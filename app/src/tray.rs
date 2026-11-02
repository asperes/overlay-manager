//! System tray icon. The menu lists every game (tick to enable/disable,
//! marked when running) and is rebuilt whenever that information changes.

use crate::state::AppState;
use crate::{set_paused, show_window, update_config};
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

const TRAY_ID: &str = "main";
const GAME_ITEM_PREFIX: &str = "game:";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(tooltip(app))
        .menu(&build_menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Rebuilds the menu from current state. Safe to call from any thread.
pub fn refresh(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(tray) = handle.tray_by_id(TRAY_ID) else { return };
        if let Ok(menu) = build_menu(&handle) {
            let _ = tray.set_menu(Some(menu));
        }
        let _ = tray.set_tooltip(Some(tooltip(&handle)));
    });
}

fn tooltip(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    if state.paused.load(Ordering::Relaxed) {
        return "Overlay Manager: paused".into();
    }
    match state.status.lock().unwrap().len() {
        0 => "Overlay Manager: watching".into(),
        1 => "Overlay Manager: 1 game running".into(),
        n => format!("Overlay Manager: {n} games running"),
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let state = app.state::<AppState>();
    let paused = state.paused.load(Ordering::Relaxed);
    let running: HashSet<String> = state.status.lock().unwrap().iter().map(|s| s.game_id.clone()).collect();
    let games = state.config.read().unwrap().games.clone();

    let menu = Menu::new(app)?;
    let header = if paused { "Paused" } else { "Watching for games" };
    menu.append(&MenuItem::with_id(app, "header", header, false, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    if games.is_empty() {
        menu.append(&MenuItem::with_id(app, "no-games", "No games set up yet", false, None::<&str>)?)?;
    }
    for game in &games {
        let name = game.display_name();
        let label = if running.contains(&game.id) { format!("{name}  (running)") } else { name };
        let id = format!("{GAME_ITEM_PREFIX}{}", game.id);
        menu.append(&CheckMenuItem::with_id(app, id, label, true, game.enabled, None::<&str>)?)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let pause_label = if paused { "Resume watching" } else { "Pause watching" };
    menu.append(&MenuItem::with_id(app, "pause", pause_label, true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "show", "Open Overlay Manager", true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)?;
    Ok(menu)
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "show" => show_window(app),
        "pause" => {
            let paused = app.state::<AppState>().paused.load(Ordering::Relaxed);
            set_paused(app, !paused);
        }
        "quit" => app.exit(0),
        id => {
            if let Some(game_id) = id.strip_prefix(GAME_ITEM_PREFIX) {
                toggle_game(app, game_id);
            }
        }
    }
}

fn toggle_game(app: &AppHandle, game_id: &str) {
    let result = update_config(app, |config| {
        if let Some(game) = config.games.iter_mut().find(|g| g.id == game_id) {
            game.enabled = !game.enabled;
        }
    });
    match result {
        // The window didn't make this change, so tell it.
        Ok(config) => {
            let _ = app.emit("config", &config);
        }
        Err(e) => eprintln!("{e}"),
    }
}
