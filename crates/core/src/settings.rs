//! Kalici ayarlar: `%APPDATA%\Iskandil\settings.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Grafik kagidi: acik zemin, murekkep, vermilyon vurgu.
    #[default]
    Light,
    /// Gece etudu: koyu zemin, ayni vurgu.
    Dark,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub last_path: Option<String>,
    pub theme: ThemeMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_path: None,
            theme: ThemeMode::Light,
        }
    }
}

pub fn settings_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Iskandil").join("settings.json")
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = settings_path();
        let Some(dir) = path.parent() else {
            return;
        };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let Ok(text) = serde_json::to_string_pretty(self) else {
            return;
        };
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, text).is_err() {
            return;
        }
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::rename(&tmp, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_light_theme() {
        let s = Settings::default();
        assert_eq!(s.theme, ThemeMode::Light);
        assert!(s.last_path.is_none());
    }

    #[test]
    fn settings_roundtrip_through_json() {
        let s = Settings {
            last_path: Some("C:\\Users".into()),
            theme: ThemeMode::Dark,
        };
        let text = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(back.theme, ThemeMode::Dark);
        assert_eq!(back.last_path.as_deref(), Some("C:\\Users"));
    }
}
