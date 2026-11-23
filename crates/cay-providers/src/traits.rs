use cay_core::models::{
    BatteryStatus, CalendarMeeting, MediaInfo, PrivacyStatus, ToastNotification,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Platform API error: {0}")]
    PlatformError(String),

    #[error("Access denied or permission not granted: {0}")]
    AccessDenied(String),

    #[error("Service unavailable or not initialized: {0}")]
    Unavailable(String),

    #[error("Internal provider error: {0}")]
    Internal(String),
}

/// Interface for operating media sessions (SMTC on Windows).
pub trait MediaProvider: Send + Sync {
    /// Retrieve current active track metadata, if any.
    fn get_current_media(&self) -> Result<Option<MediaInfo>, ProviderError>;

    /// Resume media playback.
    fn play(&self) -> Result<(), ProviderError>;

    /// Pause media playback.
    fn pause(&self) -> Result<(), ProviderError>;

    /// Skip to the next track.
    fn next_track(&self) -> Result<(), ProviderError>;

    /// Return to the previous track.
    fn previous_track(&self) -> Result<(), ProviderError>;

    /// Seek to a specific playback timestamp in seconds.
    fn seek_to(&self, position_secs: f64) -> Result<(), ProviderError>;
}

/// Interface for querying and listening to system toasts.
pub trait NotificationProvider: Send + Sync {
    /// Request OS permission to read notifications.
    fn request_permission(&self) -> Result<bool, ProviderError>;

    /// Retrieve unread or recent toasts.
    fn get_recent_notifications(&self) -> Result<Vec<ToastNotification>, ProviderError>;

    /// Dismiss or clear a specific notification by ID.
    fn dismiss_notification(&self, id: u64) -> Result<(), ProviderError>;

    /// Clear all active notifications.
    fn clear_all(&self) -> Result<(), ProviderError>;
}

/// Interface for retrieving upcoming calendar appointments.
pub trait CalendarProvider: Send + Sync {
    /// Query upcoming meetings within the given lookahead window (in minutes).
    fn get_upcoming_meetings(
        &self,
        lookahead_minutes: u32,
    ) -> Result<Vec<CalendarMeeting>, ProviderError>;
}

/// Interface for reading hardware and system battery/power/privacy telemetry.
pub trait SystemProvider: Send + Sync {
    /// Query current battery level and charging state.
    fn get_battery_status(&self) -> Result<BatteryStatus, ProviderError>;

    /// Set master audio volume percentage (0..=100).
    fn set_master_volume(&self, percent: u8) -> Result<(), ProviderError>;

    /// Query current hardware privacy status (microphone active, camera active, microphone muted).
    fn get_privacy_status(&self) -> Result<PrivacyStatus, ProviderError>;

    /// Set global microphone mute state.
    fn set_microphone_mute(&self, muted: bool) -> Result<(), ProviderError>;
}
