use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{clipboard, graphics::Viewport, mouse, renderer::Style},
    event::Status,
    keyboard::{
        Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    },
};
use iced_runtime::{UserInterface, user_interface::Cache};

use super::{
    CataloguePaths, Desktop, LibraryPage, Message, Operation, Sources,
    cover_layout_tests::paint,
    queue_tests::{setup, snapshot},
    transport_time_layout_tests::rendered_labels,
};

const FRAME: Size = Size::new(850.0, 800.0);
const SOURCE_NAME: &str = "Holloway Sessions";
const SOURCE_LOCATION: &str = "/music/Holloway";
const PROMPT: &str = "Remove Holloway Sessions?";
const DESCRIPTION: &str = "This removes the source, its indexed tracks, metadata edits and play counts from Muzo. Your music files stay where they are. Playlist entries remain, marked unavailable.";
const FAILURE: &str = "Could not read the music catalogue";

fn desktop() -> (Desktop, tempfile::TempDir) {
    let (mut desktop, _) = setup();
    let root = tempfile::tempdir().unwrap();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(root.path())));
    let mut catalogue = snapshot(&[SOURCE_NAME, "Harbour Rehearsals"]);
    catalogue.libraries[0].location = SOURCE_LOCATION.into();
    desktop.sources.snapshot = Some(catalogue);
    (desktop, root)
}

fn labels(desktop: &Desktop) -> std::collections::HashMap<String, Rectangle> {
    rendered_labels(desktop, desktop.windows.library_id(), FRAME)
}

fn request(desktop: &mut Desktop) {
    let _ = desktop.update(Message::RequestSourceRemoval(SOURCE_NAME.into()));
}

#[test]
fn it_shows_the_same_centered_confirmation_from_settings_and_the_sources_panel() {
    let (mut desktop, _root) = desktop();
    for page in [LibraryPage::Songs, LibraryPage::Settings] {
        let _ = desktop.update(Message::LibraryPage(page));
        request(&mut desktop);

        let labels = labels(&desktop);
        let title = labels[PROMPT];
        assert!((title.x - 230.0).abs() < 1.0, "{title:?}");
        assert!(
            (300.0..=320.0).contains(&title.y),
            "the prompt should begin near the center of the window: {title:?}"
        );
        assert!(labels.contains_key(DESCRIPTION));
        assert!(labels.contains_key(SOURCE_LOCATION));
        assert!(labels["Cancel"].x < labels["Remove source"].x);
        let _ = desktop.update(Message::CancelSourceRemoval);
    }
}

#[test]
fn it_keeps_pointer_and_keyboard_input_in_the_dialog_and_focuses_cancel_first() {
    let (mut desktop, _root) = desktop();
    request(&mut desktop);

    let mouse = [
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        }),
    ];
    let (status, messages) = dispatch(&desktop, &mouse, Point::new(291.0, 412.0));
    assert_eq!(status, vec![Status::Captured; mouse.len()]);
    assert!(messages.is_empty());

    send_key(&mut desktop, Named::Enter, Modifiers::empty());
    assert!(!labels(&desktop).contains_key(PROMPT));
    assert!(!desktop.sources.busy());

    request(&mut desktop);
    send_key(&mut desktop, Named::Tab, Modifiers::SHIFT);
    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    send_key(&mut desktop, Named::Enter, Modifiers::empty());
    assert!(!labels(&desktop).contains_key(PROMPT));
    assert!(!desktop.sources.busy());

    request(&mut desktop);
    send_key(&mut desktop, Named::Escape, Modifiers::empty());
    assert!(!labels(&desktop).contains_key(PROMPT));
}

#[test]
fn it_draws_the_focus_outline_after_keyboard_navigation() {
    let (mut desktop, _root) = desktop();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.appearance.accent = "glow-cyan".into();
    request(&mut desktop);

    assert!(!has_focus_outline(&desktop, 424..621));

    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    assert!(has_focus_outline(&desktop, 505..621));
    assert!(!has_focus_outline(&desktop, 424..495));

    send_key(&mut desktop, Named::Tab, Modifiers::SHIFT);
    assert!(has_focus_outline(&desktop, 424..495));
}

fn has_focus_outline(desktop: &Desktop, columns: std::ops::Range<usize>) -> bool {
    let frame = Size::new(FRAME.width as u32, FRAME.height as u32);
    let pixels = paint(desktop, desktop.windows.library_id(), frame);
    let accent = desktop.appearance.accent_ink().into_rgba8();
    (440..520).any(|y| {
        columns.clone().any(|x| {
            let offset = (y * frame.width as usize + x) * 4;
            pixels[offset..offset + 4] == accent
        })
    })
}

#[test]
fn it_preserves_the_panel_and_backdrop_when_only_the_focused_button_repaints() {
    let (mut desktop, _root) = desktop();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    request(&mut desktop);
    let size = Size::new(FRAME.width as u32, FRAME.height as u32);
    let viewport = Viewport::with_physical_size(size, 1.0);
    let mut mask = tiny_skia::Mask::new(size.width, size.height).unwrap();
    let mut frame = tiny_skia::Pixmap::new(size.width, size.height).unwrap();
    let window = desktop.windows.library_id();
    let theme = desktop.theme(window);

    let render =
        |desktop: &Desktop, frame: &mut tiny_skia::Pixmap, mask: &mut tiny_skia::Mask, damage| {
            let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
            let mut ui =
                UserInterface::build(desktop.view(window), FRAME, Cache::new(), &mut renderer);
            ui.draw(
                &mut renderer,
                &theme,
                &Style::default(),
                mouse::Cursor::Unavailable,
            );
            renderer.draw(
                &mut frame.as_mut(),
                mask,
                &viewport,
                &[damage],
                theme.palette().background,
            );
        };
    render(&desktop, &mut frame, &mut mask, Rectangle::with_size(FRAME));
    let panel = frame.pixel(250, 300).unwrap();
    let outside = frame.pixel(180, 400).unwrap();

    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    render(
        &desktop,
        &mut frame,
        &mut mask,
        Rectangle {
            x: 495.0,
            y: 451.0,
            width: 132.0,
            height: 62.0,
        },
    );

    assert_eq!(frame.pixel(250, 300).unwrap(), panel);
    assert_eq!(frame.pixel(180, 400).unwrap(), outside);
}

#[test]
fn it_keeps_the_confirmation_through_a_failed_removal_and_allows_retry() {
    let (mut desktop, _root) = desktop();
    request(&mut desktop);
    let _ = desktop.update(Message::SourceOperation(Operation::Remove(
        SOURCE_NAME.into(),
    )));

    assert!(labels(&desktop).contains_key("Removing..."));
    send_key(&mut desktop, Named::Escape, Modifiers::empty());
    assert!(labels(&desktop).contains_key(PROMPT));

    let _ = desktop.update(Message::SourcesUpdated(Err(FAILURE.into())));
    assert!(labels(&desktop).contains_key(PROMPT));
    assert!(labels(&desktop).contains_key(FAILURE));
    assert!(labels(&desktop).contains_key("Remove source"));

    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    send_key(&mut desktop, Named::Enter, Modifiers::empty());
    assert!(labels(&desktop).contains_key("Removing..."));
    assert!(!labels(&desktop).contains_key(FAILURE));
    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&[
        "Harbour Rehearsals",
    ]))));
    assert!(!labels(&desktop).contains_key(PROMPT));
}

fn send_key(desktop: &mut Desktop, name: Named, modifiers: Modifiers) {
    let keyboard = Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(name),
        modified_key: Key::Named(name),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    });
    let (statuses, messages) = dispatch(desktop, &[keyboard], Point::ORIGIN);
    assert_eq!(statuses, [Status::Captured], "{name:?}");
    for message in messages {
        let _ = desktop.update(message);
    }
}

fn dispatch(desktop: &Desktop, events: &[Event], point: Point) -> (Vec<Status>, Vec<Message>) {
    let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let mut ui = UserInterface::build(
        desktop.view(desktop.windows.library_id()),
        FRAME,
        Cache::new(),
        &mut renderer,
    );
    let mut messages = Vec::new();
    let (_, statuses) = ui.update(
        events,
        mouse::Cursor::Available(point),
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );
    (statuses, messages)
}
