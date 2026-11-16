# Prior Art & Technical Research: Windows Island Implementations

## 1. Comparative Analysis of Existing Projects

The concept of a desktop floating island on Windows has been explored across various languages, runtimes, and windowing toolkits. The table below summarizes notable open-source implementations to date.

> [!CAUTION]
> **License Compliance**: Check licenses carefully before adapting architectural ideas. For instance, `eIsland` is licensed under GPL-3.0; incorporating its source would infect the project with GPL. `Lumen`, `Ripple`, and others are MIT-licensed.

| Project | Stack | License | Key Insights & Architectural Relevance |
|---|---|---|---|
| [FlorianButz/DynamicWin](https://github.com/FlorianButz/DynamicWin) | C# + SkiaSharp | Proprietary / Releases only | Pioneer implementation on Windows; custom SkiaSharp rendering layer. Repo currently hosts compiled binaries only. |
| [w0otdev/V-Notch](https://github.com/w0otdev/V-Notch) | WPF, .NET 8, CommunityToolkit.Mvvm, WebView2, C++ helper | MIT | Integrates local AI summarization via LLamaSharp; relies on C++ helper for native hook interop. |
| [luolangaga/WinLand](https://github.com/luolangaga/WinLand) | WinUI 3, .NET 10, Windows App SDK | MIT | Modular architecture with sample plugin architecture; uses official WinUI modern controls. |
| [Glance](https://github.com/Glance-App/Glance) | WPF / C# | GPL-3.0 | Comprehensive feature set: notch layout, media playback, temporary file shelf, calendar, battery status. |
| [HeyyCzer/windows-dynamic-island](https://github.com/HeyyCzer/windows-dynamic-island) | Tauri 2 + React + Tailwind + Motion; Rust `windows` crate | MIT | **Closest to Cay's scope**. Clean separation between Rust backend (SMTC, toasts, calendar) and React frontend. Uses `rrule` crate and Google OAuth PKCE. |
| [JNTMTMTM/eIsland](https://github.com/JNTMTMTM/eIsland) | Tauri 2 (migrated from Electron) + React + GSAP | GPL-3.0 | Modular architecture: separate helper crate/package per Windows feature (SMTC, toast listener, volume, Wi-Fi). |
| [Risuleia/Lumen](https://github.com/Risuleia/Lumen) | Rust + Slint (Skia backend) | MIT | Extremely lightweight native Rust application; features real-time audio visualization using `rustfft`. |
| [TopMyster/Ripple](https://github.com/TopMyster/Ripple) | Electron + React 19 + Framer Motion | MIT | Polished cross-platform prototype exploring fluid gesture and spring animations. |
| [Taxperia/windows-notch-overlay](https://github.com/Taxperia/windows-notch-overlay) | Electron + WinRT helpers | MIT | Minimal footprint demonstrating SMTC session parsing and native toast notifications reading. |
| [devcode90/Dynamic-Island-for-Windows](https://github.com/devcode90/Dynamic-Island-for-Windows) | C++ (Windhawk mod), Direct2D + DWM | MIT | Mod injected directly into `dwm.exe` / explorer process; ultra-low overhead, but requires Windhawk engine and elevated permissions. |
| [rajsriv/dynamic-island-for-windows](https://github.com/rajsriv/dynamic-island-for-windows) | Python + PyQt6 + winsdk | MIT | Educational reference for rapid prototyping with Python WinSDK bindings. |

---

## 2. Low-Level Windows Subsystem Mechanisms

Across all analyzed implementations, four core OS interaction layers are universally required:

### A. Window Presentation & Input Transparency
- **Extended Window Styles**:
  - `WS_EX_TOPMOST` ($0x00000008$): Pins the window above all normal desktop windows.
  - `WS_EX_TOOLWINDOW` ($0x00000080$): Prevents an icon from appearing in the Windows Taskbar and Alt+Tab switcher.
  - `WS_EX_NOACTIVATE` ($0x08000000$): Prevents the window from stealing foreground focus when clicked or updated.
- **Hit-Testing**:
  - Transparent regions outside the pill must pass mouse events to the application underneath.
  - Handled either by dynamic window region clipping (`SetWindowRgn`), processing `WM_NCHITTEST` and returning `HTTRANSPARENT` for background coordinates and `HTCLIENT` for island coordinates, or relying on Rune's selective overlay input routing.

### B. Media Playback & Controls (SMTC)
- **API**: `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`
- **Capabilities**:
  - Queries `GetCurrentSession()`.
  - Subscribes to `CurrentSessionChanged` and `MediaPropertiesChanged`.
  - Extracts track title, artist, album art stream (`GetThumbnailAsync`), and playback status (`Playing`, `Paused`, `Stopped`).
  - Sends playback commands (`TryPlayAsync`, `TryPauseAsync`, `TrySkipNextAsync`, `TrySkipPreviousAsync`).
- **Reliability Note**: Some third-party browsers only update timeline positions intermittently; polling session state every 500 ms while active is commonly needed.

### C. System Notifications
- **API**: `Windows.UI.Notifications.Management.UserNotificationListener`
- **Permissions**: Requires user permission prompt via `RequestAccessAsync()`.
- **Known Gotcha**: Prior art (notably HeyyCzer) observed that `NotificationChanged` events are occasionally dropped or silently missed by the WinRT subsystem in unpackaged desktop applications. The recommended approach is to poll `GetNotificationsAsync(NotificationKinds::Toast)` on an MTA background thread at a steady interval (~500–1000 ms) while active, comparing notification IDs.
- **Packaging Identity Spike**: Unpackaged Win32 executables calling `UserNotificationListener` sometimes require sparse package identity (`AppxManifest.xml` with `userNotificationListener` capability). Verifying this is a primary objective of Phase 0.

### D. Calendar Synchronisation
- **API Realities**: Windows does not expose a unified, zero-auth local calendar WinRT API for desktop applications.
- **Proven Approaches**:
  1. **Google Calendar API**: OAuth 2.0 PKCE flow with a loopback redirect (`http://127.0.0.1:port`).
  2. **Microsoft Graph API**: OAuth 2.0 flow for Outlook / Microsoft 365.
  3. **Webcal / `.ics` URL**: Fetching and parsing standard iCalendar RFC 5545 format using a Rust parser (e.g. `icalendar` crate). Requires no developer credentials.
