pub mod stubs;
pub mod traits;
#[cfg(windows)]
pub mod windows_impl;

pub use stubs::{
    StubCalendarProvider, StubMediaProvider, StubNotificationProvider, StubSystemProvider,
};
pub use traits::{
    CalendarProvider, MediaProvider, NotificationProvider, ProviderError, SystemProvider,
};
#[cfg(windows)]
pub use windows_impl::{
    init_mta, process_working_set_bytes, WinNotificationProvider, WinSmtcMediaProvider,
    WinSystemProvider,
};
