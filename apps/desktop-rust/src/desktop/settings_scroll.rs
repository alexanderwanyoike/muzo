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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::{Desktop, LibraryPage, appearance_focus::Choice, dropbox_panel::Control};
    use iced::{Font, Pixels, Renderer, Size, futures::StreamExt};
    use iced_runtime::{Action, UserInterface, task, user_interface::Cache};

    #[derive(Default)]
    struct ScrollPositions {
        settings: Option<(Rectangle, f32)>,
        folders: Option<(Rectangle, f32)>,
        folder_list: Option<Id>,
        target_id: Option<Id>,
        target: Option<Rectangle>,
    }

    impl ScrollPositions {
        fn target_visible(&self) -> bool {
            let (outer, outer_offset) = self.settings.expect("Settings viewport");
            let (mut inner, inner_offset) = self.folders.expect("Dropbox folder viewport");
            let mut target = self.target.expect("focused folder control");
            inner.y -= outer_offset;
            target.y -= outer_offset + inner_offset;
            [outer, inner].iter().all(|viewport| {
                target.y >= viewport.y && target.y + target.height <= viewport.y + viewport.height
            })
        }
    }

    impl Operation for ScrollPositions {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }

        fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
            if id.is_some() && id == self.target_id.as_ref() {
                self.target = Some(bounds);
            }
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            _: Rectangle,
            offset: Vector,
            _: &mut dyn Scrollable,
        ) {
            if id == Some(&SCROLL.into()) {
                self.settings = Some((bounds, offset.y));
            } else if id.is_some() && id == self.folder_list.as_ref() {
                self.folders = Some((bounds, offset.y));
            }
        }
    }

    #[test]
    fn it_reveals_an_offscreen_dropbox_focus_target_inside_both_scroll_areas() {
        let (mut desktop, _) = Desktop::with_session(Err("No audio in fixture".into()));
        let window = desktop.windows.library_id();
        desktop.page = LibraryPage::Settings;
        desktop.dropbox.snapshot.account = Some("fixture".into());
        desktop.dropbox.snapshot.path = "/Music".into();
        desktop.dropbox.snapshot.folders = (1..=32)
            .map(|index| {
                (
                    format!("/Music/Archive {index}"),
                    format!("Archive {index}"),
                )
            })
            .collect();
        let last = "/Music/Archive 32".to_owned();
        desktop.appearance_focus = Some(Choice::Dropbox(Control::SelectFolder(last.clone())));
        let next = desktop.update(Message::CycleMiniFocus(window, false));
        let target = Control::OpenFolder(last);
        assert_eq!(
            desktop.appearance_focus,
            Some(Choice::Dropbox(target.clone()))
        );

        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut ui = UserInterface::build(
            desktop.view(window),
            Size::new(1050.0, 600.0),
            Cache::default(),
            &mut renderer,
        );
        let mut positions = ScrollPositions {
            folder_list: target.nested_scroll(),
            target_id: Some(target.id()),
            ..Default::default()
        };
        ui.operate(&renderer, &mut positions);
        assert!(
            !positions.target_visible(),
            "the last folder starts outside the scroll areas"
        );

        let mut actions = task::into_stream(next).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        while let Some(action) = runtime.block_on(async {
            tokio::time::timeout(std::time::Duration::from_secs(2), actions.next())
                .await
                .unwrap()
        }) {
            let Action::Widget(mut operation) = action else {
                panic!("expected a widget operation");
            };
            loop {
                ui.operate(&renderer, operation.as_mut());
                let Outcome::Chain(next) = operation.finish() else {
                    break;
                };
                operation = next;
            }
        }

        ui.operate(&renderer, &mut positions);
        assert!(
            positions.target_visible(),
            "the focused control should be visible inside both scroll areas"
        );
    }
}
