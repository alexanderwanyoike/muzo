use super::{Message, bold, label_text::single_line};
use iced::{
    Length, Pixels, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Layout, Widget, layout, mouse, renderer,
        text::{self, Paragraph as _, Renderer as _},
        widget::{Operation, Tree, tree},
    },
    alignment,
};

#[path = "sidebar_name_fit.rs"]
mod fit;

const FONT_SIZE: Pixels = Pixels(11.0);
const ELLIPSIS: &str = "…";
type Paragraph = <Renderer as text::Renderer>::Paragraph;

pub(super) struct SidebarName<'a>(pub(super) &'a str);

#[derive(Default)]
struct State {
    original: String,
    width: f32,
    visible: String,
    paragraph: Paragraph,
    ellipsis: Option<Ellipsis>,
}

struct Ellipsis {
    paragraph: Paragraph,
    offset: f32,
}

fn shape(content: &str) -> Paragraph {
    let mut paragraph = Paragraph::with_text(text::Text {
        content,
        bounds: Size::INFINITE,
        size: FONT_SIZE,
        line_height: text::LineHeight::default(),
        font: bold(),
        align_x: text::Alignment::Left,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
    });
    if paragraph.buffer().layout_runs().any(|line| line.rtl) {
        // The right edge must be finite before reading right-to-left glyph positions.
        paragraph.resize(paragraph.min_bounds());
    }
    paragraph
}

impl State {
    fn update(&mut self, name: &str, width: f32) {
        if self.original == name && self.width == width {
            return;
        }
        name.clone_into(&mut self.original);
        self.width = width;
        self.visible = single_line(name);
        self.paragraph = shape(&self.visible);
        self.ellipsis = None;
        if self.paragraph.min_width() <= width {
            return;
        }

        let paragraph = shape(ELLIPSIS);
        let (visible, offset) = fit::visible_text(
            &self.paragraph,
            &self.visible,
            width - paragraph.min_width(),
        );
        self.visible = visible;
        self.visible.push_str(ELLIPSIS);
        self.ellipsis = Some(Ellipsis { paragraph, offset });
    }
}

impl Widget<Message, Theme, Renderer> for SidebarName<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        state.update(self.0, limits.max().width);
        layout::Node::new(limits.resolve(
            Length::Fill,
            Length::Shrink,
            state.paragraph.min_bounds(),
        ))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(
            None,
            layout.bounds(),
            &tree.state.downcast_ref::<State>().visible,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if let Some(clip) = bounds.intersection(viewport) {
            let state = tree.state.downcast_ref::<State>();
            let text_clip = state.ellipsis.as_ref().map_or(Some(clip), |marker| {
                clip.intersection(&Rectangle {
                    width: marker.offset,
                    ..bounds
                })
            });
            if let Some(text_clip) = text_clip {
                renderer.fill_paragraph(
                    &state.paragraph,
                    bounds.position(),
                    style.text_color,
                    text_clip,
                );
            }
            if let Some(marker) = &state.ellipsis {
                renderer.fill_paragraph(
                    &marker.paragraph,
                    Point::new(bounds.x + marker.offset, bounds.y),
                    style.text_color,
                    clip,
                );
            }
        }
    }
}
