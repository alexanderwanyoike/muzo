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
fn it_centers_the_confirmation_on_both_library_pages() {
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
fn it_honours_a_cancellation_before_enter_in_the_same_keyboard_batch() {
    let (mut desktop, _root) = desktop();
    for (key, modifiers) in [
        (Named::Tab, Modifiers::SHIFT),
        (Named::Escape, Modifiers::empty()),
    ] {
        request(&mut desktop);
        send_key(&mut desktop, Named::Tab, Modifiers::empty());

        send_keys(
            &mut desktop,
            &[
                key_event(key, modifiers),
                key_event(Named::Enter, Modifiers::empty()),
            ],
        );

        assert!(!desktop.sources.busy(), "{key:?} must prevent removal");
        assert!(!labels(&desktop).contains_key(PROMPT));
    }

    request(&mut desktop);
    send_keys(
        &mut desktop,
        &[
            key_event(Named::Tab, Modifiers::empty()),
            key_event(Named::Enter, Modifiers::empty()),
        ],
    );
    assert!(labels(&desktop).contains_key("Removing..."));
}

#[test]
fn it_does_not_remove_a_source_without_its_open_confirmation() {
    let (mut desktop, _root) = desktop();
    let remove_another =
        || Message::SourceOperation(Operation::Remove("Harbour Rehearsals".into()));

    let _ = desktop.update(remove_another());
    assert!(!desktop.sources.busy());

    request(&mut desktop);
    let _ = desktop.update(remove_another());
    assert!(!desktop.sources.busy());
    assert!(labels(&desktop).contains_key(PROMPT));
}

#[test]
fn it_draws_the_focus_outline_after_keyboard_navigation() {
    let (mut desktop, _root) = desktop();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.appearance.accent = "glow-cyan".into();
    request(&mut desktop);

    assert_eq!(focus_outlines(&desktop), [false, false]);

    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    assert_eq!(focus_outlines(&desktop), [false, true]);

    send_key(&mut desktop, Named::Tab, Modifiers::SHIFT);
    assert_eq!(focus_outlines(&desktop), [true, false]);
}

fn focus_outlines(desktop: &Desktop) -> [bool; 2] {
    let frame = Size::new(FRAME.width as u32, FRAME.height as u32);
    let pixels = paint(desktop, desktop.windows.library_id(), frame);
    let labels = labels(desktop);
    let accent = desktop.appearance.accent_ink().into_rgba8();
    ["Cancel", "Remove source"].map(|action| {
        let label = labels[action];
        let x = label.center_x() as usize;
        let top = (label.y - 12.0) as usize;
        let bottom = (label.y + label.height + 12.0).ceil() as usize;
        (top..bottom).any(|y| {
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
    let confirm = labels(&desktop)["Remove source"];
    let click = [
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ];
    let (_, messages) = dispatch(
        &desktop,
        &click,
        Point::new(confirm.center_x(), confirm.center_y()),
    );
    for message in messages {
        let _ = desktop.update(message);
    }

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
    send_keys(desktop, &[key_event(name, modifiers)]);
}

fn key_event(name: Named, modifiers: Modifiers) -> Event {
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

fn send_keys(desktop: &mut Desktop, events: &[Event]) {
    let (statuses, messages) = dispatch(desktop, events, Point::ORIGIN);
    assert_eq!(statuses, vec![Status::Captured; events.len()], "{events:?}");
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
