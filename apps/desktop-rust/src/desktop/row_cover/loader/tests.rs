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

    let cover = load(&paths, &Remote::default(), track, false);
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
            cover: Some(Handle::from_rgba(1, 1, vec![255; 4])),
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
