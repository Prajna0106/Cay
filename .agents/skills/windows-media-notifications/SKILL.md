---
name: windows-media-notifications
description: >-
  Guides integration with Windows WinRT APIs for media controls (SMTC) and system toast
  notifications (UserNotificationListener). Use when implementing or debugging providers in cay-providers.
---

# Skill: Windows Media & Notifications Integration

This skill covers robust integration with Windows Runtime (WinRT) APIs for media transport controls and notification capture using the Microsoft `windows` crate.

## 1. System Media Transport Controls (SMTC)

Windows exposes media playback across Spotify, browsers (Chrome/Edge), iTunes, and desktop players via:
`Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`

### Implementation Pattern
```rust
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
};

pub async fn setup_smtc_listener() -> windows::core::Result<()> {
    // Request manager instance
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;

    // Initial check
    if let Ok(session) = manager.GetCurrentSession() {
        extract_session_info(&session).await?;
    }

    // Subscribe to session changes
    manager.CurrentSessionChanged(&windows::Foundation::TypedEventHandler::new(
        move |mgr, _| {
            if let Some(mgr) = mgr {
                if let Ok(session) = mgr.GetCurrentSession() {
                    // Spawn task to read updated session info
                }
            }
            Ok(())
        },
    ))?;

    Ok(())
}
```

### SMTC Gotchas
- **COM Apartment**: Always run on a multi-threaded COM apartment (`MTA`).
- **Thumbnail Buffers**: `GetThumbnailAsync()` returns an `IRandomAccessStreamWithContentType`. Read bytes using `DataReader` on a background thread.
- **Playback Polling**: Some browsers only trigger events on initial play/pause and fail to update timeline position ticks. Run a slow background poll (~1000 ms) while playback is active to keep track scrubbing accurate.

---

## 2. Windows Notifications (`UserNotificationListener`)

Notification access is governed by:
`Windows.UI.Notifications.Management.UserNotificationListener`

### Access Authorization
```rust
use windows::UI::Notifications::Management::{
    UserNotificationListener, UserNotificationListenerAccessStatus,
};

pub async fn request_notification_access() -> windows::core::Result<bool> {
    let listener = UserNotificationListener::Current()?;
    let status = listener.RequestAccessAsync()?.await?;
    Ok(status == UserNotificationListenerAccessStatus::Allowed)
}
```

### Critical Notification Gotchas
- **Packaging Identity Requirement**: On certain Windows 10/11 revisions, calling `RequestAccessAsync()` from an unpackaged Win32 `.exe` returns `DeniedBySystem`. If this occurs during the Phase 0 spike, the app must declare a sparse package identity with the `userNotificationListener` restricted capability in `AppxManifest.xml`.
- **Do Not Trust `NotificationChanged` Alone**: The WinRT event handler for toasts is notoriously unreliable in desktop processes and often drops events.
  - **Best Practice**: Spawn a background worker thread that queries `GetNotificationsAsync(NotificationKinds::Toast)` at a regular interval (500–1000 ms), diffing against known notification IDs.
- **Privacy & Cleanup**: Toasts may contain sensitive one-time passwords (OTPs) or private chat messages. Cay stores notifications strictly in transient RAM within `cay-core` and clears them on process termination.
