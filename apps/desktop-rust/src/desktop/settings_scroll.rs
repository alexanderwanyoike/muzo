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
