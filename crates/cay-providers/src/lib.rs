pub mod calendar_ics;
pub mod stubs;
pub mod traits;
#[cfg(windows)]
pub mod windows_impl;

pub use calendar_ics::IcsCalendarProvider;
pub use stubs::{
    StubCalendarProvider, StubMediaProvider, StubNotificationProvider, StubSystemProvider,
};
pub use traits::{
    CalendarProvider, MediaProvider, NotificationProvider, ProviderError, SystemProvider,
};
#[cfg(windows)]
pub use windows_impl::{
    current_local_clock_and_date, get_system_master_volume_percent, init_mta,
    is_foreground_window_fullscreen, is_windows_quiet_hours_active, open_external_url,
    process_working_set_bytes, set_windows_autostart, spawn_global_hotkey_listener,
    WinNotificationProvider, WinSmtcMediaProvider, WinSystemProvider,
};
