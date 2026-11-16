# Workflow: Phase 0 Technical Spike Execution

This workflow guides the implementation, execution, and evaluation of the 1–2 day Phase 0 technical spike on Windows 11.

## Objective
Validate the core assumptions of Cay:
1. Rune's Windows overlay implementation (`OverlaySpec`) behaves correctly on Windows 11 (topmost, non-activating, click-through, 60 fps spring morphs).
2. Live SMTC media metadata streaming works via `windows` crate.
3. WinRT `UserNotificationListener` behavior is clarified regarding package identity.

---

## Step-by-Step Procedure

### Step 1: Run the Minimal Overlay
1. Compile and launch `cay-app`:
   ```bash
   cargo run -p cay-app
   ```
2. Verify visual appearance: a $200 \times 40\text{ px}$ black pill at top-centre of the main monitor, $14\text{ px}$ below the top edge.

### Step 2: Test Window Focus & Transparency
1. Open a text editor (e.g. Notepad, VS Code) maximised behind the island.
2. Click directly on the transparent area around the pill:
   - *Expected*: Notepad receives the click and keeps keyboard focus.
3. Type in Notepad:
   - *Expected*: Keystrokes continue without interruption.
4. Click on the pill itself:
   - *Expected*: Pill registers the click (morphs or opens hub), but Notepad's title bar does NOT deactivate, and Notepad does not lose focus (`WS_EX_NOACTIVATE`).

### Step 3: Test Spring Animation & Performance
1. Click the pill to trigger morphing to $360 \times 46\text{ px}$ and back.
2. Observe with Task Manager / PresentMon:
   - Framerate should be steady 60 fps (or 120 fps on high refresh rate displays).
   - CPU usage during idle should drop to $<0.2\%$.
   - Memory footprint should stay $<30\text{ MB}$.

### Step 4: Validate SMTC Media Stream
1. Launch Spotify or a media playback source in a browser.
2. Play a song; observe whether track title, artist, and playback state reach `cay-providers` and update the view.

### Step 5: Test `UserNotificationListener` Capability
1. Invoke `UserNotificationListener::Current().RequestAccessAsync()`.
2. Determine whether Windows displays the permission prompt or throws `E_ACCESSDENIED`.
3. If access fails without package identity, test with a sparse MSIX manifest.

---

## Spike Report Template

```markdown
# Phase 0 Spike Results Report

**Date**: YYYY-MM-DD  
**Tested On**: Windows 11 (Build XXXXX)  
**Hardware**: CPU / GPU  

### Verification Matrix
- [ ] Overlay transparency outside pill: Pass / Fail
- [ ] Topmost over maximised windows: Pass / Fail
- [ ] No focus stealing (`WS_EX_NOACTIVATE`): Pass / Fail
- [ ] Click-through outside pill: Pass / Fail
- [ ] Interactive click handling on pill: Pass / Fail
- [ ] 60 fps spring morph animation: Pass / Fail
- [ ] SMTC live track update: Pass / Fail
- [ ] `UserNotificationListener` requires sparse package identity: Yes / No

### Observed Metrics
- Idle RAM: XX MB
- Idle CPU: X.X%
- Morph FPS: XX fps

### Decision & Recommendation
- **Verdict**: [Proceed with Rune / Trigger Fallback to Tauri 2]
- **Notes & Next Steps**:
```
