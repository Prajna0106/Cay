use crate::models::{
    AccentPreset, BatteryStatus, CalendarMeeting, HudState, IslandSettings, MediaInfo,
    MonitorChoice, TimerState, ToastNotification,
};
use serde::{Deserialize, Serialize};

/// Default peek duration for a newly arrived toast notification (5.0 seconds per `design/tokens.json`).
pub const NOTIFICATION_PEEK_DURATION_SECS: f32 = 5.0;

/// Default display duration for the Volume/Brightness HUD interrupt (2.0 seconds per `design/tokens.json`).
pub const HUD_DISPLAY_DURATION_SECS: f32 = 2.0;

/// Visual geometry dimensions (width, height, corner radius) in device points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IslandDimensions {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

/// The discrete states of the Cay floating island.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IslandState {
    /// Idle pill showing clock and battery status.
    Idle { battery: BatteryStatus },
    /// Media playback pill showing track info and waveform.
    NowPlaying { media: MediaInfo },
    /// Notification peek showing toast preview.
    Notification { notification: ToastNotification },
    /// Upcoming meeting alert with direct Join action.
    UpcomingEvent { event: CalendarMeeting },
    /// Active focus or countdown timer.
    Timer { timer: TimerState },
    /// Volume or brightness HUD flyout.
    Hud { hud: HudState },
    /// Expanded three-card dashboard.
    Hub {
        media: Option<MediaInfo>,
        today_events: Vec<CalendarMeeting>,
        notifications: Vec<ToastNotification>,
    },
}

impl IslandState {
    /// Returns the exact dimensions and corner radius (width, height, radius) specified in tokens.json.
    pub fn dimensions(&self) -> IslandDimensions {
        match self {
            IslandState::Idle { .. } => IslandDimensions {
                width: 200.0,
                height: 40.0,
                radius: 20.0,
            },
            IslandState::NowPlaying { .. } => IslandDimensions {
                width: 360.0,
                height: 46.0,
                radius: 23.0,
            },
            IslandState::Notification { .. } => IslandDimensions {
                width: 440.0,
                height: 78.0,
                radius: 30.0,
            },
            IslandState::UpcomingEvent { .. } => IslandDimensions {
                width: 400.0,
                height: 62.0,
                radius: 31.0,
            },
            IslandState::Timer { .. } => IslandDimensions {
                width: 280.0,
                height: 46.0,
                radius: 23.0,
            },
            IslandState::Hud { .. } => IslandDimensions {
                width: 300.0,
                height: 46.0,
                radius: 23.0,
            },
            IslandState::Hub { .. } => IslandDimensions {
                width: 820.0,
                height: 330.0,
                radius: 40.0,
            },
        }
    }

    /// Numeric priority tier (higher integer = higher priority in arbitration).
    pub fn priority_rank(&self) -> u8 {
        match self {
            IslandState::Hud { .. } => 6,
            IslandState::Notification { .. } => 5,
            IslandState::UpcomingEvent { .. } => 4,
            IslandState::Timer { .. } => 3,
            IslandState::NowPlaying { .. } => 2,
            IslandState::Idle { .. } => 1,
            // Hub is user-toggled; priority ranking applies to automatic pill states.
            IslandState::Hub { .. } => 10,
        }
    }
}

/// Resolves which state should be displayed when multiple events are active.
pub fn resolve_highest_priority(
    hud: Option<&HudState>,
    notification: Option<&ToastNotification>,
    upcoming_event: Option<&CalendarMeeting>,
    timer: Option<&TimerState>,
    media: Option<&MediaInfo>,
    battery: Option<&BatteryStatus>,
) -> IslandState {
    if let Some(hud) = hud {
        return IslandState::Hud { hud: *hud };
    }

    if let Some(notification) = notification {
        return IslandState::Notification {
            notification: notification.clone(),
        };
    }

    if let Some(event) = upcoming_event {
        return IslandState::UpcomingEvent {
            event: event.clone(),
        };
    }

    if let Some(timer) = timer {
        if timer.is_running {
            return IslandState::Timer {
                timer: timer.clone(),
            };
        }
    }

    if let Some(media) = media {
        if media.is_playing || !media.title.is_empty() {
            return IslandState::NowPlaying {
                media: media.clone(),
            };
        }
    }

    IslandState::Idle {
        battery: battery.copied().unwrap_or_default(),
    }
}

/// Manages active state, history, and hub expansion.
#[derive(Clone, Debug, PartialEq)]
pub struct IslandStateMachine {
    active_state: IslandState,
    is_hub_expanded: bool,
    is_hovered: bool,
    is_fullscreen_concealed: bool,
    settings: IslandSettings,
    latest_media: Option<MediaInfo>,
    active_notifications: Vec<ToastNotification>,
    seen_notification_ids: Vec<u64>,
    peek_remaining_secs: f32,
    upcoming_events: Vec<CalendarMeeting>,
    active_timer: Option<TimerState>,
    timer_frac_secs: f32,
    active_hud: Option<HudState>,
    hud_remaining_secs: f32,
    battery_status: BatteryStatus,
}

impl Default for IslandStateMachine {
    fn default() -> Self {
        Self {
            active_state: IslandState::Idle {
                battery: BatteryStatus::default(),
            },
            is_hub_expanded: false,
            is_hovered: false,
            is_fullscreen_concealed: false,
            settings: IslandSettings::default(),
            latest_media: None,
            active_notifications: Vec::new(),
            seen_notification_ids: Vec::new(),
            peek_remaining_secs: 0.0,
            upcoming_events: Vec::new(),
            active_timer: None,
            timer_frac_secs: 0.0,
            active_hud: None,
            hud_remaining_secs: 0.0,
            battery_status: BatteryStatus::default(),
        }
    }
}

impl IslandStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current visual state to display.
    pub fn current_state(&self) -> IslandState {
        if self.is_hub_open() {
            IslandState::Hub {
                media: self.latest_media.clone(),
                today_events: self.upcoming_events.clone(),
                notifications: self.active_notifications.clone(),
            }
        } else {
            self.active_state.clone()
        }
    }

    pub fn is_hub_open(&self) -> bool {
        self.is_hub_expanded || (self.settings.hover_expand_enabled && self.is_hovered)
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn set_hover_expand_enabled(&mut self, enabled: bool) {
        self.settings.hover_expand_enabled = enabled;
    }

    pub fn settings(&self) -> &IslandSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut IslandSettings {
        &mut self.settings
    }

    pub fn set_settings(&mut self, settings: IslandSettings) {
        self.settings = settings;
        self.recompute_state();
    }

    pub fn set_accent(&mut self, accent: AccentPreset) {
        self.settings.accent = accent;
    }

    pub fn set_monitor_choice(&mut self, choice: MonitorChoice) {
        self.settings.monitor_choice = choice;
    }

    pub fn is_do_not_disturb(&self) -> bool {
        self.settings.do_not_disturb
    }

    pub fn set_do_not_disturb(&mut self, dnd: bool) {
        self.settings.do_not_disturb = dnd;
        if dnd {
            self.peek_remaining_secs = 0.0;
        }
        self.recompute_state();
    }

    pub fn is_app_blocked(&self, app_name: &str) -> bool {
        self.settings
            .blocked_notification_apps
            .iter()
            .any(|blocked| blocked.eq_ignore_ascii_case(app_name.trim()))
    }

    pub fn is_fullscreen_concealed(&self) -> bool {
        self.is_fullscreen_concealed
    }

    pub fn set_fullscreen_concealed(&mut self, concealed: bool) {
        self.is_fullscreen_concealed = concealed;
    }

    pub fn toggle_hub(&mut self) {
        self.is_hub_expanded = !self.is_hub_expanded;
    }

    pub fn close_hub(&mut self) {
        self.is_hub_expanded = false;
        self.is_hovered = false;
    }

    pub fn set_media(&mut self, media: Option<MediaInfo>) {
        self.latest_media = media;
        self.recompute_state();
    }

    pub fn latest_media(&self) -> Option<&MediaInfo> {
        self.latest_media.as_ref()
    }

    /// Pushes a newly arrived toast notification and triggers the 5.0-second peek window
    /// (unless filtered by `blocked_notification_apps` or suppressed by Do Not Disturb).
    pub fn push_notification(&mut self, notification: ToastNotification) {
        if self.is_app_blocked(&notification.app_name) {
            return;
        }
        if !self.seen_notification_ids.contains(&notification.id) {
            self.seen_notification_ids.push(notification.id);
        }
        self.active_notifications
            .retain(|n| n.id != notification.id);
        self.active_notifications.insert(0, notification);
        if self.active_notifications.len() > 10 {
            self.active_notifications.truncate(10);
        }
        if !self.settings.do_not_disturb {
            self.peek_remaining_secs = NOTIFICATION_PEEK_DURATION_SECS;
        }
        self.recompute_state();
    }

    /// Synchronizes a polled snapshot of active OS toasts. Only newly discovered notification IDs
    /// trigger the 5.0-second peek banner.
    pub fn sync_notifications(&mut self, toasts: Vec<ToastNotification>) {
        let mut has_new = false;
        for toast in toasts {
            if self.is_app_blocked(&toast.app_name) {
                continue;
            }
            if !self.seen_notification_ids.contains(&toast.id) {
                self.seen_notification_ids.push(toast.id);
                self.active_notifications.retain(|n| n.id != toast.id);
                self.active_notifications.insert(0, toast);
                has_new = true;
            }
        }
        if self.active_notifications.len() > 10 {
            self.active_notifications.truncate(10);
        }
        if has_new && !self.settings.do_not_disturb {
            self.peek_remaining_secs = NOTIFICATION_PEEK_DURATION_SECS;
        }
        self.recompute_state();
    }

    pub fn clear_notifications(&mut self) {
        self.active_notifications.clear();
        self.peek_remaining_secs = 0.0;
        self.recompute_state();
    }

    pub fn set_upcoming_event(&mut self, event: Option<CalendarMeeting>) {
        self.upcoming_events = event.into_iter().collect();
        self.recompute_state();
    }

    pub fn set_upcoming_events(&mut self, events: Vec<CalendarMeeting>) {
        self.upcoming_events = events;
        self.recompute_state();
    }

    pub fn set_timer(&mut self, timer: Option<TimerState>) {
        self.active_timer = timer;
        self.timer_frac_secs = 0.0;
        self.recompute_state();
    }

    pub fn start_focus_timer(&mut self, duration_secs: u32, label: impl Into<String>) {
        self.active_timer = Some(TimerState {
            label: label.into(),
            remaining_seconds: duration_secs,
            is_running: true,
        });
        self.timer_frac_secs = 0.0;
        self.recompute_state();
    }

    pub fn toggle_timer_pause(&mut self) {
        if let Some(timer) = &mut self.active_timer {
            timer.is_running = !timer.is_running;
        }
        self.recompute_state();
    }

    pub fn stop_timer(&mut self) {
        self.active_timer = None;
        self.timer_frac_secs = 0.0;
        self.recompute_state();
    }

    pub fn trigger_hud(&mut self, hud: HudState) {
        self.active_hud = Some(hud);
        self.hud_remaining_secs = HUD_DISPLAY_DURATION_SECS;
        self.recompute_state();
    }

    pub fn clear_hud(&mut self) {
        self.active_hud = None;
        self.hud_remaining_secs = 0.0;
        self.recompute_state();
    }

    pub fn set_battery(&mut self, battery: BatteryStatus) {
        self.battery_status = battery;
        self.recompute_state();
    }

    /// Steps time-dependent state durations (5.0s notification peek, 2.0s HUD interrupt,
    /// and 1.0s active focus timer countdown). Returns `true` if the resolved island state changed.
    pub fn tick(&mut self, dt_secs: f32) -> bool {
        let before = self.current_state();

        if self.peek_remaining_secs > 0.0 {
            self.peek_remaining_secs = (self.peek_remaining_secs - dt_secs).max(0.0);
        }

        if self.hud_remaining_secs > 0.0 {
            self.hud_remaining_secs = (self.hud_remaining_secs - dt_secs).max(0.0);
            if self.hud_remaining_secs == 0.0 {
                self.active_hud = None;
            }
        }

        if let Some(timer) = &mut self.active_timer {
            if timer.is_running {
                self.timer_frac_secs += dt_secs.max(0.0);
                let elapsed_whole = self.timer_frac_secs.floor() as u32;
                if elapsed_whole > 0 {
                    self.timer_frac_secs -= elapsed_whole as f32;
                    timer.remaining_seconds = timer.remaining_seconds.saturating_sub(elapsed_whole);
                    if timer.remaining_seconds == 0 {
                        self.active_timer = None;
                        self.timer_frac_secs = 0.0;
                    }
                }
            }
        }

        self.recompute_state();
        self.current_state() != before
    }

    fn recompute_state(&mut self) {
        let peek_notification = if self.settings.enable_notifications
            && !self.settings.do_not_disturb
            && self.peek_remaining_secs > 0.0
        {
            self.active_notifications.first()
        } else {
            None
        };
        let next_event = if self.settings.enable_calendar {
            self.upcoming_events.first()
        } else {
            None
        };
        let media = if self.settings.enable_media {
            self.latest_media.as_ref()
        } else {
            None
        };

        self.active_state = resolve_highest_priority(
            self.active_hud.as_ref(),
            peek_notification,
            next_event,
            self.active_timer.as_ref(),
            media,
            Some(&self.battery_status),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HudKind, ToastNotification};

    #[test]
    fn test_dimensions_match_design_spec() {
        let idle = IslandState::Idle {
            battery: BatteryStatus::default(),
        };
        assert_eq!(idle.dimensions().width, 200.0);
        assert_eq!(idle.dimensions().height, 40.0);
        assert_eq!(idle.dimensions().radius, 20.0);

        let now_playing = IslandState::NowPlaying {
            media: MediaInfo::default(),
        };
        assert_eq!(now_playing.dimensions().width, 360.0);
        assert_eq!(now_playing.dimensions().height, 46.0);
        assert_eq!(now_playing.dimensions().radius, 23.0);

        let notification = IslandState::Notification {
            notification: ToastNotification {
                id: 1,
                app_name: "Test".into(),
                channel: None,
                sender: "Sender".into(),
                message: "Msg".into(),
                received_timestamp: 0,
            },
        };
        assert_eq!(notification.dimensions().width, 440.0);
        assert_eq!(notification.dimensions().height, 78.0);
        assert_eq!(notification.dimensions().radius, 30.0);

        let hub = IslandState::Hub {
            media: None,
            today_events: vec![],
            notifications: vec![],
        };
        assert_eq!(hub.dimensions().width, 820.0);
        assert_eq!(hub.dimensions().height, 330.0);
        assert_eq!(hub.dimensions().radius, 40.0);
    }

    #[test]
    fn test_priority_hierarchy_resolution() {
        let hud = HudState {
            kind: HudKind::Volume,
            level_percent: 50,
        };
        let notification = ToastNotification {
            id: 1,
            app_name: "Slack".into(),
            channel: None,
            sender: "Alex".into(),
            message: "Hello".into(),
            received_timestamp: 100,
        };
        let media = MediaInfo {
            title: "Track".into(),
            artist: "Artist".into(),
            album: None,
            is_playing: true,
            position_secs: 10.0,
            duration_secs: 200.0,
            source_app: "Spotify".into(),
            thumbnail_bytes: None,
        };

        // When media only -> NowPlaying
        let state = resolve_highest_priority(None, None, None, None, Some(&media), None);
        assert!(matches!(state, IslandState::NowPlaying { .. }));

        // When notification arrives -> Notification wins over Media
        let state =
            resolve_highest_priority(None, Some(&notification), None, None, Some(&media), None);
        assert!(matches!(state, IslandState::Notification { .. }));

        // When HUD triggers -> HUD interrupts everything
        let state = resolve_highest_priority(
            Some(&hud),
            Some(&notification),
            None,
            None,
            Some(&media),
            None,
        );
        assert!(matches!(state, IslandState::Hud { .. }));
    }

    #[test]
    fn test_state_machine_hub_toggle() {
        let mut sm = IslandStateMachine::new();
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));

        sm.toggle_hub();
        assert!(sm.is_hub_open());
        assert!(matches!(sm.current_state(), IslandState::Hub { .. }));

        sm.toggle_hub();
        assert!(!sm.is_hub_open());
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));
    }

    #[test]
    fn test_state_machine_hover_expansion() {
        let mut sm = IslandStateMachine::new();
        assert!(!sm.is_hub_open());

        // Hover expands by default
        sm.set_hovered(true);
        assert!(sm.is_hub_open());
        assert!(matches!(sm.current_state(), IslandState::Hub { .. }));

        sm.set_hovered(false);
        assert!(!sm.is_hub_open());
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));

        // When hover_expand_enabled is false, hover does not expand
        sm.set_hover_expand_enabled(false);
        sm.set_hovered(true);
        assert!(!sm.is_hub_open());
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));
    }

    #[test]
    fn test_notification_peek_auto_dismisses_and_remains_in_hub() {
        let mut sm = IslandStateMachine::new();
        sm.set_media(Some(MediaInfo {
            title: "Song".into(),
            artist: "Artist".into(),
            album: None,
            is_playing: true,
            position_secs: 0.0,
            duration_secs: 180.0,
            source_app: "Spotify".into(),
            thumbnail_bytes: None,
        }));
        assert!(matches!(sm.current_state(), IslandState::NowPlaying { .. }));

        // Pushing a notification overrides NowPlaying for 5.0s
        sm.push_notification(ToastNotification {
            id: 42,
            app_name: "Slack".into(),
            channel: Some("#eng".into()),
            sender: "Priya".into(),
            message: "Check PR".into(),
            received_timestamp: 0,
        });
        assert!(matches!(
            sm.current_state(),
            IslandState::Notification { .. }
        ));

        // After 3.0s, still peeking
        assert!(!sm.tick(3.0));
        assert!(matches!(
            sm.current_state(),
            IslandState::Notification { .. }
        ));

        // After another 2.5s (total 5.5s), folds back to NowPlaying
        assert!(sm.tick(2.5));
        assert!(matches!(sm.current_state(), IslandState::NowPlaying { .. }));

        // Opening the Hub still lists the notification
        sm.toggle_hub();
        if let IslandState::Hub { notifications, .. } = sm.current_state() {
            assert_eq!(notifications.len(), 1);
            assert_eq!(notifications[0].id, 42);
        } else {
            panic!("Expected Hub state");
        }
    }

    #[test]
    fn test_focus_timer_countdown_and_completion() {
        let mut sm = IslandStateMachine::new();
        sm.start_focus_timer(3, "Focus");
        assert!(matches!(
            sm.current_state(),
            IslandState::Timer {
                timer: TimerState {
                    remaining_seconds: 3,
                    ..
                }
            }
        ));

        // 1 second elapsed -> 2 seconds left
        assert!(sm.tick(1.0));
        assert!(matches!(
            sm.current_state(),
            IslandState::Timer {
                timer: TimerState {
                    remaining_seconds: 2,
                    ..
                }
            }
        ));

        // Pause timer -> tick does not decrement
        sm.toggle_timer_pause();
        let _ = sm.tick(2.0);
        // Resume timer -> 2 more seconds completes timer and reverts to Idle
        sm.toggle_timer_pause();
        assert!(sm.tick(2.0));
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));
    }

    #[test]
    fn test_notification_blocked_apps_and_do_not_disturb() {
        let mut sm = IslandStateMachine::new();
        sm.settings_mut()
            .blocked_notification_apps
            .push("Discord".to_string());

        // Blocked app notification is ignored completely
        sm.push_notification(ToastNotification {
            id: 1,
            app_name: "discord".into(),
            channel: None,
            sender: "User".into(),
            message: "Spam".into(),
            received_timestamp: 0,
        });
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));

        // Enable Do Not Disturb: allowed app notification is added to Hub list without peeking
        sm.set_do_not_disturb(true);
        sm.push_notification(ToastNotification {
            id: 2,
            app_name: "Slack".into(),
            channel: None,
            sender: "Priya".into(),
            message: "Silent message".into(),
            received_timestamp: 0,
        });
        assert!(matches!(sm.current_state(), IslandState::Idle { .. }));

        sm.toggle_hub();
        if let IslandState::Hub { notifications, .. } = sm.current_state() {
            assert_eq!(notifications.len(), 1);
            assert_eq!(notifications[0].id, 2);
        } else {
            panic!("Expected Hub state");
        }
    }
}
