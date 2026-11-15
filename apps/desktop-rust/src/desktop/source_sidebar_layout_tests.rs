use std::path::{Path, PathBuf};

use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{
        Layout, Shell, clipboard, layout, mouse,
        renderer::{Headless, Style},
        widget::Tree,
    },
    futures::StreamExt,
    time, window,
};
use iced_runtime::{Action, UserInterface, task, user_interface::Cache};

use crate::appearance::sidebar::SidebarWidth;

use super::{
    CataloguePaths, Desktop, Message,
    cover_layout_tests::paint,
    library_sources::{Operation, Sources, run},
    queue_tests::{setup, snapshot},
    transport_time_layout_tests::rendered_labels,
};

const FRAME: Size<u32> = Size::new(850, 800);
const SOURCE_NAME_CLICK_X: f32 = 150.0;
const FIRST_SOURCE_Y: usize = 377;
const SECOND_SOURCE_Y: usize = FIRST_SOURCE_Y + 32;

fn desktop_with_sources() -> Desktop {
    let (mut desktop, _) = setup();
    let mut catalogue = snapshot(&["Selected library", "Other library"]);
    catalogue.selected = Some(catalogue.libraries[0].id.clone());
    desktop.sources.snapshot = Some(catalogue);
    desktop.sidebar.set(SidebarWidth::new(312.0));
    desktop.sidebar.layout_width(FRAME.width as f32);
    desktop
}

fn desktop_with_two_indexed_sources(root: &Path, track: Option<&Path>) -> (Desktop, String) {
    let paths = CataloguePaths::in_directory(root);
    let first = root.join("Harbour Rehearsals");
    let second = root.join("Holloway Sessions");
    std::fs::create_dir(&first).unwrap();
    std::fs::create_dir(&second).unwrap();
    if let Some(track) = track {
        std::fs::copy(track, second.join("Recording.mp3")).unwrap();
    }
    let previous = run(&paths, Operation::Add(first)).unwrap().selected;
    let mut catalogue = run(&paths, Operation::Add(second)).unwrap();
    let target = catalogue.selected.replace(previous.unwrap()).unwrap();
    let mut desktop = desktop_with_sources();
    desktop.sources = Sources::new(Some(paths));
    desktop.sources.snapshot = Some(catalogue);
    (desktop, target)
}

#[test]
fn it_draws_the_source_selection_across_the_whole_row_and_reserves_its_accent_edge() {
    const ACCENT: [u8; 4] = [0, 217, 237, 255];

    let mut desktop = desktop_with_sources();
    desktop.appearance.accent = "glow-cyan".into();

    for (material, mode, selected) in [
        ("walnut", "dark", [30, 53, 51, 255]),
        ("pulse", "light", [216, 216, 216, 255]),
        ("pulse", "dark", [42, 42, 42, 255]),
    ] {
        desktop.appearance.theme = material.into();
        desktop.appearance.mode = mode.into();
        let panel = desktop.appearance.surfaces().raised.into_rgba8();
        let pixels = paint(&desktop, desktop.windows.library_id(), FRAME);
        let at = |x, y| {
            let start = (y * FRAME.width as usize + x) * 4;
            &pixels[start..start + 4]
        };

        for x in [17, 18] {
            assert_eq!(
                at(x, FIRST_SOURCE_Y),
                ACCENT,
                "the selected {material} row's 2px accent edge"
            );
            assert_eq!(
                at(x, SECOND_SOURCE_Y),
                panel,
                "the unselected {material} row reserves an empty edge"
            );
        }
        for x in [19, 150, 293] {
            assert_eq!(
                at(x, FIRST_SOURCE_Y),
                selected,
                "the {material} selection at x={x}"
            );
        }
    }
}

#[test]
fn it_keeps_the_source_selection_and_remove_clicks_in_the_original_hit_areas() {
    let mut desktop = desktop_with_sources();
    desktop.sources.snapshot.as_mut().unwrap().selected = None;

    for (y, name) in [
        (FIRST_SOURCE_Y, "Selected library"),
        (SECOND_SOURCE_Y, "Other library"),
    ] {
        let y = y as f32;
        let select = click(&desktop, Point::new(272.0, y));
        assert!(
            matches!(select.as_slice(), [Message::SourceRowClicked(id)] if id == name),
            "the last pixel in the selection area should select {name}: {select:?}"
        );
        let remove = click(&desktop, Point::new(273.5, y));
        assert!(
            matches!(remove.as_slice(), [Message::RequestSourceRemoval(id)] if id == name),
            "the first pixel in the remove area should request removal of {name}: {remove:?}"
        );
        for outside in [y - 12.0, y + 12.0] {
            assert!(
                click(&desktop, Point::new(284.0, outside)).is_empty(),
                "clicking just above or below the remove button should leave {name} alone"
            );
        }
    }
}

#[test]
fn it_closes_the_selected_song_list_on_a_second_source_click_and_keeps_playing() {
    let directory = tempfile::tempdir().unwrap();
    let folder = directory.path().join("Harbour Rehearsals");
    std::fs::create_dir(&folder).unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    std::fs::copy(fixture, folder.join("tone.mp3")).unwrap();
    let paths = CataloguePaths::in_directory(directory.path());
    let snapshot = run(&paths, Operation::Add(folder)).unwrap();
    let playing_id = snapshot.tracks[0].id.clone();

    let mut desktop = desktop_with_sources();
    desktop.sources = Sources::new(Some(paths));
    desktop.sources.snapshot = Some(snapshot);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let apply = |desktop: &mut Desktop, message| {
        let mut pending = desktop.update(message);
        while let Some(mut actions) = task::into_stream(pending) {
            let Some(Action::Output(updated)) = runtime.block_on(actions.next()) else {
                panic!("the catalogue task should return an update");
            };
            pending = desktop.update(updated);
        }
    };
    apply(&mut desktop, Message::PlayIndexed(0));
    assert!(desktop.session.as_ref().unwrap().is_playing());

    let [message]: [Message; 1] = click(
        &desktop,
        Point::new(SOURCE_NAME_CLICK_X, FIRST_SOURCE_Y as f32),
    )
    .try_into()
    .expect("a click on the selected source row produces one action");
    apply(&mut desktop, message);

    assert_eq!(desktop.sources.snapshot.as_ref().unwrap().selected, None);
    assert!(
        rendered_labels(
            &desktop,
            desktop.windows.library_id(),
            Size::new(FRAME.width as f32, FRAME.height as f32),
        )
        .contains_key("No music yet")
    );
    let session = desktop.session.as_ref().unwrap();
    assert_eq!(
        session.current().and_then(crate::track::Track::indexed_id),
        Some(playing_id.as_str())
    );
    assert!(session.is_playing());
}

#[test]
fn it_clears_the_selected_source_while_the_catalogue_is_loading() {
    let directory = tempfile::tempdir().unwrap();
    let folder = directory.path().join("Harbour Rehearsals");
    std::fs::create_dir(&folder).unwrap();
    let paths = CataloguePaths::in_directory(directory.path());
    let snapshot = run(&paths, Operation::Add(folder)).unwrap();
    let mut desktop = desktop_with_sources();
    desktop.sources = Sources::new(Some(paths));
    desktop.sources.snapshot = Some(snapshot);
    let reload = desktop.update(Message::SourceOperation(Operation::Reload));
    assert!(desktop.sources.busy());

    let [message]: [Message; 1] = click(
        &desktop,
        Point::new(SOURCE_NAME_CLICK_X, FIRST_SOURCE_Y as f32),
    )
    .try_into()
    .expect("the selected source row remains clickable while the catalogue loads");
    let cleared = desktop.update(message);
    assert_eq!(cleared.units(), 0, "clearing needs no catalogue task");
    assert_eq!(desktop.sources.snapshot.as_ref().unwrap().selected, None);

    let mut actions = task::into_stream(reload).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let Some(Action::Output(updated)) = runtime.block_on(actions.next()) else {
        panic!("expected the in-flight response with the previous selection");
    };
    let _ = desktop.update(updated);
    assert!(!desktop.sources.busy());
    assert_eq!(desktop.sources.snapshot.as_ref().unwrap().selected, None);
}

#[test]
fn it_tracks_two_clicks_on_another_source_while_the_catalogue_is_loading() {
    let directory = tempfile::tempdir().unwrap();
    let (mut desktop, target) = desktop_with_two_indexed_sources(directory.path(), None);
    let reload = desktop.update(Message::SourceOperation(Operation::Reload));

    for expected in [Some(target.as_str()), None] {
        let [message]: [Message; 1] = click(
            &desktop,
            Point::new(SOURCE_NAME_CLICK_X, SECOND_SOURCE_Y as f32),
        )
        .try_into()
        .expect("the other source row must remain clickable during the reload");
        let _ = desktop.update(message);
        assert_eq!(
            desktop
                .sources
                .snapshot
                .as_ref()
                .unwrap()
                .selected
                .as_deref(),
            expected
        );
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let finish = |desktop: &mut Desktop, action| {
        if let Some(mut task) = task::into_stream(action) {
            let Some(Action::Output(updated)) = runtime.block_on(task.next()) else {
                panic!("expected the catalogue response");
            };
            let _ = desktop.update(updated);
        }
        assert_eq!(desktop.sources.snapshot.as_ref().unwrap().selected, None);
    };
    finish(&mut desktop, reload);
    let queued = desktop.update(Message::SourceWatchTick);
    finish(&mut desktop, queued);
}

#[test]
fn it_shows_loading_tracks_when_the_selected_source_waits_for_a_scan() {
    let directory = tempfile::tempdir().unwrap();
    let track = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let (mut desktop, target) = desktop_with_two_indexed_sources(directory.path(), Some(&track));
    let reload = desktop.update(Message::SourceOperation(Operation::Reload));
    assert!(reload.units() > 0);
    let [choice]: [Message; 1] = click(
        &desktop,
        Point::new(SOURCE_NAME_CLICK_X, SECOND_SOURCE_Y as f32),
    )
    .try_into()
    .expect("the indexed source can be chosen during a scan");
    let _ = desktop.update(choice);
    assert_eq!(
        desktop.sources.snapshot.as_ref().unwrap().selected.as_ref(),
        Some(&target),
    );

    let labels = |desktop: &Desktop| {
        rendered_labels(
            desktop,
            desktop.windows.library_id(),
            (FRAME.width, FRAME.height).into(),
        )
    };
    let pending = labels(&desktop);
    assert!(
        pending.contains_key("Loading tracks..."),
        "The selected source is waiting for its read, but Songs displayed {pending:?}",
    );

    const MAX_STATUS_ALIGNMENT_DELTA: f32 = 12.0;
    const MAX_STATUS_GAP_BELOW_TOOLBAR: f32 = 80.0;
    let toolbar = pending["LIBRARY"];
    let loading = pending["Loading tracks..."];
    assert!(
        (loading.x - toolbar.x).abs() <= MAX_STATUS_ALIGNMENT_DELTA
            && (toolbar.y..=toolbar.y + MAX_STATUS_GAP_BELOW_TOOLBAR).contains(&loading.y),
        "the loading message should sit near the left edge under the toolbar: {loading:?}"
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let finish = |desktop: &mut Desktop, action| {
        let mut actions = task::into_stream(action).expect("catalogue task");
        let Some(Action::Output(updated)) = runtime.block_on(actions.next()) else {
            panic!("expected the catalogue response");
        };
        let _ = desktop.update(updated);
    };
    finish(&mut desktop, reload);
    assert!(labels(&desktop).contains_key("Loading tracks..."));
    let selection = desktop.update(Message::SourceWatchTick);
    assert!(labels(&desktop).contains_key("Loading tracks..."));
    finish(&mut desktop, selection);

    let snapshot = desktop.sources.snapshot.as_ref().unwrap();
    assert_eq!(snapshot.selected.as_ref(), Some(&target));
    assert_eq!(
        snapshot.tracks.len(),
        1,
        "the selected source was not read: {:?}",
        desktop.sources.source_tracks_error
    );
    assert!(!labels(&desktop).contains_key("Loading tracks..."));
}

#[test]
fn it_shows_the_source_read_error_instead_of_an_empty_track_list() {
    const READ_ERROR: &str = "The selected track catalogue could not be read.";
    const EMPTY: &str = "No tracks yet. Muzo syncs filesystem libraries automatically.";

    let directory = tempfile::tempdir().unwrap();
    let mut desktop = desktop_with_sources();
    let catalogue = desktop.sources.snapshot.take();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
    desktop.sources.snapshot = catalogue;
    let selection = desktop.update(Message::SourceRowClicked("Other library".into()));
    assert!(selection.units() > 0);

    let _ = desktop.update(Message::SourcesUpdated(Err(READ_ERROR.into())));

    let labels = rendered_labels(
        &desktop,
        desktop.windows.library_id(),
        (FRAME.width, FRAME.height).into(),
    );
    assert!(labels.contains_key(READ_ERROR));
    assert!(
        !labels.contains_key(EMPTY),
        "a read error must not present the selected source as empty"
    );
    assert!(
        desktop.sources.error.is_none(),
        "track errors belong to the selected source's track list"
    );
}

#[test]
fn it_discards_a_source_read_error_after_selection_changes() {
    const READ_ERROR: &str = "The earlier track catalogue could not be read.";

    for next in ["Other library", "Selected library"] {
        let directory = tempfile::tempdir().unwrap();
        let mut desktop = desktop_with_sources();
        let catalogue = desktop.sources.snapshot.take();
        desktop.sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
        desktop.sources.snapshot = catalogue;
        let selection = desktop.update(Message::SourceRowClicked("Other library".into()));
        assert!(selection.units() > 0);
        let _ = desktop.update(Message::SourceRowClicked(next.into()));

        let _ = desktop.update(Message::SourcesUpdated(Err(READ_ERROR.into())));

        let labels = rendered_labels(
            &desktop,
            desktop.windows.library_id(),
            (FRAME.width, FRAME.height).into(),
        );
        assert!(
            !labels.contains_key(READ_ERROR),
            "a failed read from the earlier selection must not appear after choosing {next}"
        );
    }
}

#[test]
fn it_does_not_click_the_replacement_for_a_source_removed_before_release() {
    let mut desktop = desktop_with_sources();
    let remaining = desktop.sources.snapshot.as_ref().unwrap().libraries[1]
        .id
        .clone();
    let id = desktop.windows.library_id();
    let size = (FRAME.width, FRAME.height).into();
    let cursor = mouse::Cursor::Available(Point::new(SOURCE_NAME_CLICK_X, FIRST_SOURCE_Y as f32));
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut ui = UserInterface::build(desktop.workspace(id), size, Cache::new(), &mut renderer);
    let mut messages = Vec::new();
    ui.update(
        &[Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        ))],
        cursor,
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );
    assert!(messages.is_empty());

    let cache = ui.into_cache();
    let mut next = desktop.sources.snapshot.clone().unwrap();
    next.libraries.remove(0);
    next.selected = None;
    let _ = desktop.update(Message::SourcesUpdated(Ok(next)));
    let mut ui = UserInterface::build(desktop.workspace(id), size, cache, &mut renderer);
    ui.update(
        &[Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))],
        cursor,
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );
    assert!(
        !messages.iter().any(
            |message| matches!(message, Message::SourceRowClicked(source) if source == &remaining)
        ),
        "releasing the removed row clicked its replacement: {messages:?}"
    );
}

#[test]
fn it_draws_distinct_glyphs_for_dropbox_and_filesystem_sources() {
    let mut desktop = desktop_with_sources();
    let id = desktop.windows.library_id();
    let cloud = paint(&desktop, id, FRAME);
    desktop.sources.snapshot.as_mut().unwrap().libraries[0].kind = "filesystem".into();
    let folder = paint(&desktop, id, FRAME);

    assert!((369..385).any(|y| {
        let row = y * FRAME.width as usize * 4;
        let glyph = row + 25 * 4..row + 42 * 4;
        cloud[glyph.clone()] != folder[glyph]
    }));
}

#[test]
fn it_aligns_source_names_and_tabular_track_counts_to_the_original_columns() {
    load_fonts();
    let mut desktop = desktop_with_sources();
    let catalogue = desktop.sources.snapshot.as_mut().unwrap();
    catalogue.selected = None;
    catalogue.track_counts = [
        ("Selected library".into(), 11),
        ("Other library".into(), 48),
    ]
    .into();
    let labels = rendered_labels(
        &desktop,
        desktop.windows.library_id(),
        Size::new(FRAME.width as f32, FRAME.height as f32),
    );

    for (name, count) in [
        ("Selected library", "11 tracks"),
        ("Other library", "48 tracks"),
    ] {
        let title = labels[name];
        let tracks = labels[count];
        assert!((title.x - 47.0).abs() < 0.25, "{name}: {title:?}");
        assert!((title.height - 17.0).abs() < 0.25, "{name}: {title:?}");
        assert!((tracks.height - 13.0).abs() < 0.25, "{count}: {tracks:?}");
        assert!(
            (tracks.x + tracks.width - 267.0).abs() < 0.25,
            "{count}: {tracks:?}"
        );
    }
    assert!((labels["11 tracks"].width - labels["48 tracks"].width).abs() < 0.1);
}

#[test]
fn it_draws_source_name_and_count_ink_on_the_original_baselines() {
    load_fonts();
    let mut desktop = desktop_with_sources();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.appearance.accent = "glow-cyan".into();
    let catalogue = desktop.sources.snapshot.as_mut().unwrap();
    catalogue.libraries[0].name = "Holloway Sessions".into();
    catalogue
        .track_counts
        .insert(catalogue.libraries[0].id.clone(), 2);
    let frame = paint_source_sidebar(&desktop, mouse::Cursor::Unavailable);
    let ink_rows = |range: std::ops::RangeInclusive<usize>| {
        let has_ink = |x: usize, y: usize| {
            let start = (y * FRAME.width as usize + x) * 4;
            frame[start..start + 3].iter().any(|channel| *channel > 130)
        };
        let rows: Vec<_> = (364..396)
            .filter(|&y| range.clone().any(|x| has_ink(x, y)))
            .collect();
        (*rows.first().unwrap(), *rows.last().unwrap())
    };

    assert_eq!(ink_rows(47..=226), (373, 384));
    assert_eq!(ink_rows(223..=267), (375, 380));
}

#[test]
fn it_uses_the_pulse_button_hover_fill_only_in_the_selection_area() {
    let mut desktop = desktop_with_sources();
    desktop.appearance.theme = "pulse".into();

    for (mode, hover, selected, panel) in [("light", 222, 216, 255), ("dark", 42, 42, 31)] {
        desktop.appearance.mode = mode.into();
        for (y, remove_fill) in [(FIRST_SOURCE_Y, selected), (SECOND_SOURCE_Y, panel)] {
            let cursor = mouse::Cursor::Available(Point::new(180.0, y as f32));
            let frame = paint_source_sidebar(&desktop, cursor);
            let at = |x: usize| {
                let start = (y * FRAME.width as usize + x) * 4;
                &frame[start..start + 4]
            };
            assert_eq!(at(180), [hover, hover, hover, 255], "Pulse {mode}");
            assert_eq!(
                at(291),
                [remove_fill, remove_fill, remove_fill, 255],
                "Pulse {mode} remove cell at y={y}"
            );
        }
    }
}

#[test]
fn it_preserves_the_original_gap_between_filtered_count_and_source_list() {
    const ACCENT: [u8; 4] = [0, 217, 237, 255];
    let mut desktop = desktop_with_sources();
    desktop.appearance.accent = "glow-cyan".into();
    desktop.sources.search = "Selected".into();
    let labels = rendered_labels(
        &desktop,
        desktop.windows.library_id(),
        (FRAME.width, FRAME.height).into(),
    );
    let count = labels["1 of 2 sources"];
    assert!((count.x - 26.0).abs() < 0.25, "{count:?}");
    assert!((count.y - 360.0).abs() < 0.5, "{count:?}");
    assert!((count.height - 14.0).abs() < 0.25, "{count:?}");

    let frame = paint_source_sidebar(&desktop, mouse::Cursor::Unavailable);
    let marked_rows: Vec<_> = (350..430)
        .filter(|&y| {
            let start = (y * FRAME.width as usize + 17) * 4;
            frame[start..start + 4] == ACCENT
        })
        .collect();
    assert_eq!(marked_rows, (383..414).collect::<Vec<_>>());
}

fn load_fonts() {
    let mut fonts = iced::advanced::graphics::text::font_system()
        .write()
        .unwrap();
    for font in crate::fonts::settings().fonts {
        fonts.load_font(font);
    }
}

fn paint_source_sidebar(desktop: &Desktop, cursor: mouse::Cursor) -> Vec<u8> {
    let id = desktop.windows.library_id();
    let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let mut ui = UserInterface::build(
        desktop.workspace(id),
        (FRAME.width, FRAME.height).into(),
        Cache::new(),
        &mut renderer,
    );
    ui.update(
        &[Event::Window(window::Event::RedrawRequested(
            time::Instant::now(),
        ))],
        cursor,
        &mut renderer,
        &mut clipboard::Null,
        &mut Vec::new(),
    );
    let theme = desktop.theme(id);
    ui.draw(&mut renderer, &theme, &Style::default(), cursor);
    renderer.screenshot(FRAME, 1.0, theme.palette().background)
}

fn click(desktop: &Desktop, point: Point) -> Vec<Message> {
    let mut view = desktop.workspace(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let bounds = Rectangle::with_size(Size::new(FRAME.width as f32, FRAME.height as f32));
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(bounds.size(), bounds.size()),
    );
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        widget.update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(point),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &bounds,
        );
    }
    messages
}

#[test]
fn it_routes_reload_track_errors_only_to_the_source_still_selected() {
    let directory = tempfile::tempdir().unwrap();
    let track = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let (mut desktop, target) = desktop_with_two_indexed_sources(directory.path(), Some(&track));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let finish = |desktop: &mut Desktop, task| {
        let mut actions = task::into_stream(task).expect("catalogue command");
        let Some(Action::Output(updated)) = runtime.block_on(actions.next()) else {
            panic!("expected a catalogue result");
        };
        let _ = desktop.update(updated);
    };
    let select = desktop.update(Message::SourceRowClicked(target.clone()));
    finish(&mut desktop, select);
    let db = rusqlite::Connection::open(directory.path().join("muzo-rust.sqlite")).unwrap();
    db.execute(
        "UPDATE tracks SET duration_seconds='invalid' WHERE library_id=?1",
        [&target],
    )
    .unwrap();

    let reload = desktop.update(Message::SourceOperation(Operation::Reload));
    finish(&mut desktop, reload);
    assert!(
        desktop.sources.source_tracks_error.is_some(),
        "the current track error belongs in Songs"
    );
    assert!(desktop.sources.error.is_none());

    let reload = desktop.update(Message::SourceOperation(Operation::Reload));
    let _ = desktop.update(Message::SourceRowClicked(target));
    finish(&mut desktop, reload);
    assert!(desktop.sources.source_tracks_error.is_none());
    assert!(
        desktop.sources.error.is_none(),
        "a stale read error escaped into the toolbar"
    );
}

#[test]
fn it_keeps_shared_catalogue_failures_visible_when_a_source_read_is_superseded() {
    let directory = tempfile::tempdir().unwrap();
    let (mut desktop, target) = desktop_with_two_indexed_sources(directory.path(), None);
    let db = rusqlite::Connection::open(directory.path().join("muzo-rust.sqlite")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=OFF; DROP TABLE playlists")
        .unwrap();
    let select = desktop.update(Message::SourceRowClicked(target.clone()));
    let _ = desktop.update(Message::SourceRowClicked(target));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut actions = task::into_stream(select).expect("source selection command");
    let Some(Action::Output(updated)) = runtime.block_on(actions.next()) else {
        panic!("expected a catalogue result");
    };
    let _ = desktop.update(updated);
    assert!(
        desktop.sources.error.is_some(),
        "a shared catalogue failure was discarded"
    );
    assert!(desktop.sources.source_tracks_error.is_none());
}
