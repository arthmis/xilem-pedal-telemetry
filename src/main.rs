mod mapped_view;
mod pedals_graph;
mod widgets;

use std::{thread, time::Duration};

use compio::time::Interval;
use futures::{StreamExt, pin_mut};
use xilem::{
    AppState, Color, EventLoop, WindowId, WindowView, Xilem,
    core::fork,
    dpi::LogicalSize,
    style::Style,
    view::{CrossAxisAlignment, MainAxisAlignment, flex_col, task_raw, text_button},
    winit::window::WindowLevel,
};

use crate::mapped_view::MappedView;

struct State {
    receiver: flume::Receiver<Inputs>,
    inputs: Inputs,
    window_id: WindowId,
    window_closed: bool,
}

struct GameState {
    view: MappedView,
    ticker: Interval,
}

impl AppState for State {
    fn keep_running(&self) -> bool {
        true
    }
}

fn app_logic(state: &mut State) -> impl Iterator<Item = WindowView<State>> + use<> {
    let graph = pedals_graph::pedals_graph(state.inputs).padding(8.);
    let close_button = text_button("X", |state: &mut State| {
        state.window_closed = true;
    });

    let receiver = state.receiver.clone();
    let task = task_raw(
        move |proxy| {
            let receiver = receiver.clone();
            async move {
                loop {
                    if let Ok(inputs) = receiver.recv_async().await
                        && let Err(err) = proxy.message(inputs)
                    {
                        dbg!(err);
                    }
                }
            }
        },
        |state: &mut State, inputs: Inputs| {
            state.inputs = inputs;
        },
    );
    let root_view = fork(
        flex_col((close_button, graph))
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .background_color(Color::from_rgb8(22, 22, 22)),
        task,
    );

    let window_view = xilem::window(state.window_id, "Inputs", root_view).with_options(|o| {
        o.with_transparent(true)
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_initial_inner_size(LogicalSize::new(800., 125.))
            .with_min_inner_size(LogicalSize::new(200., 100.))
    });

    let mut windows = vec![window_view];
    if state.window_closed {
        windows.clear();
        return windows.into_iter();
    }

    windows.into_iter()
}

fn main() {
    let (sender, receiver) = flume::bounded::<Inputs>(1000);
    let _thread_guard = thread::spawn(move || {
        compio::runtime::Runtime::new().unwrap().block_on(async {
            let view: MappedView = loop {
                let view = MappedView::open(
                    windows::core::w!("Local\\acevo_pmf_physics"),
                    size_of::<PhysicsPage>(),
                );

                match view {
                    Err(_) => {
                        thread::sleep(Duration::from_millis(5000));
                        continue;
                    }
                    Ok(view) => break view,
                }
            };

            let ticker = compio::time::interval(Duration::from_millis(PERIOD_MS));
            let data_stream =
                futures::stream::unfold(GameState { view, ticker }, async |mut state| {
                    state.ticker.tick().await;
                    let input = unsafe { state.view.read() };
                    Some((input, state))
                });

            pin_mut!(data_stream);
            while let Some(physics) = data_stream.next().await {
                sender.send(Inputs::from(physics)).ok();
            }
        });
    });

    let event_loop = EventLoop::with_user_event();

    let state = State {
        receiver,
        inputs: Inputs {
            id: 0,
            throttle: 0.0,
            brake: 100.0,
        },
        window_id: WindowId::next(),
        window_closed: false,
    };
    // let window_options = WindowOptions::new("Telemetry")
    //     .with_decorations(false)
    //     .with_window_level(WindowLevel::AlwaysOnTop)
    //     .with_initial_inner_size(LogicalSize::new(800., 125.))
    //     .with_min_inner_size(LogicalSize::new(200., 100.));
    // let app = Xilem::new_simple(state, app_logic, window_options);
    let app = Xilem::new(state, app_logic);
    app.run_in(event_loop).unwrap();
}

/// `WaitForSingleObject` timeout value meaning "wait forever".
pub const INFINITE: u32 = 0xFFFF_FFFF;

/// Timer period: ~3 ms ≈ 333 Hz.
pub const PERIOD_MS: u64 = 16;

/// Same period in 100-ns intervals; negative value means relative to now.
pub const PERIOD_100NS: i64 = -30_000;

/// Mirrors the first three fields of `SPageFilePhysics`.
///
/// The struct uses `#pragma pack(4)` and every field is 4 bytes wide, so the
/// layout is a flat byte blob with no padding:
///
/// | offset | type | field      |
/// |--------|------|------------|
/// |  0     | i32  | packet_id  |
/// |  4     | f32  | throttle   |
/// |  8     | f32  | brake      |
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PhysicsPage {
    packet_id: i32,
    /// Throttle input in the range `0.0–1.0`.
    throttle: f32,
    /// Brake input in the range `0.0–1.0`.
    brake: f32,
}

impl PhysicsPage {
    pub fn new(packet_id: i32, throttle: f32, brake: f32) -> Self {
        Self {
            packet_id,
            throttle,
            brake,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inputs {
    id: i32,
    throttle: f64,
    brake: f64,
}

impl Inputs {
    pub fn new(id: i32, throttle: f64, brake: f64) -> Self {
        Self {
            id,
            throttle,
            brake,
        }
    }

    #[inline(always)]
    pub fn throttle(&self) -> f64 {
        self.throttle
    }

    #[inline(always)]
    pub fn brake(&self) -> f64 {
        self.brake
    }
}

impl From<PhysicsPage> for Inputs {
    fn from(page: PhysicsPage) -> Self {
        Self {
            id: page.packet_id,
            throttle: (page.throttle * 100.) as f64,
            brake: (page.brake * 100.) as f64,
        }
    }
}
