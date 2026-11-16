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
