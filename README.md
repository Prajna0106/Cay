# Cay

**Cay** is an ambient desktop floating island for Windows 10/11. Anchored to the top centre of the desktop as a hardware-accelerated, solid-black pill, Cay morphs smoothly to surface active media playback, incoming toast notifications, upcoming calendar appointments, and system HUD indicators without stealing window focus or cluttering the taskbar.

---

## Current Status: Phase 0 Spike Skeleton

The project is currently in **Phase 0 (Technical Spike)**. The workspace skeleton, architecture specifications, design tokens, and agent guidance system are fully established.

- **Phase 0 Spike**: Native Rune desktop overlay proof-of-concept on Windows 11.
- **UI Seam**: Strict separation between `cay-core` (domain state), `cay-providers` (WinRT OS interop), and `cay-app` (Rune presentation layer).
- **Fallback Guarantee**: If Rune encounters platform blockers on Windows 11, the architecture enables an immediate fallback of the presentation layer to Tauri 2 + React + Motion without changing core models or providers.

---

## Workspace Structure

```text
Cay/
├── AGENTS.md                    # Single source of truth for coding agents
├── CLAUDE.md                    # Claude Code pointer (@AGENTS.md)
├── Cargo.toml                   # Workspace manifest
├── design/
│   ├── tokens.json              # Source of truth design tokens (colors, sizes, motion)
│   └── reference/               # HTML/CSS prototype reference files
├── docs/
│   ├── PRD.md                   # Product Requirements Document
│   ├── DESIGN.md                # Design specifications & state geometry table
│   ├── ARCHITECTURE.md          # Crate boundaries, threading, & UI seam
│   ├── RESEARCH.md              # Prior art analysis & WinRT mechanisms
│   ├── ROADMAP.md               # Milestones (Phase 0 Spike → MVP → P1 → P2)
│   └── adr/
│       └── 0001-ui-framework-rune.md # Decision record for Rune + Tauri 2 fallback
├── .agents/
│   ├── rules/                   # Workspace guidelines (Rust, WinRT, Design, Testing)
│   ├── workflows/               # Step-by-step procedures (Spike, Add State, Verify UI)
│   └── skills/                  # Specialized runbooks (Rune UI, WinRT Media & Toasts)
└── crates/
    ├── cay-core/                # Pure domain state machine & priority arbitration (no OS/UI deps)
    ├── cay-providers/           # WinRT integrations (SMTC, Toasts, Calendar) behind traits
    └── cay-app/                 # Native Rune overlay shell & reactive views
```

---

## Building & Running

### Prerequisites
- Windows 10 (version 1903+) or Windows 11
- Rust 1.80.0+ (MSRV)
- Visual Studio C++ Build Tools (for Windows SDK and MSVC linker)

### Commands
```powershell
# Build entire workspace
cargo build

# Run Cay Island overlay
cargo run -p cay-app

# Run all unit tests
cargo test --workspace

# Run Clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Render headless state snapshot
cargo run -p cay-app -- --snap snapshots/idle.png --scene idle
```

---

## Design Tokens & States

All visual parameters are defined in `design/tokens.json`:

| State | Size ($W \times H$) | Corner Radius | Content Summary |
|---|---|---|---|
| **Idle** | $200 \times 40\text{ px}$ | $20\text{ px}$ | System clock & battery percentage |
| **Now Playing** | $360 \times 46\text{ px}$ | $23\text{ px}$ | Album art, track title, artist, live waveform |
| **Notification** | $440 \times 78\text{ px}$ | $30\text{ px}$ | 44 px app icon, header metadata, sender, message preview |
| **Upcoming Event** | $400 \times 62\text{ px}$ | $31\text{ px}$ | Date tile, event title, countdown, Join button |
| **Timer / Focus** | $280 \times 46\text{ px}$ | $23\text{ px}$ | Timer label & tabular countdown |
| **HUD** | $300 \times 46\text{ px}$ | $23\text{ px}$ | Volume / brightness slider & level number |
| **Expanded Hub** | $820 \times 330\text{ px}$ | $40\text{ px}$ | 3 equal cards: Media scrub, Today's events, Notifications |

---

## Documentation Links
- [PRD (Requirements & Priorities)](./docs/PRD.md)
- [Design Spec](./docs/DESIGN.md)
- [Architecture & UI Seam](./docs/ARCHITECTURE.md)
- [Research & Prior Art](./docs/RESEARCH.md)
- [Roadmap & Exit Criteria](./docs/ROADMAP.md)
- [ADR 0001 (Rune & Tauri Fallback)](./docs/adr/0001-ui-framework-rune.md)
- [Agent Guidelines (AGENTS.md)](./AGENTS.md)

---

## License

Licensed under the [MIT License](./LICENSE) (pending final release).
