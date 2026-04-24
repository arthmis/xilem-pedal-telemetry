use std::collections::VecDeque;

use masonry::{
    accesskit::Role,
    core::{ChildrenIds, HasProperty, Widget, WidgetMut},
    kurbo::{BezPath, Join, PathEl, Point, Stroke},
    peniko::{Brush, Fill},
};
use xilem::{Affine, Color, style::Padding};

use crate::Inputs;

pub struct PedalsPlotWidget {
    inputs: VecDeque<Inputs>,
}

impl PedalsPlotWidget {
    pub fn new() -> Self {
        Self {
            inputs: VecDeque::from(
                [Inputs {
                    id: 0,
                    throttle: 0.0,
                    brake: 100.0,
                }; 100],
            ),
        }
    }

    pub fn update(&mut self, inputs: Inputs) {
        self.inputs.push_back(inputs);
        self.inputs.pop_front();
    }

    fn throttle_path(&self) -> impl Iterator<Item = PathEl> + '_ {
        let move_to = [PathEl::MoveTo(Point::new(
            0.,
            self.inputs.front().unwrap().throttle,
        ))];
        let throttle_inputs = self.inputs.iter().enumerate().map(|(i, v)| {
            let y = 100. - v.throttle;
            let x = i as f64;
            PathEl::LineTo(Point::new(x, y))
        });

        let output = move_to.into_iter().chain(throttle_inputs);
        output
    }

    fn brake_path(&self) -> impl Iterator<Item = PathEl> + '_ {
        let move_to = [PathEl::MoveTo(Point::new(
            0.,
            self.inputs.front().unwrap().brake,
        ))];
        let throttle_inputs = self.inputs.iter().enumerate().map(|(i, v)| {
            let y = 100. - v.brake;
            let x = i as f64;
            PathEl::LineTo(Point::new(x, y))
        });

        let output = move_to.into_iter().chain(throttle_inputs);
        output
    }
}

impl PedalsPlotWidget {
    pub fn redraw(this: &mut WidgetMut<'_, Self>) {
        this.ctx.request_render();
    }
}

impl HasProperty<Padding> for PedalsPlotWidget {}

// #[derive(Debug)]
// pub struct NoAction;

impl Widget for PedalsPlotWidget {
    // type Action = NoAction;
    type Action = ();

    fn register_children(&mut self, _ctx: &mut masonry::core::RegisterCtx<'_>) {}

    fn layout(
        &mut self,
        _ctx: &mut masonry::core::LayoutCtx<'_>,
        _props: &mut masonry::core::PropertiesMut<'_>,
        bc: &masonry::core::BoxConstraints,
    ) -> masonry::kurbo::Size {
        bc.max()
    }

    fn paint(
        &mut self,
        ctx: &mut masonry::core::PaintCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        scene: &mut masonry::vello::Scene,
    ) {
        let identity = Affine::IDENTITY;

        let fill_style = Fill::NonZero;
        let brush = Brush::Solid(Color::from_rgb8(33, 33, 33));
        let rect = ctx.bounding_rect();
        scene.fill(fill_style, identity, brush, None, &rect);

        let border_width = 1.0;
        let stroke = Stroke::new(border_width).with_join(Join::Bevel);
        let throttle_brush_color = Brush::Solid(Color::from_rgb8(0, 255, 0));

        let throttle_path = BezPath::from_iter(self.throttle_path());
        scene.stroke(
            &stroke,
            identity,
            throttle_brush_color,
            None,
            &throttle_path,
        );
        let brake_brush_color = Brush::Solid(Color::from_rgb8(255, 0, 0));

        let brake_path = BezPath::from_iter(self.brake_path());
        scene.stroke(&stroke, identity, brake_brush_color, None, &brake_path);
    }

    fn accessibility_role(&self) -> masonry::accesskit::Role {
        Role::GenericContainer
    }

    fn accessibility(
        &mut self,
        _ctx: &mut masonry::core::AccessCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        _node: &mut masonry::accesskit::Node,
    ) {
    }

    fn children_ids(&self) -> masonry::core::ChildrenIds {
        ChildrenIds::new()
    }
}
