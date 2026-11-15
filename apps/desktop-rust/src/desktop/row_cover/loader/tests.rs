use std::{fs, path::PathBuf};

use crate::desktop::{
    library_sources::{Operation, run},
    queue_tests::snapshot,
};

use super::*;

#[test]
fn it_loads_an_unplayed_local_cover_after_delivering_the_source_rows() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temporary.path());
    let folder = temporary.path().join("music");
    fs::create_dir(&folder).unwrap();
    fs::write(
        folder.join("record.mp3"),
        include_bytes!("../../../../tests/fixtures/embedded-cover.mp3"),
    )
    .unwrap();
    let snapshot = run(&paths, Operation::Add(folder)).unwrap();
    assert!(snapshot.covers.is_empty());
    let track = &snapshot.tracks[0];

    let cover = load(
        &paths,
        &ArtworkAccess::default(),
        Candidate::indexed(track, &snapshot.libraries),
    )
    .unwrap();
    assert!(cover.is_some());
    assert!(paths.artwork_cache().exists());
    assert_eq!(track.plays, 0);
}

#[test]
fn it_prioritizes_visible_rows_past_the_previous_probe_limit() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temporary.path());
    let mut snapshot = snapshot(&["source"]);
    snapshot.selected = Some("source".into());
    for index in 0..1_000 {
        snapshot.tracks.push(IndexedTrack {
            id: index.to_string(),
            title: format!("Track {index}"),
            artist: String::new(),
            album: None,
            metadata_overridden: false,
            path: PathBuf::from(format!("track-{index}.mp3")),
            plays: 0,
            duration_seconds: 0,
            library_id: "source".into(),
            file_size: 0,
            file_mtime: 0,
        });
    }
    let mut loader = Loader::default();
    loader.visible(LibraryPage::Songs, 850..860);
    let task = loader.request(&paths, &mut snapshot, LibraryPage::Songs);
    assert_eq!(task.units(), CONCURRENT_READS);
    assert_eq!(
        loader.pending,
        HashSet::from([(0, "850".into()), (0, "851".into())])
    );
    assert_eq!(
        loader
            .request(&paths, &mut snapshot, LibraryPage::Songs)
            .units(),
        0
    );
}

#[test]
fn it_discards_artwork_completed_after_a_catalogue_revision() {
    let mut snapshot = snapshot(&["source"]);
    let mut loader = Loader::default();
    loader.pending.insert((0, "track".into()));
    loader.catalogue_changed();
    let result = loader.complete(
        Complete {
            generation: 0,
            id: "track".into(),
            target: Target::Library,
            cover: Ok(Some(Handle::from_rgba(1, 1, vec![255; 4]))),
        },
        Some(&mut snapshot.covers),
    );
    assert!(result.is_none());
    assert!(loader.pending.is_empty());
    assert!(snapshot.covers.is_empty());
}

#[test]
fn it_loads_the_first_covers_when_a_short_source_replaces_a_scrolled_one() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temporary.path());
    let mut snapshot = snapshot(&["source"]);
    snapshot.selected = Some("source".into());
    for index in 0..12 {
        snapshot.tracks.push(IndexedTrack {
            id: index.to_string(),
            title: format!("Track {index}"),
            artist: String::new(),
            album: None,
            metadata_overridden: false,
            path: PathBuf::from(format!("track-{index}.mp3")),
            plays: 0,
            duration_seconds: 0,
            library_id: "source".into(),
            file_size: 0,
            file_mtime: 0,
        });
    }
    let mut loader = Loader::default();
    loader.visible(LibraryPage::Songs, 35..51);
    loader.catalogue_changed();
    let task = loader.request(&paths, &mut snapshot, LibraryPage::Songs);

    assert_eq!(task.units(), CONCURRENT_READS);
    assert_eq!(
        loader.pending,
        HashSet::from([(1, "0".into()), (1, "1".into())])
    );
}

#[test]
fn it_reports_every_visible_song_when_a_tall_viewport_needs_no_scrollbar() {
    use iced::{Event, Font, Pixels, Renderer, Size, advanced::clipboard, mouse, window};
    use iced_runtime::{UserInterface, user_interface::Cache};

    use crate::desktop::queue_tests::setup;

    let (mut desktop, _) = setup();
    let mut source = snapshot(&["source"]);
    source.selected = Some("source".into());
    let count = 25;
    source.tracks = (0..count).map(indexed_track).collect();
    desktop.sources.snapshot = Some(source);
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut ui = UserInterface::build(
        desktop.workspace(window::Id::unique()),
        Size::new(1440.0, 2000.0),
        Cache::default(),
        &mut renderer,
    );
    let mut messages = Vec::new();
    ui.update(
        &[Event::Window(window::Event::RedrawRequested(
            iced::time::Instant::now(),
        ))],
        mouse::Cursor::Unavailable,
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );
    assert!(
        messages.into_iter().any(|message| matches!(
            message,
            Message::ArtworkViewport(LibraryPage::Songs, rows) if rows == (0..count)
        )),
        "Songs should request all {count} visible covers when every row fits"
    );
}

fn indexed_track(index: usize) -> IndexedTrack {
    IndexedTrack {
        id: index.to_string(),
        title: format!("Track {index}"),
        artist: String::new(),
        album: None,
        metadata_overridden: false,
        path: PathBuf::from(format!("track-{index}.mp3")),
        plays: 0,
        duration_seconds: 0,
        library_id: "source".into(),
        file_size: 0,
        file_mtime: 0,
    }
}

#[test]
fn it_keeps_the_thumbnail_limit_after_refresh_preserves_loaded_covers() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temporary.path());
    let mut source = snapshot(&["source"]);
    source.tracks = (0..ROW_LIMIT + 32).map(indexed_track).collect();
    let cover = Handle::from_rgba(1, 1, vec![255; 4]);
    for track in &source.tracks[..ROW_LIMIT] {
        source.covers.insert(track.id.clone(), cover.clone());
    }
    let mut loader = Loader::default();
    loader.catalogue_changed();
    loader.visible(LibraryPage::Songs, ROW_LIMIT..ROW_LIMIT + 16);
    let _ = loader.request(&paths, &mut source, LibraryPage::Songs);
    let next = ROW_LIMIT.to_string();
    assert!(loader.pending.contains(&(1, next.clone())));
    let _ = loader.complete(
        Complete {
            generation: 1,
            id: next.clone(),
            target: Target::Library,
            cover: Ok(Some(cover)),
        },
        Some(&mut source.covers),
    );
    assert!(source.covers.contains_key(&next));
    assert!(
        source.covers.len() <= ROW_LIMIT,
        "preserving ready covers across refresh must keep the thumbnail limit"
    );
}

#[test]
fn it_bounds_resident_covers_when_more_rows_fit_than_the_thumbnail_limit() {
    let temporary = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temporary.path());
    let mut source = snapshot(&["source"]);
    source.tracks = (0..ROW_LIMIT + 1).map(indexed_track).collect();
    let cover = Handle::from_rgba(1, 1, vec![255; 4]);
    for track in &source.tracks[..ROW_LIMIT] {
        source.covers.insert(track.id.clone(), cover.clone());
    }
    let mut loader = Loader::default();
    loader.visible(LibraryPage::Songs, 0..ROW_LIMIT + 1);

    assert_eq!(
        loader
            .request(&paths, &mut source, LibraryPage::Songs)
            .units(),
        0,
        "an oversized viewport must preserve the thumbnail limit"
    );
    assert_eq!(source.covers.len(), ROW_LIMIT);
}

#[test]
fn it_loads_an_unplayed_cover_when_opening_the_mini_queue_away_from_songs() {
    use iced::{Size, futures::StreamExt};
    use iced_runtime::{Action, task};

    use crate::desktop::{library_sources::Sources, queue_tests::setup};

    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let folder = root.path().join("music");
    fs::create_dir(&folder).unwrap();
    for name in ["first.mp3", "second.mp3"] {
        fs::write(
            folder.join(name),
            include_bytes!("../../../../tests/fixtures/embedded-cover.mp3"),
        )
        .unwrap();
    }
    let source = run(&paths, Operation::Add(folder)).unwrap();
    let next = source.tracks[1].id.clone();
    let (mut desktop, _) = setup();
    desktop.sources = Sources::new(Some(paths));
    desktop.sources.snapshot = Some(source);
    let _ = desktop.update(Message::PlayIndexed(0));
    assert!(desktop.queue_covers.is_empty());
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));
    let _ = desktop.update(Message::ShowMini);
    let _ = desktop.update(Message::ToggleQueue);
    let id = desktop.windows.mini_id().unwrap();
    desktop.windows.resized(id, Size::new(740.0, 560.0));
    assert!(desktop.windows.queue_open());

    let mut stream = task::into_stream(desktop.load_row_covers())
        .expect("the expanded queue should schedule its own visible cover reads");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    while let Some(Action::Output(message)) = runtime.block_on(stream.next()) {
        let _ = desktop.update(message);
    }
    assert!(desktop.queue_covers.contains_key(&next));
    assert_ne!(desktop.session.as_ref().unwrap().current_index(), Some(1));
}

#[test]
fn it_leaves_queue_artwork_workers_available_after_navigating_from_the_sources_sidebar() {
    for navigation in [
        Message::OpenSourceSettings,
        Message::SourceRowClicked("other".into()),
    ] {
        let (_root, mut desktop) = open_queue_with_unloaded_covers();
        let _ = desktop.update(navigation.clone());
        assert_eq!(
            desktop.load_row_covers().units(),
            CONCURRENT_READS,
            "discarded nested artwork tasks should not reserve both workers after {navigation:?}"
        );
    }
}

#[test]
fn it_leaves_artwork_workers_for_visible_songs_once_the_mini_queue_closes() {
    let (_root, mut desktop) = open_queue_with_unloaded_covers();

    let _ = desktop.update(Message::ShowLibrary);

    assert_eq!(
        desktop.load_row_covers().units(),
        1,
        "closing the mini queue should schedule the one visible song"
    );
}

fn open_queue_with_unloaded_covers() -> (tempfile::TempDir, crate::desktop::Desktop) {
    use iced::Size;

    use crate::desktop::{library_sources::Sources, queue_tests::setup};

    let root = tempfile::tempdir().unwrap();
    let (mut desktop, local) = setup();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(root.path())));
    let mut source = snapshot(&["source", "other"]);
    source.selected = Some("source".into());
    source.tracks.push(indexed_track(0));
    desktop.sources.snapshot = Some(source);
    desktop.session.as_mut().unwrap().replace(
        ["first", "second"]
            .into_iter()
            .map(|id| {
                local
                    .clone()
                    .with_indexed_id(id.into())
                    .with_library("other".into())
            })
            .collect(),
    );
    let _ = desktop.update(Message::ShowMini);
    let _ = desktop.update(Message::ToggleQueue);
    let id = desktop.windows.mini_id().unwrap();
    desktop.windows.resized(id, Size::new(740.0, 560.0));
    (root, desktop)
}

#[test]
fn it_retries_a_failed_cover_read_when_returning_to_that_list() {
    use iced::futures::StreamExt;
    use iced_runtime::{Action, task};

    let temporary = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temporary.path());
    let folder = temporary.path().join("music");
    fs::create_dir(&folder).unwrap();
    let audio = folder.join("record.mp3");
    let embedded = include_bytes!("../../../../tests/fixtures/embedded-cover.mp3");
    fs::write(&audio, embedded).unwrap();
    let mut snapshot = run(&paths, Operation::Add(folder)).unwrap();
    let track = snapshot.tracks[0].id.clone();
    let mut loader = Loader::default();
    loader.visible(LibraryPage::Songs, 0..16);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let completed = |request| {
        let mut stream = task::into_stream(request).expect("a cover read should be scheduled");
        let Some(Action::Output(Message::ArtworkReady(result))) = runtime.block_on(stream.next())
        else {
            panic!("the worker should report a cover result");
        };
        result
    };
    fs::remove_file(&audio).unwrap();
    let unavailable = loader.request(&paths, &mut snapshot, LibraryPage::Songs);
    let _ = loader.complete(completed(unavailable), Some(&mut snapshot.covers));
    assert!(!snapshot.covers.contains_key(&track));
    assert_eq!(
        loader
            .request(&paths, &mut snapshot, LibraryPage::Songs)
            .units(),
        0
    );

    fs::write(&audio, embedded).unwrap();
    loader.visible(LibraryPage::Playlists, 0..16);
    loader.visible(LibraryPage::Songs, 0..16);
    let restored = loader.request(&paths, &mut snapshot, LibraryPage::Songs);
    assert_eq!(restored.units(), 1, "the failed cover should be retried");
    let _ = loader.complete(completed(restored), Some(&mut snapshot.covers));
    assert!(snapshot.covers.contains_key(&track));
}

#[test]
fn it_preserves_ready_covers_until_the_backing_audio_changes() {
    use crate::desktop::{Desktop, library_sources::Sources};

    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let audio = root.path().join("music");
    fs::create_dir(&audio).unwrap();
    for name in ["first.mp3", "second.mp3"] {
        fs::write(
            audio.join(name),
            include_bytes!("../../../../tests/fixtures/embedded-cover.mp3"),
        )
        .unwrap();
    }
    let mut snapshot = run(&paths, Operation::Add(audio)).unwrap();
    let library = snapshot.selected.clone().unwrap();
    let handle = Handle::from_rgba(1, 1, vec![255; 4]);
    for track in &snapshot.tracks {
        snapshot.covers.insert(track.id.clone(), handle.clone());
    }
    let unchanged = snapshot.tracks[0].id.clone();
    let updated = snapshot.tracks[1].id.clone();
    let mut desktop = Desktop::with_session(Err("No audio in fixture".into())).0;
    desktop.sources = Sources::new(Some(paths.clone()));
    desktop.sources.snapshot = Some(snapshot);

    let database = rusqlite::Connection::open(root.path().join("muzo-rust.sqlite")).unwrap();
    database
        .execute(
            "UPDATE tracks SET file_mtime = file_mtime + 1 WHERE id = ?1",
            [&updated],
        )
        .unwrap();
    let refreshed = run(&paths, Operation::Select(library)).unwrap();
    let _ = desktop.update(Message::SourcesUpdated(Ok(refreshed)));

    let covers = &desktop.sources.snapshot.as_ref().unwrap().covers;
    assert_eq!(
        covers.get(&unchanged),
        Some(&handle),
        "an unchanged cover should stay visible"
    );
    assert!(
        !covers.contains_key(&updated),
        "changed audio must load fresh artwork"
    );
}

#[test]
fn it_keeps_rows_after_an_expanded_editor_in_the_cover_viewport() {
    use iced::advanced::widget::operation::scrollable::{AbsoluteOffset, scroll_to};
    use iced::{Event, Font, Pixels, Renderer, Size, advanced::clipboard, mouse, window};
    use iced_runtime::{UserInterface, user_interface::Cache};

    use crate::desktop::{
        library_sources::TrackEditor, queue_tests::setup, track_editor_focus::SONGS_SCROLL,
    };

    let (mut desktop, _) = setup();
    let mut source = snapshot(&["source"]);
    source.selected = Some("source".into());
    source.tracks = (0..40)
        .map(|index| IndexedTrack {
            id: index.to_string(),
            title: format!("Track {index}"),
            artist: String::new(),
            album: None,
            metadata_overridden: false,
            path: PathBuf::from(format!("track-{index}.mp3")),
            plays: 0,
            duration_seconds: 0,
            library_id: "source".into(),
            file_size: 0,
            file_mtime: 0,
        })
        .collect();
    desktop.sources.snapshot = Some(source);
    let visible_start = |desktop: &crate::desktop::Desktop| {
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut ui = UserInterface::build(
            desktop.workspace(window::Id::unique()),
            Size::new(1040.0, 750.0),
            Cache::default(),
            &mut renderer,
        );
        ui.operate(
            &renderer,
            &mut scroll_to(
                SONGS_SCROLL.into(),
                AbsoluteOffset {
                    x: None,
                    y: Some(620.0),
                },
            ),
        );
        let mut messages = Vec::new();
        ui.update(
            &[Event::Window(window::Event::RedrawRequested(
                iced::time::Instant::now(),
            ))],
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        messages
            .into_iter()
            .find_map(|message| {
                if let Message::ArtworkViewport(LibraryPage::Songs, rows) = message
                    && !rows.is_empty()
                {
                    Some(rows.start)
                } else {
                    None
                }
            })
            .expect("Songs should report the viewport after scrolling")
    };
    let collapsed = visible_start(&desktop);
    desktop.sources.editing = Some(Box::new(TrackEditor {
        track_id: "0".into(),
        library_id: "source".into(),
        fields: Default::default(),
        overridden: false,
        error: None,
        action_focus: None,
    }));
    let expanded = visible_start(&desktop);
    assert!(
        expanded < collapsed,
        "the expanded row should shift which covers are visible"
    );
}
