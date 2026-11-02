//! Functions the window calls through `invoke`.

use crate::config::{Config, Overlay};
use crate::processes::{self, file_name};
use crate::program_info::ProgramInfo;
use crate::state::{AppState, GameStatus, LogEntry, LogLevel};
use crate::{apply_config, set_paused as apply_paused};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Config {
    state.config.read().unwrap().clone()
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: Config) -> Result<(), String> {
    apply_config(&app, config)
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> Vec<GameStatus> {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_log(state: State<AppState>) -> Vec<LogEntry> {
    state.log_entries()
}

#[tauri::command]
pub fn get_paused(state: State<AppState>) -> bool {
    state.paused.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn set_paused(app: AppHandle, paused: bool) {
    apply_paused(&app, paused);
}

#[derive(Serialize)]
pub struct RunningProgram {
    name: String,
    exe: Option<String>,
}

/// Distinct running programs, for picking a game that's already open.
#[tauri::command]
pub async fn list_running_programs() -> Result<Vec<RunningProgram>, String> {
    let mut sys = System::new().map_err(|e| format!("Can't read running processes: {e}"))?;
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );
    let mut programs = BTreeMap::new();
    for p in sys.processes().values() {
        let name = p.name().to_string_lossy().into_owned();
        let exe = p.exe().map(|e| e.to_string_lossy().into_owned());
        let key = exe.clone().unwrap_or_else(|| name.clone()).to_lowercase();
        programs.entry(key).or_insert(RunningProgram { name, exe });
    }
    let mut programs: Vec<_> = programs.into_values().collect();
    programs.sort_by_key(|p| p.name.to_lowercase());
    Ok(programs)
}

#[tauri::command]
pub async fn program_info(path: String) -> ProgramInfo {
    crate::program_info::program_info(&path)
}

/// Starts the given overlays right away (ignoring delays) so the user can check
/// their setup without launching the game. Returns how many started.
#[tauri::command]
pub fn test_overlays(app: AppHandle, state: State<AppState>, overlays: Vec<Overlay>) -> usize {
    let mut started = 0;
    for overlay in overlays.iter().filter(|o| o.enabled && !o.path.trim().is_empty()) {
        let name = file_name(&overlay.path);
        match processes::start(&overlay.path) {
            Ok(_) => {
                started += 1;
                state.log(&app, LogLevel::Info, format!("Test: started {name}"));
            }
            Err(e) => state.log(&app, LogLevel::Error, format!("Test: couldn't start {name}: {e}")),
        }
    }
    started
}
