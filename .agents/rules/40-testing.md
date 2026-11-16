# Rule 40: Testing, Snapshot Verification & Quality Assurance

## Multi-Layer Verification Strategy

Quality assurance in Cay relies on three complementary testing tiers:

### 1. Unit Testing (`cay-core`)
- Comprehensive automated test coverage for `IslandStateMachine`:
  - Priority resolution between competing states (e.g., Notification arriving while Media is playing).
  - Timeout transitions (e.g., Notification expiring after 5.0 s and reverting to Now Playing).
  - Edge cases (empty media sessions, rapid consecutive toasts, hub toggle).
- Run regularly via:
  ```bash
  cargo test -p cay-core
  ```

### 2. Headless Snapshot Verification (`cay-app`)
- Rune supports native headless rendering to PNG files (`--snap <output.png> --scene <state_name>`).
- Every island state must have a corresponding scene registered in `cay-app`:
  - `idle`, `now_playing`, `notification`, `upcoming_event`, `timer`, `hud`, `hub`.
- Snapshot commands:
  ```bash
  cargo run -p cay-app -- --snap snapshots/idle.png --scene idle
  cargo run -p cay-app -- --snap snapshots/now_playing.png --scene now_playing
  cargo run -p cay-app -- --snap snapshots/hub.png --scene hub
  ```
- Before landing changes that alter views, compare generated snapshots visually or via pixel-diff tools against the design specifications.

### 3. Manual Windows Integration Checklist
When testing on a live Windows 10/11 machine, verify:
- [ ] Island sits top-centre, $14\text{ px}$ from top edge.
- [ ] Completely transparent outside the pill silhouette.
- [ ] Clicking on the desktop or other apps behind the island passes clicks straight through.
- [ ] Clicking inside the pill registers clicks without stealing window focus (`WS_EX_NOACTIVATE`).
- [ ] Morphs between states play at consistent 60 fps without tearing or dropped frames.
- [ ] Opening fullscreen games or videos conceals the island.
- [ ] Tray icon menu enables quitting and toggling settings.
