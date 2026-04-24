mod mapped_view;
mod pedals_graph;
mod widgets;

use std::{collections::VecDeque, thread, time::Duration};

use compio::time::Interval;
use futures::{StreamExt, pin_mut};
use xilem::{
    AppState, EventLoop, WidgetView, WindowOptions, Xilem,
    core::fork,
    dpi::LogicalSize,
    style::{Padding, Style},
    view::{flex_col, task, task_raw},
    winit::window::WindowLevel,
};

use crate::mapped_view::MappedView;

struct State {
    receiver: flume::Receiver<Inputs>,
    inputs: Inputs,
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

fn app_logic(state: &mut State) -> impl WidgetView<State> + use<> {
    // let header_text = label("todos").text_size(80.);
    let graph = pedals_graph::pedals_graph(
        // state.receiver.clone(),
        state.inputs,
    )
    .padding(Padding::all(12.));

    let receiver = state.receiver.clone();
    let task = task_raw(
        move |proxy| {
            let receiver = receiver.clone();
            async move {
                loop {
                    if let Ok(inputs) = receiver.recv_async().await {
                        if let Err(err) = proxy.message(inputs) {
                            dbg!(err);
                        }
                    }
                }
            }
        },
        |state: &mut State, inputs: Inputs| {
            // state.inputs.push_back(inputs);
            // state.inputs.pop_front();
            state.inputs = inputs;
        },
    );
    fork(flex_col(graph), task)
    // flex_col(graph)
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
    let queue = VecDeque::from(
        [Inputs {
            id: 0,
            throttle: 0.0,
            brake: 100.0,
        }; 1000],
    );
    let state = State {
        receiver,
        inputs: Inputs {
            id: 0,
            throttle: 0.0,
            brake: 100.0,
        },
    };
    let window_options = WindowOptions::new("Telemetry")
        .with_decorations(true)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_initial_inner_size(LogicalSize::new(300., 100.))
        .with_min_inner_size(LogicalSize::new(200., 100.));
    let app = Xilem::new_simple(state, app_logic, window_options);
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
