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
    init_mta, open_external_url, process_working_set_bytes, set_windows_autostart,
    WinNotificationProvider, WinSmtcMediaProvider, WinSystemProvider,
};
