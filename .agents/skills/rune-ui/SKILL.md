---
name: rune-ui
description: >-
  Guides building, animating, and testing native desktop views with the Rune UI framework.
  Use when writing views in cay-app, configuring OverlaySpec, implementing spring animations,
  or running headless snapshot tests.
---

# Skill: Building Views with Rune

Rune is a high-performance native Rust UI framework with its own renderer, text engine (Taffy 0.15), and spring animation system (`rune-motion`).

## Essential Rules Before Coding
1. **Always read upstream documentation**: Review Rune's API concepts (`row()`, `col()`, `el()`, `text()`, `AppSpec`, `OverlaySpec`) before writing views.
2. **Consult Reference Examples**:
   - `examples/hello`: Basic reactive view, state updates, and spring motion.
   - `examples/overlay`: Native overlay mechanics, click-through, and `OverlaySpec`.
   - `examples/pacing_overlay`: Pinned capsule/overlay layout and windowless apps.

---

## 1. Configuring `OverlaySpec` for Cay Island

Cay uses an edge-pinned overlay anchored to the top centre of the desktop:

```rust
use rune::{Align, AppSpec, Cx, El, OverlaySpec, ScreenEdge};

pub fn create_island_overlay<T: 'static>(view_fn: fn(&T, &mut Cx<T>) -> El) -> OverlaySpec<T> {
    OverlaySpec {
        name: "Cay Island",
        size: (200.0, 40.0),       // Default Idle size in points
        edge: ScreenEdge::Top,     // Pin to top screen boundary
        margin: 14.0,              // 14 px offset from top edge
        align: Align::Center,      // Centred horizontally
        inset: 0.0,
        interactive: true,         // Clicks reach the island
        view: view_fn,
    }
}
```

- When the overlay is created with `app.overlay(spec, initial_state)`, it operates as a topmost non-activating window on Windows (`WS_EX_TOPMOST | WS_EX_NOACTIVATE`).
- The overlay's root view should have no fill outside the pill, keeping all exterior regions completely transparent and click-through.

---

## 2. Composing Island Views

Rune views are declarative functions returning `El`:

```rust
use rune::tok::motion;
use rune::{Color, Cx, El, col, el, row, text};

pub fn island_view(state: &AppState, cx: &mut Cx<AppState>) -> El {
    let (width, height, radius) = state.current_state.dimensions();

    // Solid black pill container
    el()
        .key("island-pill")
        .size(width, height)
        .radius(radius)
        .fill(Color { r: 0.043, g: 0.043, b: 0.059, a: 1.0 }) // #0B0B0F
        .border(1.0, Color { r: 0.149, g: 0.149, b: 0.180, a: 1.0 }) // #26262E
        .motion(motion::SNAPPY) // Analytic spring morph
        .on_press(|s: &mut AppState| {
            s.toggle_hub();
        })
        .child(render_content(state))
}
```

### Layout Gotchas in Rune
- `.grow(g)` is equivalent to `flex: g 1 0`. Inside an unconstrained parent, it collapses. Give parents bounded `.h(...)` or `.h_full()`.
- Use `.min_h(0.0)` on flex containers if scrollable items refuse to shrink.
- Use `Tx::instant()` if a layout change should skip animation (e.g. initial setup).

---

## 3. Headless Verification & Snapshot Testing

Rune provides built-in headless capture:
```bash
# Snapshot a specific scene to PNG
cargo run -p cay-app -- --snap snapshots/idle.png --scene idle

# Dump element tree layout
cargo run -p cay-app -- --tree --scene idle
```

In `cay-app`, define static test scenes in `AppSpec`:
```rust
scenes: &[
    ("idle", |s| s.set_state(IslandState::Idle)),
    ("now_playing", |s| s.set_state(IslandState::NowPlaying(...))),
]
```
