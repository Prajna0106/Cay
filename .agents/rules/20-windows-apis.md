# Rule 20: Windows APIs & Subsystem Interoperability

## WinRT Integration via `windows` Crate
- Use the official Microsoft `windows` crate (`features = ["Media_Control", "UI_Notifications_Management", "System_Power", ...]` etc.).
- Avoid legacy raw Win32 bindings where safe WinRT projections exist.

## COM Apartments & Concurrency Rules
1. **Thread Apartment Initialization**:
   - WinRT background threads must initialize COM via `windows::core::RoInitialize(RO_INIT_MULTITHREADED)` or `CoInitializeEx(None, COINIT_MULTITHREADED)`.
   - Never initialize COM single-threaded apartments (STA) on background worker pools unless a specific legacy control mandates it.
2. **Never Block the Main UI Thread**:
   - WinRT async calls (`IAsyncOperation`, `IAsyncAction`) must be awaited on dedicated background worker threads or Tokio runtimes, never on the Rune UI thread.
   - Synchronous blocking waits (e.g. `.get()?`) on the UI thread will cause visual stuttering and freeze spring animations.
3. **Event Notification Gotchas**:
   - `UserNotificationListener`: Do not rely exclusively on the `NotificationChanged` event, as Windows WinRT notifications can silently drop events in unpackaged desktop environments. Implement background polling of `GetNotificationsAsync(NotificationKinds::Toast)` at ~500–1000 ms intervals.
   - `GlobalSystemMediaTransportControlsSessionManager`: Listen to `CurrentSessionChanged` and `MediaPropertiesChanged`, but verify playback info with periodic sanity checks while active.
4. **Thumbnail Stream Handling**:
   - WinRT thumbnails (`IRandomAccessStreamWithContentType`) must be asynchronously read into memory bytes on worker threads, decoded, and converted to Rune image textures before being handed to the UI.
