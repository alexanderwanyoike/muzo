use iced::{
    Color, Length, Rectangle, Renderer, Size, Theme,
    advanced::{Layout, Renderer as _, Widget, layout, mouse, renderer, widget::Tree},
    border, gradient,
};

use super::Message;

pub(super) struct SeekHalo {
    accent: Color,
    progress: f32,
}

impl SeekHalo {
    pub(super) fn new(accent: Color, progress: f32) -> Self {
        Self { accent, progress }
    }
}

impl Widget<Message, Theme, Renderer> for SeekHalo {
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
        if self.progress == 0.0 {
            return;
        }
        const HEIGHT: f32 = 12.0;
        const END_OVERHANG: f32 = 2.0;
        const BLURRED_RAIL: [(f32, f32); 7] = [
            (0.0, 0.0),
            (0.10, 0.01),
            (0.25, 0.03),
            (0.50, 0.055),
            (0.75, 0.03),
            (0.90, 0.01),
            (1.0, 0.0),
        ];
        let rail = layout.bounds();
        let fill = gradient::Linear::new(0.0).add_stops(BLURRED_RAIL.map(|(offset, alpha)| {
            gradient::ColorStop {
                offset,
                color: self.accent.scale_alpha(alpha),
            }
        }));
        // Use the full halo bounds so partial redraws clear the glow outside the rail.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: rail.x - END_OVERHANG,
                    y: rail.center().y - HEIGHT / 2.0,
                    width: rail.width * self.progress + END_OVERHANG * 2.0,
                    height: HEIGHT,
                },
                border: border::rounded(HEIGHT / 2.0),
                ..Default::default()
            },
            fill,
        );
    }
}
