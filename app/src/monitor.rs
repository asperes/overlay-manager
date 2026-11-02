//! Background thread: polls running processes, feeds them to the detector,
//! and starts/stops overlays as it decides.

use crate::config::MIN_POLL_MS;
use crate::detector::{self, Action, GameState, Proc, Snapshot};
use crate::processes::{self, file_name};
use crate::state::{AppState, GameStatus, LogLevel};
use crate::tray;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Emitter, Manager};

static MONITOR: OnceLock<thread::Thread> = OnceLock::new();

pub fn start(app: AppHandle) {
    let handle = thread::Builder::new()
        .name("monitor".into())
        .spawn(move || run(&app))
        .expect("failed to spawn monitor thread");
    MONITOR.set(handle.thread().clone()).ok();
}

/// Makes the monitor check right away, e.g. after the config changed.
pub fn wake() {
    if let Some(t) = MONITOR.get() {
        t.unpark();
    }
}

fn run(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut sys = match System::new() {
        Ok(sys) => sys,
        Err(e) => {
            state.log(app, LogLevel::Error, format!("Can't read running processes: {e}"));
            return;
        }
    };
    let mut games: HashMap<String, GameState> = HashMap::new();

    loop {
        let config = state.config.read().unwrap().clone();
        let paused = state.paused.load(Ordering::Relaxed);
        let snapshot = snapshot(&mut sys);
        let now = Instant::now();

        games.retain(|id, _| config.games.iter().any(|g| &g.id == id));
        for game in &config.games {
            let game_state = games.entry(game.id.clone()).or_default();
            let was_running = matches!(game_state, GameState::Running(_));
            let actions = detector::step(game, game_state, &snapshot, paused, now);

            match (was_running, matches!(game_state, GameState::Running(_))) {
                (false, true) => state.log(app, LogLevel::Info, format!("{} started", game.display_name())),
                (true, false) => state.log(app, LogLevel::Info, format!("{} closed", game.display_name())),
                _ => {}
            }

            for action in actions {
                match action {
                    Action::Launch(overlay) => match processes::start(&overlay.path) {
                        Ok(pid) => {
                            game_state.record_launch(&overlay, pid, unix_now());
                            state.log(app, LogLevel::Info, format!("Started {}", file_name(&overlay.path)));
                        }
                        Err(e) => state.log(
                            app,
                            LogLevel::Error,
                            format!("Couldn't start {}: {e}", file_name(&overlay.path)),
                        ),
                    },
                    Action::Close { path, pids } => close_in_background(app, path, pids),
                }
            }
        }

        publish_status(app, &state, &games);

        let poll = Duration::from_millis(config.poll_interval_ms.max(MIN_POLL_MS));
        let wait = games
            .values()
            .filter_map(GameState::next_due)
            .min()
            .map_or(poll, |due| due.saturating_duration_since(Instant::now()).min(poll));
        thread::park_timeout(wait);
    }
}

/// Closing can take seconds (and may show a UAC prompt), so it runs on its own
/// thread and reports the result to the Activity page.
fn close_in_background(app: &AppHandle, path: String, pids: Vec<u32>) {
    let app = app.clone();
    thread::spawn(move || {
        let state = app.state::<AppState>();
        let name = file_name(&path);
        let left = processes::stop(&pids);
        if left.is_empty() {
            state.log(&app, LogLevel::Info, format!("Closed {name}"));
        } else {
            state.log(
                &app,
                LogLevel::Error,
                format!("Couldn't close {name} (PID {left:?}). It needs admin rights and the prompt was declined."),
            );
        }
    });
}

fn snapshot(sys: &mut System) -> Snapshot {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    let procs = sys
        .processes()
        .iter()
        .map(|(pid, p)| {
            Proc::new(
                pid.as_u32(),
                p.parent().map(|parent| parent.as_u32()),
                p.start_time(),
                &p.name().to_string_lossy(),
                p.exe().and_then(|e| e.to_str()),
            )
        })
        .collect();
    Snapshot::new(procs)
}

fn publish_status(app: &AppHandle, state: &AppState, games: &HashMap<String, GameState>) {
    let mut status: Vec<GameStatus> = games
        .iter()
        .filter_map(|(id, game_state)| match game_state {
            GameState::Running(session) => Some(GameStatus { game_id: id.clone(), overlays: session.overlays.len() }),
            GameState::Idle => None,
        })
        .collect();
    status.sort_by(|a, b| a.game_id.cmp(&b.game_id));

    let mut current = state.status.lock().unwrap();
    if *current != status {
        let _ = app.emit("status", &status);
        *current = status;
        drop(current);
        tray::refresh(app);
    }
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}
