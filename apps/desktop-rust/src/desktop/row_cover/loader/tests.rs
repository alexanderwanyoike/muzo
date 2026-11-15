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

    let cover = load(&paths, &Remote::default(), track, false).unwrap();
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
    loader.complete(
        Complete {
            generation: 0,
            id: "track".into(),
            cover: Ok(Some(Handle::from_rgba(1, 1, vec![255; 4]))),
        },
        Some(&mut snapshot),
    );
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
    loader.complete(completed(unavailable), Some(&mut snapshot));
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
    loader.complete(completed(restored), Some(&mut snapshot));
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
