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
