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
    desktop.loading_remote = true;

    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&["second"]))));
    let _ = desktop.update(Message::RemoteTrackReady(
        0,
        1,
        Ok(local.with_library("second".into())),
    ));

    assert_eq!(desktop.now_playing().map(|(index, _)| index), Some(0));
}
