use super::*;
use std::num::NonZero;

fn setup() -> (Desktop, Track) {
    let (mixer, _) = rodio::mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let mut session = Session::new(Playback::new(mixer));
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let local = read_track(path);
    session.extend([
        local.clone().with_library("first".into()),
        local.clone().with_library("second".into()),
    ]);
    (Desktop::with_session(Ok(session)).0, local)
}

fn source(id: &str) -> crate::catalogue::Library {
    crate::catalogue::Library {
        id: id.into(),
        name: id.into(),
        kind: "dropbox".into(),
        location: PathBuf::from(id),
    }
}

fn snapshot(ids: &[&str]) -> Snapshot {
    Snapshot {
        libraries: ids.iter().map(|id| source(id)).collect(),
        selected: None,
        tracks: vec![],
        warnings: vec![],
        playlists: vec![],
        selected_playlist: None,
        playlist_entries: vec![],
        editor: None,
    }
}

#[test]
fn it_retries_the_same_track_after_an_earlier_source_is_removed() {
    let (mut desktop, _) = setup();
    desktop.sources.snapshot = Some(snapshot(&["first", "second"]));
    desktop.failed_track = Some(1);

    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&["second"]))));
    let _ = desktop.update(Message::TogglePlayback);

    assert_eq!(desktop.now_playing().map(|(index, _)| index), Some(0));
}

#[test]
fn it_drops_the_retry_when_the_failed_source_is_removed() {
    let (mut desktop, _) = setup();
    desktop.sources.snapshot = Some(snapshot(&["first", "second"]));
    desktop.failed_track = Some(1);
    desktop.error = Some("Audio fetch failed".into());

    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&["first"]))));

    assert_eq!(desktop.failed_track, None);
    assert_eq!(desktop.error, None);
}

#[test]
fn it_reevaluates_a_pending_track_index_when_an_unrelated_source_is_removed() {
    let (mut desktop, local) = setup();
    desktop.sources.snapshot = Some(snapshot(&["first", "second"]));
    desktop.pending_track = Some(1);

    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&["second"]))));
    let _ = desktop.update(Message::RemoteTrackReady(
        0,
        Ok(local.with_library("second".into())),
    ));

    assert_eq!(desktop.now_playing().map(|(index, _)| index), Some(0));
}

#[test]
fn it_pauses_current_audio_even_after_another_track_fails_to_load() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::Select(0));
    assert!(desktop.session.as_ref().unwrap().is_playing());
    desktop.failed_track = Some(1);

    let _ = desktop.update(Message::TogglePlayback);

    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.current_index(), Some(0));
    assert!(!session.is_playing());
    assert_eq!(desktop.failed_track, Some(1));
}

#[test]
fn it_keeps_the_catalogued_title_when_a_cached_download_has_no_embedded_title() {
    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let mut catalogue = paths.open().unwrap();
    catalogue
        .save_dropbox_account("account", "app-key")
        .unwrap();
    let library = catalogue.add_dropbox("account", "/Music", "Music").unwrap();
    let entry = serde_json::from_str(
        r#"{".tag":"file","name":"Track.mp3","path_display":"/Music/Track.mp3","size":48,"rev":"1"}"#,
    )
    .unwrap();
    catalogue.sync_dropbox(&library.id, &[entry]).unwrap();
    let indexed = catalogue.tracks(&library.id).unwrap().remove(0);
    let (_, audio) = setup();
    let local = root.path().join("8b2e-2c9f.mp3");
    std::fs::copy(audio.path(), &local).unwrap();

    let track = Track::new(PathBuf::new(), indexed.title.clone(), None, None, None)
        .with_indexed_id(indexed.id);
    let played = prepare_dropbox_track(&paths, track, local).unwrap();

    assert_eq!(played.title(), indexed.title);
    assert_eq!(
        catalogue.tracks(&library.id).unwrap()[0].title,
        indexed.title
    );
}

#[test]
fn it_plays_a_download_if_its_optional_metadata_cache_cannot_be_opened() {
    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    std::fs::create_dir(root.path().join("muzo-rust.sqlite")).unwrap();
    let (_, audio) = setup();
    let path = audio.path().to_path_buf();
    let result = prepare_dropbox_track(&paths, audio.with_indexed_id("track".into()), path.clone());

    assert_eq!(result.unwrap().path(), path);
}
