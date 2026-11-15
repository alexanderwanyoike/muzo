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
    line_height: text::LineHeight,
    centered: bool,
}

impl<'a> EllipsizedLabel<'a> {
    pub(super) fn new(content: impl Into<Cow<'a, str>>, size: f32) -> Self {
        Self {
            content: content.into(),
            size: Pixels(size),
            font: None,
            color: None,
            line_height: text::LineHeight::default(),
            centered: false,
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

    pub(super) fn line_height(mut self, height: impl Into<text::LineHeight>) -> Self {
        self.line_height = height.into();
        self
    }

    pub(super) fn center_x(mut self) -> Self {
        self.centered = true;
        self
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Typography {
    size: Pixels,
    font: Font,
    line_height: text::LineHeight,
}

#[derive(Default)]
struct State {
    original: String,
    width: f32,
    typography: Option<Typography>,
    visible: String,
    paragraph: Paragraph,
    ellipsis: Option<Ellipsis>,
}

struct Ellipsis {
    paragraph: Paragraph,
    offset: f32,
}

fn shape(content: &str, typography: Typography) -> Paragraph {
    let mut paragraph = Paragraph::with_text(text::Text {
        content,
        bounds: Size::INFINITE,
        size: typography.size,
        line_height: typography.line_height,
        font: typography.font,
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
    fn update(&mut self, content: &str, width: f32, typography: Typography) {
        if self.original == content && self.width == width && self.typography == Some(typography) {
            return;
        }
        content.clone_into(&mut self.original);
        self.width = width;
        self.typography = Some(typography);
        self.visible = single_line(content);
        self.paragraph = shape(&self.visible, typography);
        self.ellipsis = None;
        if self.paragraph.min_width() <= width {
            return;
        }

        let paragraph = shape(ELLIPSIS, typography);
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
            Typography {
                size: self.size,
                font: self.font.unwrap_or_else(|| renderer.default_font()),
                line_height: self.line_height,
            },
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
            let mut origin = bounds.position();
            if self.centered {
                let ink_width = state.ellipsis.as_ref().map_or_else(
                    || state.paragraph.min_width(),
                    |marker| marker.offset + marker.paragraph.min_width(),
                );
                origin.x += (bounds.width - ink_width).max(0.0) / 2.0;
            }
            let text_clip = state.ellipsis.as_ref().map_or(Some(clip), |marker| {
                clip.intersection(&Rectangle {
                    x: origin.x,
                    width: marker.offset,
                    ..bounds
                })
            });
            if let Some(text_clip) = text_clip {
                renderer.fill_paragraph(&state.paragraph, origin, color, text_clip);
            }
            if let Some(marker) = &state.ellipsis {
                renderer.fill_paragraph(
                    &marker.paragraph,
                    Point::new(origin.x + marker.offset, origin.y),
                    color,
                    clip,
                );
            }
        }
    }
}
