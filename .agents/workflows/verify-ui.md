# Workflow: Headless UI Verification

This workflow details how to render and audit every island state headlessly to ensure visual fidelity without running on a live display.

## Execution Steps

### 1. Build and Render Headless Snapshots
Run the snapshot rendering command for all registered island scenes:

```powershell
# Create snapshot output directory
New-Item -ItemType Directory -Force -Path snapshots

# Render all registered scenes
cargo run -p cay-app -- --snap snapshots/idle.png --scene idle
cargo run -p cay-app -- --snap snapshots/now_playing.png --scene now_playing
cargo run -p cay-app -- --snap snapshots/notification.png --scene notification
cargo run -p cay-app -- --snap snapshots/upcoming_event.png --scene upcoming_event
cargo run -p cay-app -- --snap snapshots/timer.png --scene timer
cargo run -p cay-app -- --snap snapshots/hud.png --scene hud
cargo run -p cay-app -- --snap snapshots/hub.png --scene hub
```

### 2. Inspect Element Hierarchy Tree
To inspect the resolved layout tree (dimensions, padding, computed constraints):
```powershell
cargo run -p cay-app -- --tree --scene hub
```

### 3. Verification Checklist per State
Compare generated images in `snapshots/` against specifications in `docs/DESIGN.md`:

| State | Target Size | Target Radius | Elements to Verify |
|---|---|---|---|
| **Idle** | $200 \times 40\text{ px}$ | $20\text{ px}$ | Clock left, battery glyph + % right |
| **Now Playing** | $360 \times 46\text{ px}$ | $23\text{ px}$ | 30 px album art, title/artist, 5 accent waveform bars |
| **Notification**| $440 \times 78\text{ px}$ | $30\text{ px}$ | 44 px app icon, header metadata, sender, message preview |
| **Upcoming** | $400 \times 62\text{ px}$ | $31\text{ px}$ | Date tile, title, time countdown, Join button |
| **Timer** | $280 \times 46\text{ px}$ | $23\text{ px}$ | Timer icon, Focus label, tabular numbers in warm accent |
| **HUD** | $300 \times 46\text{ px}$ | $23\text{ px}$ | Volume icon, level slider bar, level digits |
| **Hub** | $820 \times 330\text{ px}$ | $40\text{ px}$ | 3 equal cards, padding 18 px, gap 12 px, media controls |

### 4. Reporting Discrepancies
If any snapshot deviates from the design spec:
1. Note the pixel deviation or misalignment.
2. Adjust styles or padding in `crates/cay-app/src/views/`.
3. Re-run snapshot rendering to verify resolution before submitting PR.
