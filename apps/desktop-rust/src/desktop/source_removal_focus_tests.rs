use iced::{
    Event, Font, Pixels, Rectangle, Renderer, Size, Task, Vector,
    advanced::{
        clipboard, mouse,
        widget::{
            Id, Operation,
            operation::{Focusable, Outcome, Scrollable},
        },
    },
    futures::{FutureExt, StreamExt},
    keyboard::{
        Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    },
    window,
};
use iced_runtime::{Action, UserInterface, task, user_interface::Cache};

use super::{
    CataloguePaths, Desktop, LibraryPage, Message, input_message,
    library_sources::{Operation as SourceOperation, Sources},
    queue_tests::{setup, snapshot},
};

#[test]
fn it_opens_the_matching_source_from_the_filtered_sidebar_and_restores_focus_on_cancel() {
    let mut library = KeyboardLibrary::new(LibraryPage::Songs);
    let _ = library
        .desktop
        .update(Message::SourceSearch("Harbour".into()));
    let target = "source-remove-sidebar:Harbour Rehearsals";
    library.tab_to(target, Modifiers::empty());
    assert!(library.desktop.sources.removal.is_none());

    library.press(Named::Enter, Modifiers::empty());
    library.assert_confirmation_for("Harbour Rehearsals");
    library.press(Named::Escape, Modifiers::empty());

    assert!(library.desktop.sources.removal.is_none());
    assert_eq!(library.focus().id, Some(target.to_owned().into()));
}

#[test]
fn it_reveals_the_indexed_source_action_in_settings_and_opens_it_with_space_release() {
    let mut library = KeyboardLibrary::new(LibraryPage::Settings);
    let target = "source-remove-settings:Harbour Rehearsals";
    library.tab_to(target, Modifiers::SHIFT);
    assert!(library.focus().settings_scroll > 0.0);

    library.press(Named::Space, Modifiers::empty());
    assert!(library.desktop.sources.removal.is_none());
    library.input(release_space());
    library.assert_confirmation_for("Harbour Rehearsals");
    library.press(Named::Enter, Modifiers::empty());
    assert!(
        library.desktop.sources.removal.is_none(),
        "Cancel is focused on entry"
    );
    assert_eq!(library.focus().id, Some(target.to_owned().into()));
}

#[test]
fn it_does_not_open_the_previous_source_when_tab_and_space_release_arrive_together() {
    let mut library = KeyboardLibrary::new(LibraryPage::Songs);
    library.tab_to("source-remove-sidebar:Canal Sessions", Modifiers::empty());
    library.press(Named::Space, Modifiers::empty());
    library.inputs(&[key_press(Named::Tab, Modifiers::empty()), release_space()]);

    assert!(library.desktop.sources.removal.is_none());
    assert_eq!(
        library.focus().id,
        Some("source-remove-sidebar:Harbour Rehearsals".to_owned().into()),
    );
}

#[test]
fn it_restores_launcher_focus_when_the_user_cancels_after_a_failed_removal() {
    let profile = tempfile::tempdir().unwrap();
    let mut library = KeyboardLibrary::new(LibraryPage::Songs);
    let snapshot = library.desktop.sources.snapshot.take();
    library.desktop.sources = Sources::new(Some(CataloguePaths::in_directory(profile.path())));
    library.desktop.sources.snapshot = snapshot;
    let target = "source-remove-sidebar:Harbour Rehearsals";
    library.tab_to(target, Modifiers::empty());
    library.press(Named::Enter, Modifiers::empty());

    let _ = library
        .desktop
        .update(Message::SourceOperation(SourceOperation::Remove(
            "Harbour Rehearsals".into(),
        )));
    assert!(library.desktop.sources.busy());
    library.focus();
    let _ = library
        .desktop
        .update(Message::SourcesUpdated(Err("Catalogue unavailable".into())));
    library.press(Named::Escape, Modifiers::empty());

    assert!(library.desktop.sources.removal.is_none());
    assert_eq!(library.focus().id, Some(target.to_owned().into()));
}

fn key_press(name: Named, modifiers: Modifiers) -> Event {
    Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(name),
        modified_key: Key::Named(name),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })
}

fn release_space() -> Event {
    Event::Keyboard(iced::keyboard::Event::KeyReleased {
        key: Key::Named(Named::Space),
        modified_key: Key::Named(Named::Space),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
    })
}

struct KeyboardLibrary {
    desktop: Desktop,
    renderer: Renderer,
    cache: Cache,
}

impl KeyboardLibrary {
    fn new(page: LibraryPage) -> Self {
        let (mut desktop, _) = setup();
        desktop.sources.snapshot = Some(snapshot(&["Canal Sessions", "Harbour Rehearsals"]));
        let _ = desktop.update(Message::LibraryPage(page));
        Self {
            desktop,
            renderer: Renderer::new(Font::with_name("Manrope"), Pixels(16.0)),
            cache: Cache::new(),
        }
    }

    fn tab_to(&mut self, target: &str, modifiers: Modifiers) {
        let target = Id::from(target.to_owned());
        for _ in 0..8 {
            self.press(Named::Tab, modifiers);
            self.input(Event::Window(window::Event::RedrawRequested(
                iced::time::Instant::now(),
            )));
            if self.focus().id.as_ref() == Some(&target) {
                return;
            }
        }
        panic!("Tab did not reach {target:?}");
    }

    fn assert_confirmation_for(&self, id: &str) {
        let removal = self
            .desktop
            .sources
            .removal
            .as_ref()
            .expect("keyboard opened a confirmation");
        assert_eq!(removal.source.id, id);
        assert!(removal.cancel_focused && removal.show_keyboard_focus);
    }

    fn press(&mut self, name: Named, modifiers: Modifiers) {
        self.input(key_press(name, modifiers));
    }

    fn input(&mut self, event: Event) {
        self.inputs(&[event]);
    }

    fn inputs(&mut self, events: &[Event]) {
        let id = self.desktop.windows.library_id();
        let mut ui = UserInterface::build(
            self.desktop.view(id),
            Size::new(800.0, 600.0),
            std::mem::replace(&mut self.cache, Cache::new()),
            &mut self.renderer,
        );
        let mut messages = Vec::new();
        let (_, statuses) = ui.update(
            events,
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        self.cache = ui.into_cache();
        messages.extend(
            events
                .iter()
                .cloned()
                .zip(statuses)
                .filter_map(|(event, status)| input_message(event, status, id)),
        );
        let tasks = Task::batch(
            messages
                .into_iter()
                .map(|message| self.desktop.update(message)),
        );
        if let Some(mut actions) = task::into_stream(tasks) {
            while let Some(action) = actions
                .next()
                .now_or_never()
                .expect("synchronous widget action")
            {
                let Action::Widget(mut operation) = action else {
                    panic!("expected a focus or scroll operation");
                };
                self.with_ui(|ui, renderer| {
                    loop {
                        ui.operate(renderer, operation.as_mut());
                        let Outcome::Chain(next) = operation.finish() else {
                            break;
                        };
                        operation = next;
                    }
                });
            }
        }
    }

    fn focus(&mut self) -> Focus {
        let mut focus = Focus::default();
        self.with_ui(|ui, renderer| ui.operate(renderer, &mut focus));
        focus
    }

    fn with_ui(
        &mut self,
        inspect: impl FnOnce(&mut UserInterface<'_, Message, iced::Theme, Renderer>, &Renderer),
    ) {
        let mut ui = UserInterface::build(
            self.desktop.view(self.desktop.windows.library_id()),
            Size::new(800.0, 600.0),
            std::mem::replace(&mut self.cache, Cache::new()),
            &mut self.renderer,
        );
        inspect(&mut ui, &self.renderer);
        self.cache = ui.into_cache();
    }
}

#[derive(Default)]
struct Focus {
    id: Option<Id>,
    settings_scroll: f32,
}

impl Operation for Focus {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.id = id.cloned();
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        _: Rectangle,
        _: Rectangle,
        translation: Vector,
        _: &mut dyn Scrollable,
    ) {
        if id == Some(&super::appearance_focus::SCROLL.into()) {
            self.settings_scroll = translation.y;
        }
    }
}
