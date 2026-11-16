# Cay: Agent Guidelines & Project Instructions

Cay is a floating island desktop application for Windows 10/11: an ambient, hardware-accelerated solid-black pill anchored top-centre on the desktop that presents glanceable notifications, media controls, upcoming meetings, and system HUD flyouts without stealing keyboard focus or obstructing active workflows.

## Documentation Index
- [Product Requirements Document (PRD)](file:///docs/PRD.md)
- [Design Specification](file:///docs/DESIGN.md) & [Design Tokens](file:///design/tokens.json)
- [Architecture & UI Seam](file:///docs/ARCHITECTURE.md)
- [Prior Art & Technical Research](file:///docs/RESEARCH.md)
- [Project Roadmap](file:///docs/ROADMAP.md)
- [ADR 0001: UI Framework (Rune & Fallback)](file:///docs/adr/0001-ui-framework-rune.md)

## Tech Stack & Pinned Revisions
- **Language**: Rust (2021 Edition, MSRV 1.80+)
- **UI Framework**: Rune (Git pinned: `rev = "55ef5268f22038a95344b3e9d2b4a25ddba731b1"`)
- **OS Interop**: Microsoft `windows` crate (WinRT: SMTC, UserNotificationListener, Power)
- **Domain State**: Pure Rust state machine with analytic spring interpolation

## Standard Commands
- **Build**: `cargo build`
- **Run**: `cargo run -p cay-app`
- **Test**: `cargo test --workspace`
- **Lint**: `cargo clippy --workspace --all-targets -- -D warnings`
- **Format Check**: `cargo fmt --all -- --check`
- **Headless Snapshots**: `cargo run -p cay-app -- --snap snapshots/island.png --scene idle`

## Crate Architecture & Strict Boundaries
- `crates/cay-core`: Pure UI-agnostic domain models, state machine, priority arbitration, and token layout logic. **Zero dependencies on Rune, Windows API, or OS crates**.
- `crates/cay-providers`: Windows platform traits (`MediaProvider`, `NotificationProvider`, `CalendarProvider`, `SystemProvider`) and WinRT implementations. **Zero dependencies on Rune or UI crates**. Stubs provided for headless testing.
- `crates/cay-app`: Rune overlay shell, views, animations, and provider dispatch loop. Wires `cay-providers` $\to$ `cay-core` $\to$ Rune views.

## Coding Standards Summary
1. **Error Handling**: Use `thiserror` in `cay-core` and `cay-providers`; use `anyhow` in `cay-app`. Never use `unwrap()` or `expect()` in providers or production paths.
2. **WinRT & COM**: WinRT listeners must run on background MTA worker threads. Never block the main UI thread.
3. **Design Fidelity**: Never hardcode colors, padding, or dimensions. Reference `design/tokens.json` and `docs/DESIGN.md` exclusively.
4. **Touch & Ergonomics**: All interactive click targets must be $\ge 44\text{ px}$. Use vector stroke icons (1.8–2.0 px); never emoji.

## Definition of Done (DoD)
Every pull request or feature milestone must satisfy:
1. `cargo test --workspace` passes with zero failures.
2. `cargo clippy --workspace --all-targets -- -D warnings` produces zero warnings.
3. `cargo fmt --all -- --check` reports clean formatting.
4. Headless PNG snapshots generated and verified against design specs for any altered states.
5. Corresponding documentation updated in `docs/` if behavior or architecture changed.

## Ask Before Rules
Always request explicit user confirmation before:
1. Adding new third-party dependencies to `Cargo.toml`.
2. Altering design tokens in `design/tokens.json`.
3. Modifying release workflows, packaging scripts, or signing credentials.
4. Triggering the UI framework fallback from Rune to Tauri 2.
