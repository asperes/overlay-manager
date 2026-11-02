//! The settings file: which games to watch and which overlays go with each.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MIN_POLL_MS: u64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    #[serde(alias = "profiles")] // name used by 0.1.0
    pub games: Vec<Game>,
    pub poll_interval_ms: u64,
    pub autostart: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { games: Vec::new(), poll_interval_ms: 2000, autostart: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    /// Full path to the game's program, or just its file name (e.g. "game.exe").
    #[serde(alias = "gameExe")] // name used by 0.1.0
    pub exe: String,
    pub enabled: bool,
    pub overlays: Vec<Overlay>,
}

impl Game {
    /// Name to show: the given name, else the exe's file name.
    pub fn display_name(&self) -> String {
        if !self.name.trim().is_empty() {
            return self.name.trim().to_string();
        }
        match self.exe.rsplit(['\\', '/']).next() {
            Some(exe) if !exe.trim().is_empty() => exe.to_string(),
            _ => "New game".to_string(),
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self { id: String::new(), name: String::new(), exe: String::new(), enabled: true, overlays: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Overlay {
    /// Stable key for the UI; assigned by the frontend.
    pub id: String,
    pub enabled: bool,
    pub path: String,
    pub delay_ms: u64,
    pub close_on_exit: bool,
    pub skip_if_running: bool,
}

impl Default for Overlay {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: true,
            path: String::new(),
            delay_ms: 0,
            close_on_exit: true,
            skip_if_running: true,
        }
    }
}

impl Config {
    /// Loads the config, falling back to defaults. A file that fails to parse is
    /// moved aside to `config.json.bak` so the next save doesn't destroy it.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        match serde_json::from_str::<Config>(&text) {
            Ok(mut config) => {
                config.poll_interval_ms = config.poll_interval_ms.max(MIN_POLL_MS);
                config
            }
            Err(err) => {
                eprintln!("config parse error ({err}); backing up and starting fresh");
                let _ = fs::rename(path, backup_path(path));
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        // Write to a temp file and rename so a crash mid-write can't corrupt the config.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&tmp, path)
    }
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_get_defaults() {
        let config: Config =
            serde_json::from_str(r#"{"games":[{"id":"a","exe":"x.exe","overlays":[{"path":"o.exe"}]}]}"#).unwrap();
        assert_eq!(config.poll_interval_ms, 2000);
        let game = &config.games[0];
        assert!(game.enabled);
        assert!(game.overlays[0].enabled);
        assert!(game.overlays[0].close_on_exit);
        assert!(game.overlays[0].skip_if_running);
    }

    #[test]
    fn reads_0_1_0_configs() {
        let config: Config = serde_json::from_str(
            r#"{"profiles":[{"id":"a","gameExe":"x.exe","overlays":[{"path":"o.exe","args":"--x"}]}]}"#,
        )
        .unwrap();
        assert_eq!(config.games[0].exe, "x.exe");
        assert_eq!(config.games[0].overlays[0].path, "o.exe");
    }

    #[test]
    fn round_trip_and_corrupt_backup() {
        let dir = std::env::temp_dir().join(format!("om-test-{}", std::process::id()));
        let path = dir.join("config.json");
        let mut config = Config::default();
        config.games.push(Game { id: "1".into(), name: "G".into(), ..Default::default() });
        config.save(&path).unwrap();
        assert_eq!(Config::load(&path), config);

        fs::write(&path, "{not json").unwrap();
        assert_eq!(Config::load(&path), Config::default());
        assert!(backup_path(&path).exists());
        let _ = fs::remove_dir_all(dir);
    }
}
