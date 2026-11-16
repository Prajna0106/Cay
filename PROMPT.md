# Prompt for Claude Code: set up the "Cay" floating island project

> **Before you paste this:** edit the two values in "Settings" below.
> If you also downloaded `design-source/` (Main.dc.html, States.dc.html), put that folder next to where you run Claude Code, or give its path in DESIGN_SOURCE.

---

## Settings

- TARGET_FOLDER: `D:\Projects\cay`   ← change to the folder you want
- PRODUCT_NAME: `Cay`                 ← working name; use it everywhere, so a rename is a find-and-replace
- DESIGN_SOURCE: `./design-source`    ← folder holding Main.dc.html and States.dc.html (optional)

---

## Your task

You are setting up a new project in TARGET_FOLDER for **PRODUCT_NAME**, a floating "island" for Windows 10/11: a black pill at the top centre of the screen that shows notifications, music, calendar events and more, and grows or shrinks to fit what is happening (in the spirit of Apple's Dynamic Island, but never use that name in the product).

The project will be developed agentically in **Google Antigravity** (and also Claude Code). Your job in this session is to produce the **product, design and agent-context files plus a project skeleton**, not to build the features. Everything you need is below. Do not invent facts beyond it; where something is unknown, write it down as an open question.

Work in this order and keep me posted after each step:

1. Create TARGET_FOLDER (if it doesn't exist) and `git init` it.
2. Write the docs (`docs/`), the design spec (`design/`), and the agent files (`AGENTS.md`, `CLAUDE.md`, `.agents/`).
3. Create the Cargo workspace skeleton (see "Skeleton" below). Code should compile on Windows but implement nothing beyond stubs.
4. Copy DESIGN_SOURCE files into `design/reference/` if they exist.
5. Commit everything as `chore: project bootstrap (PRD, design, agent context)`.
6. Finish with a short report: the file tree, anything you could not verify, and the open questions I need to answer.

Before writing the Antigravity files, check the current docs at https://antigravity.google/docs/rules (and the workflows and skills pages) and follow their exact folder names and frontmatter. As of Nov 2026: workspace rules go in `.agents/rules/*.md`, workflows in `.agents/workflows/*.md`, skills in `.agents/skills/<name>/SKILL.md`, and Antigravity reads `AGENTS.md` at the workspace root. `GEMINI.md` would override `AGENTS.md`, so **do not create a GEMINI.md**; keep `AGENTS.md` the single source of truth.

---

## Context 1: research on existing Windows island projects

These open-source projects already do this. Use them as reference only; check each license before copying any code (eIsland is GPL-3.0, so copying it would make our code GPL too; Lumen and Ripple are MIT).

| Project | Stack | Notes |
|---|---|---|
| FlorianButz/DynamicWin | C# + SkiaSharp | Most popular; repo now hosts releases only, no source |
| w0otdev/V-Notch | WPF, .NET 8, CommunityToolkit.Mvvm, WebView2, C++ helper | Local AI via LLamaSharp |
| luolangaga/WinLand | WinUI 3, .NET 10, Windows App SDK | Plugin system, plugin samples |
| Glance | WPF | Notch, media, shelf, calendar, battery |
| HeyyCzer/windows-dynamic-island | Tauri 2 + React + Tailwind + Motion; Rust `windows` crate | Music, notifications, calendar (Google OAuth PKCE + .ics, `rrule` crate). Closest to our scope |
| JNTMTMTM/eIsland | Tauri 2 (was Electron) + React + GSAP | One helper package per Windows feature (SMTC, toast listener, volume, Wi-Fi…) |
| Risuleia/Lumen | Rust + Slint (Skia renderer) | Small and fast; rustfft audio visualiser |
| TopMyster/Ripple | Electron + React 19 + Framer Motion | Cross-platform |
| Taxperia/windows-notch-overlay | Electron + WinRT helpers | SMTC helper + toast reader |
| devcode90/Dynamic-Island-for-Windows | Single C++ Windhawk mod, Direct2D + DWM | Media, lyrics, calendar grid, weather, HW monitor, cam/mic dots |
| rajsriv/dynamic-island-for-windows | Python + PyQt6 + winsdk | Easiest prototype |

How they all work underneath:
- **Window:** borderless, transparent, always-on-top, not in the taskbar, never steals focus; click-through outside the pill.
- **Music:** `GlobalSystemMediaTransportControlsSessionManager` (SMTC) — title, artist, album art, playback state, play/pause/skip for any app that reports to Windows (Spotify, browsers, etc.).
- **Notifications:** `Windows.UI.Notifications.Management.UserNotificationListener` — the user must grant access once. HeyyCzer polls `GetNotificationsAsync(NotificationKinds::Toast)` instead of relying on the `NotificationChanged` event, which they found unreliable.
- **Calendar:** no simple Windows API; projects use Google OAuth, Microsoft Graph, or an `.ics` URL.

## Context 2: technology decision — Rune (with Tauri 2 fallback)

UI framework: **Rune** — https://github.com/anomalyco/rune — a native GPU-rendered Rust UI framework (own renderer, text, layout and spring animations), MIT licensed.

Why it fits:
- Edge overlays (`App::overlay` / `OverlaySpec`): on Windows a topmost tool window with `WS_EX_NOACTIVATE`, click-through unless `interactive`, resizable at runtime (`OverlayHandle::resize`), pinned to the top edge and centred.
- Spring animations (`motion::SNAPPY` etc.) for the pill morphing.
- Headless mode: render any state to PNG and dump the element tree — use this for automated visual checks of every island state.
- UI Automation accessibility on Windows.

Risks (record in the ADR and PRD):
- Rune is **pre-release**; pin a specific revision of its `dev` branch in Cargo.toml (look up the current dev HEAD commit and pin it; never track a moving branch).
- Its Windows overlay path is only compile-checked by its authors (`cargo xwin clippy`), not exercised on a real desktop.
- System-material blur works on macOS only; on Windows it draws the fill (fine: our pill is solid black).
- The project accepts issues, not pull requests.
- Rune doesn't read media or notifications: use Microsoft's `windows` crate for SMTC and UserNotificationListener. `rune-notify` only *posts* toasts.

Plan: **Phase 0 is a 1–2 day spike** — Rune overlay pill + live SMTC track info + one spring resize, run on a real Windows 11 PC. Exit criteria: transparent, topmost, no focus steal, click-through outside the pill, clicks work on the pill, 60 fps morph, works over a maximised window. If it fails, switch the UI layer to **Tauri 2 + React + Motion**; the providers crate (SMTC, notifications, calendar) is UI-agnostic and carries over unchanged. Architecture must keep that seam clean.

Also verify in Phase 0 (open question): whether `UserNotificationListener` needs package identity (MSIX or sparse package with the `userNotificationListener` capability) for an unpackaged Rust exe. This decides the packaging plan.

## Context 3: the design (source of truth for `docs/DESIGN.md` and `design/tokens.json`)

A design canvas was made in Claude Design with two artboards: an interactive prototype and an "All island states" sheet. If DESIGN_SOURCE exists, those are its source files (`.dc.html`, readable HTML with `{{}}` holes); otherwise use this spec.

**Look:** a solid black pill on the desktop, top centre, 14 px from the top edge. Inter for UI text, Space Grotesk only for marketing/headings. One accent colour.

**Tokens**
- island background `#0B0B0F`, island border `#26262E` (1 px), shadow `0 18px 40px rgba(11,11,15,0.28)`
- card surface (inside hub) `#17171D`, card radius 22
- track/progress background `#2E2E38`
- text primary `#F5F5F7`, text secondary `#D4D4D8`, text muted `#A1A1AA`
- accent (default) `#7CC4FF`; alternatives offered as a user setting: `#A3E635`, `#FFB86B`, `#F0ABFC`
- warm/timer `#FFB86B`
- type: 15/600 titles in cards, 14/600 primary lines, 13 body, 12 meta, 11/600 caps labels with 0.8 px tracking; tabular numbers for times/timers
- touch targets ≥ 44 px; icons are 1.8–2 px stroke line icons, never emoji

**States (width × height, corner radius)**
1. **Idle** — 200 × 40, r20. Clock left ("7:08 PM"), battery icon + % right.
2. **Now playing** — 360 × 46, r23. 30 px album-art tile (r9), "Title · Artist" (artist muted), 5-bar animated waveform in accent on the right; bars freeze when paused.
3. **Notification** — 440 × 78, r30. 44 px app icon tile (r14), line 1 "App · channel" + "now" (muted 12), line 2 sender (14/600), line 3 message (13, one line, ellipsis). Shows ~5 s then folds back.
4. **Upcoming event** — 400 × 62, r31. Date tile (month in accent caps + day), event title, "in 12 min · Microsoft Teams", accent "Join" pill button. Appears 15 min before a meeting.
5. **Timer / focus** — 280 × 46, r23. Timer icon + "Focus" left, countdown right in `#FFB86B`.
6. **Volume / brightness HUD** — 300 × 46, r23. Speaker icon, level bar (white on `#2E2E38`), number. Replaces the Windows flyout.
7. **Expanded hub** — 820 × 330, r40, padding 18, three equal cards with 12 gap:
   - Now playing: source label, 64 px art, title/artist, progress bar with times, prev / play-pause (44 px white circle) / next.
   - Today: date label, up to 3 events each with a 3 px colour bar (accent, warm, grey), "Open calendar" link.
   - Notifications: header with "Clear", 3 recent items (32 px icon tile, sender, one-line message).

**Motion:** width, height and radius animate together with a slight overshoot spring (prototype used `cubic-bezier(.34,1.4,.5,1)` over 450 ms; in Rune use an equivalent spring). Content cross-fades with a small scale-in (0.96 → 1, ~350 ms). Respect "reduce motion": no overshoot, fades only.

**Interaction:** click the pill to open or close the hub; hover-to-expand is a setting (open question). Priority when several things happen at once: notification > upcoming event > timer > now playing > idle; HUD interrupts briefly over anything.

## Context 4: what the PRD should contain (`docs/PRD.md`)

Write a real PRD, concise, with these sections:
1. Summary and problem (Windows has no glanceable, always-visible place for media, meetings and notifications).
2. Goals, non-goals (non-goals for v1: macOS/Linux, plugin marketplace, AI features, cloud account).
3. Target users (developers, students, people in back-to-back meetings, music listeners).
4. Feature list with priorities:
   - **P0 (MVP):** idle pill; now playing (SMTC) with controls; notification peek + hub list; upcoming event + Join; expanded hub; tray icon with settings/quit; start with Windows; hide when a fullscreen app/game runs; settings (accent, position offset, which modules are on).
   - **P1:** timer/focus; volume/brightness HUD; multi-monitor choice; notification app allow/deny list; Do Not Disturb sync with Windows Focus.
   - **P2:** synced lyrics; file/clipboard shelf; weather; plugin API.
5. User stories and acceptance criteria per P0 feature.
6. Behaviour rules (state priority, durations, queueing of multiple notifications, what happens with no media).
7. Calendar sources: Google (OAuth PKCE), Microsoft 365 / Outlook (Microsoft Graph), `.ics` URL. Which ship in MVP is an open question.
8. Non-functional targets (to confirm in Phase 0): idle RAM, idle CPU near 0%, 60 fps animations, cold start time, binary size.
9. Privacy and permissions: notification content stays on the device, no telemetry by default, tokens stored in Windows Credential Manager.
10. Accessibility: UI Automation names/roles, keyboard shortcut to open hub, reduce-motion, contrast ≥ 4.5:1.
11. Distribution: installer type and packaging (depends on the package-identity question), auto-update, code signing.
12. Milestones (Phase 0 spike → MVP → P1 → P2), risks, open questions, success metrics.

## Files to create

```
TARGET_FOLDER/
├── README.md                    # what it is, status, how to build, links to docs
├── AGENTS.md                    # single source of truth for all coding agents (see below)
├── CLAUDE.md                    # one line: "@AGENTS.md" so Claude Code loads the same rules
├── .gitignore                   # Rust + Windows + editor files
├── docs/
│   ├── PRD.md
│   ├── DESIGN.md                # Context 3, written as a spec with a table of states
│   ├── ARCHITECTURE.md          # crates, threads, data flow provider → state → view, the UI seam
│   ├── RESEARCH.md              # Context 1 table + how-they-work notes, with links
│   ├── ROADMAP.md               # phases with exit criteria
│   └── adr/0001-ui-framework-rune.md   # decision, alternatives (Tauri 2, WinUI 3, Slint), risks, fallback trigger
├── design/
│   ├── tokens.json              # colours, radii, type scale, state sizes, motion
│   └── reference/               # Main.dc.html, States.dc.html if provided
├── .agents/
│   ├── rules/
│   │   ├── 00-project.md        # product, scope, where docs live, "PRD and DESIGN.md win over guesses"
│   │   ├── 10-rust.md           # edition, clippy -D warnings, rustfmt, error handling (anyhow in app, thiserror in libs), no unwrap in providers
│   │   ├── 20-windows-apis.md   # WinRT via `windows` crate, COM/apartment notes, background threads, never block the UI thread
│   │   ├── 30-design-fidelity.md# use tokens.json only, sizes from DESIGN.md, 44 px targets, no emoji
│   │   └── 40-testing.md        # unit tests for state machine, headless PNG snapshots per state, manual Windows checklist
│   ├── workflows/
│   │   ├── phase0-spike.md      # step-by-step spike with exit criteria and a report template
│   │   ├── add-island-state.md  # add a state: tokens → state enum → view → snapshot → docs
│   │   ├── verify-ui.md         # render every state headless, compare to design, list diffs
│   │   └── release.md           # version bump, changelog, build, sign, package
│   └── skills/
│       ├── rune-ui/SKILL.md     # how to build views with Rune: start from examples/hello and examples/overlay, OverlaySpec, springs, headless --snap; read crates/rune/README.md before coding
│       └── windows-media-notifications/SKILL.md  # SMTC session manager usage, UserNotificationListener access + polling, pitfalls
├── Cargo.toml                   # workspace
└── crates/
    ├── cay-core/                # UI-agnostic: island state machine, priorities, models (no Rune, no windows crate)
    ├── cay-providers/           # Windows integrations behind traits: media (SMTC), notifications, calendar, system (battery)
    └── cay-app/                 # Rune overlay + views; wires providers → core → UI
```

**AGENTS.md** must include: one-paragraph product summary; links to every doc; the stack and pinned Rune revision; build/run/test commands (`cargo build`, `cargo test`, `cargo clippy -- -D warnings`, headless snapshot command); the crate boundaries (core has no platform deps; providers have no UI deps); coding rules summary; "definition of done" (tests pass, clippy clean, snapshots updated, docs updated); and "ask before" rules (adding dependencies, changing tokens, touching the release workflow). Keep it under ~150 lines; details go in `.agents/rules/`.

**Skeleton:** workspace with the three crates; `cay-core` has the `IslandState` enum (Idle, NowPlaying, Notification, UpcomingEvent, Timer, Hud, Hub) with sizes from tokens and a priority function plus unit tests; `cay-providers` has traits (`MediaProvider`, `NotificationProvider`, `CalendarProvider`) with `todo!()`-free stub impls that return nothing; `cay-app` has a `main.rs` that opens the Rune overlay with the Idle pill only. Rune is a git dependency pinned to a `rev`. If you cannot compile for Windows here, say so in the report instead of guessing.

## Rules for this session

- Don't build features beyond the skeleton; stop after the commit and report.
- Don't contact any external service except to read docs and look up the Rune dev commit.
- Mark every number you weren't given (RAM targets etc.) as "target, to confirm in Phase 0".
- Write plain, short docs. Tables over walls of text.
