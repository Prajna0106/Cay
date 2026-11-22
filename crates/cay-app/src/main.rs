use cay_core::models::{
    AccentPreset, BatteryStatus, CalendarMeeting, HudKind, HudState, IslandSettings, MediaInfo,
    MonitorChoice, TimerState, ToastNotification,
};
use cay_core::state::{IslandDimensions, IslandState, IslandStateMachine};
#[cfg(windows)]
use cay_providers::{
    current_local_clock_and_date, get_system_master_volume_percent, init_mta,
    is_foreground_window_fullscreen, is_windows_quiet_hours_active, open_external_url,
    process_working_set_bytes, set_windows_autostart, spawn_global_hotkey_listener,
    WinNotificationProvider, WinSmtcMediaProvider, WinSystemProvider,
};
use cay_providers::{
    CalendarProvider, IcsCalendarProvider, MediaProvider, NotificationProvider, SystemProvider,
};
use rune::{
    col, el, icon, row, text, Align, App, AppSpec, Color, Cx, DisplayTarget, El, Icon, ImageCache,
    Key, KeyEvent, OverlayInput, OverlayOptions, OverlaySpec, RunOptions, ScreenEdge, Spring,
    SurfaceFit, TextStyle, Tray, TrayEvent, TrayIcon, TrayItem, TraySpec, Tx, WindowSpec,
};
use std::sync::{Arc, OnceLock};

// Design Tokens from design/tokens.json (Linear sRGB for Rune's Color pipeline)
const ISLAND_BG: Color = Color {
    r: 0.00182,
    g: 0.00182,
    b: 0.01033,
    a: 1.0,
}; // #06061A
const ISLAND_BORDER: Color = Color {
    r: 0.0975,
    g: 0.1273,
    b: 0.8714,
    a: 0.32,
}; // #5864F0 @ 32%
const CARD_SURFACE: Color = Color {
    r: 0.00401,
    g: 0.00401,
    b: 0.01621,
    a: 1.0,
}; // #0D0D22
const TRACK_BG: Color = Color {
    r: 0.00769,
    g: 0.00769,
    b: 0.02611,
    a: 1.0,
}; // #15152A
const TEXT_PRIMARY: Color = Color {
    r: 0.9131,
    g: 0.9131,
    b: 0.9301,
    a: 1.0,
}; // #F5F5F7
const TEXT_SECONDARY: Color = Color {
    r: 0.6584,
    g: 0.6584,
    b: 0.6867,
    a: 1.0,
}; // #D4D4D8
const TEXT_MUTED: Color = Color {
    r: 0.0768,
    g: 0.0768,
    b: 0.2070,
    a: 1.0,
}; // #48487E
const ACCENT_SKY: Color = Color {
    r: 0.0476,
    g: 0.6844,
    b: 1.0000,
    a: 1.0,
}; // #3DD8FF
const ACCENT_LIME: Color = Color {
    r: 0.1752,
    g: 1.0000,
    b: 0.0391,
    a: 1.0,
}; // #74FF38
const ACCENT_WARM: Color = Color {
    r: 1.0000,
    g: 0.3208,
    b: 0.0638,
    a: 1.0,
}; // #FF9A47
const ACCENT_PURPLE: Color = Color {
    r: 0.5411,
    g: 0.1680,
    b: 1.0000,
    a: 1.0,
}; // #BF72FF
const ALBUM_TILE_BG: Color = Color {
    r: 0.0437,
    g: 0.0284,
    b: 0.1470,
    a: 1.0,
}; // #3B2F6B
const ALBUM_ICON_COLOR: Color = Color {
    r: 0.5841,
    g: 0.5333,
    b: 0.9131,
    a: 1.0,
}; // #C9C1F5
const NOTIF_SLACK_BG: Color = Color {
    r: 0.1465,
    g: 0.01765,
    b: 0.2791,
    a: 1.0,
}; // #6B2490
const NOTIF_WHATSAPP_BG: Color = Color {
    r: 0.01227,
    g: 0.2619,
    b: 0.1023,
    a: 1.0,
}; // #1D8C5A
const NOTIF_GITHUB_BG: Color = Color {
    r: 0.04672,
    g: 0.08848,
    b: 0.1385,
    a: 1.0,
}; // #3D5468
const CAL_NEUTRAL_BAR: Color = Color {
    r: 0.03688,
    g: 0.03688,
    b: 0.08042,
    a: 1.0,
}; // #363650
const CAL_TILE_BG: Color = Color {
    r: 0.00401,
    g: 0.01371,
    b: 0.03688,
    a: 1.0,
}; // #0D1F36
const CAL_FEATURED_BORDER: Color = Color {
    r: 0.0476,
    g: 0.6844,
    b: 1.0000,
    a: 0.30,
}; // #3DD8FF @ 30%
const PILL_BG: Color = Color {
    r: 0.0476,
    g: 0.6844,
    b: 1.0000,
    a: 0.15,
}; // #3DD8FF @ 15% for countdown pill bg

// Typography Tokens from design/tokens.json
const FONT_CARD_TITLE: TextStyle = TextStyle::new(15.0, 600.0);
const FONT_PRIMARY_LINE: TextStyle = TextStyle::new(14.0, 600.0).tracking(0.2);
const FONT_CLOCK: TextStyle = TextStyle::new(14.0, 600.0)
    .tracking(0.2)
    .features("'tnum' 1");
const FONT_BODY: TextStyle = TextStyle::new(13.0, 400.0);
const FONT_BODY_SEMIBOLD: TextStyle = TextStyle::new(13.0, 600.0);
const FONT_META: TextStyle = TextStyle::new(12.0, 400.0);
const FONT_META_TABULAR: TextStyle = TextStyle::new(12.0, 400.0).features("'tnum' 1");
const FONT_CAPS_LABEL: TextStyle = TextStyle::new(11.0, 600.0).tracking(0.8);
const FONT_CHIP: TextStyle = TextStyle::new(10.0, 600.0);
const FONT_TIMER: TextStyle = TextStyle::new(16.0, 600.0).features("'tnum' 1");

// Motion Token: 450ms spring with 0.2 bounce
const PILL_SPRING: Spring = Spring::new(0.45, 0.2);

// Vector stroke icons (1.8-2.0px stroke, never emoji)
static ICON_BATTERY: Icon = Icon {
    name: "cay-battery",
    view: 24.0,
    body: r#"<rect x="2" y="6" width="17" height="12" rx="2.5" fill="none" stroke="currentColor" stroke-width="1.8"/><rect x="4.5" y="8.5" width="10" height="7" rx="1" fill="currentColor" stroke="none"/><path d="M21 10v4" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>"#,
};

static ICON_MUSIC: Icon = Icon {
    name: "cay-music",
    view: 24.0,
    body: r#"<path d="M9 18V5l12-2v13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><circle cx="6" cy="18" r="3" fill="none" stroke="currentColor" stroke-width="2"/><circle cx="18" cy="16" r="3" fill="none" stroke="currentColor" stroke-width="2"/>"#,
};

static ICON_TIMER: Icon = Icon {
    name: "cay-timer",
    view: 24.0,
    body: r#"<circle cx="12" cy="13" r="8" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 9v4l2 2" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M9 2h6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
};

static ICON_VOLUME: Icon = Icon {
    name: "cay-volume",
    view: 24.0,
    body: r#"<path d="M11 5 6 9H2v6h4l5 4V5Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M15.5 8.5a5 5 0 0 1 0 7" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M19 5a10 10 0 0 1 0 14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
};

static ICON_PREV: Icon = Icon {
    name: "cay-prev",
    view: 24.0,
    body: r#"<path d="M19 20 9 12l10-8v16Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M5 19V5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
};

static ICON_NEXT: Icon = Icon {
    name: "cay-next",
    view: 24.0,
    body: r#"<path d="m5 4 10 8-10 8V4Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M19 5v14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
};

static ICON_PAUSE: Icon = Icon {
    name: "cay-pause",
    view: 24.0,
    body: r#"<rect x="6" y="4" width="4" height="16" rx="1" fill="currentColor"/><rect x="14" y="4" width="4" height="16" rx="1" fill="currentColor"/>"#,
};

static ICON_PLAY: Icon = Icon {
    name: "cay-play",
    view: 24.0,
    body: r#"<path d="M7 4v16l13-8L7 4Z" fill="currentColor"/>"#,
};

static ICON_CALENDAR: Icon = Icon {
    name: "cay-calendar",
    view: 24.0,
    body: r#"<rect x="3" y="4" width="18" height="18" rx="3" fill="none" stroke="currentColor" stroke-width="1.8"/><path d="M16 2v4M8 2v4M3 10h18" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>"#,
};

static ICON_BELL: Icon = Icon {
    name: "cay-bell",
    view: 24.0,
    body: r#"<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M13.73 21a2 2 0 0 1-3.46 0" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>"#,
};

static ICON_CLOSE: Icon = Icon {
    name: "cay-close",
    view: 24.0,
    body: r#"<path d="M18 6 6 18M6 6l12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

static ICON_HEADPHONES: Icon = Icon {
    name: "cay-headphones",
    view: 24.0,
    body: r#"<path d="M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7a9 9 0 0 1 18 0v7a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

static ICON_DND: Icon = Icon {
    name: "cay-dnd",
    view: 24.0,
    body: r#"<circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.8"/><path d="M8 12h8" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>"#,
};

static ICON_CHECK: Icon = Icon {
    name: "cay-check",
    view: 24.0,
    body: r#"<path d="M20 6 9 17l-5-5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

fn resolve_accent_color(preset: AccentPreset) -> Color {
    match preset {
        AccentPreset::Sky => ACCENT_SKY,
        AccentPreset::Lime => ACCENT_LIME,
        AccentPreset::Warm => ACCENT_WARM,
        AccentPreset::Purple => ACCENT_PURPLE,
    }
}

fn format_mm_ss(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u32;
    let mins = total / 60;
    let secs = total % 60;
    format!("{mins}:{secs:02}")
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    let clean = text.replace(['\r', '\n'], " ");
    let trimmed = clean.trim();
    if trimmed.chars().count() > max_chars {
        let prefix: String = trimmed.chars().take(max_chars).collect();
        format!("{prefix}…")
    } else {
        trimmed.to_string()
    }
}

fn image_cache() -> &'static ImageCache {
    static CACHE: OnceLock<ImageCache> = OnceLock::new();
    CACHE.get_or_init(|| ImageCache::new(Arc::new(|| {})))
}

fn render_album_tile(size: f32, radius: f32, icon_size: f32, thumbnail_bytes: Option<&[u8]>) -> El {
    if let Some(bytes) = thumbnail_bytes {
        if !bytes.is_empty() {
            let img = image_cache().load(bytes.to_vec());
            if img.is_ready() && img.error().is_none() {
                return el()
                    .size(size, size)
                    .radius(radius)
                    .fill(ALBUM_TILE_BG)
                    .pixels_fit(img.pixels(), SurfaceFit::Fill);
            }
        }
    }
    el().size(size, size)
        .radius(radius)
        .fill(ALBUM_TILE_BG)
        .center()
        .child(icon(&ICON_MUSIC, icon_size).color(ALBUM_ICON_COLOR))
}

#[derive(Clone, Debug)]
struct AppState {
    clock_text: String,
    today_header: String,
    waveform_phase: usize,
    state_machine: IslandStateMachine,
}

impl Default for AppState {
    fn default() -> Self {
        let mut sm = IslandStateMachine::new();
        sm.set_battery(BatteryStatus {
            percentage: 82,
            is_charging: false,
            is_present: true,
        });
        Self {
            clock_text: "7:08 PM".to_string(),
            today_header: "TODAY · SUN 15 NOV".to_string(),
            waveform_phase: 0,
            state_machine: sm,
        }
    }
}

fn sample_media() -> MediaInfo {
    MediaInfo {
        title: "Midnight Drive".to_string(),
        artist: "Lumen Avenue".to_string(),
        album: Some("Nightfall".to_string()),
        is_playing: true,
        position_secs: 94.0,
        duration_secs: 228.0,
        source_app: "Spotify".to_string(),
        thumbnail_bytes: None,
    }
}

fn sample_notification() -> ToastNotification {
    ToastNotification {
        id: 1,
        app_name: "Slack".to_string(),
        channel: Some("#product".to_string()),
        sender: "Priya Sharma".to_string(),
        message: "Can you share the island PRD before tomorrow's sync?".to_string(),
        received_timestamp: 0,
    }
}

fn sample_event() -> CalendarMeeting {
    CalendarMeeting {
        id: "evt-1".to_string(),
        title: "Design review".to_string(),
        start_timestamp: 0,
        duration_minutes: 40,
        service_name: "Microsoft Teams".to_string(),
        join_url: Some("https://teams.microsoft.com".to_string()),
    }
}

fn render_idle(clock: &str, battery: &BatteryStatus) -> El {
    let mut r = row()
        .key("content-idle")
        .w_full()
        .h_full()
        .px(18.0)
        .center_y()
        .child(text(clock).font(FONT_CLOCK).color(TEXT_PRIMARY))
        .child(el().grow(1.0));

    if battery.is_present {
        r = r.child(
            row()
                .gap(6.0)
                .center_y()
                .child(icon(&ICON_BATTERY, 16.0).color(TEXT_MUTED))
                .child(
                    text(format!("{}%", battery.percentage))
                        .font(FONT_META_TABULAR)
                        .color(TEXT_MUTED),
                ),
        );
    }
    r
}

fn render_now_playing(media: &MediaInfo, accent: Color, waveform_phase: usize) -> El {
    let heights = [8.0_f32, 14.0, 10.0, 16.0, 6.0];
    let mut waveform = row().gap(3.0).center_y().h(16.0);
    for idx in 0..heights.len() {
        let bar_h = heights[(idx + waveform_phase) % heights.len()];
        let h = if media.is_playing { bar_h } else { 5.0 };
        waveform = waveform.child(
            el().key(format!("wave-{idx}"))
                .size(3.0, h)
                .radius(2.0)
                .fill(accent)
                .motion(PILL_SPRING),
        );
    }

    let title_text = truncate_preview(&media.title, 18);
    let artist_str = if media.artist.is_empty() {
        format!("· {}", media.source_app)
    } else {
        format!("· {}", media.artist)
    };
    let artist_text = truncate_preview(&artist_str, 16);

    row()
        .key("content-now-playing")
        .w_full()
        .h_full()
        .px(12.0)
        .gap(10.0)
        .center_y()
        .child(render_album_tile(
            30.0,
            9.0,
            14.0,
            media.thumbnail_bytes.as_deref(),
        ))
        .child(
            row()
                .grow(1.0)
                .min_w(0.0)
                .gap(6.0)
                .center_y()
                .child(
                    text(title_text)
                        .font(FONT_BODY_SEMIBOLD)
                        .color(TEXT_PRIMARY),
                )
                .child(text(artist_text).font(FONT_BODY).color(TEXT_MUTED)),
        )
        .child(waveform)
}

fn render_notification(notif: &ToastNotification) -> El {
    let header_left = match &notif.channel {
        Some(ch) => format!("{} · {}", notif.app_name, ch),
        None => notif.app_name.clone(),
    };
    let badge_char = notif
        .app_name
        .chars()
        .next()
        .unwrap_or('N')
        .to_ascii_uppercase()
        .to_string();

    let header_display = truncate_preview(&header_left, 24);
    let sender_display = truncate_preview(&notif.sender, 28);
    let message_display = truncate_preview(&notif.message, 38);

    row()
        .key("content-notification")
        .w_full()
        .h_full()
        .px(16.0)
        .gap(12.0)
        .center_y()
        .child(
            el().size(44.0, 44.0)
                .radius(14.0)
                .fill(NOTIF_SLACK_BG)
                .center()
                .child(
                    text(badge_char)
                        .font(TextStyle::new(18.0, 700.0))
                        .color(TEXT_PRIMARY),
                ),
        )
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(2.0)
                .child(
                    row()
                        .w_full()
                        .min_w(0.0)
                        .child(text(header_display).font(FONT_META).color(TEXT_MUTED))
                        .child(el().grow(1.0))
                        .child(text("now").font(FONT_META).color(TEXT_MUTED)),
                )
                .child(
                    text(sender_display)
                        .font(FONT_PRIMARY_LINE)
                        .color(TEXT_PRIMARY),
                )
                .child(text(message_display).font(FONT_BODY).color(TEXT_SECONDARY)),
        )
}

fn render_upcoming_event(event: &CalendarMeeting, accent: Color) -> El {
    let join_url = event.join_url.clone();
    let title_display = truncate_preview(&event.title, 18);
    let sub_str = format!("in 12 min · {}", event.service_name);
    let sub_display = truncate_preview(&sub_str, 22);

    row()
        .key("content-upcoming-event")
        .w_full()
        .h_full()
        .px(12.0)
        .gap(12.0)
        .center_y()
        .child(
            col()
                .size(38.0, 38.0)
                .radius(12.0)
                .fill(CAL_TILE_BG)
                .center()
                .child(
                    text("NOV")
                        .font(TextStyle::new(9.0, 700.0).tracking(0.6))
                        .color(accent),
                )
                .child(
                    text("15")
                        .font(TextStyle::new(15.0, 700.0))
                        .color(TEXT_PRIMARY),
                ),
        )
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(2.0)
                .child(
                    text(title_display)
                        .font(FONT_PRIMARY_LINE)
                        .color(TEXT_PRIMARY),
                )
                .child(text(sub_display).font(FONT_META).color(TEXT_MUTED)),
        )
        .child(
            el().key("event-btn-join")
                .role("button")
                .label("Join meeting")
                .size(64.0, 44.0)
                .radius(22.0)
                .fill(accent)
                .center()
                .on_press(move |_: &mut AppState| {
                    #[cfg(windows)]
                    if let Some(url) = &join_url {
                        let _ = open_external_url(url);
                    }
                })
                .child(text("Join").font(FONT_BODY_SEMIBOLD).color(ISLAND_BG)),
        )
}

fn render_timer(timer: &TimerState) -> El {
    let mins = timer.remaining_seconds / 60;
    let secs = timer.remaining_seconds % 60;
    let label_display = truncate_preview(&timer.label, 14);

    row()
        .key("content-timer")
        .w_full()
        .h_full()
        .px(16.0)
        .center_y()
        .child(
            row()
                .grow(1.0)
                .min_w(0.0)
                .gap(8.0)
                .center_y()
                .child(icon(&ICON_TIMER, 18.0).color(ACCENT_WARM))
                .child(
                    text(label_display)
                        .font(FONT_BODY_SEMIBOLD)
                        .color(TEXT_PRIMARY),
                ),
        )
        .child(
            text(format!("{mins:02}:{secs:02}"))
                .font(FONT_TIMER)
                .color(ACCENT_WARM),
        )
}

fn render_hud(hud: &HudState) -> El {
    let bar_fill_w = 160.0 * (f32::from(hud.level_percent) / 100.0);
    row()
        .key("content-hud")
        .w_full()
        .h_full()
        .px(16.0)
        .gap(12.0)
        .center_y()
        .child(icon(&ICON_VOLUME, 18.0).color(TEXT_PRIMARY))
        .child(
            el().size(160.0, 6.0)
                .radius(3.0)
                .fill(TRACK_BG)
                .child(el().size(bar_fill_w, 6.0).radius(3.0).fill(TEXT_PRIMARY)),
        )
        .child(el().grow(1.0))
        .child(
            text(format!("{}", hud.level_percent))
                .font(FONT_META_TABULAR)
                .color(TEXT_MUTED),
        )
}

fn render_hub(state: &AppState, accent: Color) -> El {
    let default_media = sample_media();
    let media_opt = state.state_machine.latest_media();
    let active_media = media_opt.unwrap_or(&default_media);
    let raw_source = if active_media.source_app.is_empty() {
        "MUSIC".to_string()
    } else {
        active_media.source_app.to_ascii_uppercase()
    };
    let source_app = truncate_preview(&raw_source, 16);
    let ratio = if active_media.duration_secs > 0.0 {
        (active_media.position_secs / active_media.duration_secs).clamp(0.0, 1.0) as f32
    } else {
        0.42
    };
    let fill_px = (216.0 * ratio).round().clamp(0.0, 216.0);
    let pos_label = format_mm_ss(active_media.position_secs);
    let dur_label = format_mm_ss(active_media.duration_secs);
    let play_pause_icon = if active_media.is_playing {
        &ICON_PAUSE
    } else {
        &ICON_PLAY
    };
    let is_currently_playing = active_media.is_playing;

    // Mini waveform for alongside artist name
    let waveform_heights: [f32; 7] = [5.0, 9.0, 7.0, 11.0, 6.0, 10.0, 4.0];
    let mut mini_wave = row().gap(2.0).center_y().h(12.0);
    for (idx, &bar_h) in waveform_heights.iter().enumerate() {
        let h = if active_media.is_playing { bar_h } else { 3.0 };
        mini_wave = mini_wave.child(
            el().key(format!("mwave-{idx}"))
                .size(2.0, h)
                .radius(1.5)
                .fill(accent)
                .motion(PILL_SPRING),
        );
    }

    // Media app control toggle chips (arranged in 2 rows to prevent horizontal overflow)
    let make_media_chip = |app_name: &'static str| {
        let is_enabled = state.state_machine.is_media_app_enabled(app_name);
        let chip_bg = if is_enabled { PILL_BG } else { TRACK_BG };
        let chip_border = if is_enabled {
            CAL_FEATURED_BORDER
        } else {
            ISLAND_BORDER
        };
        let chip_color = if is_enabled { accent } else { TEXT_MUTED };
        let chip_label = if is_enabled {
            format!("{app_name} ✓")
        } else {
            format!("{app_name} ✕")
        };
        let app_str = app_name.to_string();
        el().key(format!("chip-media-{app_name}"))
            .role("button")
            .label(format!("Toggle media control for {app_name}"))
            .px(7.0)
            .py(2.0)
            .radius(7.0)
            .fill(chip_bg)
            .border(1.0, chip_border)
            .on_press(move |s: &mut AppState| {
                s.state_machine.toggle_media_app(&app_str);
                let _ = s.state_machine.settings().save_default();
            })
            .child(text(chip_label).font(FONT_CHIP).color(chip_color))
    };

    let media_chips = col()
        .w_full()
        .min_w(0.0)
        .gap(4.0)
        .child(
            row()
                .gap(5.0)
                .center_y()
                .child(make_media_chip("Spotify"))
                .child(make_media_chip("Chrome")),
        )
        .child(
            row()
                .gap(5.0)
                .center_y()
                .child(make_media_chip("Edge"))
                .child(make_media_chip("VLC")),
        );

    let title_display = truncate_preview(&active_media.title, 18);
    let artist_display = truncate_preview(&active_media.artist, 14);

    // Card 1: Now Playing
    let media_card = col()
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .padding(12.0)
        .gap(6.0)
        .radius(22.0)
        .fill(CARD_SURFACE)
        .child(
            row()
                .w_full()
                .min_w(0.0)
                .center_y()
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .child(icon(&ICON_MUSIC, 11.0).color(TEXT_MUTED))
                        .child(text(source_app).font(FONT_CAPS_LABEL).color(TEXT_MUTED)),
                )
                .child(el().grow(1.0))
                .child(
                    el().px(5.0)
                        .py(1.0)
                        .radius(4.0)
                        .fill(PILL_BG)
                        .border(1.0, CAL_FEATURED_BORDER)
                        .child(text("FLAC 24-bit").font(FONT_CHIP).color(accent)),
                ),
        )
        .child(media_chips)
        .child(
            row()
                .w_full()
                .min_w(0.0)
                .gap(10.0)
                .center_y()
                .child(render_album_tile(
                    52.0,
                    12.0,
                    22.0,
                    active_media.thumbnail_bytes.as_deref(),
                ))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(3.0)
                        .child(
                            text(title_display)
                                .font(FONT_CARD_TITLE)
                                .color(TEXT_PRIMARY),
                        )
                        .child(
                            row()
                                .gap(6.0)
                                .center_y()
                                .child(mini_wave)
                                .child(text(artist_display).font(FONT_BODY).color(TEXT_MUTED)),
                        ),
                ),
        )
        .child(
            col()
                .gap(3.0)
                .child(
                    el().w_full()
                        .h(4.0)
                        .radius(2.0)
                        .fill(TRACK_BG)
                        .child(el().size(fill_px, 4.0).radius(2.0).fill(accent)),
                )
                .child(
                    row()
                        .w_full()
                        .child(text(pos_label).font(FONT_META_TABULAR).color(TEXT_MUTED))
                        .child(el().grow(1.0))
                        .child(text(dur_label).font(FONT_META_TABULAR).color(TEXT_MUTED)),
                ),
        )
        .child(
            row()
                .w_full()
                .center()
                .gap(16.0)
                .child(
                    el().key("hub-btn-prev")
                        .role("button")
                        .label("Previous track")
                        .size(34.0, 34.0)
                        .center()
                        .on_press(|_: &mut AppState| {
                            #[cfg(windows)]
                            std::thread::spawn(|| {
                                let _ = WinSmtcMediaProvider::new().previous_track();
                            });
                        })
                        .child(icon(&ICON_PREV, 16.0).color(TEXT_SECONDARY)),
                )
                .child(
                    el().key("hub-btn-play-pause")
                        .role("button")
                        .label(if is_currently_playing {
                            "Pause"
                        } else {
                            "Play"
                        })
                        .size(38.0, 38.0)
                        .radius(19.0)
                        .fill(accent)
                        .center()
                        .on_press(move |s: &mut AppState| {
                            if let Some(mut m) = s.state_machine.latest_media().cloned() {
                                m.is_playing = !m.is_playing;
                                s.state_machine.set_media(Some(m));
                            }
                            #[cfg(windows)]
                            std::thread::spawn(move || {
                                let smtc = WinSmtcMediaProvider::new();
                                if is_currently_playing {
                                    let _ = smtc.pause();
                                } else {
                                    let _ = smtc.play();
                                }
                            });
                        })
                        .child(icon(play_pause_icon, 16.0).color(ISLAND_BG)),
                )
                .child(
                    el().key("hub-btn-next")
                        .role("button")
                        .label("Next track")
                        .size(34.0, 34.0)
                        .center()
                        .on_press(|_: &mut AppState| {
                            #[cfg(windows)]
                            std::thread::spawn(|| {
                                let _ = WinSmtcMediaProvider::new().next_track();
                            });
                        })
                        .child(icon(&ICON_NEXT, 16.0).color(TEXT_SECONDARY)),
                ),
        )
        .child(el().grow(1.0))
        .child(
            row()
                .w_full()
                .min_w(0.0)
                .center_y()
                .child(
                    row()
                        .gap(5.0)
                        .center_y()
                        .child(icon(&ICON_HEADPHONES, 11.0).color(accent))
                        .child(
                            text("Default Endpoint")
                                .font(FONT_META)
                                .color(TEXT_SECONDARY),
                        ),
                )
                .child(el().grow(1.0))
                .child(
                    row()
                        .gap(4.0)
                        .center_y()
                        .child(icon(&ICON_VOLUME, 11.0).color(TEXT_MUTED))
                        .child(text("74%").font(FONT_META_TABULAR).color(TEXT_MUTED)),
                ),
        );

    // Card 2: Today's Calendar
    let today_events = state.state_machine.upcoming_events();
    let bar_colors = [accent, ACCENT_WARM, CAL_NEUTRAL_BAR];
    let (featured_title, featured_sub, featured_svc, secondary_rows) = if today_events.len() >= 2 {
        let feat = &today_events[0];
        let feat_sub = format!("{} min", feat.duration_minutes);
        let secondary: Vec<(String, String, Color)> = today_events
            .iter()
            .skip(1)
            .take(2)
            .enumerate()
            .map(|(i, evt)| {
                let t = format!("{} min · {}", evt.duration_minutes, evt.service_name);
                (evt.title.clone(), t, bar_colors[(i + 1) % bar_colors.len()])
            })
            .collect();
        (
            feat.title.clone(),
            feat_sub,
            feat.service_name.clone(),
            secondary,
        )
    } else if let Some(evt) = today_events.first() {
        (
            evt.title.clone(),
            format!("{} min", evt.duration_minutes),
            evt.service_name.clone(),
            vec![
                ("Gym".to_string(), "8:30 – 9:30 PM".to_string(), ACCENT_WARM),
                (
                    "Call with Mom".to_string(),
                    "10:00 PM".to_string(),
                    CAL_NEUTRAL_BAR,
                ),
            ],
        )
    } else {
        (
            "Design review".to_string(),
            "40 min".to_string(),
            "Microsoft Teams".to_string(),
            vec![
                ("Gym".to_string(), "8:30 – 9:30 PM".to_string(), ACCENT_WARM),
                (
                    "Call with Mom".to_string(),
                    "10:00 PM".to_string(),
                    CAL_NEUTRAL_BAR,
                ),
            ],
        )
    };

    let feat_title_disp = truncate_preview(&featured_title, 15);
    let feat_meta_disp = truncate_preview(&format!("{} · {}", featured_sub, featured_svc), 20);

    let mut today_card = col()
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .padding(12.0)
        .gap(6.0)
        .radius(22.0)
        .fill(CARD_SURFACE)
        .child(
            row()
                .w_full()
                .min_w(0.0)
                .center_y()
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .child(icon(&ICON_CALENDAR, 11.0).color(TEXT_MUTED))
                        .child(
                            text(truncate_preview(&state.today_header, 20))
                                .font(FONT_CAPS_LABEL)
                                .color(TEXT_MUTED),
                        ),
                )
                .child(el().grow(1.0))
                .child(
                    row()
                        .gap(4.0)
                        .center_y()
                        .child(el().size(5.0, 5.0).radius(2.5).fill(ACCENT_LIME))
                        .child(text(".ics Synced").font(FONT_CHIP).color(TEXT_MUTED)),
                ),
        )
        .child(
            col()
                .w_full()
                .min_w(0.0)
                .padding(8.0)
                .gap(3.0)
                .radius(12.0)
                .fill(CAL_TILE_BG)
                .border(1.0, CAL_FEATURED_BORDER)
                .child(
                    row()
                        .w_full()
                        .min_w(0.0)
                        .center_y()
                        .child(text("Starts in 18 min").font(FONT_CHIP).color(accent))
                        .child(el().grow(1.0))
                        .child(text("11:00 AM").font(FONT_META).color(TEXT_MUTED)),
                )
                .child(
                    row()
                        .w_full()
                        .min_w(0.0)
                        .center_y()
                        .child(
                            col().grow(1.0).min_w(0.0).child(
                                text(feat_title_disp)
                                    .font(FONT_PRIMARY_LINE)
                                    .color(TEXT_PRIMARY),
                            ),
                        )
                        .child(
                            el().px(7.0)
                                .py(2.0)
                                .radius(8.0)
                                .fill(accent)
                                .child(text("JOIN →").font(FONT_CHIP).color(ISLAND_BG)),
                        ),
                )
                .child(text(feat_meta_disp).font(FONT_META).color(TEXT_MUTED)),
        );

    for (title, sub, bar_color) in secondary_rows {
        let t_disp = truncate_preview(&title, 18);
        let s_disp = truncate_preview(&sub, 22);
        today_card = today_card.child(
            row()
                .w_full()
                .min_w(0.0)
                .gap(8.0)
                .center_y()
                .child(el().size(3.0, 24.0).radius(1.5).fill(bar_color))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(text(t_disp).font(FONT_BODY_SEMIBOLD).color(TEXT_PRIMARY))
                        .child(text(s_disp).font(FONT_META).color(TEXT_MUTED)),
                ),
        );
    }

    today_card = today_card.child(el().grow(1.0)).child(
        row()
            .w_full()
            .min_w(0.0)
            .center_y()
            .child(
                row()
                    .gap(4.0)
                    .center_y()
                    .child(icon(&ICON_CHECK, 11.0).color(ACCENT_LIME))
                    .child(text("No conflicts").font(FONT_META).color(ACCENT_LIME)),
            )
            .child(el().grow(1.0))
            .child(
                el().key("hub-btn-open-cal")
                    .role("button")
                    .label("Open calendar")
                    .on_press(|_: &mut AppState| {
                        #[cfg(windows)]
                        let _ = open_external_url("https://calendar.google.com");
                    })
                    .child(text("Open calendar →").font(FONT_META).color(accent)),
            ),
    );

    // Card 3: Recent Notifications + App Notification Filters (arranged in 2 rows to prevent overflow)
    let make_notif_chip = |app_name: &'static str| {
        let is_enabled = state.state_machine.is_notification_app_enabled(app_name);
        let chip_bg = if is_enabled { PILL_BG } else { TRACK_BG };
        let chip_border = if is_enabled {
            CAL_FEATURED_BORDER
        } else {
            ISLAND_BORDER
        };
        let chip_color = if is_enabled { accent } else { TEXT_MUTED };
        let chip_label = if is_enabled {
            format!("{app_name} ✓")
        } else {
            format!("{app_name} ✕")
        };
        let app_str = app_name.to_string();
        el().key(format!("chip-notif-{app_name}"))
            .role("button")
            .label(format!("Toggle notifications for {app_name}"))
            .px(6.0)
            .py(2.0)
            .radius(7.0)
            .fill(chip_bg)
            .border(1.0, chip_border)
            .on_press(move |s: &mut AppState| {
                s.state_machine.toggle_notification_app(&app_str);
                let _ = s.state_machine.settings().save_default();
            })
            .child(text(chip_label).font(FONT_CHIP).color(chip_color))
    };

    let notif_chips = col()
        .w_full()
        .min_w(0.0)
        .gap(4.0)
        .child(
            row()
                .gap(4.0)
                .center_y()
                .child(make_notif_chip("Slack"))
                .child(make_notif_chip("WhatsApp")),
        )
        .child(
            row()
                .gap(4.0)
                .center_y()
                .child(make_notif_chip("Teams"))
                .child(make_notif_chip("Discord"))
                .child(make_notif_chip("Outlook")),
        );

    let tile_bgs = [NOTIF_SLACK_BG, NOTIF_WHATSAPP_BG, NOTIF_GITHUB_BG];
    let notifications = state.state_machine.active_notifications();
    let notif_count = notifications.len().clamp(1, 99);
    let notif_rows: Vec<(String, String, String, String, Color)> = if notifications.len() >= 2 {
        notifications
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, n)| {
                let badge = n
                    .app_name
                    .chars()
                    .next()
                    .unwrap_or('N')
                    .to_ascii_uppercase()
                    .to_string();
                (
                    badge,
                    n.app_name.clone(),
                    n.sender.clone(),
                    n.message.clone(),
                    tile_bgs[i % tile_bgs.len()],
                )
            })
            .collect()
    } else {
        let first_sender = notifications
            .first()
            .map(|n| n.sender.clone())
            .unwrap_or_else(|| "Priya Sharma".to_string());
        let first_msg = notifications
            .first()
            .map(|n| n.message.clone())
            .unwrap_or_else(|| "Can you share the island PRD…".to_string());
        vec![
            (
                "S".to_string(),
                "Slack".to_string(),
                first_sender,
                first_msg,
                NOTIF_SLACK_BG,
            ),
            (
                "W".to_string(),
                "WhatsApp".to_string(),
                "Family".to_string(),
                "Dinner at 9?".to_string(),
                NOTIF_WHATSAPP_BG,
            ),
            (
                "G".to_string(),
                "GitHub".to_string(),
                "PR #42 approved".to_string(),
                "cay-app: merged".to_string(),
                NOTIF_GITHUB_BG,
            ),
        ]
    };
    let timestamps = ["2m", "14m", "1h"];

    let is_dnd = state.state_machine.is_do_not_disturb();
    let dnd_color = if is_dnd { ACCENT_WARM } else { TEXT_MUTED };
    let dnd_label = if is_dnd { "DND: On" } else { "DND: Off" };
    let dnd_btn_bg = if is_dnd { PILL_BG } else { TRACK_BG };
    let dnd_border = if is_dnd {
        CAL_FEATURED_BORDER
    } else {
        ISLAND_BORDER
    };

    let mut notif_card = col()
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .padding(12.0)
        .gap(6.0)
        .radius(22.0)
        .fill(CARD_SURFACE)
        .child(
            row()
                .w_full()
                .gap(6.0)
                .center_y()
                .child(icon(&ICON_BELL, 11.0).color(TEXT_MUTED))
                .child(text("UNREAD").font(FONT_CAPS_LABEL).color(TEXT_MUTED))
                .child(
                    el().px(6.0).py(2.0).radius(8.0).fill(NOTIF_SLACK_BG).child(
                        text(format!("{notif_count}"))
                            .font(FONT_CAPS_LABEL)
                            .color(TEXT_PRIMARY),
                    ),
                )
                .child(el().grow(1.0))
                .child(
                    el().key("hub-btn-clear")
                        .role("button")
                        .label("Clear notifications")
                        .on_press(|s: &mut AppState| {
                            s.state_machine.clear_notifications();
                            #[cfg(windows)]
                            std::thread::spawn(|| {
                                let _ = WinNotificationProvider::new().clear_all();
                            });
                        })
                        .child(text("Clear All").font(FONT_META).color(TEXT_MUTED)),
                ),
        )
        .child(notif_chips);

    for (i, (badge, app_name, sender, msg, tile_bg)) in notif_rows.into_iter().enumerate() {
        let sender_disp = truncate_preview(&sender, 16);
        let preview_disp = truncate_preview(&msg, 20);
        let ts = timestamps.get(i).copied().unwrap_or("?");
        let _ = app_name;
        notif_card = notif_card.child(
            row()
                .w_full()
                .min_w(0.0)
                .gap(8.0)
                .center_y()
                .child(
                    el().size(28.0, 28.0)
                        .radius(9.0)
                        .fill(tile_bg)
                        .center()
                        .child(text(badge).font(FONT_BODY_SEMIBOLD).color(TEXT_PRIMARY)),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(1.0)
                        .child(
                            text(sender_disp)
                                .font(FONT_BODY_SEMIBOLD)
                                .color(TEXT_PRIMARY),
                        )
                        .child(text(preview_disp).font(FONT_META).color(TEXT_MUTED)),
                )
                .child(text(ts).font(FONT_META_TABULAR).color(TEXT_MUTED)),
        );
    }

    notif_card = notif_card.child(el().grow(1.0)).child(
        row()
            .w_full()
            .min_w(0.0)
            .center_y()
            .child(
                row()
                    .gap(4.0)
                    .center_y()
                    .child(icon(&ICON_DND, 11.0).color(dnd_color))
                    .child(text(dnd_label).font(FONT_META).color(dnd_color)),
            )
            .child(el().grow(1.0))
            .child(
                el().key("hub-btn-toggle-dnd")
                    .role("button")
                    .label("Toggle Do Not Disturb")
                    .px(7.0)
                    .py(2.0)
                    .radius(7.0)
                    .fill(dnd_btn_bg)
                    .border(1.0, dnd_border)
                    .on_press(|s: &mut AppState| {
                        let next = !s.state_machine.is_do_not_disturb();
                        s.state_machine.set_do_not_disturb(next);
                        let _ = s.state_machine.settings().save_default();
                    })
                    .child(
                        text(if is_dnd { "Active" } else { "Toggle" })
                            .font(FONT_CHIP)
                            .color(dnd_color),
                    ),
            ),
    );

    let top_bar = row()
        .w_full()
        .min_w(0.0)
        .h(16.0)
        .px(2.0)
        .center_y()
        .child(
            row()
                .gap(3.0)
                .center_y()
                .child(el().size(3.0, 3.0).radius(1.5).fill(TEXT_MUTED))
                .child(el().size(3.0, 3.0).radius(1.5).fill(TEXT_MUTED))
                .child(el().size(3.0, 3.0).radius(1.5).fill(TEXT_MUTED))
                .child(el().size(3.0, 3.0).radius(1.5).fill(TEXT_MUTED))
                .child(
                    text("  CAY · DYNAMIC HUD DECK")
                        .font(FONT_CAPS_LABEL)
                        .color(TEXT_MUTED),
                ),
        )
        .child(el().grow(1.0))
        .child(
            row()
                .gap(8.0)
                .center_y()
                .child(text("Esc to collapse").font(FONT_META).color(TEXT_MUTED))
                .child(
                    el().key("hub-btn-close")
                        .role("button")
                        .label("Close Hub")
                        .size(18.0, 18.0)
                        .radius(9.0)
                        .fill(TRACK_BG)
                        .center()
                        .on_press(|s: &mut AppState| {
                            s.state_machine.close_hub();
                        })
                        .child(icon(&ICON_CLOSE, 10.0).color(TEXT_MUTED)),
                ),
        );

    let bottom_bar = row()
        .w_full()
        .min_w(0.0)
        .h(14.0)
        .px(2.0)
        .center_y()
        .child(
            row()
                .gap(6.0)
                .center_y()
                .child(el().size(5.0, 5.0).radius(2.5).fill(ACCENT_LIME))
                .child(
                    text("DirectX 12 Composited Overlay · GPU: 0.4%")
                        .font(FONT_META)
                        .color(TEXT_MUTED),
                ),
        )
        .child(el().grow(1.0))
        .child(
            row()
                .gap(6.0)
                .center_y()
                .child(
                    el().key("hub-btn-toggle-shortcut")
                        .role("button")
                        .label("Toggle deck shortcut")
                        .px(5.0)
                        .py(1.0)
                        .radius(4.0)
                        .fill(TRACK_BG)
                        .on_press(|s: &mut AppState| {
                            s.state_machine.toggle_hub();
                        })
                        .child(text("Ctrl + Alt + I").font(FONT_CHIP).color(accent)),
                )
                .child(
                    el().key("hub-btn-toggle-label")
                        .role("button")
                        .label("Toggle deck")
                        .on_press(|s: &mut AppState| {
                            s.state_machine.toggle_hub();
                        })
                        .child(text("to toggle deck").font(FONT_META).color(TEXT_MUTED)),
                ),
        );

    col()
        .key("content-hub")
        .w_full()
        .h_full()
        .padding(12.0)
        .gap(6.0)
        .child(top_bar)
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_w(0.0)
                .gap(10.0)
                .child(media_card)
                .child(today_card)
                .child(notif_card),
        )
        .child(bottom_bar)
}

fn island_view(state: &AppState, _: &mut Cx<AppState>) -> El {
    let current = state.state_machine.current_state();
    let IslandDimensions {
        width,
        height,
        radius,
    } = current.dimensions();
    let accent = resolve_accent_color(state.state_machine.settings().accent);

    let inner = match &current {
        IslandState::Idle { battery } => render_idle(&state.clock_text, battery),
        IslandState::NowPlaying { media } => {
            render_now_playing(media, accent, state.waveform_phase)
        }
        IslandState::Notification { notification } => render_notification(notification),
        IslandState::UpcomingEvent { event } => render_upcoming_event(event, accent),
        IslandState::Timer { timer } => render_timer(timer),
        IslandState::Hud { hud } => render_hud(hud),
        IslandState::Hub { .. } => render_hub(state, accent),
    };

    let pill = el()
        .key("cay-island-pill")
        .hit()
        .role("button")
        .label("Cay floating island")
        .size(width, height)
        .radius(radius)
        .fill(ISLAND_BG)
        .border(1.0, ISLAND_BORDER)
        .motion(PILL_SPRING)
        .on_hover_change(|s: &mut AppState, inside, _| {
            s.state_machine.set_hovered(inside);
        })
        .on_press(|s: &mut AppState| {
            s.state_machine.toggle_hub();
        })
        .child(inner);

    col().w_full().h_full().center_y().child(pill)
}

fn build_tray_icon() -> TrayIcon {
    let (w, h) = (32_u32, 32_u32);
    let mut rgba = vec![0_u8; (w * h * 4) as usize];
    for y in 8..24 {
        for x in 3..29 {
            let idx = ((y * w + x) * 4) as usize;
            // Solid dark pill (#06061A) with sky accent dot (#3DD8FF)
            if (13..19).contains(&y) && (20..26).contains(&x) {
                rgba[idx] = 61;
                rgba[idx + 1] = 216;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            } else {
                rgba[idx] = 6;
                rgba[idx + 1] = 6;
                rgba[idx + 2] = 26;
                rgba[idx + 3] = 255;
            }
        }
    }
    TrayIcon::rgba(w, h, rgba)
}

fn build_tray_menu() -> Vec<TrayItem> {
    vec![
        TrayItem::action("toggle_hub", "Toggle Expanded Hub"),
        TrayItem::Separator,
        TrayItem::action("timer_start_25", "Focus Timer: Start 25 min"),
        TrayItem::action("timer_pause_resume", "Focus Timer: Pause / Resume"),
        TrayItem::action("timer_stop", "Focus Timer: Stop"),
        TrayItem::Separator,
        TrayItem::action("notif_slack", "Notif App: Slack (Toggle)"),
        TrayItem::action("notif_whatsapp", "Notif App: WhatsApp (Toggle)"),
        TrayItem::action("notif_teams", "Notif App: Teams (Toggle)"),
        TrayItem::action("notif_discord", "Notif App: Discord (Toggle)"),
        TrayItem::action("notif_outlook", "Notif App: Outlook (Toggle)"),
        TrayItem::Separator,
        TrayItem::action("media_spotify", "Media Control: Spotify (Toggle)"),
        TrayItem::action("media_chrome", "Media Control: Chrome (Toggle)"),
        TrayItem::action("media_edge", "Media Control: Edge (Toggle)"),
        TrayItem::action("media_vlc", "Media Control: VLC (Toggle)"),
        TrayItem::Separator,
        TrayItem::action("accent_sky", "Accent: Sky (#3DD8FF)"),
        TrayItem::action("accent_lime", "Accent: Lime (#74FF38)"),
        TrayItem::action("accent_warm", "Accent: Warm (#FF9A47)"),
        TrayItem::action("accent_purple", "Accent: Purple (#BF72FF)"),
        TrayItem::Separator,
        TrayItem::action("monitor_primary", "Display: Primary Monitor"),
        TrayItem::action("monitor_pointer", "Display: Follow Pointer Monitor"),
        TrayItem::Separator,
        TrayItem::action("toggle_dnd", "Toggle Do Not Disturb (Suppress Peeks)"),
        TrayItem::action("toggle_hover", "Toggle Hover Expansion"),
        TrayItem::action("enable_autostart", "Enable Start with Windows"),
        TrayItem::action("disable_autostart", "Disable Start with Windows"),
        TrayItem::Separator,
        TrayItem::action("quit", "Quit Cay"),
    ]
}

fn handle_key(s: &mut AppState, event: &KeyEvent, _: &mut Tx) -> bool {
    if event.key == Key::Escape && s.state_machine.is_hub_open() {
        s.state_machine.close_hub();
        return true;
    }
    false
}

fn spec() -> AppSpec<AppState> {
    AppSpec {
        on_key: Some(handle_key),
        scenes: &[
            ("idle", |s| {
                *s = AppState::default();
            }),
            ("now_playing", |s| {
                s.state_machine.set_media(Some(sample_media()));
            }),
            ("notification", |s| {
                s.state_machine.push_notification(sample_notification());
            }),
            ("upcoming_event", |s| {
                s.state_machine.set_upcoming_event(Some(sample_event()));
            }),
            ("timer", |s| {
                s.state_machine.set_timer(Some(TimerState {
                    label: "Focus".to_string(),
                    remaining_seconds: 24 * 60 + 12,
                    is_running: true,
                }));
            }),
            ("hud", |s| {
                s.state_machine.trigger_hud(HudState {
                    kind: HudKind::Volume,
                    level_percent: 60,
                });
            }),
            ("hub", |s| {
                s.state_machine.set_media(Some(sample_media()));
                s.state_machine.set_upcoming_event(Some(sample_event()));
                s.state_machine.push_notification(sample_notification());
                s.state_machine.toggle_hub();
            }),
        ],
        ..AppSpec::new("Cay", (860.0, 360.0), AppState::default, island_view)
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    rune::run_windows_with(RunOptions { accessory: true }, |_| {
        let mut app = App::new(spec()).windowless();

        let mut initial_state = AppState::default();
        let loaded_settings = IslandSettings::load_default();
        let top_margin = loaded_settings.top_offset_px.max(0.0);
        let initial_display = match loaded_settings.monitor_choice {
            MonitorChoice::Primary => DisplayTarget::Primary,
            MonitorChoice::UnderPointer => DisplayTarget::UnderPointer,
        };
        initial_state.state_machine.set_settings(loaded_settings);

        let pill = app.overlay(
            OverlaySpec {
                name: "Cay Island",
                size: (860.0, 360.0),
                edge: ScreenEdge::Top,
                margin: top_margin,
                align: Align::Center,
                inset: 0.0,
                interactive: true,
                view: island_view,
            }
            .options(OverlayOptions {
                display: initial_display,
                input: OverlayInput::Regions,
                prewarm: true,
                ..Default::default()
            })
            .on_key(handle_key),
            initial_state,
        );

        pill.show();

        // Register global keyboard shortcuts (Ctrl + Alt + I, Win + Shift + I, Win + Alt + I) to toggle the Expanded Hub
        #[cfg(windows)]
        {
            let hotkey_pill = pill.clone();
            spawn_global_hotkey_listener(move || {
                hotkey_pill.update(|s, _| s.state_machine.toggle_hub());
            });
        }

        // Initialize Windows System Tray Icon & Context Menu
        let tray_pill = pill.clone();
        let tray_app = app.handle();
        let tray = Tray::new(
            TraySpec::new("cay.island", "Cay", build_tray_icon())
                .tooltip("Cay Floating Island")
                .menu(build_tray_menu()),
            move |event| match event {
                TrayEvent::Activate { .. } => {
                    tray_pill.update(|s, _| s.state_machine.toggle_hub());
                }
                TrayEvent::Menu(id) => match id.as_str() {
                    "toggle_hub" => tray_pill.update(|s, _| s.state_machine.toggle_hub()),
                    "timer_start_25" => tray_pill.update(|s, _| {
                        s.state_machine.start_focus_timer(25 * 60, "Focus");
                    }),
                    "timer_pause_resume" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_timer_pause();
                    }),
                    "timer_stop" => tray_pill.update(|s, _| {
                        s.state_machine.stop_timer();
                    }),
                    "notif_slack" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_notification_app("Slack");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "notif_whatsapp" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_notification_app("WhatsApp");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "notif_teams" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_notification_app("Teams");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "notif_discord" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_notification_app("Discord");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "notif_outlook" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_notification_app("Outlook");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "media_spotify" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_media_app("Spotify");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "media_chrome" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_media_app("Chrome");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "media_edge" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_media_app("Edge");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "media_vlc" => tray_pill.update(|s, _| {
                        s.state_machine.toggle_media_app("VLC");
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "accent_sky" => tray_pill.update(|s, _| {
                        s.state_machine.set_accent(AccentPreset::Sky);
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "accent_lime" => tray_pill.update(|s, _| {
                        s.state_machine.set_accent(AccentPreset::Lime);
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "accent_warm" => tray_pill.update(|s, _| {
                        s.state_machine.set_accent(AccentPreset::Warm);
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "accent_purple" => tray_pill.update(|s, _| {
                        s.state_machine.set_accent(AccentPreset::Purple);
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "monitor_primary" => {
                        tray_pill.set_display(DisplayTarget::Primary);
                        tray_pill.update(|s, _| {
                            s.state_machine.set_monitor_choice(MonitorChoice::Primary);
                            let _ = s.state_machine.settings().save_default();
                        });
                    }
                    "monitor_pointer" => {
                        tray_pill.set_display(DisplayTarget::UnderPointer);
                        tray_pill.refollow_pointer();
                        tray_pill.update(|s, _| {
                            s.state_machine
                                .set_monitor_choice(MonitorChoice::UnderPointer);
                            let _ = s.state_machine.settings().save_default();
                        });
                    }
                    "toggle_dnd" => tray_pill.update(|s, _| {
                        let next = !s.state_machine.is_do_not_disturb();
                        s.state_machine.set_do_not_disturb(next);
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "toggle_hover" => tray_pill.update(|s, _| {
                        let next = !s.state_machine.settings().hover_expand_enabled;
                        s.state_machine.set_hover_expand_enabled(next);
                        let _ = s.state_machine.settings().save_default();
                    }),
                    "enable_autostart" => {
                        #[cfg(windows)]
                        let _ = set_windows_autostart(true);
                    }
                    "disable_autostart" => {
                        #[cfg(windows)]
                        let _ = set_windows_autostart(false);
                    }
                    "quit" => tray_app.quit(),
                    _ => {}
                },
                _ => {}
            },
        );
        // Keep Tray handle alive for the lifetime of the application
        if let Ok(tray_handle) = tray {
            std::mem::forget(tray_handle);
        }

        // Spawn background MTA worker thread for Windows providers
        #[cfg(windows)]
        {
            let worker_pill = pill.clone();
            let app_handle = app.handle();
            let is_spike_probe = std::env::var_os("CAY_SPIKE_PROBE").is_some();

            let _ = std::thread::Builder::new()
                .name("cay-provider-worker".into())
                .spawn(move || {
                    init_mta();
                    let smtc = WinSmtcMediaProvider::new();
                    let sys = WinSystemProvider::new();
                    let notif = WinNotificationProvider::new();
                    let cal = IcsCalendarProvider::new(
                        IslandSettings::load_default()
                            .ics_source
                            .map(std::path::PathBuf::from)
                            .or_else(|| {
                                std::env::var_os("CAY_ICS_PATH").map(std::path::PathBuf::from)
                            }),
                    );

                    if is_spike_probe {
                        std::thread::sleep(std::time::Duration::from_millis(600));
                        let ram_mb = process_working_set_bytes()
                            .map(|b| b as f64 / (1024.0 * 1024.0))
                            .unwrap_or(0.0);
                        let battery_res = sys.get_battery_status();
                        let smtc_res = smtc.get_current_media();
                        let notif_access = notif.probe_access_status();
                        let (clock_now, today_now) = current_local_clock_and_date();
                        let fullscreen_now = is_foreground_window_fullscreen();
                        let volume_now = get_system_master_volume_percent();
                        let quiet_hours_now = is_windows_quiet_hours_active();

                        tracing::info!(
                            availability = ?worker_pill.availability(),
                            failure = ?worker_pill.failure(),
                            surface = ?worker_pill.surface_description(),
                            capabilities = ?worker_pill.capabilities(),
                            working_set_mb = format!("{ram_mb:.2}"),
                            clock = %clock_now,
                            today = %today_now,
                            fullscreen_active = fullscreen_now,
                            quiet_hours_active = quiet_hours_now,
                            volume_percent = ?volume_now,
                            battery = ?battery_res,
                            smtc_media = ?smtc_res,
                            notification_access = ?notif_access,
                            "Phase 0 Spike: Overlay & WinRT providers probed"
                        );

                        let morph_media = smtc_res.ok().flatten().unwrap_or_else(sample_media);

                        // Trigger spring morph: Idle -> NowPlaying
                        worker_pill.update(move |s, _| {
                            s.state_machine.set_media(Some(morph_media));
                        });
                        std::thread::sleep(std::time::Duration::from_millis(700));

                        // Trigger spring morph: NowPlaying -> Hub
                        worker_pill.update(|s, _| {
                            s.state_machine.toggle_hub();
                        });
                        std::thread::sleep(std::time::Duration::from_millis(700));

                        // Trigger spring morph: Hub -> NowPlaying -> Idle
                        worker_pill.update(|s, _| {
                            s.state_machine.close_hub();
                            s.state_machine.set_media(None);
                        });
                        std::thread::sleep(std::time::Duration::from_millis(600));

                        let stats = worker_pill.surface_stats();
                        let final_ram_mb = process_working_set_bytes()
                            .map(|b| b as f64 / (1024.0 * 1024.0))
                            .unwrap_or(0.0);
                        tracing::info!(
                            stats = ?stats,
                            working_set_mb = format!("{final_ram_mb:.2}"),
                            "Phase 0 Spike: Completed spring morph cycle"
                        );
                        app_handle.quit();
                        return;
                    }

                    // Live background polling loop (1000ms interval on MTA worker thread)
                    let mut was_fullscreen = false;
                    let mut last_volume = get_system_master_volume_percent();
                    loop {
                        let is_fullscreen = is_foreground_window_fullscreen();
                        if is_fullscreen != was_fullscreen {
                            was_fullscreen = is_fullscreen;
                            if is_fullscreen {
                                worker_pill.hide();
                            } else {
                                worker_pill.show();
                            }
                        }

                        // Step pointer-display tracking if DisplayTarget::UnderPointer is active
                        worker_pill.refollow_pointer();

                        let current_vol = get_system_master_volume_percent();
                        let changed_vol = if current_vol.is_some() && current_vol != last_volume {
                            last_volume = current_vol;
                            current_vol
                        } else {
                            None
                        };

                        let (clock_now, today_now) = current_local_clock_and_date();
                        let quiet_hours_now = is_windows_quiet_hours_active();
                        let battery_opt = sys.get_battery_status().ok();
                        let media_opt = smtc.get_current_media().ok().flatten();
                        if let Some(bytes) = media_opt
                            .as_ref()
                            .and_then(|m| m.thumbnail_bytes.as_deref())
                        {
                            let _ = image_cache().load(bytes.to_vec());
                        }
                        let toasts_opt = notif.get_recent_notifications().ok();
                        let meetings_opt = cal.get_upcoming_meetings(15).ok();

                        worker_pill.update_if(move |s, _| {
                            let before_state = s.state_machine.current_state();
                            let before_clock = s.clock_text.clone();
                            let before_phase = s.waveform_phase;

                            s.clock_text = clock_now;
                            s.today_header = today_now;
                            s.state_machine.set_fullscreen_concealed(is_fullscreen);
                            s.state_machine.set_os_quiet_hours(quiet_hours_now);

                            if let Some(vol) = changed_vol {
                                s.state_machine.trigger_hud(HudState {
                                    kind: HudKind::Volume,
                                    level_percent: vol,
                                });
                            }
                            if let Some(bat) = battery_opt {
                                s.state_machine.set_battery(bat);
                            }
                            let is_playing_now = media_opt.as_ref().is_some_and(|m| m.is_playing);
                            s.state_machine.set_media(media_opt);
                            if is_playing_now {
                                s.waveform_phase = (s.waveform_phase + 1) % 5;
                            }
                            if let Some(toasts) = toasts_opt {
                                s.state_machine.sync_notifications(toasts);
                            }
                            if let Some(meetings) = meetings_opt {
                                s.state_machine.set_upcoming_events(meetings);
                            }
                            let _ = s.state_machine.tick(1.0);

                            s.state_machine.current_state() != before_state
                                || s.clock_text != before_clock
                                || s.waveform_phase != before_phase
                        });

                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                });
        }

        vec![WindowSpec::from_app("cay", app)]
    });
}
