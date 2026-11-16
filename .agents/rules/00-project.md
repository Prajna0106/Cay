# Rule 00: Project Scope & Documentation Authority

## Product Overview
**Cay** is a floating "island" for Windows 10/11: a solid-black pill anchored top-centre on the desktop ($14\text{ px}$ from the top edge) that dynamically expands to display media playback, incoming toast notifications, upcoming calendar meetings, timers, and HUD indicators, shrinking back to an unobtrusive clock when idle.

## Single Source of Truth
- When implementing features, resolving ambiguities, or making architectural decisions, **the written documentation in `docs/` and `design/tokens.json` takes absolute precedence over assumptions or guesses**.
- Do not invent facts, API behavior, or visual styling beyond what is specified in:
  - `docs/PRD.md`: Requirements, priority rules, user stories, acceptance criteria.
  - `docs/DESIGN.md`: Visual layout, dimensions, typography, and interaction patterns.
  - `docs/ARCHITECTURE.md`: Crate boundaries, threading rules, and fallback mechanisms.
  - `design/tokens.json`: Exact colors, radii, dimensions, and motion settings.
- If a requirement or edge case is missing or unknown, document it explicitly as an **Open Question** rather than silently making an assumption.

## Crate Responsibilities & Boundaries
1. **`crates/cay-core`**: UI-agnostic domain logic. Must compile with standard Rust toolchains on any host. Contains state models and priority logic. Never import `windows` or `rune` here.
2. **`crates/cay-providers`**: Windows platform traits and WinRT integrations. Encapsulates all OS calls behind traits. Never import `rune` or UI crates here.
3. **`crates/cay-app`**: Native Rune application shell and view rendering. Connects providers to core state and renders overlay views.
