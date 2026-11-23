use crate::traits::{
    CalendarProvider, MediaProvider, NotificationProvider, ProviderError, SystemProvider,
};
use cay_core::models::{
    BatteryStatus, CalendarMeeting, MediaInfo, PrivacyStatus, ToastNotification,
};

/// In-memory stub for media operations, returning no active session.
#[derive(Clone, Debug, Default)]
pub struct StubMediaProvider;

impl MediaProvider for StubMediaProvider {
    fn get_current_media(&self) -> Result<Option<MediaInfo>, ProviderError> {
        Ok(None)
    }

    fn play(&self) -> Result<(), ProviderError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), ProviderError> {
        Ok(())
    }

    fn next_track(&self) -> Result<(), ProviderError> {
        Ok(())
    }

    fn previous_track(&self) -> Result<(), ProviderError> {
        Ok(())
    }

    fn seek_to(&self, _position_secs: f64) -> Result<(), ProviderError> {
        Ok(())
    }
}

/// In-memory stub for notifications, returning no toasts.
#[derive(Clone, Debug, Default)]
pub struct StubNotificationProvider;

impl NotificationProvider for StubNotificationProvider {
    fn request_permission(&self) -> Result<bool, ProviderError> {
        Ok(true)
    }

    fn get_recent_notifications(&self) -> Result<Vec<ToastNotification>, ProviderError> {
        Ok(Vec::new())
    }

    fn dismiss_notification(&self, _id: u64) -> Result<(), ProviderError> {
        Ok(())
    }

    fn clear_all(&self) -> Result<(), ProviderError> {
        Ok(())
    }
}

/// In-memory stub for calendar events, returning an empty schedule.
#[derive(Clone, Debug, Default)]
pub struct StubCalendarProvider;

impl CalendarProvider for StubCalendarProvider {
    fn get_upcoming_meetings(
        &self,
        _lookahead_minutes: u32,
    ) -> Result<Vec<CalendarMeeting>, ProviderError> {
        Ok(Vec::new())
    }
}

/// In-memory stub for system telemetry, returning default battery and privacy.
#[derive(Clone, Debug, Default)]
pub struct StubSystemProvider;

impl SystemProvider for StubSystemProvider {
    fn get_battery_status(&self) -> Result<BatteryStatus, ProviderError> {
        Ok(BatteryStatus {
            percentage: 100,
            is_charging: false,
            is_present: true,
        })
    }

    fn set_master_volume(&self, _percent: u8) -> Result<(), ProviderError> {
        Ok(())
    }

    fn get_privacy_status(&self) -> Result<PrivacyStatus, ProviderError> {
        Ok(PrivacyStatus::default())
    }

    fn set_microphone_mute(&self, _muted: bool) -> Result<(), ProviderError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stubs_return_clean_defaults() {
        let media = StubMediaProvider;
        assert_eq!(media.get_current_media().unwrap(), None);
        assert!(media.seek_to(45.0).is_ok());

        let notif = StubNotificationProvider;
        assert!(notif.get_recent_notifications().unwrap().is_empty());

        let cal = StubCalendarProvider;
        assert!(cal.get_upcoming_meetings(15).unwrap().is_empty());

        let sys = StubSystemProvider;
        assert_eq!(sys.get_battery_status().unwrap().percentage, 100);
        assert!(sys.set_master_volume(70).is_ok());
        assert!(!sys.get_privacy_status().unwrap().is_mic_active);
        assert!(sys.set_microphone_mute(true).is_ok());
    }
}
