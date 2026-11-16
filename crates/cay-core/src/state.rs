use crate::models::{
    BatteryStatus, CalendarMeeting, HudState, MediaInfo, TimerState, ToastNotification,
};
use serde::{Deserialize, Serialize};

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
    latest_media: Option<MediaInfo>,
    active_notifications: Vec<ToastNotification>,
    upcoming_events: Vec<CalendarMeeting>,
    active_timer: Option<TimerState>,
    active_hud: Option<HudState>,
    battery_status: BatteryStatus,
}

impl Default for IslandStateMachine {
    fn default() -> Self {
        Self {
            active_state: IslandState::Idle {
                battery: BatteryStatus::default(),
            },
            is_hub_expanded: false,
            latest_media: None,
            active_notifications: Vec::new(),
            upcoming_events: Vec::new(),
            active_timer: None,
            active_hud: None,
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
        if self.is_hub_expanded {
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
        self.is_hub_expanded
    }

    pub fn toggle_hub(&mut self) {
        self.is_hub_expanded = !self.is_hub_expanded;
    }

    pub fn close_hub(&mut self) {
        self.is_hub_expanded = false;
    }

    pub fn set_media(&mut self, media: Option<MediaInfo>) {
        self.latest_media = media;
        self.recompute_state();
    }

    pub fn push_notification(&mut self, notification: ToastNotification) {
        self.active_notifications.insert(0, notification);
        self.recompute_state();
    }

    pub fn clear_notifications(&mut self) {
        self.active_notifications.clear();
        self.recompute_state();
    }

    pub fn set_upcoming_event(&mut self, event: Option<CalendarMeeting>) {
        self.upcoming_events = event.into_iter().collect();
        self.recompute_state();
    }

    pub fn set_timer(&mut self, timer: Option<TimerState>) {
        self.active_timer = timer;
        self.recompute_state();
    }

    pub fn trigger_hud(&mut self, hud: HudState) {
        self.active_hud = Some(hud);
        self.recompute_state();
    }

    pub fn clear_hud(&mut self) {
        self.active_hud = None;
        self.recompute_state();
    }

    pub fn set_battery(&mut self, battery: BatteryStatus) {
        self.battery_status = battery;
        self.recompute_state();
    }

    fn recompute_state(&mut self) {
        let first_notification = self.active_notifications.first();
        let next_event = self.upcoming_events.first();

        self.active_state = resolve_highest_priority(
            self.active_hud.as_ref(),
            first_notification,
            next_event,
            self.active_timer.as_ref(),
            self.latest_media.as_ref(),
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
}
