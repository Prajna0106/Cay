use cay_core::state::{IslandDimensions, IslandStateMachine};
use rune::tok::motion;
use rune::{
    el, row, text, Align, App, AppSpec, Color, Cx, El, OverlaySpec, RunOptions, ScreenEdge,
    WindowSpec,
};

#[derive(Clone, Debug)]
struct AppState {
    state_machine: IslandStateMachine,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            state_machine: IslandStateMachine::new(),
        }
    }
}

fn island_view(state: &AppState, _: &mut Cx<AppState>) -> El {
    let current = state.state_machine.current_state();
    let IslandDimensions {
        width,
        height,
        radius,
    } = current.dimensions();

    // Dark pill container (#0B0B0F, 1px border #26262E)
    let pill_bg = Color {
        r: 0.043,
        g: 0.043,
        b: 0.059,
        a: 1.0,
    };
    let border_color = Color {
        r: 0.149,
        g: 0.149,
        b: 0.180,
        a: 1.0,
    };
    let text_primary = Color {
        r: 0.961,
        g: 0.961,
        b: 0.969,
        a: 1.0,
    };
    let text_muted = Color {
        r: 0.631,
        g: 0.631,
        b: 0.667,
        a: 1.0,
    };

    // Idle view: Clock on left, Battery percentage on right
    let content = row()
        .w_full()
        .h_full()
        .padding(14.0)
        .center()
        .child(
            text("7:08 PM")
                .color(text_primary)
                .font(rune::tok::type_::TITLE),
        )
        .child(el().grow(1.0))
        .child(text("82%").color(text_muted).font(rune::tok::type_::LABEL));

    el().key("cay-island-pill")
        .size(width, height)
        .radius(radius)
        .fill(pill_bg)
        .border(1.0, border_color)
        .motion(motion::SNAPPY)
        .child(content)
}

fn main() {
    tracing_subscriber::fmt::init();

    rune::run_windows_with_args(RunOptions { accessory: true }, [], |_| {
        let spec = AppSpec::new("Cay", (200.0, 40.0), AppState::default, island_view);
        let mut app = App::new(spec).windowless();

        let pill = app.overlay(
            OverlaySpec {
                name: "Cay Island",
                size: (200.0, 40.0),
                edge: ScreenEdge::Top,
                margin: 14.0,
                align: Align::Center,
                inset: 0.0,
                interactive: true,
                view: island_view,
            },
            AppState::default(),
        );

        pill.show();
        vec![WindowSpec::from_app("cay", app)]
    });
}
