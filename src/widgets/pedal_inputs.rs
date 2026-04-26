use std::collections::VecDeque;

use masonry::{
    accesskit::Role,
    core::{ChildrenIds, HasProperty, Widget, WidgetMut},
    kurbo::{BezPath, Join, PathEl, Point, Size, Stroke},
    peniko::{Brush, Fill},
};
use xilem::{Affine, Color, style::Padding};

use crate::Inputs;

const PEDAL_LIMIT: f64 = 100.;

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
                    brake: 0.0,
                }; 1000],
            ),
        }
    }

    pub fn update(this: &mut WidgetMut<'_, Self>, inputs: Inputs) {
        this.widget.inputs.push_back(inputs);
        this.widget.inputs.pop_front();
        this.ctx.request_render();
    }

    fn throttle_path(&self, height: f64) -> impl Iterator<Item = PathEl> + '_ {
        let scale = scale(height);
        let move_to = [PathEl::MoveTo(Point::new(
            0.,
            (PEDAL_LIMIT - self.inputs.front().unwrap().throttle) * scale,
        ))];
        let throttle_inputs = self.inputs.iter().enumerate().map(move |(i, v)| {
            let y = (PEDAL_LIMIT - v.throttle) * scale;
            let x = i as f64;
            PathEl::LineTo(Point::new(x, y))
        });

        move_to.into_iter().chain(throttle_inputs)
    }

    fn brake_path(&self, height: f64) -> impl Iterator<Item = PathEl> + '_ {
        let scale = scale(height);
        let move_to = [PathEl::MoveTo(Point::new(
            0.,
            (PEDAL_LIMIT - self.inputs.front().unwrap().brake) * scale,
        ))];
        let throttle_inputs = self.inputs.iter().enumerate().map(move |(i, v)| {
            let y = (PEDAL_LIMIT - v.brake) * scale;
            let x = i as f64;
            PathEl::LineTo(Point::new(x, y))
        });

        move_to.into_iter().chain(throttle_inputs)
    }

    fn y_markers(&self, max_size: Size) -> BezPath {
        let base = |line_height| max_size.height / 5. * line_height;
        let heights = [base(1.), base(2.), base(3.), base(4.), base(5.)];

        let mut path = BezPath::new();
        for height in heights {
            path.move_to(Point::new(0., height));
            path.line_to(Point::new(max_size.width, height));
        }

        path
    }

    fn scale_width(&mut self, max_width: f64) {
        let max_width = max_width as usize;

        if max_width == self.inputs.len() {
            return;
        }

        if max_width > self.inputs.len() {
            let amount_to_add = max_width - self.inputs.len();
            for _ in 0..amount_to_add {
                self.inputs.push_back(Inputs {
                    id: 0,
                    throttle: 0.,
                    brake: 0.,
                });
            }
            self.inputs.reserve(amount_to_add);
            return;
        }

        let amount_to_drain = self.inputs.len() - max_width;

        {
            self.inputs.drain(0..amount_to_drain);
        }
        self.inputs.reserve(self.inputs.len());
    }
}

fn scale(height: f64) -> f64 {
    height / PEDAL_LIMIT
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
        props: &mut masonry::core::PropertiesMut<'_>,
        bc: &masonry::core::BoxConstraints,
    ) -> masonry::kurbo::Size {
        let padding = props.get::<Padding>();
        padding.layout_down(*bc).max()
    }

    fn paint(
        &mut self,
        ctx: &mut masonry::core::PaintCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        scene: &mut masonry::vello::Scene,
    ) {
        let identity = Affine::IDENTITY;

        let fill_style = Fill::NonZero;
        let brush = Brush::Solid(Color::from_rgb8(22, 22, 22));
        let rect = ctx.size().to_rect();
        scene.fill(fill_style, identity, brush, None, &rect);

        let brush_transformation = None;

        let max_size = ctx.size();
        let y_marker_color = Brush::Solid(Color::from_rgb8(180, 180, 180));
        let stroke = Stroke::new(0.5).with_join(Join::Bevel);
        let y_axis_markers = self.y_markers(max_size);
        scene.stroke(
            &stroke,
            identity,
            y_marker_color,
            brush_transformation,
            &y_axis_markers,
        );

        self.scale_width(max_size.width);
        let border_width = 2.0;
        let stroke = Stroke::new(border_width).with_join(Join::Bevel);
        let throttle_brush_color = Brush::Solid(Color::from_rgb8(0, 255, 0));

        let throttle_path = BezPath::from_iter(self.throttle_path(max_size.height));
        scene.stroke(
            &stroke,
            identity,
            throttle_brush_color,
            brush_transformation,
            &throttle_path,
        );

        let brake_brush_color = Brush::Solid(Color::from_rgb8(255, 0, 0));
        let brake_path = BezPath::from_iter(self.brake_path(max_size.height));
        scene.stroke(
            &stroke,
            identity,
            brake_brush_color,
            brush_transformation,
            &brake_path,
        );
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
