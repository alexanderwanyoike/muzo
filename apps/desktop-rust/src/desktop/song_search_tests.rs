use iced::{
    Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree, operation::TextInput},
    },
};

use super::{Desktop, queue_tests::setup};

#[derive(Default)]
struct SearchInputs(Vec<Id>);

impl Operation for SearchInputs {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text_input(&mut self, id: Option<&Id>, _: Rectangle, _: &mut dyn TextInput) {
        self.0.extend(id.cloned());
    }
}

fn search_inputs(desktop: &Desktop) -> Vec<Id> {
    let mut view = desktop.workspace(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let frame = Size::new(1000.0, 800.0);
    let widget = view.as_widget_mut();
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(frame, frame));
    let mut inputs = SearchInputs::default();
    widget.operate(&mut tree, Layout::new(&node), &renderer, &mut inputs);
    inputs.0
}

#[test]
fn it_offers_global_song_search_even_without_a_selected_source() {
    let (desktop, _) = setup();

    assert!(
        search_inputs(&desktop).contains(&Id::new("track-search")),
        "Songs must offer search before the user selects a source"
    );
}

fn next_output(
    task: iced::Task<super::Message>,
    accept: impl Fn(&super::Message) -> bool,
) -> super::Message {
    use iced::futures::StreamExt;
    use iced_runtime::{Action, task};

    let mut events = task::into_stream(task).expect("the search action should schedule work");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    runtime.block_on(async {
        while let Some(action) = events.next().await {
            if let Action::Output(message) = action
                && accept(&message)
            {
                return message;
            }
        }
        panic!("search work should produce an output");
    })
}

#[test]
fn it_finds_indexed_local_and_dropbox_songs_and_opens_a_result_without_switching_scope() {
    use super::{
        Message,
        library_sources::{CataloguePaths, Operation as SourceOperation, Sources, run},
    };
    use crate::{catalogue::MetadataOverride, dropbox::Entry};

    let data = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(data.path());
    let mut catalogue = paths.open().unwrap();
    let music = data.path().join("local");
    std::fs::create_dir(&music).unwrap();
    std::fs::write(
        music.join("south.mp3"),
        include_bytes!("../../tests/fixtures/tone.mp3"),
    )
    .unwrap();
    let local = catalogue.add_folder(&music).unwrap();
    let local_track = catalogue.tracks(&local.id).unwrap().remove(0);
    let local_metadata = MetadataOverride {
        title: Some("Beacon on the Hill".into()),
        artist: Some("Björk".into()),
        album: Some("Late Sessions".into()),
        ..Default::default()
    };
    catalogue
        .override_library_track(&local_track.id, &local.id, &local_metadata)
        .unwrap();
    catalogue
        .save_dropbox_account("fixture-account", "fixture-key")
        .unwrap();
    let remote = catalogue
        .add_dropbox_indexed(
            "fixture-account",
            "/Music",
            "Dropbox",
            &[Entry {
                kind: "file".into(),
                name: "Beacon in the bay.mp3".into(),
                path_display: Some("/Music/Beacon in the bay.mp3".into()),
                path_lower: None,
                size: Some(500),
                server_modified: None,
                rev: Some("r1".into()),
            }],
        )
        .unwrap();
    let remote_track = catalogue.tracks(&remote.id).unwrap().remove(0);
    catalogue
        .override_library_track(&remote_track.id, &remote.id, &local_metadata)
        .unwrap();
    drop(catalogue);

    let (mut desktop, _) = setup();
    desktop.sources = Sources::new(Some(paths.clone()));
    desktop.sources.snapshot = Some(run(&paths, SourceOperation::Startup).unwrap());

    let task = desktop.update(Message::TrackSearchText("SESSIONS bjÖrK".into()));
    let result = next_output(task, |event| {
        matches!(event, Message::TrackSearchFinished(..))
    });
    let _ = desktop.update(result);
    let found = &desktop.sources.visible_snapshot().unwrap().tracks;
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|track| track.id == remote_track.id));
    let index = found
        .iter()
        .position(|track| track.id == local_track.id)
        .unwrap();
    let local_result = found[index].clone();
    let play = desktop.update(Message::PlayIndexed(index));
    let prepared = next_output(play, |event| matches!(event, Message::RemoteTrackReady(..)));
    let _ = desktop.update(prepared);
    assert_eq!(
        desktop
            .session
            .as_ref()
            .unwrap()
            .current()
            .unwrap()
            .indexed_id(),
        Some(local_track.id.as_str())
    );

    let task = desktop.update(Message::EditTrack(local_result));
    let result = next_output(task, |event| matches!(event, Message::SourcesUpdated(_)));
    let _ = desktop.update(result);
    assert!(
        desktop
            .sources
            .snapshot
            .as_ref()
            .unwrap()
            .selected
            .is_none(),
        "opening a global result must not switch to that track's source"
    );
    assert_eq!(desktop.sources.visible_snapshot().unwrap().tracks.len(), 2);
    assert_eq!(
        desktop.sources.editing.as_ref().unwrap().track_id,
        local_track.id
    );
}

#[test]
fn it_finds_a_track_within_the_selected_source_or_playlist_and_plays_its_original_queue() {
    use super::{LibraryPage, Message, queue_tests::snapshot};
    use crate::catalogue::{IndexedTrack, Playlist, PlaylistEntry};

    let (mut desktop, file) = setup();
    let mut source = snapshot(&["local"]);
    source.libraries[0].kind = "filesystem".into();
    source.selected = Some("local".into());
    let song = |id: &str, title: &str| IndexedTrack {
        id: id.into(),
        title: title.into(),
        artist: "Fixture artist".into(),
        album: None,
        metadata_overridden: false,
        path: file.path().to_owned(),
        plays: 0,
        duration_seconds: 1,
        library_id: "local".into(),
        file_size: 0,
        file_mtime: 0,
    };
    source.tracks = vec![song("first", "Ferry lights"), song("second", "Fern grove")];
    source.playlists = vec![Playlist {
        id: "mix".into(),
        name: "After hours".into(),
    }];
    source.selected_playlist = Some("mix".into());
    source.playlist_entries = source
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| PlaylistEntry {
            id: format!("entry-{index}"),
            track: Some(track.clone()),
        })
        .collect();
    desktop.sources.snapshot = Some(source);

    let _ = desktop.update(Message::TrackSearchText("FERN".into()));
    let visible = &desktop.sources.visible_snapshot().unwrap().tracks;
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "second");
    let play = desktop.update(Message::PlayIndexed(0));
    let prepared = next_output(play, |event| matches!(event, Message::RemoteTrackReady(..)));
    let _ = desktop.update(prepared);
    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.current().unwrap().indexed_id(), Some("second"));
    assert_eq!(
        session.tracks().len(),
        2,
        "the source search changed its playback queue"
    );

    let _ = desktop.update(Message::LibraryPage(LibraryPage::Playlists));
    let _ = desktop.update(Message::TrackSearchText("lights".into()));
    let visible = &desktop.sources.visible_snapshot().unwrap().playlist_entries;
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "entry-0");
    let entry = visible[0].id.clone();
    let play = desktop.update(Message::PlayPlaylist(entry));
    let prepared = next_output(play, |event| matches!(event, Message::RemoteTrackReady(..)));
    let _ = desktop.update(prepared);
    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.current().unwrap().playlist_entry(), Some("entry-0"));
    assert_eq!(session.tracks().len(), 2);
}

fn search_row(id: &str, title: &str, path: &std::path::Path) -> crate::catalogue::IndexedTrack {
    crate::catalogue::IndexedTrack {
        id: id.into(),
        title: title.into(),
        artist: "Fixture artist".into(),
        album: None,
        metadata_overridden: false,
        path: path.to_owned(),
        plays: 0,
        duration_seconds: 1,
        library_id: "local".into(),
        file_size: 0,
        file_mtime: 0,
    }
}

fn search_reply(
    task: iced::Task<super::Message>,
    tracks: Vec<crate::catalogue::IndexedTrack>,
) -> super::Message {
    use super::Message;
    let Message::TrackSearchFinished(serial, _) = next_output(task, |event| {
        matches!(event, Message::TrackSearchFinished(..))
    }) else {
        unreachable!()
    };
    Message::TrackSearchFinished(serial, Ok(tracks))
}

#[test]
fn it_keeps_new_play_counts_when_a_global_result_arrives_late() {
    use super::{
        Message,
        library_sources::{CataloguePaths, Sources},
        queue_tests::snapshot,
    };

    let data = tempfile::tempdir().unwrap();
    let (mut desktop, file) = setup();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(data.path())));
    desktop.sources.snapshot = Some(snapshot(&["local"]));
    let task = desktop.update(Message::TrackSearchText("fern".into()));
    let stale = search_reply(task, vec![search_row("fern", "Fern grove", file.path())]);
    let _ = desktop.update(Message::PlayRecorded("fern".into(), Ok(2)));
    let _ = desktop.update(stale);

    assert_eq!(
        desktop.sources.visible_snapshot().unwrap().tracks[0].plays,
        2,
        "the late search result lost the confirmed play count"
    );
}

#[test]
fn it_closes_the_editor_when_its_global_result_stops_matching() {
    use super::{
        Message,
        library_sources::{CataloguePaths, Sources, TrackEditor},
        queue_tests::snapshot,
    };

    let data = tempfile::tempdir().unwrap();
    let (mut desktop, file) = setup();
    desktop.sources = Sources::new(Some(CataloguePaths::in_directory(data.path())));
    desktop.sources.snapshot = Some(snapshot(&["local"]));
    let task = desktop.update(Message::TrackSearchText("fern".into()));
    let first = search_reply(task, vec![search_row("fern", "Fern grove", file.path())]);
    let _ = desktop.update(first);
    desktop.sources.editing = Some(Box::new(TrackEditor {
        track_id: "fern".into(),
        library_id: "local".into(),
        fields: Default::default(),
        overridden: false,
        error: None,
        action_focus: None,
    }));
    let refreshed = search_reply(desktop.refresh_track_search(), Vec::new());
    let _ = desktop.update(refreshed);

    assert!(
        desktop.sources.editing.is_none(),
        "an invisible editor still held keyboard navigation"
    );
}

#[test]
fn it_keeps_unfiltered_row_clicks_on_the_original_track_during_filtering() {
    use super::{Message, queue_tests::snapshot};

    let (mut desktop, file) = setup();
    let mut source = snapshot(&["local"]);
    source.libraries[0].kind = "filesystem".into();
    source.selected = Some("local".into());
    source.tracks = vec![
        search_row("ferry", "Ferry lights", file.path()),
        search_row("fern", "Fern grove", file.path()),
    ];
    desktop.sources.snapshot = Some(source);

    let click = Message::PlayIndexed(0);
    let _ = desktop.update(Message::TrackSearchText("fern".into()));
    let play = desktop.update(click);
    let prepared = next_output(play, |event| matches!(event, Message::RemoteTrackReady(..)));
    let _ = desktop.update(prepared);
    assert_eq!(
        desktop
            .session
            .as_ref()
            .unwrap()
            .current()
            .unwrap()
            .indexed_id(),
        Some("ferry"),
        "a stale unfiltered row index played a different track"
    );
    let play = desktop.update(Message::PlaySearchResult("fern".into()));
    let prepared = next_output(play, |event| matches!(event, Message::RemoteTrackReady(..)));
    let _ = desktop.update(prepared);
    assert_eq!(
        desktop
            .session
            .as_ref()
            .unwrap()
            .current()
            .unwrap()
            .indexed_id(),
        Some("fern")
    );
}

#[test]
fn it_opens_global_search_from_the_keyboard_while_a_source_is_selected() {
    use super::{SongScope, input_message_for_platform, queue_tests::snapshot};
    use iced::{
        Event,
        event::Status,
        keyboard::{
            Event as Keyboard, Key, Location, Modifiers,
            key::{NativeCode, Physical},
        },
    };

    for (macos, command) in [(false, Modifiers::CTRL), (true, Modifiers::LOGO)] {
        let (mut desktop, _) = setup();
        let mut source = snapshot(&["local"]);
        source.selected = Some("local".into());
        desktop.sources.snapshot = Some(source);
        let id = desktop.windows.library_id();
        let key = Event::Keyboard(Keyboard::KeyPressed {
            key: Key::Character("f".into()),
            modified_key: Key::Character("F".into()),
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers: command | Modifiers::SHIFT,
            text: None,
            repeat: false,
        });
        let message = input_message_for_platform(key, Status::Ignored, id, macos)
            .expect("the global search keyboard shortcut was unavailable");
        let focus = desktop.update(message);
        assert!(
            focus.units() > 0,
            "the search field should receive keyboard focus"
        );
        assert_eq!(desktop.sources.track_search.scope, SongScope::AllSongs);
        assert_eq!(
            desktop
                .sources
                .snapshot
                .as_ref()
                .unwrap()
                .selected
                .as_deref(),
            Some("local")
        );
    }
}

#[test]
fn it_clears_playlist_search_when_the_selected_playlist_changes() {
    use super::{LibraryPage, Message, queue_tests::snapshot};
    use crate::catalogue::Playlist;

    let (mut desktop, _) = setup();
    let mut source = snapshot(&[]);
    source.playlists = vec![
        Playlist {
            id: "one".into(),
            name: "First".into(),
        },
        Playlist {
            id: "two".into(),
            name: "Second".into(),
        },
    ];
    source.selected_playlist = Some("one".into());
    desktop.sources.snapshot = Some(source.clone());
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Playlists));
    let _ = desktop.update(Message::TrackSearchText("fern".into()));
    let _ = desktop.update(Message::SourcesUpdated(Ok(source.clone())));
    assert_eq!(
        desktop
            .sources
            .track_search
            .query(LibraryPage::Playlists, Some(&source)),
        "fern"
    );
    source.selected_playlist = Some("two".into());
    let _ = desktop.update(Message::SourcesUpdated(Ok(source.clone())));
    assert_eq!(
        desktop
            .sources
            .track_search
            .query(LibraryPage::Playlists, Some(&source)),
        "",
        "a new playlist inherited another playlist's search"
    );
}
