use iced::{
    Element, Length, Point, Rectangle, Renderer, Size, Theme,
    advanced::{Layout, Widget, image, layout, mouse, renderer, widget::Tree},
};

use super::Message;

pub(super) fn overhang(handle: image::Handle, edge: f32, pad: f32) -> Element<'static, Message> {
    Element::new(Sleeve { handle, edge, pad })
}

struct Sleeve {
    handle: image::Handle,
    edge: f32,
    pad: f32,
}

impl Widget<Message, Theme, Renderer> for Sleeve {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.edge), Length::Fixed(self.edge))
    }

    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        let edge = self.edge.min(limits.max().width).min(limits.max().height);
        layout::Node::new(Size::new(edge, edge))
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let frame = layout.bounds();
        let pad = self.pad * frame.width / self.edge;
        image::Renderer::draw_image(
            renderer,
            image::Image::new(self.handle.clone()),
            Rectangle::new(
                Point::new(frame.x - pad, frame.y - pad),
                Size::new(frame.width + pad * 2.0, frame.height + pad * 2.0),
            ),
            *viewport,
        );
    }
}
