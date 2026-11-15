use iced::{
    Rectangle, Task, Vector,
    advanced::widget::{
        Id, Operation, operate,
        operation::{
            Outcome,
            scrollable::{AbsoluteOffset, Scrollable, scroll_to},
        },
    },
};

use super::{Message, appearance_focus::SCROLL};

pub(super) const SOURCE_SETUP: &str = "source-setup";
const TOP_MARGIN: f32 = 16.0;

pub(super) fn reveal_sources() -> Task<Message> {
    operate(RevealSources::default()).discard()
}

pub(super) fn reveal_control(control: Id, inner: Option<Id>) -> Task<Message> {
    operate(RevealControl {
        control,
        inner,
        target: None,
        outer_viewport: None,
        inner_viewport: None,
    })
    .then(|offsets| {
        Task::batch(offsets.into_iter().map(|(id, y)| {
            iced::widget::operation::scroll_to(
                id,
                AbsoluteOffset {
                    x: None,
                    y: Some(y),
                },
            )
        }))
    })
}

struct RevealControl {
    control: Id,
    inner: Option<Id>,
    target: Option<Rectangle>,
    outer_viewport: Option<ScrollViewport>,
    inner_viewport: Option<ScrollViewport>,
}

#[derive(Clone, Copy)]
struct ScrollViewport {
    bounds: Rectangle,
    offset: f32,
    limit: f32,
}

impl ScrollViewport {
    fn reveal(self, target: Rectangle) -> f32 {
        let top = target.y - self.bounds.y;
        let bottom = top + target.height;
        if top - self.offset < TOP_MARGIN {
            (top - TOP_MARGIN).clamp(0.0, self.limit)
        } else if bottom - self.offset > self.bounds.height - TOP_MARGIN {
            (bottom + TOP_MARGIN - self.bounds.height).clamp(0.0, self.limit)
        } else {
            self.offset
        }
    }
}

impl Operation<Vec<(Id, f32)>> for RevealControl {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<Vec<(Id, f32)>>)) {
        visit(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&self.control) {
            self.target = Some(bounds);
        }
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        self.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        contents: Rectangle,
        offset: Vector,
        _: &mut dyn Scrollable,
    ) {
        let viewport = ScrollViewport {
            bounds,
            offset: offset.y,
            limit: (contents.height - bounds.height).max(0.0),
        };
        if id == Some(&SCROLL.into()) {
            self.outer_viewport = Some(viewport);
        } else if id.is_some() && id == self.inner.as_ref() {
            self.inner_viewport = Some(viewport);
        }
    }

    fn finish(&self) -> Outcome<Vec<(Id, f32)>> {
        let Some((mut target, outer)) = self.target.zip(self.outer_viewport) else {
            return Outcome::None;
        };
        let mut offsets = Vec::new();
        if let Some((id, inner)) = self.inner.as_ref().zip(self.inner_viewport) {
            let offset = inner.reveal(target);
            target.y -= offset;
            if offset != inner.offset {
                offsets.push((id.clone(), offset));
            }
        }
        let offset = outer.reveal(target);
        if offset != outer.offset {
            offsets.push((SCROLL.into(), offset));
        }
        Outcome::Some(offsets)
    }
}

#[derive(Default)]
struct RevealSources {
    viewport_top: Option<f32>,
    section_top: Option<f32>,
}

impl Operation for RevealSources {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&SOURCE_SETUP.into()) {
            self.section_top = Some(bounds.y);
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _: Rectangle,
        _: Vector,
        _: &mut dyn Scrollable,
    ) {
        if id == Some(&SCROLL.into()) {
            self.viewport_top = Some(bounds.y);
        }
    }

    fn finish(&self) -> Outcome<()> {
        let Some((section, viewport)) = self.section_top.zip(self.viewport_top) else {
            return Outcome::None;
        };
        let y = (section - viewport - TOP_MARGIN).max(0.0);
        Outcome::Chain(Box::new(scroll_to(
            SCROLL.into(),
            AbsoluteOffset {
                x: None,
                y: Some(y),
            },
        )))
    }
}
