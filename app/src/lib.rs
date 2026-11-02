//! Overlay Manager: starts a game's overlays when the game starts, and closes
//! them when it exits. This file wires the Tauri app together.

mod commands;
mod config;
mod detector;
mod monitor;
mod processes;
mod program_info;
mod state;
mod tray;

use config::{Config, MIN_POLL_MS};
use state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// Passed by the autostart entry so the app starts hidden in the tray.
const MINIMIZED_ARG: &str = "--minimized";

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn set_paused(app: &AppHandle, paused: bool) {
    app.state::<AppState>().paused.store(paused, Ordering::Relaxed);
    let _ = app.emit("paused", paused);
    tray::refresh(app);
    monitor::wake();
}

fn set_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();
    let result = if enabled { autolaunch.enable() } else { autolaunch.disable() };
    result.map_err(|e| format!("Couldn't change start-with-Windows: {e}"))
}

/// Saves a new config and applies it: autostart, monitor and tray.
fn apply_config(app: &AppHandle, mut config: Config) -> Result<(), String> {
    let state = app.state::<AppState>();
    config.poll_interval_ms = config.poll_interval_ms.max(MIN_POLL_MS);
    config.save(&state.config_path).map_err(|e| format!("Couldn't save settings: {e}"))?;

    let autostart_changed = state.config.read().unwrap().autostart != config.autostart;
    let autostart = config.autostart;
    *state.config.write().unwrap() = config;
    monitor::wake();
    tray::refresh(app);

    if autostart_changed {
        set_autostart(app, autostart)?;
    }
    Ok(())
}

/// Edits the current config in place, applies it, and returns the result.
fn update_config(app: &AppHandle, edit: impl FnOnce(&mut Config)) -> Result<Config, String> {
    let mut config = app.state::<AppState>().config.read().unwrap().clone();
    edit(&mut config);
    apply_config(app, config.clone())?;
    Ok(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first: a second launch just focuses the running app.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_window(app)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![MINIMIZED_ARG])))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            let config = Config::load(&config_path);
            let autostart = config.autostart;
            app.manage(AppState::new(config, config_path));

            let handle = app.handle();
            tray::create(handle)?;
            // Re-register if the entry went missing (e.g. the app was moved or updated).
            if autostart && !handle.autolaunch().is_enabled().unwrap_or(false) {
                let _ = set_autostart(handle, true);
            }
            if !std::env::args().any(|a| a == MINIMIZED_ARG) {
                show_window(handle);
            }
            monitor::start(handle.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window keeps the app running in the tray.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_status,
            commands::get_log,
            commands::get_paused,
            commands::set_paused,
            commands::list_running_programs,
            commands::program_info,
            commands::test_overlays,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
