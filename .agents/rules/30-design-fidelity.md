# Rule 30: Design Fidelity & Visual Ergonomics

## Strict Token Adherence
- **Zero Magic Numbers**: All interface colors, radii, spacing, typography scales, and motion parameters must be drawn directly from `design/tokens.json`.
- Island dimensions must strictly match the state geometry specifications in `docs/DESIGN.md`:
  - **Idle**: $200 \times 40\text{ px}$, $r = 20\text{ px}$
  - **Now playing**: $360 \times 46\text{ px}$, $r = 23\text{ px}$
  - **Notification**: $440 \times 78\text{ px}$, $r = 30\text{ px}$
  - **Upcoming event**: $400 \times 62\text{ px}$, $r = 31\text{ px}$
  - **Timer / focus**: $280 \times 46\text{ px}$, $r = 23\text{ px}$
  - **HUD**: $300 \times 46\text{ px}$, $r = 23\text{ px}$
  - **Expanded hub**: $820 \times 330\text{ px}$, $r = 40\text{ px}$, padding $18\text{ px}$, card gap $12\text{ px}$

## Iconography & Typography Constraints
- **Vector Line Icons**: All icons must be clean vector line paths with $1.8\text{–}2.0\text{ px}$ stroke width and rounded joins/caps.
- **Never Use Emojis**: Emojis render inconsistently across Windows versions and break the polished, native aesthetic.
- **Tabular Figures**: Always enable tabular figures for numbers, clocks, and timers (`font_variant_numeric: tabular-nums`) to prevent text jitter during ticking updates.
- **Touch & Click Targets**: Every interactive button or control must maintain a minimum target size of $44 \times 44\text{ px}$ (via padding if the visual glyph is smaller).
- **Reduced Motion**: Always inspect system motion preferences. If reduced motion is requested, disable overshoot spring bounces and execute smooth cross-fades only.
