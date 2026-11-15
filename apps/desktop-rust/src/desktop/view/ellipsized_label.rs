use std::borrow::Cow;

use super::{Message, label_text::single_line};
use iced::{
    Color, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Layout, Widget, layout, mouse, renderer,
        text::{self, Paragraph as _, Renderer as _},
        widget::{Operation, Tree, tree},
    },
    alignment,
};

#[path = "ellipsized_label_fit.rs"]
mod fit;

const ELLIPSIS: &str = "…";
type Paragraph = <Renderer as text::Renderer>::Paragraph;

pub(super) struct EllipsizedLabel<'a> {
    content: Cow<'a, str>,
    size: Pixels,
    font: Option<Font>,
    color: Option<Color>,
}

impl<'a> EllipsizedLabel<'a> {
    pub(super) fn new(content: impl Into<Cow<'a, str>>, size: f32) -> Self {
        Self {
            content: content.into(),
            size: Pixels(size),
            font: None,
            color: None,
        }
    }

    pub(super) fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    pub(super) fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Default)]
struct State {
    original: String,
    width: f32,
    typography: Option<(Pixels, Font)>,
    visible: String,
    paragraph: Paragraph,
    ellipsis: Option<Ellipsis>,
}

struct Ellipsis {
    paragraph: Paragraph,
    offset: f32,
}

fn shape(content: &str, size: Pixels, font: Font) -> Paragraph {
    let mut paragraph = Paragraph::with_text(text::Text {
        content,
        bounds: Size::INFINITE,
        size,
        line_height: text::LineHeight::default(),
        font,
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
    fn update(&mut self, content: &str, width: f32, size: Pixels, font: Font) {
        if self.original == content && self.width == width && self.typography == Some((size, font))
        {
            return;
        }
        content.clone_into(&mut self.original);
        self.width = width;
        self.typography = Some((size, font));
        self.visible = single_line(content);
        self.paragraph = shape(&self.visible, size, font);
        self.ellipsis = None;
        if self.paragraph.min_width() <= width {
            return;
        }

        let paragraph = shape(ELLIPSIS, size, font);
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

impl Widget<Message, Theme, Renderer> for EllipsizedLabel<'_> {
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
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        state.update(
            &self.content,
            limits.max().width,
            self.size,
            self.font.unwrap_or_else(|| renderer.default_font()),
        );
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
            let color = self.color.unwrap_or(style.text_color);
            let text_clip = state.ellipsis.as_ref().map_or(Some(clip), |marker| {
                clip.intersection(&Rectangle {
                    width: marker.offset,
                    ..bounds
                })
            });
            if let Some(text_clip) = text_clip {
                renderer.fill_paragraph(&state.paragraph, bounds.position(), color, text_clip);
            }
            if let Some(marker) = &state.ellipsis {
                renderer.fill_paragraph(
                    &marker.paragraph,
                    Point::new(bounds.x + marker.offset, bounds.y),
                    color,
                    clip,
                );
            }
        }
    }
}
