use crate::traits::{MediaProvider, NotificationProvider, ProviderError, SystemProvider};
use cay_core::models::{BatteryStatus, MediaInfo, ToastNotification};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionMediaProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::SystemInformation::GetLocalTime;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT, MOD_NOREPEAT, MOD_WIN};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_ACCEPTS_NOTIFICATIONS, QUNS_APP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetDesktopWindow, GetForegroundWindow, GetMessageW, GetShellWindow,
    GetWindowRect, MSG, WM_HOTKEY,
};
use windows::UI::Notifications::Management::{
    UserNotificationListener, UserNotificationListenerAccessStatus,
};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds};

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

        let thumbnail_bytes = read_smtc_thumbnail(&props);

        Ok(Some(MediaInfo {
            title,
            artist,
            album,
            is_playing,
            position_secs,
            duration_secs,
            source_app,
            thumbnail_bytes,
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

                let (sender, message) = extract_toast_text(&item, &app_name);

                results.push(ToastNotification {
                    id,
                    app_name,
                    channel: None,
                    sender,
                    message,
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

fn extract_toast_text(
    item: &windows::UI::Notifications::UserNotification,
    fallback_app: &str,
) -> (String, String) {
    let mut sender = fallback_app.to_string();
    let mut message = String::new();

    if let Ok(notif) = item.Notification() {
        if let Ok(visual) = notif.Visual() {
            let binding = KnownNotificationBindings::ToastGeneric()
                .and_then(|tmpl| visual.GetBinding(&tmpl))
                .or_else(|_| visual.Bindings().and_then(|b| b.GetAt(0)));
            if let Ok(binding) = binding {
                if let Ok(texts) = binding.GetTextElements() {
                    let count = texts.Size().unwrap_or(0);
                    if count >= 1 {
                        if let Ok(first) = texts.GetAt(0).and_then(|t| t.Text()) {
                            let s = first.to_string();
                            if !s.is_empty() {
                                sender = s;
                            }
                        }
                    }
                    let mut body_parts = Vec::new();
                    for idx in 1..count {
                        if let Ok(part) = texts.GetAt(idx).and_then(|t| t.Text()) {
                            let s = part.to_string();
                            if !s.is_empty() {
                                body_parts.push(s);
                            }
                        }
                    }
                    if !body_parts.is_empty() {
                        message = body_parts.join(" · ");
                    }
                }
            }
        }
    }

    if message.is_empty() {
        message = sender.clone();
    }
    (sender, message)
}

/// Opens an `http://` or `https://` meeting URL in the default Windows browser/handler.
pub fn open_external_url(url: &str) -> Result<(), ProviderError> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(ProviderError::Internal(
            "Refusing to open non-HTTP(S) URL".to_string(),
        ));
    }
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map_err(|e| ProviderError::PlatformError(format!("Failed to launch URL: {e}")))?;
    Ok(())
}

/// Configures or removes Cay from `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
pub fn set_windows_autostart(enable: bool) -> Result<(), ProviderError> {
    let run_key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    if enable {
        let exe_path = std::env::current_exe()
            .map_err(|e| ProviderError::Internal(format!("Cannot resolve current_exe: {e}")))?;
        let exe_str = exe_path.to_string_lossy().into_owned();
        let status = std::process::Command::new("reg")
            .args([
                "add", run_key, "/v", "Cay", "/t", "REG_SZ", "/d", &exe_str, "/f",
            ])
            .status()
            .map_err(|e| ProviderError::PlatformError(format!("Failed to run reg.exe: {e}")))?;
        if !status.success() {
            return Err(ProviderError::PlatformError(
                "reg add HKCU Run failed".to_string(),
            ));
        }
    } else {
        let _ = std::process::Command::new("reg")
            .args(["delete", run_key, "/v", "Cay", "/f"])
            .status();
    }
    Ok(())
}

/// Reads up to 2 MB of encoded image bytes from an SMTC session's thumbnail stream.
fn read_smtc_thumbnail(
    props: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<Vec<u8>> {
    let thumb_ref = props.Thumbnail().ok()?;
    let stream = thumb_ref.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()?;
    if size == 0 || size > 2 * 1024 * 1024 {
        return None;
    }
    let len = u32::try_from(size).ok()?;
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    let loaded = reader.LoadAsync(len).ok()?.join().ok()?;
    if loaded == 0 {
        return None;
    }
    let mut buf = vec![0_u8; loaded as usize];
    reader.ReadBytes(&mut buf).ok()?;
    let _ = reader.Close();
    Some(buf)
}

/// Returns the formatted local 12-hour clock string (e.g. `"7:08 PM"`) and today header
/// (e.g. `"TODAY · THU 19 NOV"`).
pub fn current_local_clock_and_date() -> (String, String) {
    // SAFETY: GetLocalTime takes no pointers and returns a value SYSTEMTIME struct.
    let st = unsafe { GetLocalTime() };
    let is_pm = st.wHour >= 12;
    let hour12 = match st.wHour % 12 {
        0 => 12,
        h => h,
    };
    let am_pm = if is_pm { "PM" } else { "AM" };
    let clock = format!("{hour12}:{:02} {am_pm}", st.wMinute);

    let weekday = match st.wDayOfWeek {
        0 => "SUN",
        1 => "MON",
        2 => "TUE",
        3 => "WED",
        4 => "THU",
        5 => "FRI",
        6 => "SAT",
        _ => "DAY",
    };
    let month = match st.wMonth {
        1 => "JAN",
        2 => "FEB",
        3 => "MAR",
        4 => "APR",
        5 => "MAY",
        6 => "JUN",
        7 => "JUL",
        8 => "AUG",
        9 => "SEP",
        10 => "OCT",
        11 => "NOV",
        12 => "DEC",
        _ => "MON",
    };
    let today_header = format!("TODAY · {weekday} {} {month}", st.wDay);
    (clock, today_header)
}

/// Detects whether the current foreground application occupies the entire physical monitor
/// (e.g. fullscreen game, fullscreen video player, or presentation).
pub fn is_foreground_window_fullscreen() -> bool {
    // SAFETY: Win32 window queries on thread-safe HWND handles.
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() || fg == GetDesktopWindow() || fg == GetShellWindow() {
            return false;
        }

        let mut class_buf = [0_u16; 64];
        let class_len = GetClassNameW(fg, &mut class_buf);
        if class_len > 0 {
            let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);
            if matches!(
                class_name.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            ) {
                return false;
            }
        }

        let mut win_rect = RECT::default();
        if GetWindowRect(fg, &mut win_rect).is_err() {
            return false;
        }

        let hmon = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        if hmon.0.is_null() {
            return false;
        }

        let mut mon_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmon, &mut mon_info).as_bool() {
            return false;
        }

        let m = mon_info.rcMonitor;
        win_rect.left <= m.left
            && win_rect.top <= m.top
            && win_rect.right >= m.right
            && win_rect.bottom >= m.bottom
    }
}

/// Queries the current Windows master audio output volume percentage (0..=100) via Core Audio
/// (`IMMDeviceEnumerator` + `IAudioEndpointVolume`).
pub fn get_system_master_volume_percent() -> Option<u8> {
    init_mta();
    // SAFETY: Standard COM activation on an MTA thread; pointers are managed by `windows` smart pointers.
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let endpoint_vol: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).ok()?;
        let scalar = endpoint_vol.GetMasterVolumeLevelScalar().ok()?;
        Some((scalar.clamp(0.0, 1.0) * 100.0).round() as u8)
    }
}

/// Queries Windows Shell (`SHQueryUserNotificationState`) to determine whether Focus Assist,
/// Quiet Hours, Presentation Mode, or D3D Fullscreen mode is currently suppressing notifications.
pub fn is_windows_quiet_hours_active() -> bool {
    // SAFETY: SHQueryUserNotificationState takes a valid pointer to a stack-allocated enum.
    unsafe {
        let Ok(state) = SHQueryUserNotificationState() else {
            return false;
        };
        state != QUNS_ACCEPTS_NOTIFICATIONS && state != QUNS_APP
    }
}

/// Spawns a dedicated background message thread that registers the global shortcut `Win + Alt + I`
/// and invokes `on_hotkey` whenever the user presses it.
pub fn spawn_global_hotkey_listener(on_hotkey: impl Fn() + Send + 'static) {
    let _ = std::thread::Builder::new()
        .name("cay-hotkey-listener".into())
        .spawn(move || {
            // SAFETY: RegisterHotKey and GetMessageW operate on this thread's win32 message queue.
            unsafe {
                let mods = MOD_WIN | MOD_ALT | MOD_NOREPEAT;
                if RegisterHotKey(None, 1, mods, u32::from(b'I')).is_err() {
                    return;
                }
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                    if msg.message == WM_HOTKEY {
                        on_hotkey();
                    }
                }
            }
        });
}
