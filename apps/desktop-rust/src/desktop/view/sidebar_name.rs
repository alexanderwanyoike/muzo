use iced::{
    Length, Pixels, Rectangle, Renderer, Size, Theme,
    advanced::{
        Layout, Widget, layout, mouse, renderer,
        text::{self, Paragraph as _, Renderer as _},
        widget::{Operation, Tree, tree},
    },
    alignment,
};
use unicode_segmentation::UnicodeSegmentation;

use super::{Message, bold};

const FONT_SIZE: Pixels = Pixels(11.0);
const ELLIPSIS: char = '…';
type Paragraph = <Renderer as text::Renderer>::Paragraph;

pub(super) struct SidebarName<'a>(pub(super) &'a str);

#[derive(Default)]
struct State {
    original: String,
    width: f32,
    visible: String,
    paragraph: Paragraph,
}

fn shape(content: &str) -> Paragraph {
    Paragraph::with_text(text::Text {
        content,
        bounds: Size::INFINITE,
        size: FONT_SIZE,
        line_height: text::LineHeight::default(),
        font: bold(),
        align_x: text::Alignment::Left,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
    })
}

impl State {
    fn update(&mut self, name: &str, width: f32) {
        if self.original == name && self.width == width {
            return;
        }
        name.clone_into(&mut self.original);
        self.width = width;
        name.clone_into(&mut self.visible);
        self.paragraph = shape(&self.visible);
        if self.paragraph.min_width() <= width {
            return;
        }

        let boundaries: Vec<_> = name
            .grapheme_indices(true)
            .map(|(offset, _)| offset)
            .collect();
        let mut candidate = String::new();
        let fitted = boundaries.partition_point(|&offset| {
            candidate.clear();
            candidate.push_str(&name[..offset]);
            candidate.push(ELLIPSIS);
            shape(&candidate).min_width() <= width
        });
        let end = fitted.checked_sub(1).map_or(0, |index| boundaries[index]);
        self.visible.truncate(end);
        self.visible.push(ELLIPSIS);
        self.paragraph = shape(&self.visible);
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
            renderer.fill_paragraph(
                &tree.state.downcast_ref::<State>().paragraph,
                bounds.position(),
                style.text_color,
                clip,
            );
        }
    }
}
