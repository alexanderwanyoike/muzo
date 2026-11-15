use super::*;
use std::num::NonZero;

pub(super) fn setup() -> (Desktop, Track) {
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

pub(super) fn snapshot(ids: &[&str]) -> Snapshot {
    Snapshot {
        libraries: ids.iter().map(|id| source(id)).collect(),
        selected: None,
        tracks: vec![],
        covers: Default::default(),
        track_counts: Default::default(),
        playlist_counts: Default::default(),
        warnings: vec![],
        playlists: vec![],
        selected_playlist: None,
        playlist_entries: vec![],
        editor: None,
    }
}

#[test]
fn it_clears_a_source_search_without_changing_the_selected_library() {
    let (mut desktop, _) = setup();
    let mut sources = snapshot(&["first", "second"]);
    sources.selected = Some("second".into());
    desktop.sources.snapshot = Some(sources);
    let _ = desktop.update(Message::SourceSearch("archive".into()));

    let clear = desktop.update(Message::ClearSourceSearch);

    assert!(desktop.sources.search.is_empty());
    assert_eq!(
        desktop
            .sources
            .snapshot
            .as_ref()
            .unwrap()
            .selected
            .as_deref(),
        Some("second")
    );
    assert_ne!(clear.units(), 0);
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
fn it_drops_a_dropbox_refresh_result_after_the_source_is_removed() {
    let (mut desktop, _) = setup();
    desktop.sources.snapshot = Some(snapshot(&["first", "second"]));
    desktop.dropbox.busy = true;
    desktop.dropbox.refreshing = Some("second".into());

    let _ = desktop.update(Message::SourcesUpdated(Ok(snapshot(&["first"]))));
    let _ = desktop.update(Message::DropboxLibraryRefreshed(
        "second".into(),
        Err(dropbox_panel::RefreshError::Failed("Stale error".into())),
    ));

    assert!(desktop.dropbox.refresh_status.is_none());
    assert!(!desktop.dropbox.busy);
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

    assert!(
        desktop
            .update(Message::ActivateMiniFocus(mini_id, false))
            .units()
            > 0
    );
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

    assert!(
        desktop
            .update(Message::ActivateMiniFocus(mini_id, false))
            .units()
            > 0
    );
    assert!(!desktop.windows.is_mini(mini_id));
}

#[test]
fn it_selects_a_visible_mini_queue_entry_with_tab_and_enter() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::MiniResized(mini, iced::Size::new(740.0, 560.0)));
    assert!(desktop.windows.queue_open());

    for _ in 0..5 {
        let _ = desktop.update(Message::CycleMiniFocus(mini, false));
    }
    let _ = desktop.update(Message::ActivateMiniFocus(mini, false));

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
    let _ = desktop.update(Message::MiniResized(mini, iced::Size::new(740.0, 560.0)));

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
    let _ = desktop.update(Message::ResizeSidebar(sidebar::Resize::Preview(300.0)));
    let _ = desktop.update(Message::ResizeSidebar(sidebar::Resize::Commit(300.0)));
    assert_eq!(desktop.sidebar.visible_width(), 300.0);
    let _ = desktop.update(Message::ShowMini);
    let window = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::MiniResized(window, iced::Size::new(420.0, 560.0)));
    assert_eq!(desktop.sidebar.visible_width(), 300.0);
}

#[test]
fn it_measures_minimum_main_content_inside_the_library_chrome() {
    let mut sidebar = sidebar::Sidebar::default();
    sidebar.layout_width(850.0);
    sidebar.set(crate::appearance::sidebar::SidebarWidth::new(360.0));

    assert_eq!(sidebar.visible_width(), 329.0);
}

#[test]
fn it_saves_the_completed_sidebar_resize_even_when_the_library_closes_immediately() {
    let (mut desktop, _) = setup();
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    desktop.sources = Sources::new(Some(paths.clone()));
    let _ = desktop.update(Message::ResizeSidebar(sidebar::Resize::Preview(360.0)));
    let _ = desktop.update(Message::ResizeSidebar(sidebar::Resize::Commit(360.0)));
    let _ = desktop.update(Message::CloseRequested(desktop.windows.library_id()));

    assert_eq!(
        crate::appearance::sidebar::load_or_import(&paths.sidebar(), None)
            .unwrap()
            .visible(1199.0),
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

    assert!(desktop.cover.thumbnail().is_some());
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
        metadata_overridden: false,
        path: PathBuf::from("track.mp3"),
        plays: 0,
        library_id: "first".into(),
        duration_seconds: 0,
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

#[test]
fn it_pauses_the_active_playlist_entry_and_plays_duplicates_independently() {
    let (mut desktop, audio) = setup();
    let mut playlist = snapshot(&[]);
    playlist.selected_playlist = Some("playlist".into());
    let indexed = crate::catalogue::IndexedTrack {
        id: "shared-song".into(),
        title: audio.title().into(),
        artist: "Artist".into(),
        album: None,
        metadata_overridden: false,
        path: audio.path().into(),
        plays: 0,
        library_id: "local-library".into(),
        duration_seconds: 0,
    };
    playlist.playlist_entries = ["first", "second"]
        .into_iter()
        .map(|id| crate::catalogue::PlaylistEntry {
            id: id.into(),
            track: Some(indexed.clone()),
        })
        .collect();
    desktop.sources.snapshot = Some(playlist);

    for entry in ["first", "second"] {
        let _ = desktop.update(Message::PlayPlaylist(entry.into()));
        let index = desktop.pending_track.unwrap();
        let ready = desktop.session.as_ref().unwrap().tracks()[index].clone();
        let _ = desktop.update(Message::RemoteTrackReady(
            desktop.playback_request,
            Ok(ready),
        ));
        let session = desktop.session.as_ref().unwrap();
        assert_eq!(session.current().unwrap().playlist_entry(), Some(entry));
        assert!(session.is_playing());
    }

    desktop
        .sources
        .snapshot
        .as_mut()
        .unwrap()
        .playlist_entries
        .reverse();
    let before = desktop.session.as_ref().unwrap().start_generation();
    let _ = desktop.update(Message::PlayPlaylist("second".into()));
    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.current().unwrap().playlist_entry(), Some("second"));
    assert_eq!(session.start_generation(), before);
    assert!(!session.is_playing());
    assert!(desktop.pending_track.is_none());

    desktop
        .sources
        .snapshot
        .as_mut()
        .unwrap()
        .playlist_entries
        .retain(|entry| entry.id != "second");
    let _ = desktop.update(Message::PlayPlaylist("second".into()));
    assert!(!desktop.session.as_ref().unwrap().is_playing());

    let _ = desktop.update(Message::PlayPlaylist("missing".into()));
    assert_eq!(desktop.session.as_ref().unwrap().start_generation(), before);
}

#[test]
fn it_drops_the_mini_artwork_background_when_the_next_song_has_no_cover() {
    let (mut desktop, plain) = setup();
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/embedded-cover.mp3");
    let covered = read_track(fixture);
    desktop
        .session
        .as_mut()
        .unwrap()
        .replace(vec![covered, plain]);

    let _ = desktop.update(Message::Select(0));

    assert!(desktop.cover.background().is_some());

    let _ = desktop.update(Message::Select(1));

    assert!(desktop.cover.background().is_none());
}

#[test]
fn it_restarts_the_playback_queue_from_the_first_playable_playlist_entry() {
    let (mut desktop, audio) = setup();
    let mut playlist = snapshot(&[]);
    playlist.selected_playlist = Some("playlist".into());
    let indexed = crate::catalogue::IndexedTrack {
        id: "song".into(),
        title: audio.title().into(),
        artist: "Artist".into(),
        album: None,
        metadata_overridden: false,
        path: audio.path().into(),
        plays: 0,
        duration_seconds: 3,
        library_id: "local-library".into(),
    };
    playlist.playlist_entries = vec![
        crate::catalogue::PlaylistEntry {
            id: "unavailable".into(),
            track: None,
        },
        crate::catalogue::PlaylistEntry {
            id: "first".into(),
            track: Some(indexed.clone()),
        },
        crate::catalogue::PlaylistEntry {
            id: "second".into(),
            track: Some(indexed),
        },
    ];
    desktop.sources.snapshot = Some(playlist);
    let _ = desktop.update(Message::PlayPlaylist("first".into()));
    let ready = desktop.session.as_ref().unwrap().tracks()[0].clone();
    let _ = desktop.update(Message::RemoteTrackReady(
        desktop.playback_request,
        Ok(ready),
    ));
    let generation = desktop.session.as_ref().unwrap().start_generation();

    let _ = desktop.update(Message::StartPlaylist("playlist".into()));
    assert_eq!(desktop.pending_track, Some(0));
    let ready = desktop.session.as_ref().unwrap().tracks()[0].clone();
    let _ = desktop.update(Message::RemoteTrackReady(
        desktop.playback_request,
        Ok(ready),
    ));
    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.start_generation(), generation + 1);
    assert_eq!(session.current().unwrap().playlist_entry(), Some("first"));
}

#[test]
fn it_keeps_the_new_playlist_name_when_saving_fails() {
    let (mut desktop, _) = setup();
    let root = tempfile::tempdir().unwrap();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(root.path())));
    desktop.sources.show_playlist_form = true;
    desktop.sources.playlist_name = "After Hours".into();
    desktop.sources.playlist_search = "Other".into();
    let _ = desktop.update(Message::CreatePlaylist);
    let _ = desktop.update(Message::SourcesUpdated(Err("Cannot write database".into())));
    assert!(desktop.sources.show_playlist_form);
    assert_eq!(desktop.sources.playlist_name, "After Hours");
    assert_eq!(desktop.sources.playlist_search, "Other");
}

#[test]
fn it_clears_direct_covers_when_catalogue_playback_replaces_the_queue() {
    let (mut desktop, audio) = setup();
    let mut source = snapshot(&[]);
    source.tracks = vec![crate::catalogue::IndexedTrack {
        id: "catalogue-song".into(),
        title: audio.title().into(),
        artist: "".into(),
        album: None,
        metadata_overridden: false,
        path: audio.path().into(),
        plays: 0,
        duration_seconds: 1,
        library_id: "library".into(),
    }];
    desktop.sources.snapshot = Some(source);
    desktop.direct_covers.insert(
        PathBuf::from("previous.mp3"),
        iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 255]),
    );

    let _ = desktop.update(Message::PlayIndexed(0));

    assert!(desktop.direct_covers.is_empty());
}

#[test]
fn it_can_choose_the_next_appearance_material_with_tab_and_space() {
    let (mut desktop, _) = setup();
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::MiniSpace(library, false));

    assert_eq!(desktop.appearance.theme, "pulse");
}

#[test]
fn it_activates_the_material_clicked_after_keyboard_navigation() {
    let (mut desktop, _) = setup();
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::Material("walnut"));
    let _ = desktop.update(Message::MiniSpace(library, false));

    assert_eq!(desktop.appearance.theme, "walnut");
}

#[test]
fn it_clears_the_appearance_choice_when_a_text_entry_captures_a_tap() {
    let (mut desktop, _) = setup();
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let tap = iced::Event::Touch(iced::touch::Event::FingerPressed {
        id: iced::touch::Finger(0),
        position: iced::Point::new(50.0, 340.0),
    });
    if let Some(message) = input_message(tap, iced::event::Status::Captured, library) {
        let _ = desktop.update(message);
    }
    let _ = desktop.update(Message::ActivateMiniFocus(library, false));

    assert_eq!(desktop.appearance.theme, "studio");
}

#[test]
fn it_keeps_the_submitted_playlist_draft_during_a_pending_save() {
    let (mut desktop, _) = setup();
    let root = tempfile::tempdir().unwrap();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(root.path())));
    let _ = desktop.update(Message::ShowPlaylistForm(true));
    let _ = desktop.update(Message::PlaylistName("Evening".into()));
    let _ = desktop.update(Message::CreatePlaylist);
    let _ = desktop.update(Message::ShowPlaylistForm(false));
    let _ = desktop.update(Message::PlaylistName("Later".into()));
    let _ = desktop.update(Message::SourcesUpdated(Err("Cannot write database".into())));
    assert!(desktop.sources.show_playlist_form);
    assert_eq!(desktop.sources.playlist_name, "Evening");

    let _ = desktop.update(Message::PlaylistName("Late Evening".into()));
    assert_eq!(desktop.sources.playlist_name, "Late Evening");
}

#[test]
fn it_reports_current_after_a_successful_check_without_an_update() {
    let (mut desktop, _) = setup();
    desktop.update_status = updates::Status::Checking;
    let incoming = updates::Status::from_check(Ok(None));
    let _ = desktop.update(Message::UpdateReceived(incoming));

    assert_eq!(desktop.update_status.notice_label(), "Up to date");
    assert!(desktop.update_status.can_check());
}

#[test]
fn it_paints_the_walnut_material_in_the_mini_window() {
    use iced::{
        Color, Font, Pixels, Rectangle, Renderer, Size,
        advanced::{
            Layout, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::Material("walnut"));
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();
    let mut view = desktop.view(mini);
    let theme = desktop.theme(mini);
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut tree = Tree::new(&view);
    let bounds = Rectangle::with_size(Size::new(420.0, 560.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(bounds.size(), bounds.size()),
    );
    widget.draw(
        &tree,
        &mut renderer,
        &theme,
        &Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &bounds,
    );
    let pixels = renderer.screenshot(Size::new(420, 560), 1.0, Color::WHITE);
    let header = pixels.chunks_exact(420 * 4).nth(9).unwrap();
    let grain = &header[70 * 4..230 * 4];
    assert!(grain.chunks_exact(4).any(|pixel| pixel != &grain[..4]));
}

#[test]
fn it_opens_and_reveals_source_setup_from_the_sidebar() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Songs));

    let reveal = desktop.update(Message::OpenSourceSettings);

    assert_eq!(desktop.page, LibraryPage::Settings);
    assert_eq!(reveal.units(), 1);
}

#[test]
fn it_lists_every_indexed_source_and_location_in_settings() {
    let (mut desktop, _) = setup();
    let mut sources = snapshot(&["Records", "Field recordings"]);
    sources.libraries[0].location = PathBuf::from("/fixtures/records");
    sources.libraries[1].location = PathBuf::from("/fixtures/field recordings");
    desktop.sources.snapshot = Some(sources);
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));
    let labels = rendered_labels(&desktop);
    for location in ["/fixtures/records", "/fixtures/field recordings"] {
        assert!(
            labels.iter().any(|label| label == location),
            "missing {location}"
        );
    }
}

pub(super) fn editing_source() -> (Desktop, crate::catalogue::IndexedTrack) {
    let (mut desktop, _) = setup();
    desktop.sources = Sources::new(None);
    let track = crate::catalogue::IndexedTrack {
        id: "song".into(),
        title: "Song".into(),
        artist: "Artist".into(),
        album: None,
        metadata_overridden: false,
        path: "track.mp3".into(),
        plays: 0,
        duration_seconds: 1,
        library_id: "first".into(),
    };
    let mut selected = snapshot(&["first", "second"]);
    selected.selected = Some("first".into());
    selected.tracks.push(track.clone());
    desktop.sources.snapshot = Some(selected);
    desktop.sources.editing = Some(Box::new(library_sources::TrackEditor {
        track_id: track.id.clone(),
        library_id: track.library_id.clone(),
        fields: std::array::from_fn(|_| String::new()),
        overridden: false,
        error: None,
        action_focus: None,
    }));
    (desktop, track)
}

#[test]
fn it_discards_a_metadata_draft_when_opening_another_song() {
    let (mut desktop, mut track) = editing_source();
    track.id = "next-song".into();
    desktop
        .sources
        .snapshot
        .as_mut()
        .unwrap()
        .tracks
        .push(track.clone());

    let _ = desktop.update(Message::EditTrack(track));

    assert!(desktop.sources.editing.is_none());
}

#[test]
fn it_discards_a_metadata_draft_when_switching_sources() {
    let (mut desktop, _) = editing_source();

    let _ = desktop.update(Message::SourceOperation(Operation::Select("second".into())));

    assert!(desktop.sources.editing.is_none());
}

#[test]
fn it_does_not_reopen_the_metadata_editor_after_cancel() {
    let (mut desktop, track) = editing_source();
    let mut returned = desktop.sources.snapshot.clone().unwrap();
    returned.editor = desktop.sources.editing.clone();
    let _ = desktop.update(Message::EditTrack(track));
    let _ = desktop.update(Message::CloseTrackEdit);

    let _ = desktop.update(Message::SourcesUpdated(Ok(returned)));

    assert!(desktop.sources.editing.is_none());
}

#[test]
fn it_allows_saving_a_metadata_edit_with_a_signed_year() {
    let (mut desktop, _) = editing_source();
    let directory = tempfile::tempdir().unwrap();
    let mut sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
    sources.editing = desktop.sources.editing.take();
    desktop.sources = sources;
    let _ = desktop.update(Message::EditField(MetadataField::Year, "-120".into()));

    let save = desktop.update(Message::SaveTrackEdit(false));

    assert_ne!(save.units(), 0);
}

#[test]
fn it_keeps_the_metadata_draft_until_its_queued_save_completes() {
    let (mut desktop, _) = editing_source();
    let directory = tempfile::tempdir().unwrap();
    let mut sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
    sources.editing = desktop.sources.editing.take();
    sources.snapshot = desktop.sources.snapshot.take();
    desktop.sources = sources;
    let refreshed = desktop.sources.snapshot.clone().unwrap();
    let _ = desktop.sources.start(Operation::Reload);
    let _ = desktop.sources.start(Operation::Reload);
    let _ = desktop.update(Message::SourcesUpdated(Ok(refreshed.clone())));

    let _ = desktop.update(Message::SaveTrackEdit(false));
    let _ = desktop.sources.poll();
    let _ = desktop.update(Message::SourcesUpdated(Ok(refreshed)));

    assert!(desktop.sources.editing.is_some());
}

#[test]
fn it_clears_the_metadata_validation_error_when_cancelling() {
    let (mut desktop, _) = editing_source();
    let _ = desktop.update(Message::EditField(
        MetadataField::Track,
        "not a number".into(),
    ));
    let _ = desktop.update(Message::SaveTrackEdit(false));
    let _ = desktop.update(Message::CloseTrackEdit);

    assert!(desktop.sources.error.is_none());
}

#[test]
fn it_reports_a_metadata_write_error_after_leaving_songs() {
    let (mut desktop, _) = editing_source();
    let directory = tempfile::tempdir().unwrap();
    let mut sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
    sources.editing = desktop.sources.editing.take();
    desktop.sources = sources;
    let _ = desktop.update(Message::SaveTrackEdit(false));
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));

    let _ = desktop.update(Message::SourcesUpdated(Err("Cannot write metadata".into())));

    assert_eq!(
        desktop.sources.error.as_deref(),
        Some("Cannot write metadata")
    );
}

#[test]
fn it_requires_positive_track_and_disc_numbers() {
    let (mut desktop, _) = editing_source();

    for (field, value) in [(MetadataField::Track, "0"), (MetadataField::Disc, "-1")] {
        let _ = desktop.update(Message::EditField(field, value.into()));
        let _ = desktop.update(Message::SaveTrackEdit(false));
        assert!(desktop.sources.editing.as_ref().unwrap().error.is_some());
        let _ = desktop.update(Message::EditField(field, String::new()));
    }
}

#[test]
fn it_routes_tab_to_the_track_metadata_form_when_editing_in_songs() {
    let (mut desktop, _) = editing_source();

    let tab = desktop.update(Message::CycleMiniFocus(desktop.windows.library_id(), false));

    assert!(tab.units() > 0);
}

#[test]
fn it_reveals_the_focused_metadata_action_in_a_short_songs_viewport() {
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size, Vector,
        advanced::{
            Layout, layout,
            widget::{
                Id, Operation, Tree,
                operation::{Outcome, Scrollable},
            },
        },
    };

    #[derive(Default)]
    struct SongsOffset(f32);
    impl Operation for SongsOffset {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _: Rectangle,
            _: Rectangle,
            offset: Vector,
            _: &mut dyn Scrollable,
        ) {
            if id == Some(&"indexed-songs".into()) {
                self.0 = offset.y;
            }
        }
    }

    let (mut desktop, track) = editing_source();
    let snapshot = desktop.sources.snapshot.as_mut().unwrap();
    for index in 0..9 {
        let mut earlier = track.clone();
        earlier.id = format!("previous-{index}");
        snapshot.tracks.insert(0, earlier);
    }
    let _ = desktop.update(Message::CycleMiniFocus(desktop.windows.library_id(), false));
    let _ = desktop.update(Message::TrackEditorTab(1, false, Some(MetadataField::Year)));
    let mut view = desktop.view(desktop.windows.library_id());
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut tree = Tree::new(&view);
    let widget = view.as_widget_mut();
    let size = Size::new(800.0, 600.0);
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(size, size));
    let layout = Layout::new(&node);
    let mut reveal = track_editor_focus::reveal(TrackEditorControl::Save);
    widget.operate(&mut tree, layout, &renderer, &mut reveal);
    if let Outcome::Chain(mut scroll) = reveal.finish() {
        widget.operate(&mut tree, layout, &renderer, scroll.as_mut());
    }
    let mut offset = SongsOffset::default();
    widget.operate(&mut tree, layout, &renderer, &mut offset);

    assert!(offset.0 > 0.0);
}

#[test]
fn it_serializes_repeated_tab_requests_for_the_same_metadata_draft() {
    let (mut desktop, _) = editing_source();
    let library = desktop.windows.library_id();

    let first = desktop.update(Message::CycleMiniFocus(library, false));
    let repeated = desktop.update(Message::CycleMiniFocus(library, false));

    assert!(first.units() > 0);
    assert_eq!(repeated.units(), 0);
    let chosen = desktop.update(Message::TrackEditorTab(1, false, None));
    assert!(chosen.units() > 0);
    let next = desktop.update(Message::TrackEditorFocusApplied(1));
    assert!(next.units() > 0);
}

#[test]
fn it_ignores_an_old_tab_result_after_opening_the_same_song_again() {
    let (mut desktop, _) = editing_source();
    let previous_draft = desktop.sources.editing.clone();
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::CloseTrackEdit);
    desktop.sources.editing = previous_draft;

    let stale = desktop.update(Message::TrackEditorTab(1, false, Some(MetadataField::Year)));

    assert_eq!(stale.units(), 0);
}

pub(super) fn rendered_labels(desktop: &Desktop) -> Vec<String> {
    rendered_window_labels(
        desktop,
        desktop.windows.library_id(),
        iced::Size::new(1200.0, 800.0),
    )
}

pub(super) fn rendered_window_labels(
    desktop: &Desktop,
    id: window::Id,
    size: iced::Size,
) -> Vec<String> {
    use iced::{
        Font, Pixels, Rectangle, Renderer,
        advanced::{
            Layout, layout,
            widget::{Operation, Tree},
        },
    };

    #[derive(Default)]
    struct Labels(Vec<String>);

    impl Operation for Labels {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }

        fn text(&mut self, _: Option<&iced::advanced::widget::Id>, _: Rectangle, label: &str) {
            self.0.push(label.into());
        }
    }

    let mut view = desktop.view(id);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut tree = Tree::new(&view);
    let widget = view.as_widget_mut();
    let layout = widget.layout(&mut tree, &renderer, &layout::Limits::new(size, size));
    let mut labels = Labels::default();
    widget.operate(&mut tree, Layout::new(&layout), &renderer, &mut labels);
    let viewport = Rectangle::with_size(size);
    if let Some(mut popup) = widget.overlay(
        &mut tree,
        Layout::new(&layout),
        &renderer,
        &viewport,
        iced::Vector::ZERO,
    ) {
        let popup = popup.as_overlay_mut();
        let popup_layout = popup.layout(&renderer, size);
        popup.operate(Layout::new(&popup_layout), &renderer, &mut labels);
    }
    labels.0
}

#[test]
fn it_labels_songs_with_saved_edits_when_the_album_is_missing() {
    use crate::catalogue::{Catalogue, MetadataOverride};

    let directory = tempfile::tempdir().unwrap();
    let audio = directory.path().join("Music");
    std::fs::create_dir(&audio).unwrap();
    std::fs::write(
        audio.join("song.mp3"),
        include_bytes!("../../tests/fixtures/tone.mp3"),
    )
    .unwrap();
    let database = directory.path().join("catalogue.sqlite");
    let mut catalogue = Catalogue::open(&database, None).unwrap();
    let library = catalogue.add_folder(&audio).unwrap();
    rusqlite::Connection::open(&database)
        .unwrap()
        .execute("UPDATE tracks SET artist='Mara Vale'", [])
        .unwrap();
    let original = catalogue.tracks(&library.id).unwrap().remove(0);
    assert!(original.album.is_none());

    let (mut desktop, _) = setup();
    let mut sources = snapshot(&[&library.id]);
    sources.selected = Some(library.id.clone());
    sources.libraries = vec![library.clone()];
    desktop.sources.snapshot = Some(sources);
    let mut song_labels = || {
        desktop.sources.snapshot.as_mut().unwrap().tracks = catalogue.tracks(&library.id).unwrap();
        rendered_labels(&desktop)
    };

    assert!(song_labels().contains(&original.artist));
    let edit = MetadataOverride {
        year: Some(1998),
        ..MetadataOverride::default()
    };
    catalogue.override_track(&original.id, &edit).unwrap();

    let edited = format!("{} - Edited in Muzo", original.artist);
    assert!(song_labels().contains(&edited));

    let edit_with_album = MetadataOverride {
        album: Some("Field Recordings".into()),
        ..edit
    };
    catalogue
        .override_track(&original.id, &edit_with_album)
        .unwrap();
    let labels = song_labels();
    assert!(labels.contains(&format!("{} - Field Recordings", original.artist)));
    assert!(!labels.contains(&edited));

    let edit_with_empty_album = MetadataOverride {
        album: Some(String::new()),
        ..edit_with_album
    };
    catalogue
        .override_track(&original.id, &edit_with_empty_album)
        .unwrap();
    let labels = song_labels();
    assert!(labels.contains(&original.artist));
    assert!(!labels.contains(&edited));

    catalogue
        .override_track(&original.id, &MetadataOverride::default())
        .unwrap();
    let labels = song_labels();
    assert!(labels.contains(&original.artist));
    assert!(!labels.contains(&edited));
}

#[test]
fn it_opens_the_playlist_picker_inline_even_without_a_playlist() {
    let (mut desktop, track) = editing_source();
    let _ = desktop.update(Message::OpenPlaylistPicker(track));

    let labels = rendered_labels(&desktop);

    assert!(labels.iter().any(|label| label == "No playlists yet."));
    assert!(labels.iter().any(|label| label == "Cancel"));
    assert!(labels.iter().any(|label| label == "Song"));
    assert!(!labels.iter().any(|label| label == "Save metadata"));
}

#[test]
fn it_retains_the_playlist_choice_after_cancelling_a_track_add() {
    let (mut desktop, track) = editing_source();
    desktop.sources.snapshot.as_mut().unwrap().playlists = vec![
        crate::catalogue::Playlist {
            id: "first".into(),
            name: "First".into(),
        },
        crate::catalogue::Playlist {
            id: "second".into(),
            name: "Second".into(),
        },
    ];
    let _ = desktop.update(Message::OpenPlaylistPicker(track.clone()));
    let _ = desktop.update(Message::ChooseTrackPlaylist("second".into()));
    let _ = desktop.update(Message::ClosePlaylistPicker);
    let _ = desktop.update(Message::OpenPlaylistPicker(track));

    assert_eq!(
        desktop
            .sources
            .playlist_picker
            .as_ref()
            .unwrap()
            .playlist
            .as_deref(),
        Some("second")
    );
}

#[test]
fn it_does_not_activate_an_old_metadata_button_while_tab_is_moving_focus() {
    let (mut desktop, _) = editing_source();
    let library = desktop.windows.library_id();
    desktop.sources.editing.as_mut().unwrap().action_focus = Some(TrackEditorControl::Cancel);
    let _ = desktop.update(Message::CycleMiniFocus(library, false));

    let _ = desktop.update(Message::ActivateMiniFocus(library, false));

    assert!(desktop.sources.editing.is_some());
}

#[test]
fn it_keeps_space_on_an_unavailable_metadata_action_out_of_playback() {
    let (mut desktop, _) = editing_source();
    let library = desktop.windows.library_id();
    let directory = tempfile::tempdir().unwrap();
    let mut sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
    sources.editing = desktop.sources.editing.take();
    sources.editing.as_mut().unwrap().action_focus = Some(TrackEditorControl::Save);
    desktop.sources = sources;
    let _ = desktop.sources.start(Operation::Reload);
    let _ = desktop.update(Message::Select(0));
    assert!(desktop.session.as_ref().unwrap().is_playing());

    let _ = desktop.update(Message::MiniSpace(library, false));

    assert!(desktop.session.as_ref().unwrap().is_playing());
}

#[test]
fn it_changes_the_focused_playlist_choice_with_the_arrow_keys() {
    let (mut desktop, track) = editing_source();
    let library = desktop.windows.library_id();
    desktop.sources.snapshot.as_mut().unwrap().playlists = vec![
        crate::catalogue::Playlist {
            id: "first".into(),
            name: "First".into(),
        },
        crate::catalogue::Playlist {
            id: "second".into(),
            name: "Second".into(),
        },
    ];
    let _ = desktop.update(Message::OpenPlaylistPicker(track));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::MiniArrow(library, 1));

    assert_eq!(
        desktop
            .sources
            .playlist_picker
            .as_ref()
            .unwrap()
            .playlist
            .as_deref(),
        Some("second")
    );
    assert!(!desktop.sources.track_add_pending());
}

#[test]
fn it_keeps_space_from_pausing_music_while_the_playlist_picker_is_open() {
    let (mut desktop, track) = editing_source();
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::Select(0));
    let _ = desktop.update(Message::OpenPlaylistPicker(track));

    let _ = desktop.update(Message::MiniSpace(library, false));

    assert!(desktop.session.as_ref().unwrap().is_playing());
}

#[test]
fn it_keeps_horizontal_arrows_out_of_playback_while_playlist_cancel_is_focused() {
    let (mut desktop, track) = editing_source();
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::OpenPlaylistPicker(track));
    let _ = desktop.update(Message::CycleMiniFocus(library, true));

    assert!(desktop.route_playlist_picker_arrow(library, 1).is_some());
}

fn playlist_popup_is_visible(desktop: &Desktop) -> bool {
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size, Vector,
        advanced::{Layout, layout, widget::Tree},
    };

    let mut view = desktop.view(desktop.windows.library_id());
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let viewport = Rectangle::with_size(Size::new(1200.0, 800.0));
    let mut tree = Tree::new(&view);
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, viewport.size()),
    );
    widget
        .overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .is_some()
}

#[test]
fn it_opens_the_playlist_popup_and_confirms_a_keyboard_choice() {
    let (mut desktop, track) = editing_source();
    let library = desktop.windows.library_id();
    desktop.sources.snapshot.as_mut().unwrap().playlists = ["First", "Second"]
        .into_iter()
        .map(|name| crate::catalogue::Playlist {
            id: name.to_lowercase(),
            name: name.into(),
        })
        .collect();
    let _ = desktop.update(Message::OpenPlaylistPicker(track));
    let _ = desktop.update(Message::CycleMiniFocus(library, false));
    let _ = desktop.update(Message::ActivateMiniFocus(library, false));

    assert!(playlist_popup_is_visible(&desktop));
    let _ = desktop.update(Message::PlaylistPickerVerticalArrow(library, 1));
    assert_eq!(
        desktop
            .sources
            .playlist_picker
            .as_ref()
            .unwrap()
            .playlist
            .as_deref(),
        Some("first")
    );
    let _ = desktop.update(Message::ActivateMiniFocus(library, false));

    assert!(!playlist_popup_is_visible(&desktop));
    assert_eq!(
        desktop
            .sources
            .playlist_picker
            .as_ref()
            .unwrap()
            .playlist
            .as_deref(),
        Some("second")
    );
    assert!(!desktop.sources.track_add_pending());
}

#[test]
fn it_ignores_playlist_confirmation_repeats_and_cancels_the_preview_on_escape() {
    use iced::keyboard::{
        Event, Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    };

    let press = |desktop: &mut Desktop, name, repeat| {
        let key = Key::Named(name);
        let event = iced::Event::Keyboard(Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            text: None,
            repeat,
        });
        let library = desktop.windows.library_id();
        if let Some(message) = input_message(event, iced::event::Status::Ignored, library) {
            let _ = desktop.update(message);
        }
    };
    for confirmation in [Named::Enter, Named::Space] {
        let (mut desktop, track) = editing_source();
        desktop.sources.snapshot.as_mut().unwrap().playlists = ["First", "Second"]
            .into_iter()
            .map(|name| crate::catalogue::Playlist {
                id: name.to_lowercase(),
                name: name.into(),
            })
            .collect();
        let _ = desktop.update(Message::OpenPlaylistPicker(track));
        press(&mut desktop, Named::Tab, false);
        press(&mut desktop, confirmation, false);
        press(&mut desktop, Named::ArrowDown, true);
        press(&mut desktop, confirmation, true);

        assert!(playlist_popup_is_visible(&desktop));
        assert_eq!(
            desktop
                .sources
                .playlist_picker
                .as_ref()
                .unwrap()
                .preview
                .as_deref(),
            Some("second")
        );

        press(&mut desktop, Named::Escape, false);

        assert!(!playlist_popup_is_visible(&desktop));
        assert_eq!(
            desktop
                .sources
                .playlist_picker
                .as_ref()
                .unwrap()
                .playlist
                .as_deref(),
            Some("first")
        );
        assert!(!desktop.sources.track_add_pending());
    }
}
