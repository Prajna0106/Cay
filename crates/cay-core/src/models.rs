use serde::{Deserialize, Serialize};

/// Live playback information from media players.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub is_playing: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub source_app: String,
    #[serde(skip)]
    pub thumbnail_bytes: Option<Vec<u8>>,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album: None,
            is_playing: false,
            position_secs: 0.0,
            duration_secs: 0.0,
            source_app: String::new(),
            thumbnail_bytes: None,
        }
    }
}

/// Incoming or cached desktop toast notification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToastNotification {
    pub id: u64,
    pub app_name: String,
    pub channel: Option<String>,
    pub sender: String,
    pub message: String,
    pub received_timestamp: u64,
}

/// Scheduled calendar meeting or appointment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarMeeting {
    pub id: String,
    pub title: String,
    pub start_timestamp: i64,
    pub duration_minutes: u32,
    pub service_name: String,
    pub join_url: Option<String>,
}

/// Active countdown or focus timer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerState {
    pub label: String,
    pub remaining_seconds: u32,
    pub is_running: bool,
}

/// Volume or display brightness HUD indicator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudKind {
    Volume,
    Brightness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudState {
    pub kind: HudKind,
    pub level_percent: u8,
}

/// Device power and battery status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub percentage: u8,
    pub is_charging: bool,
    pub is_present: bool,
}

impl Default for BatteryStatus {
    fn default() -> Self {
        Self {
            percentage: 100,
            is_charging: false,
            is_present: true,
        }
    }
}

/// User-selectable accent color presets from `design/tokens.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccentPreset {
    #[default]
    Sky,
    Lime,
    Warm,
    Purple,
}

/// User preferences for Cay island behavior and appearance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IslandSettings {
    pub accent: AccentPreset,
    pub top_offset_px: f32,
    pub hover_expand_enabled: bool,
    pub enable_media: bool,
    pub enable_notifications: bool,
    pub enable_calendar: bool,
    pub ics_source: Option<String>,
}

impl Default for IslandSettings {
    fn default() -> Self {
        Self {
            accent: AccentPreset::Sky,
            top_offset_px: 14.0,
            hover_expand_enabled: true,
            enable_media: true,
            enable_notifications: true,
            enable_calendar: true,
            ics_source: None,
        }
    }
}

impl IslandSettings {
    /// Resolves `%APPDATA%\Cay\settings.json` on Windows (or `$HOME/.config/cay/settings.json`).
    pub fn default_settings_path() -> Option<std::path::PathBuf> {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return Some(
                std::path::PathBuf::from(appdata)
                    .join("Cay")
                    .join("settings.json"),
            );
        }
        std::env::var_os("HOME").map(|home| {
            std::path::PathBuf::from(home)
                .join(".config")
                .join("cay")
                .join("settings.json")
        })
    }

    pub fn load_from_path(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read settings file {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings JSON {}: {e}", path.display()))
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create settings directory {}: {e}",
                    parent.display()
                )
            })?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {e}"))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write settings file {}: {e}", path.display()))
    }

    pub fn load_default() -> Self {
        Self::default_settings_path()
            .and_then(|p| Self::load_from_path(&p).ok())
            .unwrap_or_default()
    }

    pub fn save_default(&self) -> Result<(), String> {
        let Some(path) = Self::default_settings_path() else {
            return Err("Could not determine default settings directory".to_string());
        };
        self.save_to_path(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_json_roundtrip() {
        let settings = IslandSettings {
            accent: AccentPreset::Purple,
            top_offset_px: 18.0,
            hover_expand_enabled: false,
            enable_media: true,
            enable_notifications: true,
            enable_calendar: false,
            ics_source: Some("C:\\Calendars\\work.ics".to_string()),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: IslandSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }
}
