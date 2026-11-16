# Design Specification: Cay Island

## 1. Design Overview & Philosophy

**Cay** is an ambient desktop surface that bridges Windows system state with user attention. Designed as a floating black pill anchored to the top centre of the screen, Cay communicates glanceable information and morphs smoothly into rich interactive views when needed.

### Core Principles
- **Atmospheric & Subdued**: Solid black pill (`#0B0B0F`) with subtle borders and shadows that floats seamlessly above desktop wallpapers and maximised windows.
- **Dynamic Adaptability**: Grows organically to accommodate incoming toasts, meeting reminders, or active media, shrinking back to a calm clock when idle.
- **Strict Motion Continuity**: All morphs animate dimensions, corner radii, and positions simultaneously via fluid spring curves. Content smoothly cross-fades without jump cuts.
- **Precision Typography & Iconography**: Inter for all interface elements with tabular figures for numbers. Sharp 1.8–2.0 px stroke line icons; never emoji.

---

## 2. Spatial Placement & Geometry

- **Horizontal Alignment**: Centred horizontally relative to the target display.
- **Top Offset**: Fixed `14 px` from the top boundary of the display work area.
- **Z-Order**: Always-on-top tool window (`WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`).
- **Hit-Testing**: Click-through outside the pill boundary (`WS_EX_TRANSPARENT` or regional hit-testing). Inside the pill boundary, mouse clicks and wheel interactions are fully captured.

---

## 3. Design Tokens

All values match `design/tokens.json`.

### Color Palette

| Token | Hex / Value | Usage |
|---|---|---|
| `island.background` | `#0B0B0F` | Outer island surface fill |
| `island.border` | `#26262E` | 1 px outer island stroke |
| `island.shadow` | `0 18px 40px rgba(11,11,15,0.28)` | Floating depth elevation |
| `card.surface` | `#17171D` | Interior Hub cards |
| `card.radius` | `22 px` | Hub interior card corner radius |
| `track.background` | `#2E2E38` | Progress bar background, volume slider track |
| `text.primary` | `#F5F5F7` | Track titles, clock, sender names, primary labels |
| `text.secondary` | `#D4D4D8` | Message previews, event subtitles |
| `text.muted` | `#A1A1AA` | Artist names, battery %, time stamps, caps headers |
| `accent.default` | `#7CC4FF` | Default sky blue accent (waveform, calendar, Join button) |
| `accent.lime` | `#A3E635` | Optional user accent preset |
| `accent.warm` | `#FFB86B` | Optional user accent preset / Timer & Focus state |
| `accent.purple` | `#F0ABFC` | Optional user accent preset |

### Typography Scale

- **Primary Font**: `Inter, system-ui, sans-serif`
- **Display / Marketing Font**: `Space Grotesk, Inter, sans-serif`

| Style Name | Size | Weight | Line Height | Tracking | Notes |
|---|---|---|---|---|---|
| **Card Title** | 15 px | 600 (SemiBold) | 1.2 | normal | Hub cards, track titles |
| **Primary Line** | 14 px | 600 (SemiBold) | 1.2 | 0.2 px | Clock, sender names, meeting names |
| **Body** | 13 px | 400 (Regular) | 1.3 | normal | Message previews, artist names |
| **Meta** | 12 px | 400 (Regular) | 1.3 | normal | Time stamps, channel names, battery % |
| **Caps Label** | 11 px | 600 (SemiBold) | 1.2 | 0.8 px | Section headers (`NOW PLAYING`, `TODAY`) |
| **Timer Num** | 16 px | 600 (SemiBold) | 1.0 | tabular | `tabular-nums` for countdowns |

### Ergonomics & Sizing Rules
- **Touch & Click Targets**: Minimum `44 × 44 px` interactive area for all buttons.
- **Icons**: 1.8–2.0 px stroke vector paths, rounded caps/joins. No raster emojis.

---

## 4. State Specifications

| # | State | Size ($W \times H$) | Radius | Elements & Internal Layout | Duration / Trigger |
|---|---|---|---|---|---|
| **1** | **Idle** | $200 \times 40\text{ px}$ | $20\text{ px}$ | Clock left (`14/600`), battery icon + % right (`12/muted`). | Default baseline state |
| **2** | **Now playing** | $360 \times 46\text{ px}$ | $23\text{ px}$ | 30 px album art (r9), title · artist (13/600), 5-bar accent waveform right. Bars freeze when paused. | Active SMTC media session |
| **3** | **Notification** | $440 \times 78\text{ px}$ | $30\text{ px}$ | 44 px app icon (r14), app · channel + "now" (12/muted), sender (14/600), message preview (13). | 5.0 s peek on new toast |
| **4** | **Upcoming event**| $400 \times 62\text{ px}$ | $31\text{ px}$ | Date tile (38 px, r12, month accent + day), event title (14/600), "in X min · Teams" (12/muted), "Join" pill button. | 15 min prior to event start |
| **5** | **Timer / focus** | $280 \times 46\text{ px}$ | $23\text{ px}$ | Timer icon + "Focus" left (`13/600`), countdown right (`16/600` in `#FFB86B` tabular). | Active countdown / focus |
| **6** | **HUD** | $300 \times 46\text{ px}$ | $23\text{ px}$ | Speaker/brightness icon, horizontal bar (track `#2E2E38`, fill `#F5F5F7`), level number. | 2.0 s interrupt on keypress |
| **7** | **Expanded hub** | $820 \times 330\text{ px}$ | $40\text{ px}$ | Padding 18 px, 3 equal cards with 12 px gap: (1) Now playing with scrub & controls; (2) Today calendar events; (3) Recent notifications list. | Click pill to toggle open/closed |

*Note on design reference variations*: In `States.dc.html`, the Notification and Event states were rendered at 380 px width within an artboard grid container. The production island dimensions specified above ($440 \times 78$ and $400 \times 62$) represent the nominal standalone island geometries.

---

## 5. Motion & Transitions

### Dimension & Radius Morphs
- **Spring Model**: Underdamped spring with slight overshoot.
  - Duration equivalent: `~450 ms`
  - Cubic bezier approximation: `cubic-bezier(0.34, 1.4, 0.5, 1.0)`
  - Rune motion preset: `motion::SNAPPY` or `Spring::new(0.45, 0.2)`
- **Edges**: Width, height, and corner radius animate synchronously so the pill remains visually cohesive.

### Content Transition
- **Cross-Fade**: Old view fades out while new view fades in with subtle scale transition (`0.96` to `1.0` scale, `~350 ms`).
- **Waveform Animation**: 5 accent bars bouncing asynchronously (0.35 to 1.0 vertical scale) with staggered animation delays (`0.15s`, `0.3s`, `0.45s`, `0.6s`). Freezes in current position on pause.

### Accessibility: Reduced Motion
- When Windows reports reduced motion preference:
  - Springs are disabled; strict linear or ease-out fades used (`250 ms`).
  - No overshoot bounce or scale transforms applied.

---

## 6. Interaction Model

1. **Hover Expansion (Default)**: Hovering over the pill (after a brief dwell threshold to prevent accidental triggers) smoothly expands the island into the Expanded Hub; moving the cursor away collapses it back to the active pill state.
2. **Click-to-Toggle Hub**: Clicking the pill explicitly pins/toggles the Expanded Hub open or closed (and users can switch to click-only mode in Settings).
3. **Esc Key & Outside Click**: Dismisses the Expanded Hub back to the active pill state.
4. **State Priority**:
   $$\text{HUD} > \text{Notification} > \text{Upcoming Event} > \text{Timer} > \text{Now Playing} > \text{Idle}$$
