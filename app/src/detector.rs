//! Decides when a game started or stopped and which overlay processes to close.
//!
//! Pure logic over process snapshots (no OS calls), so it's fully unit-tested.
//!
//! Both games and overlays are tracked as *process trees*: the process we saw
//! or started, plus every child it spawns. That handles launchers, which start
//! the real program and then exit (Discord's Update.exe, game launchers, ...).

use crate::config::{Game, Overlay};
use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

/// Consecutive polls the game must be gone before it counts as closed.
/// Absorbs launchers that briefly restart the game.
pub const EXIT_GRACE_POLLS: u8 = 3;

/// Processes that are never adopted into a tree or closed, whatever the config says.
const PROTECTED_NAMES: &[&str] = &[
    "system", "registry", "smss.exe", "csrss.exe", "wininit.exe", "winlogon.exe", "services.exe",
    "lsass.exe", "lsaiso.exe", "svchost.exe", "dwm.exe", "explorer.exe", "fontdrvhost.exe",
    "sihost.exe", "taskhostw.exe", "ctfmon.exe", "conhost.exe", "runtimebroker.exe", "audiodg.exe",
    "spoolsv.exe", "searchhost.exe", "startmenuexperiencehost.exe", "shellexperiencehost.exe",
    "textinputhost.exe", "applicationframehost.exe", "msmpeng.exe", "securityhealthservice.exe",
    "werfault.exe", "dllhost.exe", "wmiprvse.exe", "steam.exe", "steamwebhelper.exe",
];

/// One running process. Name and exe are normalized via [`normalize`].
#[derive(Debug, Clone)]
pub struct Proc {
    pub pid: u32,
    pub parent: Option<u32>,
    /// Unix seconds; 0 when Windows won't tell us (e.g. elevated processes).
    pub start_time: u64,
    pub name: String,
    pub exe: Option<String>,
}

impl Proc {
    pub fn new(pid: u32, parent: Option<u32>, start_time: u64, name: &str, exe: Option<&str>) -> Self {
        Self { pid, parent, start_time, name: normalize(name), exe: exe.map(normalize) }
    }

    /// Windows itself, or something we must never close.
    pub fn is_protected(&self) -> bool {
        self.pid <= 4
            || PROTECTED_NAMES.contains(&self.name.as_str())
            || self.exe.as_deref().is_some_and(|exe| exe.starts_with("c:/windows/"))
    }

    /// True if this process could be one that existed at or after `since`.
    /// Guards against Windows reusing the ID of a process that already exited.
    fn started_since(&self, since: u64) -> bool {
        self.start_time == 0 || since == 0 || self.start_time + 1 >= since
    }
}

/// All running processes, indexed by PID.
pub struct Snapshot {
    procs: Vec<Proc>,
    by_pid: HashMap<u32, usize>,
}

impl Snapshot {
    pub fn new(procs: Vec<Proc>) -> Self {
        let by_pid = procs.iter().enumerate().map(|(i, p)| (p.pid, i)).collect();
        Self { procs, by_pid }
    }

    fn get(&self, pid: u32) -> Option<&Proc> {
        self.by_pid.get(&pid).map(|&i| &self.procs[i])
    }

    fn matching<'a>(&'a self, target: &'a Target) -> impl Iterator<Item = &'a Proc> {
        self.procs.iter().filter(move |p| target.matches(p))
    }
}

/// Lowercases and uses forward slashes so Windows paths compare case-insensitively.
pub fn normalize(path: &str) -> String {
    path.trim().trim_matches('"').replace('\\', "/").to_lowercase()
}

/// Matches processes against a configured path. A bare file name ("game.exe")
/// matches by name only; a full path matches by exe path, falling back to the
/// name when Windows won't tell us the exe (e.g. elevated or protected processes).
#[derive(Debug)]
pub struct Target {
    full: Option<String>,
    name: String,
}

impl Target {
    pub fn new(path: &str) -> Option<Self> {
        let full = normalize(path);
        if full.is_empty() {
            return None;
        }
        let name = full.rsplit('/').next().unwrap_or(&full).to_string();
        let full = full.contains('/').then_some(full);
        Some(Self { full, name })
    }

    pub fn matches(&self, proc: &Proc) -> bool {
        match (&self.full, &proc.exe) {
            (Some(full), Some(exe)) => full == exe,
            _ => self.name == proc.name,
        }
    }
}

/// A process plus every descendant seen since it started.
#[derive(Debug, Clone, Default)]
pub struct ProcessTree {
    pids: BTreeSet<u32>,
    /// Unix seconds when the root started; descendants must be newer.
    since: u64,
}

impl ProcessTree {
    fn new(pid: u32, since: u64) -> Self {
        let pids = if pid == 0 { BTreeSet::new() } else { BTreeSet::from([pid]) };
        Self { pids, since }
    }

    fn add(&mut self, pid: u32) {
        self.pids.insert(pid);
    }

    /// Adopts children of members (repeatedly, for grandchildren), then drops
    /// members that exited. Adopting first keeps children whose parent was a
    /// launcher that already quit.
    fn update(&mut self, snapshot: &Snapshot) {
        loop {
            let children: Vec<u32> = snapshot
                .procs
                .iter()
                .filter(|p| {
                    p.parent.is_some_and(|parent| self.pids.contains(&parent))
                        && !self.pids.contains(&p.pid)
                        && p.started_since(self.since)
                        && !p.is_protected()
                })
                .map(|p| p.pid)
                .collect();
            if children.is_empty() {
                break;
            }
            self.pids.extend(children);
        }
        let since = self.since;
        self.pids.retain(|&pid| snapshot.get(pid).is_some_and(|p| p.started_since(since)));
    }

    pub fn pids(&self) -> impl Iterator<Item = u32> + '_ {
        self.pids.iter().copied()
    }

    fn is_empty(&self) -> bool {
        self.pids.is_empty()
    }
}

#[derive(Debug, Default)]
pub enum GameState {
    #[default]
    Idle,
    Running(Session),
}

/// A game that's running, and what we launched for it.
#[derive(Debug)]
pub struct Session {
    pub game: ProcessTree,
    misses: u8,
    pending: Vec<Pending>,
    pub overlays: Vec<LaunchedOverlay>,
}

#[derive(Debug)]
struct Pending {
    overlay: Overlay,
    due: Instant,
}

#[derive(Debug, Clone)]
pub struct LaunchedOverlay {
    pub path: String,
    pub close_on_exit: bool,
    pub tree: ProcessTree,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Launch(Overlay),
    /// Close these processes, all belonging to the overlay at `path`.
    Close { path: String, pids: Vec<u32> },
}

impl GameState {
    /// Earliest pending overlay launch, so the poll loop can wake up for it.
    pub fn next_due(&self) -> Option<Instant> {
        match self {
            Self::Running(session) => session.pending.iter().map(|p| p.due).min(),
            Self::Idle => None,
        }
    }

    /// Records an overlay we just started. `pid` is 0 when Windows didn't give
    /// us one (UAC launches); it's then found by name when closing.
    pub fn record_launch(&mut self, overlay: &Overlay, pid: u32, now_unix: u64) {
        if let Self::Running(session) = self {
            session.overlays.push(LaunchedOverlay {
                path: overlay.path.clone(),
                close_on_exit: overlay.close_on_exit,
                tree: ProcessTree::new(pid, now_unix),
            });
        }
    }
}

/// Advances one game's state given the current processes and returns what the
/// caller should do. Launch results are fed back via [`GameState::record_launch`].
pub fn step(game: &Game, state: &mut GameState, snapshot: &Snapshot, paused: bool, now: Instant) -> Vec<Action> {
    let target = Target::new(&game.exe);
    let matches: Vec<&Proc> = target.as_ref().map_or(Vec::new(), |t| snapshot.matching(t).collect());

    if let GameState::Idle = state {
        let Some(first) = matches.first() else { return Vec::new() };
        if paused || !game.enabled {
            return Vec::new();
        }
        let pending = game
            .overlays
            .iter()
            .filter(|o| o.enabled && !o.path.trim().is_empty())
            .map(|o| Pending { overlay: o.clone(), due: now + Duration::from_millis(o.delay_ms) })
            .collect();
        *state = GameState::Running(Session {
            game: ProcessTree::new(first.pid, first.start_time),
            misses: 0,
            pending,
            overlays: Vec::new(),
        });
    }

    let GameState::Running(session) = state else { unreachable!() };

    // The game is running while its program, or anything it started, is.
    for p in &matches {
        session.game.add(p.pid);
    }
    session.game.update(snapshot);
    for overlay in &mut session.overlays {
        overlay.tree.update(snapshot);
    }

    if session.game.is_empty() {
        session.misses += 1;
        if session.misses < EXIT_GRACE_POLLS {
            return Vec::new();
        }
        let actions = session
            .overlays
            .iter()
            .filter(|o| o.close_on_exit)
            .filter_map(|o| {
                let pids = close_targets(o, snapshot);
                (!pids.is_empty()).then(|| Action::Close { path: o.path.clone(), pids })
            })
            .collect();
        *state = GameState::Idle;
        return actions;
    }
    session.misses = 0;

    if paused {
        session.pending.clear();
        return Vec::new();
    }

    let (due, later): (Vec<_>, Vec<_>) = session.pending.drain(..).partition(|p| p.due <= now);
    session.pending = later;
    due.into_iter()
        .filter(|p| {
            let already_running = Target::new(&p.overlay.path)
                .is_some_and(|t| snapshot.matching(&t).next().is_some());
            !(p.overlay.skip_if_running && already_running)
        })
        .map(|p| Action::Launch(p.overlay))
        .collect()
}

/// Everything still alive in the overlay's tree. If the whole tree is gone
/// (e.g. we got no PID from a UAC launch), fall back to processes with the
/// overlay's exe that started after we launched it. Never protected processes.
fn close_targets(overlay: &LaunchedOverlay, snapshot: &Snapshot) -> Vec<u32> {
    let alive: Vec<u32> = overlay.tree.pids().filter(|&pid| snapshot.get(pid).is_some_and(|p| !p.is_protected())).collect();
    if !alive.is_empty() {
        return alive;
    }
    let Some(target) = Target::new(&overlay.path) else { return Vec::new() };
    snapshot
        .matching(&target)
        .filter(|p| p.started_since(overlay.tree.since) && !p.is_protected())
        .map(|p| p.pid)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAME: &str = r"C:\Games\Foo\Foo.exe";
    const HUD: &str = r"C:\Tools\Hud\hud.exe";
    const CHAT: &str = r"C:\Tools\chat.exe";
    const T0: u64 = 1_000;

    fn overlay(path: &str) -> Overlay {
        Overlay { id: path.into(), path: path.into(), ..Default::default() }
    }

    fn game(overlays: Vec<Overlay>) -> Game {
        Game { id: "g".into(), name: "Foo".into(), exe: GAME.into(), enabled: true, overlays }
    }

    fn proc(pid: u32, exe: &str) -> Proc {
        child(pid, 0, exe)
    }

    fn child(pid: u32, parent: u32, exe: &str) -> Proc {
        let name = exe.rsplit('\\').next().unwrap();
        let parent = (parent != 0).then_some(parent);
        Proc::new(pid, parent, T0 + 1, name, Some(exe))
    }

    fn snap(procs: &[Proc]) -> Snapshot {
        Snapshot::new(procs.to_vec())
    }

    fn launches(actions: &[Action]) -> Vec<&str> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Launch(o) => Some(o.path.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Runs `step` and records a fake PID (1000 + n) for each launch.
    fn tick(g: &Game, s: &mut GameState, procs: &[Proc]) -> Vec<Action> {
        let actions = step(g, s, &snap(procs), false, Instant::now());
        for (i, a) in actions.iter().enumerate() {
            if let Action::Launch(o) = a {
                s.record_launch(o, 1000 + i as u32, T0);
            }
        }
        actions
    }

    /// Ticks with `procs` until the exit grace period runs out.
    fn tick_exit(g: &Game, s: &mut GameState, procs: &[Proc]) -> Vec<Action> {
        for _ in 1..EXIT_GRACE_POLLS {
            assert!(tick(g, s, procs).is_empty(), "closed before the grace period ended");
        }
        tick(g, s, procs)
    }

    fn close(path: &str, pids: &[u32]) -> Action {
        Action::Close { path: path.into(), pids: pids.to_vec() }
    }

    #[test]
    fn nothing_happens_without_game() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        assert!(tick(&g, &mut s, &[proc(5, HUD)]).is_empty());
        assert!(matches!(s, GameState::Idle));
    }

    #[test]
    fn game_start_launches_and_exit_closes() {
        let g = game(vec![overlay(HUD), overlay(CHAT)]);
        let mut s = GameState::Idle;

        assert_eq!(launches(&tick(&g, &mut s, &[proc(1, GAME)])), vec![HUD, CHAT]);
        assert!(tick(&g, &mut s, &[proc(1, GAME), proc(1000, HUD), proc(1001, CHAT)]).is_empty());

        let after = [proc(1000, HUD), proc(1001, CHAT)];
        assert_eq!(tick_exit(&g, &mut s, &after), vec![close(HUD, &[1000]), close(CHAT, &[1001])]);
        assert!(matches!(s, GameState::Idle));
    }

    #[test]
    fn works_again_after_the_first_session() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        tick_exit(&g, &mut s, &[proc(1000, HUD)]);
        // Overlay closed; game started again later.
        assert_eq!(launches(&tick(&g, &mut s, &[proc(7, GAME)])), vec![HUD]);
    }

    #[test]
    fn overlay_started_through_a_launcher_is_closed() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        // hud.exe (1000) starts real.exe (1500) and exits.
        tick(&g, &mut s, &[proc(1, GAME), proc(1000, HUD), child(1500, 1000, r"C:\Tools\Hud\real.exe")]);
        tick(&g, &mut s, &[proc(1, GAME), child(1500, 1000, r"C:\Tools\Hud\real.exe")]);
        let after = [child(1500, 1000, r"C:\Tools\Hud\real.exe")];
        assert_eq!(tick_exit(&g, &mut s, &after), vec![close(HUD, &[1500])]);
    }

    #[test]
    fn game_started_through_a_launcher_keeps_running() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        // Foo.exe (1) starts the real game (20) and exits.
        let real = child(20, 1, r"C:\Games\Foo\FooGame.exe");
        tick(&g, &mut s, &[proc(1, GAME), real.clone(), proc(1000, HUD)]);
        for _ in 0..5 {
            assert!(tick(&g, &mut s, &[real.clone(), proc(1000, HUD)]).is_empty());
        }
        assert!(matches!(&s, GameState::Running(_)));
        assert_eq!(tick_exit(&g, &mut s, &[proc(1000, HUD)]), vec![close(HUD, &[1000])]);
    }

    #[test]
    fn brief_game_restart_is_not_an_exit() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        assert!(tick(&g, &mut s, &[proc(1000, HUD)]).is_empty());
        assert!(tick(&g, &mut s, &[proc(2, GAME), proc(1000, HUD)]).is_empty());
        assert!(matches!(&s, GameState::Running(_)));
    }

    #[test]
    fn protected_processes_are_never_closed() {
        let g = game(vec![overlay(HUD), overlay(r"C:\Windows\System32\notepad.exe")]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        // The HUD spawns a console host and opens Explorer; neither may be adopted.
        let running = [
            proc(1, GAME),
            proc(1000, HUD),
            child(1600, 1000, r"C:\Windows\System32\conhost.exe"),
            child(1700, 1000, r"C:\Windows\explorer.exe"),
            proc(1001, r"C:\Windows\System32\notepad.exe"),
        ];
        tick(&g, &mut s, &running);
        let after = &running[1..];
        assert_eq!(tick_exit(&g, &mut s, after), vec![close(HUD, &[1000])]);
    }

    #[test]
    fn reused_pid_is_not_closed() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        tick(&g, &mut s, &[proc(1, GAME)]); // HUD (1000) exited
        // Windows hands PID 1000 to an unrelated program.
        let after = [proc(1000, r"C:\Other\app.exe")];
        assert!(tick_exit(&g, &mut s, &after).is_empty());
    }

    #[test]
    fn close_on_exit_false_leaves_overlay() {
        let g = game(vec![Overlay { close_on_exit: false, ..overlay(HUD) }, overlay(CHAT)]);
        let mut s = GameState::Idle;
        tick(&g, &mut s, &[proc(1, GAME)]);
        let after = [proc(1000, HUD), proc(1001, CHAT)];
        assert_eq!(tick_exit(&g, &mut s, &after), vec![close(CHAT, &[1001])]);
    }

    #[test]
    fn skip_if_running_does_not_launch_or_close_existing() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        assert!(tick(&g, &mut s, &[proc(1, GAME), proc(77, HUD)]).is_empty());
        assert!(tick_exit(&g, &mut s, &[proc(77, HUD)]).is_empty());

        let g = game(vec![Overlay { skip_if_running: false, ..overlay(HUD) }]);
        let mut s = GameState::Idle;
        assert_eq!(launches(&tick(&g, &mut s, &[proc(1, GAME), proc(77, HUD)])), vec![HUD]);
    }

    #[test]
    fn no_pid_from_uac_launch_closes_by_name() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        let actions = step(&g, &mut s, &snap(&[proc(1, GAME)]), false, Instant::now());
        let Action::Launch(o) = &actions[0] else { panic!() };
        s.record_launch(o, 0, T0);
        assert_eq!(tick_exit(&g, &mut s, &[proc(4242, HUD)]), vec![close(HUD, &[4242])]);
    }

    #[test]
    fn delayed_overlay_launches_when_due() {
        let g = game(vec![Overlay { delay_ms: 5000, ..overlay(HUD) }, overlay(CHAT)]);
        let mut s = GameState::Idle;
        let now = Instant::now();
        let running = snap(&[proc(1, GAME)]);
        assert_eq!(launches(&step(&g, &mut s, &running, false, now)), vec![CHAT]);
        assert_eq!(s.next_due(), Some(now + Duration::from_millis(5000)));
        let later = now + Duration::from_millis(5000);
        assert_eq!(launches(&step(&g, &mut s, &running, false, later)), vec![HUD]);
        assert_eq!(s.next_due(), None);
    }

    #[test]
    fn disabled_overlay_is_skipped() {
        let g = game(vec![Overlay { enabled: false, ..overlay(HUD) }, overlay(CHAT)]);
        let mut s = GameState::Idle;
        assert_eq!(launches(&tick(&g, &mut s, &[proc(1, GAME)])), vec![CHAT]);
    }

    #[test]
    fn paused_or_disabled_game_does_not_launch() {
        let g = game(vec![overlay(HUD)]);
        let mut s = GameState::Idle;
        assert!(step(&g, &mut s, &snap(&[proc(1, GAME)]), true, Instant::now()).is_empty());
        assert!(matches!(s, GameState::Idle));

        let g = Game { enabled: false, ..game(vec![overlay(HUD)]) };
        assert!(tick(&g, &mut s, &[proc(1, GAME)]).is_empty());
        assert!(matches!(s, GameState::Idle));
    }

    #[test]
    fn matching_is_case_insensitive_and_falls_back_to_name() {
        let t = Target::new(GAME).unwrap();
        assert!(t.matches(&Proc::new(1, None, 0, "FOO.EXE", Some(r"c:\games\foo\FOO.exe"))));
        assert!(!t.matches(&Proc::new(1, None, 0, "foo.exe", Some(r"D:\Other\foo.exe"))));
        assert!(t.matches(&Proc::new(1, None, 0, "Foo.exe", None)));
        let t = Target::new("foo.exe").unwrap();
        assert!(t.matches(&Proc::new(1, None, 0, "foo.exe", Some(r"D:\Other\foo.exe"))));
        assert!(Target::new("  ").is_none());
    }
}
