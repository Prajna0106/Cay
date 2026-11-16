# Product Requirements Document (PRD): Cay

## 1. Summary & Problem Statement

Windows users constantly switch contexts to check basic, ongoing tasks: switching windows to see what song is playing, checking upcoming calendar appointments, or reviewing missed notifications. Unlike macOS or mobile operating systems, Windows lacks a glanceable, persistent, non-intrusive surface at the top of the screen that aggregates active tasks without stealing keyboard focus or cluttering the taskbar.

**Cay** is a floating island for Windows 10/11: a compact, solid-black pill anchored top-centre on the desktop. It morphs dynamically to present notifications, media playback, meeting alerts, and HUD indicators, shrinking back to an unobtrusive clock when idle.

---

## 2. Goals & Non-Goals

### Goals
- Provide an unobtrusive, hardware-accelerated floating island pinned to the top centre of the primary screen.
- Deliver fluid, 60 fps spring animations for all state transitions.
- Maintain zero focus-stealing: clicks reach the island when intended, while clicks outside pass straight through to background applications.
- Keep resource consumption minimal (target <30 MB RAM idle, ~0% CPU idle).
- Respect user privacy: all notification and media data stay strictly local on-device.

### Non-Goals (v1)
- Cross-platform support (no macOS or Linux support in v1).
- Extensible third-party plugin marketplace.
- AI summaries or LLM integrations.
- Cloud account syncing or external analytics/telemetry servers.

---

## 3. Target Users

- **Developers & Power Users**: Working across multi-window setups needing quick access to build alerts, music, or time without alt-tabbing.
- **Meeting-Heavy Professionals**: Needing frictionless reminders and one-click "Join" actions for Microsoft Teams or Google Meet calls.
- **Students & Remote Workers**: Who benefit from a continuous focus/timer indicator and lightweight media controls.
- **Music & Podcast Listeners**: Requiring instant media title inspection, playback control, and scrubbing without launching the full media app window.

---

## 4. Feature List & Priorities

| Priority | Feature | Description |
|---|---|---|
| **P0 (MVP)** | Idle Pill | 200 × 40 pill displaying current system time and battery percentage. |
| **P0 (MVP)** | Now Playing (SMTC) | Displays album art, track title, artist, live audio waveform, and playback controls. |
| **P0 (MVP)** | Notification Peek & Hub | Instant 5 s toast peek with sender/app info; stores recent notifications in Hub. |
| **P0 (MVP)** | Upcoming Event & Join | Meeting card appears 15 min prior to calendar events with direct "Join" button. |
| **P0 (MVP)** | Expanded Hub | 820 × 330 3-card grid (Now Playing, Today's Calendar, Recent Notifications) on click. |
| **P0 (MVP)** | System Tray & Settings | Tray icon with toggle/settings/quit menu, accent picker, position offsets, module toggles. |
| **P0 (MVP)** | Windows Autostart | Option to launch minimized on Windows startup. |
| **P0 (MVP)** | Fullscreen App Detection | Automatically conceals the island when exclusive fullscreen games or presentations run. |
| **P1** | Timer & Focus Mode | Dedicated 280 × 46 pill with countdown timer and focus state indicator. |
| **P1** | Volume / Brightness HUD | 300 × 46 pill replacing the native Windows flyout during keyboard volume/brightness adjustments. |
| **P1** | Multi-Monitor Targeting | Choose which display hosts the island (primary, active window monitor, or pointer monitor). |
| **P1** | Notification Filter | Per-app allow/deny list to suppress unwanted toasts. |
| **P1** | Windows Focus / DND Sync | Respect Windows "Do Not Disturb" / Focus assist state automatically. |
| **P2** | Synced Lyrics | Real-time scrolling lyrics line inside media pill or Hub. |
| **P2** | Clipboard / File Shelf | Drag-and-drop temporary holding zone for files and copied text snippets. |
| **P2** | Weather Widget | Compact weather condition icon and temperature in idle pill. |
| **P2** | Plugin API | Safe sandboxed plugin interface for community-built modules. |

---

## 5. User Stories & Acceptance Criteria (P0 Features)

### US-1: Idle Clock & Battery
- **Story**: As a user, I want the island to show the time and battery status when nothing is playing so the screen header feels useful and calm.
- **Acceptance Criteria**:
  - Displays formatted 12h/24h time according to Windows locale.
  - Shows battery icon and charge percentage; hides battery indicator on desktops without battery.
  - Idle footprint is exactly 200 × 40 px, r20 px.

### US-2: Now Playing Media
- **Story**: As a music listener, I want to see the currently playing track and controls without leaving my current application.
- **Acceptance Criteria**:
  - Automatically hooks into `GlobalSystemMediaTransportControlsSessionManager` (SMTC).
  - Shows 30 px album art, track title, artist, and 5-bar animated waveform.
  - Waveform animates during playback and freezes when paused.
  - In Expanded Hub, provides play/pause, previous, next, and progress bar scrub.

### US-3: Notification Peek
- **Story**: As a user, I want notification banners to appear smoothly at the island and dismiss automatically so I am informed without distraction.
- **Acceptance Criteria**:
  - Listens to incoming Windows toasts via `UserNotificationListener`.
  - Animates island to 440 × 78 px (r30 px) with app icon, app name, sender, and preview text.
  - Stays visible for 5.0 seconds before smoothly folding back to previous state.
  - Unread notifications remain accessible in the Expanded Hub.

### US-4: Upcoming Calendar Event
- **Story**: As a professional with scheduled calls, I want an alert 15 minutes before my meeting with a direct join link.
- **Acceptance Criteria**:
  - Island morphs to 400 × 62 px showing date tile (e.g. "NOV 15"), meeting title, countdown, and "Join" button.
  - Clicking "Join" opens the conference URL (Teams, Meet, Zoom) in the default browser/app.

### US-5: Expanded Hub
- **Story**: As a user, I want to click the island to view my dashboard of media, daily schedule, and recent alerts.
- **Acceptance Criteria**:
  - Clicking the pill expands to 820 × 330 px (r40 px) with 3 equal cards (Now Playing, Today's Events, Notifications).
  - Clicking outside the Hub or pressing `Esc` closes the Hub back to the active pill state.
  - No keyboard focus stolen upon opening.

### US-6: Fullscreen Concealment
- **Story**: As a gamer or presenter, I do not want the island obscuring fullscreen games or slideshows.
- **Acceptance Criteria**:
  - Queries foreground window state; if an application occupies the entire display, hide the island.
  - Unhides automatically when the user returns to windowed or desktop view.

---

## 6. Behaviour & State Arbitration Rules

### State Priority Hierarchy
When multiple events occur simultaneously, the island resolves to the highest priority active state:

$$\text{HUD (Interrupt)} > \text{Notification Peek} > \text{Upcoming Event (within 15m)} > \text{Timer/Focus} > \text{Now Playing} > \text{Idle}$$

1. **HUD Interrupt**: Volume or brightness key presses trigger a 2.0 s interrupt overlay, then yield back to whatever state was active.
2. **Notification Queueing**: If a notification arrives while another is showing, it replaces the current notification and resets the 5.0 s timer. Multiple unread notifications append to the Hub list.
3. **No Active Media**: When playback stops or the media player closes, the island transitions gracefully to Idle after a 3.0 s debounce.
4. **Hub Expansion**: Hub can be toggled by clicking the pill. When the Hub is open, new notifications append to the Notifications card without closing the Hub.

---

## 7. Calendar Sources

Calendar integration options under consideration:
- **Google Calendar**: OAuth 2.0 with PKCE for secure personal calendar retrieval.
- **Microsoft 365 / Outlook**: Microsoft Graph API for enterprise and school accounts.
- **Local / Webcal (.ics)**: Direct HTTP URL polling of `.ics` calendar feeds (requires no API credentials).

*Open Question*: Which calendar source should ship first in MVP? (Recommendation: Webcal `.ics` URL for zero-auth simplicity, followed by Google OAuth PKCE).

---

## 8. Non-Functional Targets

*(All quantitative figures are targets to confirm during Phase 0 spike)*

| Metric | Target (to confirm in Phase 0) | Notes |
|---|---|---|
| **Idle RAM** | < 30 MB | Native Rust runtime, minimal heap allocations |
| **Idle CPU** | < 0.2% | Event-driven redraws; no tight polling loops |
| **Animation Framerate** | 60 fps (120 fps on high refresh) | GPU-driven spring animations via Rune |
| **Cold Start Time** | < 400 ms | Instant startup from tray or shell autostart |
| **Binary Size** | < 15 MB | Single executable without heavy runtime dependencies |

---

## 9. Privacy, Security & Permissions

- **Local-Only Processing**: Notification content, calendar events, and media metadata never leave the device.
- **Zero Default Telemetry**: No tracking, phone-home requests, or third-party SDKs.
- **Credential Storage**: Calendar tokens or secrets are stored strictly in the Windows Credential Manager (`wincred`).
- **Permissions**: Requires user consent via Windows Settings for `UserNotificationListener`.

---

## 10. Accessibility & Ergonomics

- **UI Automation (UIA)**: Island elements declare appropriate UIA roles (`Button`, `Text`, `ProgressBar`, `Group`) and accessible names for NVDA / JAWS / Windows Narrator.
- **Reduced Motion**: Respects Windows system setting (`SPI_GETCLIENTAREAANIMATION` / `Runtime::system_reduced_motion()`). Overshoots and scales are disabled; simple cross-fades used instead.
- **Contrast**: Text and accent tokens meet WCAG 2.1 AA requirements (>= 4.5:1 against dark backgrounds).
- **Keyboard Access**: Global hotkey (default: `Win + Alt + I`) opens/closes the Expanded Hub.

---

## 11. Distribution & Packaging

- **Packaging Strategy**: Standard MSIX or lightweight WiX / Inno Setup installer. Depends on whether `UserNotificationListener` requires sparse package identity on Windows 10/11.
- **Auto-Update**: Background update checker querying GitHub Releases (verified via SHA-256).
- **Code Signing**: Sign executable and installer with Windows Authenticode certificate to prevent SmartScreen prompts.

---

## 12. Milestones, Risks & Success Metrics

### Milestones
- **Phase 0 (Spike)**: Rune overlay window on Windows 11 + live SMTC track + spring morph + verify click-through & focus behavior.
- **Phase 1 (P0 MVP)**: Core state machine, live SMTC controls, UserNotificationListener, calendar (.ics/Google), Hub view, tray icon.
- **Phase 2 (P1 Enhancements)**: Volume/brightness HUD, timer/focus, notification filters, DND sync.
- **Phase 3 (P2 Polish)**: Lyrics, shelf, themes, plugin API.

### Critical Risks & Mitigation
1. **Rune Pre-Release Stability**: Pinned to specific git commit. Fallback: switch UI layer to Tauri 2 + React + Motion. Clean seam maintained via `cay-core` and `cay-providers`.
2. **WinRT UserNotificationListener in Unpackaged App**: If Windows requires package identity, package as sparse MSIX or full MSIX.
3. **SMTC Thumbnail Extraction**: WinRT stream decoding in Rust; mitigate by extracting thumbnail buffers asynchronously into Rune textures.

### Success Metrics
- 0 window focus stealing events during normal operation.
- Consistent 60 fps during all pill morph transitions.
- Battery impact indistinguishable from baseline idle desktop.
