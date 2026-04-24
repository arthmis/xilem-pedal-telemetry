use std::{collections::VecDeque, marker::PhantomData};

use xilem::{
    Pod, ViewCtx,
    core::{MessageResult, View, ViewMarker},
};

use crate::{
    Inputs,
    widgets::{self, pedal_inputs::PedalsPlotWidget},
};

pub fn pedals_graph<State, Action>(
    // receiver: flume::Receiver<Inputs>,
    inputs: Inputs,
) -> PedalsPlot<State, Action>
where
    // F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
    State: 'static,
{
    PedalsPlot {
        // receiver,
        inputs,
        phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct PedalsPlot<State, Action> {
    // receiver: flume::Receiver<Inputs>,
    inputs: Inputs,
    phantom: PhantomData<fn(State) -> Action>,
}

impl<State, Action> ViewMarker for PedalsPlot<State, Action> {}
impl<State: 'static, Action: 'static> View<State, Action, ViewCtx> for PedalsPlot<State, Action> {
    type Element = Pod<widgets::pedal_inputs::PedalsPlotWidget>;

    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let element = ctx.create_pod(widgets::pedal_inputs::PedalsPlotWidget::new());
        (element, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: xilem::core::Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // if let Ok(inputs) = self.receiver.try_recv() {

        if prev.inputs != self.inputs {
            dbg!(prev.inputs, self.inputs);
            element.widget.update(self.inputs);
            PedalsPlotWidget::redraw(&mut element);
        }
        // PedalsPlotWidget::redraw(&mut element);
        // tracing::debug!("{:?}", inputs);
        // dbg!(inputs);
        // }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: xilem::core::Mut<'_, Self::Element>,
    ) {
        ctx.teardown_leaf(element);
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut xilem::core::MessageContext,
        element: xilem::core::Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) -> xilem::core::MessageResult<Action> {
        MessageResult::Nop
        // match message.take_message::<Inputs>() {
        //     Some(inputs) => {
        //         dbg!(&inputs);
        //         element.widget.update(*inputs);
        //         MessageResult::RequestRebuild
        //     }
        //     None => {
        //         tracing::error!("Wrong message type in Checkbox::message, got {message:?}.");
        //         MessageResult::Stale
        //     }
        // }
        // xilem::core::MessageResult::RequestRebuild
    }
}
