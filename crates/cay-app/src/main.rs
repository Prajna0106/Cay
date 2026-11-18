use cay_core::models::{
    AccentPreset, BatteryStatus, CalendarMeeting, HudKind, HudState, MediaInfo, TimerState,
    ToastNotification,
};
use cay_core::state::{IslandDimensions, IslandState, IslandStateMachine};
#[cfg(windows)]
use cay_providers::{
    init_mta, open_external_url, process_working_set_bytes, set_windows_autostart,
    WinNotificationProvider, WinSmtcMediaProvider, WinSystemProvider,
};
use cay_providers::{
    CalendarProvider, IcsCalendarProvider, MediaProvider, NotificationProvider, SystemProvider,
};
use rune::{
    col, el, icon, row, text, Align, App, AppSpec, Color, Cx, El, Icon, OverlayInput,
    OverlayOptions, OverlaySpec, RunOptions, ScreenEdge, Spring, TextStyle, Tray, TrayEvent,
    TrayIcon, TrayItem, TraySpec, WindowSpec,
};

// Design Tokens from design/tokens.json (Linear sRGB for Rune's Color pipeline)
const ISLAND_BG: Color = Color {
    r: 0.0033,
    g: 0.0033,
    b: 0.0048,
    a: 1.0,
}; // #0B0B0F
const ISLAND_BORDER: Color = Color {
    r: 0.0194,
    g: 0.0194,
    b: 0.0273,
    a: 1.0,
}; // #26262E
const CARD_SURFACE: Color = Color {
    r: 0.0086,
    g: 0.0086,
    b: 0.0123,
    a: 1.0,
}; // #17171D
const TRACK_BG: Color = Color {
    r: 0.0273,
    g: 0.0273,
    b: 0.0396,
    a: 1.0,
}; // #2E2E38
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
    r: 0.3564,
    g: 0.3564,
    b: 0.4019,
    a: 1.0,
}; // #A1A1AA
const ACCENT_SKY: Color = Color {
    r: 0.2016,
    g: 0.5520,
    b: 1.0000,
    a: 1.0,
}; // #7CC4FF
const ACCENT_LIME: Color = Color {
    r: 0.3663,
    g: 0.7874,
    b: 0.0369,
    a: 1.0,
}; // #A3E635
const ACCENT_WARM: Color = Color {
    r: 1.0000,
    g: 0.4793,
    b: 0.1470,
    a: 1.0,
}; // #FFB86B
const ACCENT_PURPLE: Color = Color {
    r: 0.8714,
    g: 0.4072,
    b: 0.9686,
    a: 1.0,
}; // #F0ABFC
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
    r: 0.0685,
    g: 0.0075,
    b: 0.0704,
    a: 1.0,
}; // #4A154B
const CAL_TILE_BG: Color = Color {
    r: 0.0116,
    g: 0.0232,
    b: 0.0467,
    a: 1.0,
}; // #1C2A3D

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

#[derive(Clone, Debug)]
struct AppState {
    clock_text: String,
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

fn render_now_playing(media: &MediaInfo, accent: Color) -> El {
    let heights = [8.0_f32, 14.0, 10.0, 16.0, 6.0];
    let mut waveform = row().gap(3.0).center_y().h(16.0);
    for (idx, bar_h) in heights.into_iter().enumerate() {
        let h = if media.is_playing { bar_h } else { 5.0 };
        waveform = waveform.child(
            el().key(format!("wave-{idx}"))
                .size(3.0, h)
                .radius(2.0)
                .fill(accent),
        );
    }

    let artist_text = if media.artist.is_empty() {
        format!("· {}", media.source_app)
    } else {
        format!("· {}", media.artist)
    };

    row()
        .key("content-now-playing")
        .w_full()
        .h_full()
        .px(12.0)
        .gap(10.0)
        .center_y()
        .child(
            el().size(30.0, 30.0)
                .radius(9.0)
                .fill(ALBUM_TILE_BG)
                .center()
                .child(icon(&ICON_MUSIC, 14.0).color(ALBUM_ICON_COLOR)),
        )
        .child(
            row()
                .grow(1.0)
                .gap(6.0)
                .center_y()
                .child(
                    text(&media.title)
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
                .gap(2.0)
                .child(
                    row()
                        .w_full()
                        .child(text(header_left).font(FONT_META).color(TEXT_MUTED))
                        .child(el().grow(1.0))
                        .child(text("now").font(FONT_META).color(TEXT_MUTED)),
                )
                .child(
                    text(&notif.sender)
                        .font(FONT_PRIMARY_LINE)
                        .color(TEXT_PRIMARY),
                )
                .child(text(&notif.message).font(FONT_BODY).color(TEXT_SECONDARY)),
        )
}

fn render_upcoming_event(event: &CalendarMeeting, accent: Color) -> El {
    let join_url = event.join_url.clone();
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
                .gap(2.0)
                .child(
                    text(&event.title)
                        .font(FONT_PRIMARY_LINE)
                        .color(TEXT_PRIMARY),
                )
                .child(
                    text(format!("in 12 min · {}", event.service_name))
                        .font(FONT_META)
                        .color(TEXT_MUTED),
                ),
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
    row()
        .key("content-timer")
        .w_full()
        .h_full()
        .px(16.0)
        .center_y()
        .child(
            row()
                .gap(8.0)
                .center_y()
                .child(icon(&ICON_TIMER, 18.0).color(ACCENT_WARM))
                .child(
                    text(&timer.label)
                        .font(FONT_BODY_SEMIBOLD)
                        .color(TEXT_PRIMARY),
                ),
        )
        .child(el().grow(1.0))
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

fn render_hub(
    media: Option<&MediaInfo>,
    today_events: &[CalendarMeeting],
    notifications: &[ToastNotification],
    accent: Color,
) -> El {
    let default_media = sample_media();
    let active_media = media.unwrap_or(&default_media);
    let source_header = if active_media.source_app.is_empty() {
        "NOW PLAYING".to_string()
    } else {
        format!(
            "NOW PLAYING · {}",
            active_media.source_app.to_ascii_uppercase()
        )
    };
    let ratio = if active_media.duration_secs > 0.0 {
        (active_media.position_secs / active_media.duration_secs).clamp(0.0, 1.0) as f32
    } else {
        0.42
    };
    let fill_px = (228.0 * ratio).round();
    let pos_label = format_mm_ss(active_media.position_secs);
    let dur_label = format_mm_ss(active_media.duration_secs);
    let play_pause_icon = if active_media.is_playing {
        &ICON_PAUSE
    } else {
        &ICON_PLAY
    };
    let is_currently_playing = active_media.is_playing;

    // Card 1: Now Playing
    let media_card = col()
        .grow(1.0)
        .h_full()
        .padding(16.0)
        .gap(12.0)
        .radius(22.0)
        .fill(CARD_SURFACE)
        .child(text(source_header).font(FONT_CAPS_LABEL).color(TEXT_MUTED))
        .child(
            row()
                .gap(12.0)
                .center_y()
                .child(
                    el().size(64.0, 64.0)
                        .radius(14.0)
                        .fill(ALBUM_TILE_BG)
                        .center()
                        .child(icon(&ICON_MUSIC, 26.0).color(ALBUM_ICON_COLOR)),
                )
                .child(
                    col()
                        .gap(4.0)
                        .child(
                            text(&active_media.title)
                                .font(FONT_CARD_TITLE)
                                .color(TEXT_PRIMARY),
                        )
                        .child(text(&active_media.artist).font(FONT_BODY).color(TEXT_MUTED)),
                ),
        )
        .child(
            col()
                .gap(6.0)
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
                .gap(18.0)
                .child(
                    el().key("hub-btn-prev")
                        .role("button")
                        .label("Previous track")
                        .size(44.0, 44.0)
                        .center()
                        .on_press(|_: &mut AppState| {
                            #[cfg(windows)]
                            std::thread::spawn(|| {
                                let _ = WinSmtcMediaProvider::new().previous_track();
                            });
                        })
                        .child(icon(&ICON_PREV, 20.0).color(TEXT_PRIMARY)),
                )
                .child(
                    el().key("hub-btn-play-pause")
                        .role("button")
                        .label(if is_currently_playing {
                            "Pause"
                        } else {
                            "Play"
                        })
                        .size(44.0, 44.0)
                        .radius(22.0)
                        .fill(TEXT_PRIMARY)
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
                        .child(icon(play_pause_icon, 18.0).color(ISLAND_BG)),
                )
                .child(
                    el().key("hub-btn-next")
                        .role("button")
                        .label("Next track")
                        .size(44.0, 44.0)
                        .center()
                        .on_press(|_: &mut AppState| {
                            #[cfg(windows)]
                            std::thread::spawn(|| {
                                let _ = WinSmtcMediaProvider::new().next_track();
                            });
                        })
                        .child(icon(&ICON_NEXT, 20.0).color(TEXT_PRIMARY)),
                ),
        );

    // Card 2: Today's Calendar
    let first_event_title = today_events
        .first()
        .map(|e| e.title.as_str())
        .unwrap_or("Design review");
    let today_card = col()
        .grow(1.0)
        .h_full()
        .padding(16.0)
        .gap(10.0)
        .radius(22.0)
        .fill(CARD_SURFACE)
        .child(
            text("TODAY · SUN 15 NOV")
                .font(FONT_CAPS_LABEL)
                .color(TEXT_MUTED),
        )
        .child(
            row()
                .gap(10.0)
                .center_y()
                .child(el().size(3.0, 34.0).radius(2.0).fill(accent))
                .child(
                    col()
                        .gap(2.0)
                        .child(
                            text(first_event_title)
                                .font(FONT_PRIMARY_LINE)
                                .color(TEXT_PRIMARY),
                        )
                        .child(
                            text("7:20 – 8:00 PM · Teams")
                                .font(FONT_META)
                                .color(TEXT_MUTED),
                        ),
                ),
        )
        .child(
            row()
                .gap(10.0)
                .center_y()
                .child(el().size(3.0, 34.0).radius(2.0).fill(ACCENT_WARM))
                .child(
                    col()
                        .gap(2.0)
                        .child(text("Gym").font(FONT_PRIMARY_LINE).color(TEXT_PRIMARY))
                        .child(text("8:30 – 9:30 PM").font(FONT_META).color(TEXT_MUTED)),
                ),
        )
        .child(el().grow(1.0))
        .child(text("Open calendar").font(FONT_META).color(accent));

    // Card 3: Recent Notifications
    let first_sender = notifications
        .first()
        .map(|n| n.sender.as_str())
        .unwrap_or("Priya Sharma");
    let first_msg = notifications
        .first()
        .map(|n| n.message.as_str())
        .unwrap_or("Can you share the island PRD…");
    let notif_card = col()
        .grow(1.0)
        .h_full()
        .padding(16.0)
        .gap(10.0)
        .radius(22.0)
        .fill(CARD_SURFACE)
        .child(
            row()
                .w_full()
                .center_y()
                .child(
                    text("NOTIFICATIONS")
                        .font(FONT_CAPS_LABEL)
                        .color(TEXT_MUTED),
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
                        .child(text("Clear").font(FONT_META).color(TEXT_MUTED)),
                ),
        )
        .child(
            row()
                .gap(10.0)
                .center_y()
                .child(
                    el().size(32.0, 32.0)
                        .radius(10.0)
                        .fill(NOTIF_SLACK_BG)
                        .center()
                        .child(text("S").font(FONT_BODY_SEMIBOLD).color(TEXT_PRIMARY)),
                )
                .child(
                    col()
                        .gap(2.0)
                        .child(
                            text(first_sender)
                                .font(FONT_BODY_SEMIBOLD)
                                .color(TEXT_PRIMARY),
                        )
                        .child(text(first_msg).font(FONT_META).color(TEXT_MUTED)),
                ),
        );

    row()
        .key("content-hub")
        .w_full()
        .h_full()
        .padding(18.0)
        .gap(12.0)
        .child(media_card)
        .child(today_card)
        .child(notif_card)
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
        IslandState::NowPlaying { media } => render_now_playing(media, accent),
        IslandState::Notification { notification } => render_notification(notification),
        IslandState::UpcomingEvent { event } => render_upcoming_event(event, accent),
        IslandState::Timer { timer } => render_timer(timer),
        IslandState::Hud { hud } => render_hud(hud),
        IslandState::Hub {
            media,
            today_events,
            notifications,
        } => render_hub(media.as_ref(), today_events, notifications, accent),
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
            // Solid dark pill (#0B0B0F) with sky accent dot (#7CC4FF)
            if (13..19).contains(&y) && (20..26).contains(&x) {
                rgba[idx] = 124;
                rgba[idx + 1] = 196;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            } else {
                rgba[idx] = 11;
                rgba[idx + 1] = 11;
                rgba[idx + 2] = 15;
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
        TrayItem::action("accent_sky", "Accent: Sky (#7CC4FF)"),
        TrayItem::action("accent_lime", "Accent: Lime (#A3E635)"),
        TrayItem::action("accent_warm", "Accent: Warm (#FFB86B)"),
        TrayItem::action("accent_purple", "Accent: Purple (#F0ABFC)"),
        TrayItem::Separator,
        TrayItem::action("toggle_hover", "Toggle Hover Expansion"),
        TrayItem::action("enable_autostart", "Enable Start with Windows"),
        TrayItem::action("disable_autostart", "Disable Start with Windows"),
        TrayItem::Separator,
        TrayItem::action("quit", "Quit Cay"),
    ]
}

fn spec() -> AppSpec<AppState> {
    AppSpec {
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

        let pill = app.overlay(
            OverlaySpec {
                name: "Cay Island",
                size: (860.0, 360.0),
                edge: ScreenEdge::Top,
                margin: 14.0,
                align: Align::Center,
                inset: 0.0,
                interactive: true,
                view: island_view,
            }
            .options(OverlayOptions {
                input: OverlayInput::Regions,
                prewarm: true,
                ..Default::default()
            }),
            AppState::default(),
        );

        pill.show();

        // Initialize Windows System Tray Icon & Context Menu
        let tray_pill = pill.clone();
        let tray_app = app.handle();
        let tray =
            Tray::new(
                TraySpec::new("cay.island", "Cay", build_tray_icon())
                    .tooltip("Cay Floating Island")
                    .menu(build_tray_menu()),
                move |event| match event {
                    TrayEvent::Activate { .. } => {
                        tray_pill.update(|s, _| s.state_machine.toggle_hub());
                    }
                    TrayEvent::Menu(id) => {
                        match id.as_str() {
                            "toggle_hub" => tray_pill.update(|s, _| s.state_machine.toggle_hub()),
                            "accent_sky" => tray_pill
                                .update(|s, _| s.state_machine.set_accent(AccentPreset::Sky)),
                            "accent_lime" => tray_pill
                                .update(|s, _| s.state_machine.set_accent(AccentPreset::Lime)),
                            "accent_warm" => tray_pill
                                .update(|s, _| s.state_machine.set_accent(AccentPreset::Warm)),
                            "accent_purple" => tray_pill
                                .update(|s, _| s.state_machine.set_accent(AccentPreset::Purple)),
                            "toggle_hover" => tray_pill.update(|s, _| {
                                let next = !s.state_machine.settings().hover_expand_enabled;
                                s.state_machine.set_hover_expand_enabled(next);
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
                        }
                    }
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
                    let cal = IcsCalendarProvider::default();

                    if is_spike_probe {
                        std::thread::sleep(std::time::Duration::from_millis(600));
                        let ram_mb = process_working_set_bytes()
                            .map(|b| b as f64 / (1024.0 * 1024.0))
                            .unwrap_or(0.0);
                        let battery_res = sys.get_battery_status();
                        let smtc_res = smtc.get_current_media();
                        let notif_access = notif.probe_access_status();

                        tracing::info!(
                            availability = ?worker_pill.availability(),
                            failure = ?worker_pill.failure(),
                            surface = ?worker_pill.surface_description(),
                            capabilities = ?worker_pill.capabilities(),
                            working_set_mb = format!("{ram_mb:.2}"),
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
                    loop {
                        let battery_opt = sys.get_battery_status().ok();
                        let media_opt = smtc.get_current_media().ok().flatten();
                        let toasts_opt = notif.get_recent_notifications().ok();
                        let meetings_opt = cal.get_upcoming_meetings(15).ok();

                        worker_pill.update_if(move |s, _| {
                            let before = s.state_machine.current_state();
                            if let Some(bat) = battery_opt {
                                s.state_machine.set_battery(bat);
                            }
                            s.state_machine.set_media(media_opt);
                            if let Some(toasts) = toasts_opt {
                                s.state_machine.sync_notifications(toasts);
                            }
                            if let Some(meetings) = meetings_opt {
                                s.state_machine.set_upcoming_events(meetings);
                            }
                            let _ = s.state_machine.tick(1.0);
                            s.state_machine.current_state() != before
                        });

                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                });
        }

        vec![WindowSpec::from_app("cay", app)]
    });
}
