use iced::{
    Color, Event, Font, Pixels, Point, Rectangle, Renderer, Size, Vector,
    advanced::{
        clipboard,
        graphics::Viewport,
        mouse,
        renderer::{Headless, Style},
        widget::{Id, Operation as WidgetOperation, operation::Scrollable},
    },
    event::Status,
    futures::{FutureExt, StreamExt},
    keyboard::{
        Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    },
    window,
};
use iced_runtime::{Action, UserInterface, task, user_interface::Cache};

use crate::appearance::sidebar::SidebarWidth;

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
fn it_confirms_with_space_only_after_release_and_preserves_cancellation() {
    let (mut desktop, _root) = desktop();
    let release = Event::Keyboard(iced::keyboard::Event::KeyReleased {
        key: Key::Named(Named::Space),
        modified_key: Key::Named(Named::Space),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
    });

    request(&mut desktop);
    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    send_key(&mut desktop, Named::Space, Modifiers::empty());
    assert!(
        !desktop.sources.busy(),
        "holding Space must not remove a source"
    );
    send_key(&mut desktop, Named::Escape, Modifiers::empty());
    assert!(!labels(&desktop).contains_key(PROMPT));

    request(&mut desktop);
    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    send_keys(&mut desktop, std::slice::from_ref(&release));
    assert!(
        !desktop.sources.busy(),
        "a release without a press must leave the dialog open"
    );
    send_key(&mut desktop, Named::Space, Modifiers::empty());
    send_key(&mut desktop, Named::Tab, Modifiers::SHIFT);
    send_keys(&mut desktop, std::slice::from_ref(&release));
    assert!(labels(&desktop).contains_key(PROMPT));
    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    send_key(&mut desktop, Named::Space, Modifiers::empty());
    send_keys(&mut desktop, &[release]);
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
fn it_updates_the_library_layout_when_the_window_resizes_behind_the_dialog() {
    let (mut desktop, _root) = desktop();
    let preferred = SidebarWidth::new(360.0);
    desktop.sidebar.set(preferred);
    desktop.sidebar.layout_width(1200.0);
    request(&mut desktop);

    let resized = [Event::Window(window::Event::Resized(Size::new(
        800.0, 600.0,
    )))];
    let (_, messages) = dispatch(&desktop, &resized, Point::ORIGIN);
    for message in messages {
        let _ = desktop.update(message);
    }
    send_key(&mut desktop, Named::Escape, Modifiers::empty());

    assert_eq!(desktop.sidebar.viewport_width(), 800.0);
    assert_eq!(desktop.sidebar.visible_width(), preferred.visible(799.0));
}

#[test]
fn it_keeps_both_actions_visible_at_the_minimum_window_size_with_long_source_details() {
    let (mut desktop, _root) = desktop();
    let source = &mut desktop.sources.snapshot.as_mut().unwrap().libraries[0];
    source.name = "Holloway Sessions ".repeat(140);
    source.location = format!("/music/{}", "late-night-rehearsals/".repeat(150)).into();
    request(&mut desktop);

    let frame = Size::new(800.0, 600.0);
    let labels = rendered_labels(&desktop, desktop.windows.library_id(), frame);
    for action in ["Cancel", "Remove source"] {
        let bounds = labels[action];
        assert!(bounds.height >= 18.0, "{action}: {bounds:?}");
        assert!(
            bounds.y + bounds.height < frame.height - 25.0,
            "{action}: {bounds:?}"
        );
    }
}

#[test]
fn it_reaches_the_source_location_using_the_keyboard_when_the_details_overflow() {
    let (mut desktop, _root) = desktop();
    desktop.sources.snapshot.as_mut().unwrap().libraries[0].name = "Holloway Sessions ".repeat(140);
    request(&mut desktop);

    let tasks: Vec<_> = [Named::ArrowDown, Named::PageDown, Named::End, Named::Home]
        .into_iter()
        .map(|key| {
            let (status, messages) = dispatch(
                &desktop,
                &[key_event(key, Modifiers::empty())],
                Point::ORIGIN,
            );
            assert_eq!(status, [Status::Captured]);
            iced::Task::batch(messages.into_iter().map(|message| desktop.update(message)))
        })
        .collect();
    let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let mut ui = UserInterface::build(
        desktop.view(desktop.windows.library_id()),
        Size::new(800.0, 600.0),
        Cache::new(),
        &mut renderer,
    );
    let mut details = CopyPosition::default();
    ui.operate(&renderer, &mut details);
    assert!(details.location_y > details.bottom);

    let mut offsets = Vec::new();
    for task in tasks {
        let mut actions = task::into_stream(task).expect("scroll key schedules a widget action");
        let Some(Action::Widget(mut operation)) = actions.next().now_or_never().flatten() else {
            panic!("expected a scroll operation for the confirmation");
        };
        ui.operate(&renderer, operation.as_mut());
        ui.operate(&renderer, &mut details);
        offsets.push(details.offset);
        if offsets.len() == 3 {
            assert!(details.location_y - details.offset <= details.bottom);
        }
    }
    assert!(offsets[0] > 0.0);
    assert!(offsets[1] > offsets[0]);
    assert!(offsets[2] > offsets[1]);
    assert_eq!(offsets[3], 0.0);
}

#[derive(Default)]
struct CopyPosition {
    offset: f32,
    bottom: f32,
    location_y: f32,
    gutter: f32,
}

impl WidgetOperation for CopyPosition {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn WidgetOperation)) {
        visit(self);
    }

    fn scrollable(
        &mut self,
        _: Option<&Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _: &mut dyn Scrollable,
    ) {
        self.bottom = bounds.y + bounds.height;
        self.offset = translation.y;
        self.gutter = bounds.width - content.width;
    }

    fn text(&mut self, _: Option<&Id>, bounds: Rectangle, text: &str) {
        if text == SOURCE_LOCATION {
            self.location_y = bounds.y + bounds.height;
        }
    }
}

#[test]
fn it_reserves_space_for_the_scrollbar_only_when_source_details_overflow() {
    let (mut desktop, _root) = desktop();
    request(&mut desktop);
    assert_eq!(details_gutter(&desktop), 0.0);
    let _ = desktop.update(Message::CancelSourceRemoval);
    desktop.sources.snapshot.as_mut().unwrap().libraries[0].location =
        format!("/music/{}", "c".repeat(3_800)).into();
    request(&mut desktop);
    let gutter = details_gutter(&desktop);
    assert!(
        gutter > 10.0,
        "expected a gap beyond the 10px rail: {gutter}"
    );
}

fn details_gutter(desktop: &Desktop) -> f32 {
    let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let mut ui = UserInterface::build(
        desktop.view(desktop.windows.library_id()),
        Size::new(800.0, 600.0),
        Cache::new(),
        &mut renderer,
    );
    let mut details = CopyPosition::default();
    ui.operate(&renderer, &mut details);
    details.gutter
}

#[test]
fn it_draws_the_focus_outline_after_keyboard_navigation() {
    let (mut desktop, _root) = desktop();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.appearance.accent = "glow-cyan".into();
    request(&mut desktop);

    let accent = desktop.appearance.accent_ink();
    assert_eq!(focus_outlines(&desktop, accent), [false, false]);

    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    assert_eq!(focus_outlines(&desktop, accent), [false, true]);

    send_key(&mut desktop, Named::Tab, Modifiers::SHIFT);
    assert_eq!(focus_outlines(&desktop, accent), [true, false]);
}

#[test]
fn it_uses_pulses_neutral_focus_and_hover_colors_in_both_modes() {
    let (mut desktop, _root) = desktop();
    desktop.appearance.theme = "pulse".into();
    desktop.appearance.accent = "glow-cyan".into();
    for (mode, ink, hover) in [
        ("light", iced::color!(0x171717), [222, 222, 222, 255]),
        ("dark", Color::WHITE, [42, 42, 42, 255]),
    ] {
        desktop.appearance.mode = mode.into();
        request(&mut desktop);
        send_key(&mut desktop, Named::Tab, Modifiers::SHIFT);
        send_key(&mut desktop, Named::Tab, Modifiers::empty());

        assert_eq!(focus_outlines(&desktop, ink), [true, false], "{mode}");
        assert_eq!(cancel_hover_pixel(&desktop), hover, "{mode}");
        let _ = desktop.update(Message::CancelSourceRemoval);
    }
}

fn cancel_hover_pixel(desktop: &Desktop) -> [u8; 4] {
    let cancel = labels(desktop)["Cancel"];
    let sample = Point::new(cancel.x - 5.0, cancel.center_y());
    let cursor = mouse::Cursor::Available(sample);
    let window = desktop.windows.library_id();
    let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let mut ui = UserInterface::build(desktop.view(window), FRAME, Cache::new(), &mut renderer);
    ui.update(
        &[Event::Window(window::Event::RedrawRequested(
            iced::time::Instant::now(),
        ))],
        cursor,
        &mut renderer,
        &mut clipboard::Null,
        &mut Vec::new(),
    );
    let theme = desktop.theme(window);
    ui.draw(&mut renderer, &theme, &Style::default(), cursor);
    let frame = Size::new(FRAME.width as u32, FRAME.height as u32);
    let pixels = renderer.screenshot(frame, 1.0, theme.palette().background);
    let offset = (sample.y as usize * frame.width as usize + sample.x as usize) * 4;
    pixels[offset..offset + 4].try_into().unwrap()
}

fn focus_outlines(desktop: &Desktop, ink: Color) -> [bool; 2] {
    let frame = Size::new(FRAME.width as u32, FRAME.height as u32);
    let pixels = paint(desktop, desktop.windows.library_id(), frame);
    let labels = labels(desktop);
    let ink = ink.into_rgba8();
    ["Cancel", "Remove source"].map(|action| {
        let label = labels[action];
        let x = label.center_x() as usize;
        let top = (label.y - 12.0) as usize;
        let bottom = (label.y + label.height + 12.0).ceil() as usize;
        (top..bottom).any(|y| {
            let offset = (y * frame.width as usize + x) * 4;
            pixels[offset..offset + 4] == ink
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
    let labels = labels(&desktop);
    let action = labels["Remove source"];
    let title = labels[PROMPT];
    let damage = Rectangle {
        x: action.x - 18.0,
        y: action.y - 12.0,
        width: action.width + 36.0,
        height: action.height + 24.0,
    };
    let inspect_button = |frame: &tiny_skia::Pixmap| {
        let rows = damage.y as u32..(damage.y + damage.height) as u32;
        rows.map(|y| frame.pixel(action.center_x() as u32, y).unwrap())
            .collect::<Vec<_>>()
    };

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
    let panel = (title.x as u32, (title.y - 8.0) as u32);
    let backdrop = ((title.x - 50.0) as u32, (action.y - 65.0) as u32);
    let inspect_surfaces =
        |frame: &tiny_skia::Pixmap| [panel, backdrop].map(|(x, y)| frame.pixel(x, y).unwrap());
    let surfaces = inspect_surfaces(&frame);
    let unfocused_button = inspect_button(&frame);

    send_key(&mut desktop, Named::Tab, Modifiers::empty());
    render(&desktop, &mut frame, &mut mask, damage);

    assert_ne!(inspect_button(&frame), unfocused_button);
    assert_eq!(inspect_surfaces(&frame), surfaces);
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
