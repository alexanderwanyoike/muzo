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
    let played = prepare_dropbox_track(&paths, track, local);

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

    assert_eq!(result.path(), path);
}

#[test]
fn it_starts_no_second_install_while_the_first_is_running() {
    let (mut desktop, _) = setup();
    let update = crate::distribution::Update {
        version: semver::Version::new(0, 1, 0),
        asset: crate::distribution::Artifact {
            name: "Muzo-0.1.0-linux-x64.AppImage".into(),
            platform: "linux".into(),
            arch: "x64".into(),
            size: 1,
            sha256: "a".repeat(64),
        },
    };
    let path = PathBuf::from("download.AppImage");
    desktop.update_status = updates::Status::Ready(update.clone(), path.clone());

    let _ = desktop.update(Message::InstallUpdate(update, path));
    assert!(matches!(
        desktop.update_status,
        updates::Status::Confirming(_, _)
    ));
    let first = desktop.update(Message::ConfirmInstallUpdate);
    let second = desktop.update(Message::ConfirmInstallUpdate);

    assert!(first.units() > 0);
    assert_eq!(second.units(), 0);
}

#[test]
fn it_keeps_current_audio_until_dropbox_disconnect_succeeds() {
    let (mut desktop, _) = setup();
    desktop.sources.snapshot = Some(snapshot(&["first", "second"]));
    let _ = desktop.update(Message::Select(0));

    let _ = desktop.update(Message::Dropbox(dropbox_panel::Action::Disconnect));

    assert_eq!(desktop.session.as_ref().unwrap().tracks().len(), 2);
    assert_eq!(desktop.now_playing().map(|(index, _)| index), Some(0));
}

#[test]
fn it_ignores_late_audio_from_a_disconnected_dropbox_account() {
    let (mut desktop, remote_audio) = setup();
    desktop.pending_track = Some(0);
    let snapshot = dropbox_panel::Snapshot {
        disconnected_libraries: vec!["first".into()],
        ..Default::default()
    };

    let _ = desktop.update(Message::DropboxReady(Ok(snapshot)));
    let _ = desktop.update(Message::RemoteTrackReady(0, Ok(remote_audio)));

    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.current_index(), None);
    assert_eq!(session.tracks().len(), 1);
    assert_eq!(session.tracks()[0].library(), Some("second"));
}

#[test]
fn it_can_expand_the_mini_queue_from_the_keyboard() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::ShowMini);
    let mini_id = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::CycleMiniFocus(mini_id, false));
    let _ = desktop.update(Message::CycleMiniFocus(mini_id, false));

    assert!(desktop.update(Message::ActivateMiniFocus(mini_id)).units() > 0);
    assert!(desktop.windows.queue_pending());
}

#[test]
fn it_skips_disabled_transport_when_cycling_mini_controls() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::ShowMini);
    let mini_id = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::CycleMiniFocus(mini_id, true));
    let _ = desktop.update(Message::CycleMiniFocus(mini_id, true));
    let _ = desktop.update(Message::CycleMiniFocus(mini_id, true));

    assert!(desktop.update(Message::ActivateMiniFocus(mini_id)).units() > 0);
    assert!(!desktop.windows.is_mini(mini_id));
}

#[test]
fn it_selects_a_visible_mini_queue_entry_with_tab_and_enter() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::MiniResized(mini, iced::Size::new(426.0, 434.0)));
    assert!(desktop.windows.queue_open());

    for _ in 0..5 {
        let _ = desktop.update(Message::CycleMiniFocus(mini, false));
    }
    let _ = desktop.update(Message::ActivateMiniFocus(mini));

    assert_eq!(desktop.now_playing().map(|(index, _)| index), Some(1));
}

#[test]
fn it_scrolls_to_a_focused_row_in_a_long_mini_queue() {
    let (mut desktop, track) = setup();
    desktop
        .session
        .as_mut()
        .unwrap()
        .extend(std::iter::repeat_n(track, 18));
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::MiniResized(mini, iced::Size::new(426.0, 434.0)));

    let reveal = desktop.update(Message::CycleMiniFocus(mini, true));

    assert_eq!(desktop.mini_focus, Some(MiniControl::Track(19)));
    assert!(reveal.units() > 0);
}

#[test]
fn it_rebases_mini_queue_focus_after_a_source_is_removed() {
    let (mut desktop, _) = setup();
    desktop.sources.snapshot = Some(snapshot(&["first", "second"]));
    desktop.mini_focus = Some(MiniControl::Track(1));

    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&["second"]))));

    assert_eq!(desktop.mini_focus, Some(MiniControl::Track(0)));
}

#[test]
fn it_uses_the_resized_sidebar_in_the_library_without_changing_mini_geometry() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::ResizeSidebar(0.39));
    assert_eq!(desktop.sidebar.visible_width(), 278.0);
    let _ = desktop.update(Message::ShowMini);
    let window = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::MiniResized(window, iced::Size::new(426.0, 236.0)));
    assert_eq!(desktop.sidebar.visible_width(), 278.0);
}

#[test]
fn it_measures_minimum_main_content_inside_the_library_chrome() {
    let mut sidebar = sidebar::Sidebar::default();
    sidebar.layout_width(900.0);
    sidebar.set(crate::appearance::sidebar::SidebarWidth::new(360.0));

    assert_eq!(sidebar.visible_width(), 328.0);
}

#[test]
fn it_saves_the_completed_sidebar_resize_even_when_the_library_closes_immediately() {
    let (mut desktop, _) = setup();
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    desktop.sources = Sources::new(Some(paths.clone()));
    let _ = desktop.update(Message::ResizeSidebar(0.30));
    let _ = desktop.update(Message::CloseRequested(desktop.windows.library_id()));

    assert_eq!(
        crate::appearance::sidebar::load_or_import(&paths.sidebar(), None)
            .unwrap()
            .visible(850.0),
        desktop.sidebar.visible_width(),
    );
}

#[test]
fn it_keeps_install_download_progress_when_another_check_arrives() {
    let (mut desktop, _) = setup();
    desktop.update_status = updates::Status::Downloading(42);

    let check = desktop.update(Message::CheckUpdate);

    assert_eq!(check.units(), 0);
    assert!(matches!(
        desktop.update_status,
        updates::Status::Downloading(42)
    ));
}

#[test]
fn it_can_reopen_cached_catalogue_artwork_without_the_original_audio() {
    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let mut catalogue = paths.open().unwrap();
    let media = root.path().join("music");
    std::fs::create_dir(&media).unwrap();
    let file = media.join("song.mp3");
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/embedded-cover.mp3");
    std::fs::copy(fixture, &file).unwrap();
    let library = catalogue.add_folder(&media).unwrap();
    let row = catalogue.tracks(&library.id).unwrap().remove(0);
    let item = Track::new(file.clone(), row.title, None, None, None)
        .with_indexed_id(row.id)
        .with_library(library.id);

    let first = prepare_local_track(Some(&paths), item.clone());
    assert!(first.artwork().is_some());
    std::fs::remove_file(file).unwrap();
    let reopened = prepare_local_track(Some(&paths), item);
    assert_eq!(reopened.artwork(), first.artwork());
}

#[test]
fn it_saves_a_successful_play_before_the_next_background_poll() {
    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let mut catalogue = paths.open().unwrap();
    let media = root.path().join("music");
    std::fs::create_dir(&media).unwrap();
    let (mut desktop, audio) = setup();
    let file = media.join("song.mp3");
    std::fs::copy(audio.path(), &file).unwrap();
    let library = catalogue.add_folder(&media).unwrap();
    let row = catalogue.tracks(&library.id).unwrap().remove(0);
    let row_id = row.id.clone();
    let item = read_track(file)
        .with_indexed_id(row.id)
        .with_library(library.id.clone())
        .with_artwork(vec![0]);
    desktop.sources = Sources::new(Some(paths.clone()));
    desktop.session.as_mut().unwrap().replace(vec![item]);

    let write = desktop.update(Message::Select(0));
    assert!(write.units() > 0);
    assert_eq!(desktop.update(Message::TogglePlayback).units(), 0);
    assert_eq!(desktop.update(Message::TogglePlayback).units(), 0);
    let count = persist_play(&paths, "event", &library.id, &row_id, 1).unwrap();
    let _ = desktop.update(Message::PlayRecorded(row_id, Ok(count)));

    assert_eq!(
        paths.open().unwrap().tracks(&library.id).unwrap()[0].plays,
        1
    );
}

#[test]
fn it_keeps_a_cached_dropbox_cover_visible_when_the_audio_request_fails() {
    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let mut catalogue = paths.open().unwrap();
    catalogue
        .save_dropbox_account("account", "app-key")
        .unwrap();
    let library = catalogue.add_dropbox("account", "/Music", "Music").unwrap();
    let entry = serde_json::from_str(
        r#"{".tag":"file","name":"Track.mp3","path_display":"/Music/Track.mp3","size":48,"rev":"1"}"#,
    ).unwrap();
    catalogue.sync_dropbox(&library.id, &[entry]).unwrap();
    let row = catalogue.tracks(&library.id).unwrap().remove(0);
    let revision = catalogue.artwork_revision(&library.id, &row.id).unwrap();
    crate::artwork_cache::ArtworkCache::new(paths.artwork_cache())
        .get_or_read(&revision, || {
            Some(include_bytes!("../../../desktop/assets/icons/muzo.png").to_vec())
        })
        .unwrap();
    let (mut desktop, _) = setup();
    let track = Track::new(PathBuf::new(), row.title, None, None, None)
        .with_indexed_id(row.id)
        .with_library(library.id.clone())
        .with_remote(library.id, "/Music/Track.mp3".into());
    desktop.session.as_mut().unwrap().replace(vec![track]);
    desktop.sources = Sources::new(Some(paths));
    let _ = desktop.update(Message::Select(0));
    let request = desktop.playback_request;
    let _ = desktop.update(Message::RemoteTrackReady(request, Err("Offline".into())));

    assert!(desktop.artwork.is_some());
    assert_eq!(desktop.error.as_deref(), Some("Offline"));
}

#[test]
fn it_does_not_regress_a_committed_play_count_when_an_older_snapshot_arrives() {
    let (mut desktop, _) = setup();
    let mut older = snapshot(&["first"]);
    let row = crate::catalogue::IndexedTrack {
        id: "track".into(),
        title: "Track".into(),
        artist: String::new(),
        album: None,
        path: PathBuf::from("track.mp3"),
        plays: 0,
        library_id: "first".into(),
    };
    older.tracks.push(row);
    let mut newer = older.clone();
    newer.tracks[0].plays = 1;
    desktop.sources.snapshot = Some(newer);

    let _ = desktop.update(Message::SourcesUpdated(Ok(older)));

    assert_eq!(desktop.sources.snapshot.unwrap().tracks[0].plays, 1);
}

#[test]
fn it_dispatches_play_history_off_the_transport_thread_and_waits_for_it_on_close() {
    let root = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(root.path());
    let (mut desktop, audio) = setup();
    let track = audio
        .with_indexed_id("track".into())
        .with_library("library".into())
        .with_artwork(vec![0]);
    desktop.sources = Sources::new(Some(paths));
    desktop.session.as_mut().unwrap().replace(vec![track]);

    assert!(desktop.update(Message::Select(0)).units() > 0);
    assert_eq!(
        desktop
            .update(Message::CloseRequested(desktop.windows.library_id()))
            .units(),
        0
    );
}
