use crate::traits::{MediaProvider, NotificationProvider, ProviderError, SystemProvider};
use cay_core::models::{BatteryStatus, MediaInfo, ToastNotification};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::UI::Notifications::Management::{
    UserNotificationListener, UserNotificationListenerAccessStatus,
};
use windows::UI::Notifications::NotificationKinds;

fn map_win_err(err: windows::core::Error) -> ProviderError {
    ProviderError::PlatformError(format!("{} (0x{:08X})", err.message(), err.code().0 as u32))
}

/// Ensures the current background worker thread is initialized in the COM Multi-Threaded Apartment (MTA).
pub fn init_mta() {
    // SAFETY: CoInitializeEx is safe to call per thread; S_FALSE or RPC_E_CHANGED_MODE is benign.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

/// Returns the current process working set (resident RAM) in bytes.
pub fn process_working_set_bytes() -> Option<u64> {
    let mut counters = PROCESS_MEMORY_COUNTERS {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ..Default::default()
    };
    // SAFETY: GetCurrentProcess returns pseudo-handle (-1); counters is a valid initialized struct.
    let ok = unsafe {
        K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    if ok.as_bool() {
        Some(counters.WorkingSetSize as u64)
    } else {
        None
    }
}

/// Windows System Media Transport Controls (SMTC) provider.
/// Must be invoked from a background MTA worker thread, never on the main UI thread.
#[derive(Clone, Debug, Default)]
pub struct WinSmtcMediaProvider;

impl WinSmtcMediaProvider {
    pub fn new() -> Self {
        Self
    }

    fn current_session(
        &self,
    ) -> Result<Option<GlobalSystemMediaTransportControlsSession>, ProviderError> {
        init_mta();
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .map_err(map_win_err)?
            .join()
            .map_err(map_win_err)?;

        match manager.GetCurrentSession() {
            Ok(session) => Ok(Some(session)),
            // When no media player session is registered, Windows returns an empty/error HRESULT.
            Err(_) => Ok(None),
        }
    }
}

impl MediaProvider for WinSmtcMediaProvider {
    fn get_current_media(&self) -> Result<Option<MediaInfo>, ProviderError> {
        let Some(session) = self.current_session()? else {
            return Ok(None);
        };

        let props = session
            .TryGetMediaPropertiesAsync()
            .map_err(map_win_err)?
            .join()
            .map_err(map_win_err)?;

        let title = props.Title().map(|s| s.to_string()).unwrap_or_default();
        let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
        let album = props
            .AlbumTitle()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let is_playing = session
            .GetPlaybackInfo()
            .and_then(|info| info.PlaybackStatus())
            .map(|status| {
                status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
            })
            .unwrap_or(false);

        let (position_secs, duration_secs) = session
            .GetTimelineProperties()
            .map(|tl| {
                // TimeSpan Duration is in 100-nanosecond ticks (1e-7 seconds)
                let pos = tl
                    .Position()
                    .map(|t| t.Duration as f64 * 1e-7)
                    .unwrap_or(0.0);
                let end = tl
                    .EndTime()
                    .map(|t| t.Duration as f64 * 1e-7)
                    .unwrap_or(0.0);
                (pos.max(0.0), end.max(0.0))
            })
            .unwrap_or((0.0, 0.0));

        let source_app = session
            .SourceAppUserModelId()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "Media".to_string());

        if title.is_empty() && artist.is_empty() && !is_playing {
            return Ok(None);
        }

        Ok(Some(MediaInfo {
            title,
            artist,
            album,
            is_playing,
            position_secs,
            duration_secs,
            source_app,
        }))
    }

    fn play(&self) -> Result<(), ProviderError> {
        if let Some(session) = self.current_session()? {
            let _ = session
                .TryPlayAsync()
                .map_err(map_win_err)?
                .join()
                .map_err(map_win_err)?;
        }
        Ok(())
    }

    fn pause(&self) -> Result<(), ProviderError> {
        if let Some(session) = self.current_session()? {
            let _ = session
                .TryPauseAsync()
                .map_err(map_win_err)?
                .join()
                .map_err(map_win_err)?;
        }
        Ok(())
    }

    fn next_track(&self) -> Result<(), ProviderError> {
        if let Some(session) = self.current_session()? {
            let _ = session
                .TrySkipNextAsync()
                .map_err(map_win_err)?
                .join()
                .map_err(map_win_err)?;
        }
        Ok(())
    }

    fn previous_track(&self) -> Result<(), ProviderError> {
        if let Some(session) = self.current_session()? {
            let _ = session
                .TrySkipPreviousAsync()
                .map_err(map_win_err)?
                .join()
                .map_err(map_win_err)?;
        }
        Ok(())
    }
}

/// Windows `UserNotificationListener` provider.
/// Must be invoked from a background MTA worker thread.
#[derive(Clone, Debug, Default)]
pub struct WinNotificationProvider;

impl WinNotificationProvider {
    pub fn new() -> Self {
        Self
    }

    /// Queries `UserNotificationListener::Current().RequestAccessAsync()` and returns a diagnostic
    /// description of the access status (`Allowed`, `DeniedBySystem`, `DeniedByUser`, `Unspecified`).
    pub fn probe_access_status(&self) -> Result<String, ProviderError> {
        init_mta();
        let listener = UserNotificationListener::Current().map_err(map_win_err)?;
        let status = listener
            .RequestAccessAsync()
            .map_err(map_win_err)?
            .join()
            .map_err(map_win_err)?;

        let label = match status {
            UserNotificationListenerAccessStatus::Allowed => "Allowed",
            UserNotificationListenerAccessStatus::Denied => "Denied",
            UserNotificationListenerAccessStatus::Unspecified => "Unspecified",
            _ => "Unknown",
        };
        Ok(format!("{label} (raw={})", status.0))
    }
}

impl NotificationProvider for WinNotificationProvider {
    fn request_permission(&self) -> Result<bool, ProviderError> {
        init_mta();
        let listener = UserNotificationListener::Current().map_err(map_win_err)?;
        let status = listener
            .RequestAccessAsync()
            .map_err(map_win_err)?
            .join()
            .map_err(map_win_err)?;

        Ok(status == UserNotificationListenerAccessStatus::Allowed)
    }

    fn get_recent_notifications(&self) -> Result<Vec<ToastNotification>, ProviderError> {
        init_mta();
        let listener = UserNotificationListener::Current().map_err(map_win_err)?;
        let status = listener.GetAccessStatus().map_err(map_win_err)?;
        if status != UserNotificationListenerAccessStatus::Allowed {
            return Err(ProviderError::AccessDenied(format!(
                "UserNotificationListener access status: {}",
                status.0
            )));
        }

        let list = listener
            .GetNotificationsAsync(NotificationKinds::Toast)
            .map_err(map_win_err)?
            .join()
            .map_err(map_win_err)?;

        let count = list.Size().map_err(map_win_err)?;
        let mut results = Vec::new();
        for i in 0..count.min(10) {
            if let Ok(item) = list.GetAt(i) {
                let id = u64::from(item.Id().unwrap_or(0));
                let app_name = item
                    .AppInfo()
                    .and_then(|info| info.DisplayInfo())
                    .and_then(|disp| disp.DisplayName())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "System".to_string());

                results.push(ToastNotification {
                    id,
                    app_name: app_name.clone(),
                    channel: None,
                    sender: app_name,
                    message: "Notification".to_string(),
                    received_timestamp: 0,
                });
            }
        }
        Ok(results)
    }

    fn dismiss_notification(&self, id: u64) -> Result<(), ProviderError> {
        init_mta();
        let listener = UserNotificationListener::Current().map_err(map_win_err)?;
        if let Ok(id_u32) = u32::try_from(id) {
            listener.RemoveNotification(id_u32).map_err(map_win_err)?;
        }
        Ok(())
    }

    fn clear_all(&self) -> Result<(), ProviderError> {
        init_mta();
        let listener = UserNotificationListener::Current().map_err(map_win_err)?;
        listener.ClearNotifications().map_err(map_win_err)?;
        Ok(())
    }
}

/// Windows battery & power telemetry provider via `GetSystemPowerStatus`.
#[derive(Clone, Debug, Default)]
pub struct WinSystemProvider;

impl WinSystemProvider {
    pub fn new() -> Self {
        Self
    }
}

impl SystemProvider for WinSystemProvider {
    fn get_battery_status(&self) -> Result<BatteryStatus, ProviderError> {
        let mut status = SYSTEM_POWER_STATUS::default();
        // SAFETY: status is a valid stack-allocated SYSTEM_POWER_STATUS struct.
        let res = unsafe { GetSystemPowerStatus(&mut status) };
        if let Err(err) = res {
            return Err(map_win_err(err));
        }

        // BatteryFlag == 128 indicates no system battery is present; 255 indicates unknown.
        let is_present = status.BatteryFlag != 128 && status.BatteryFlag != 255;
        let is_charging = status.ACLineStatus == 1 || (status.BatteryFlag & 8) != 0;
        let percentage = if status.BatteryLifePercent <= 100 {
            status.BatteryLifePercent
        } else {
            100
        };

        Ok(BatteryStatus {
            percentage,
            is_charging,
            is_present,
        })
    }
}
