use iced::{
    Color, Length, Rectangle, Renderer, Size, Theme,
    advanced::{Layout, Renderer as _, Widget, layout, mouse, renderer, widget::Tree},
    border, gradient,
};

use super::{Message, compact::VOLUME_THUMB_RADIUS, volume::thumb_travel};

pub(super) struct VolumeHalo {
    accent: Color,
    level: f32,
}

impl VolumeHalo {
    pub(super) fn new(accent: Color, level: f32) -> Self {
        Self { accent, level }
    }
}

impl Widget<Message, Theme, Renderer> for VolumeHalo {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fill)
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        const RAIL_HEIGHT: f32 = 12.0;
        const RAIL_STOPS: [(f32, f32); 5] = [
            (0.0, 0.0),
            (0.25, 0.052),
            (0.5, 0.10),
            (0.75, 0.052),
            (1.0, 0.0),
        ];
        const THUMB_STOPS: [(f32, f32); 4] =
            [(5.0, 0.012), (3.5, 0.026), (2.0, 0.045), (0.75, 0.075)];

        let rail = layout.bounds();
        let centre = rail.center().y;
        let active_width = thumb_travel(rail.width) * self.level + VOLUME_THUMB_RADIUS;
        let thumb_x = rail.x + active_width;
        let rail_fill =
            gradient::Linear::new(0.0).add_stops(RAIL_STOPS.map(|(offset, opacity)| {
                gradient::ColorStop {
                    offset,
                    color: self.accent.scale_alpha(opacity),
                }
            }));
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: rail.x,
                    y: centre - RAIL_HEIGHT / 2.0,
                    width: active_width,
                    height: RAIL_HEIGHT,
                },
                border: border::rounded(RAIL_HEIGHT / 2.0),
                ..Default::default()
            },
            rail_fill,
        );

        // Tiny-skia only invalidates the quad bounds when the slider changes.
        for (outset, opacity) in THUMB_STOPS {
            let radius = VOLUME_THUMB_RADIUS + outset;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: thumb_x - radius,
                        y: centre - radius,
                        width: radius * 2.0,
                        height: radius * 2.0,
                    },
                    border: border::rounded(radius),
                    ..Default::default()
                },
                self.accent.scale_alpha(opacity),
            );
        }
    }
}
