//! State shared between the window, the tray and the monitor thread.

use crate::config::Config;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const LOG_CAPACITY: usize = 200;

pub struct AppState {
    pub config: RwLock<Config>,
    pub config_path: PathBuf,
    pub paused: AtomicBool,
    /// Games running right now.
    pub status: Mutex<Vec<GameStatus>>,
    log: Mutex<VecDeque<LogEntry>>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    pub game_id: String,
    /// Overlays started for this session that are still tracked.
    pub overlays: usize,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    time_ms: u64,
    level: LogLevel,
    message: String,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            config: RwLock::new(config),
            config_path,
            paused: AtomicBool::new(false),
            status: Mutex::new(Vec::new()),
            log: Mutex::new(VecDeque::new()),
        }
    }

    /// Adds a line to the Activity page (and pushes it to the window live).
    pub fn log(&self, app: &AppHandle, level: LogLevel, message: impl Into<String>) {
        let time_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
        let entry = LogEntry { time_ms, level, message: message.into() };
        let _ = app.emit("log", &entry);
        let mut log = self.log.lock().unwrap();
        if log.len() == LOG_CAPACITY {
            log.pop_front();
        }
        log.push_back(entry);
    }

    pub fn log_entries(&self) -> Vec<LogEntry> {
        self.log.lock().unwrap().iter().cloned().collect()
    }
}
