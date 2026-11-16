# ADR 0001: Selection of Rune as Primary UI Framework with Tauri 2 Fallback

## Status
Accepted (Subject to Phase 0 Spike verification)

## Context
Cay requires a high-performance desktop presentation layer on Windows 10/11 that satisfies strict ergonomic and visual constraints:
1. **Window Overlay Mechanics**: Transparent, borderless, topmost desktop overlay that never steals focus (`WS_EX_NOACTIVATE`), allows click-through on transparent areas, and captures input on active elements.
2. **Fluid Physics Animations**: Spring-based dimension and radius transitions (450 ms spring morphs) without jank or dropped frames.
3. **Resource Efficiency**: Low idle RAM (<30 MB) and CPU usage (~0%), avoiding heavy webview runtimes where possible.
4. **Automated Verification**: Ability to render headless state snapshots to PNG and inspect element trees in CI/test pipelines.

## Decision
We select **Rune** (https://github.com/anomalyco/rune) as the primary UI framework for `cay-app`.

Rune is a native, GPU-rendered Rust UI toolkit providing its own text layout (Taffy 0.15 integration), vector rendering, analytic spring system (`rune-motion`), and edge-pinned overlay primitives (`OverlaySpec`).

### Why Rune Fits
- **`OverlaySpec`**: Built-in support for edge-pinned desktop overlays with configurable margin (`14 px`), alignment (`Align::Center`), click-through control (`interactive`), and runtime resizing via `OverlayHandle`.
- **Spring Animations**: Native analytic springs (`motion::SNAPPY`, `Spring::new(duration, bounce)`) that run on the compositor loop with retargeting support.
- **Headless Mode**: Headless rendering pipeline (`--snap <file.png> --scene <name>`) allows automated snapshot comparison for all island states without needing a live display.
- **Accessibility**: Native UI Automation integration on Windows.
- **Pure Rust Pipeline**: Directly links with `cay-core` and `cay-providers` without foreign function interfaces (FFI) or IPC overhead.

## Alternatives Considered

| Alternative | Pros | Cons | Verdict |
|---|---|---|---|
| **Tauri 2 + React + Motion** | Extremely mature, vast UI ecosystem, proven by HeyyCzer. | Higher memory footprint (~70–120 MB via WebView2), multi-process IPC serialization overhead. | **Designated Primary Fallback** |
| **WinUI 3 (.NET / C++)** | Official Microsoft design language, deep Windows integration. | Heavyweight runtime dependencies, complex build toolchain, significant memory footprint. | Rejected |
| **Slint (Skia backend)** | Lightweight native Rust GUI, fast startup. | Less flexible spring physics compared to Rune/Framer Motion, proprietary DSL requirement. | Rejected |
| **Direct2D / Win32 Mod** | Lowest possible overhead. | High maintenance burden, fragile custom layout engine, lacks automated testability. | Rejected |

## Risks & Mitigations

1. **Rune is Pre-Release Software**:
   - *Mitigation*: Pin an exact commit hash on the `dev` branch (`55ef5268f22038a95344b3e9d2b4a25ddba731b1`) in `Cargo.toml`. Never track a moving branch.
2. **Windows Overlay Maturity**:
   - *Risk*: The Windows overlay codepath in Rune is primarily compile-tested by upstream authors (`cargo xwin clippy`) rather than battle-tested across thousands of consumer machines.
   - *Mitigation*: Phase 0 spike specifically validates Windows 11 desktop behavior (click-through, focus non-activation, multi-monitor behavior).
3. **System Material Blur Limitation**:
   - *Risk*: Acrylic / Mica / system blur currently operates only on macOS in Rune; on Windows, Rune falls back to solid fill.
   - *Mitigation*: Cay's design specification mandates a solid black pill (`#0B0B0F`) with a 1 px border; background blur is explicitly unnecessary.
4. **Upstream Contribution Model**:
   - *Risk*: Upstream Rune repository accepts issue reports but does not accept public pull requests.
   - *Mitigation*: Maintain internal forks if patches or workarounds are required.

## Fallback Trigger Criteria
If the **Phase 0 Spike** reveals that Rune fails any of the following exit criteria on Windows 11:
- Non-activating window behavior fails (clicks on the pill steal focus from active windows).
- Transparent overlay regions fail to pass clicks through to background desktop windows.
- Morphing animations cannot maintain 60 fps.

Then the fallback is immediately triggered:
- The UI layer (`cay-app`) is replaced with Tauri 2 + React + Framer Motion.
- `cay-core` and `cay-providers` are preserved without code modifications.
