use super::{
    queue_tests::{editing_source, rendered_labels, setup, snapshot},
    *,
};

const SAVED_NAME_WITH_SEPARATOR: &str = "Studio\u{2029}حفلة";
const DISPLAYED_SAVED_NAME: &str = "Studio حفلة";

fn playlist_with_separator() -> crate::catalogue::Playlist {
    crate::catalogue::Playlist {
        id: "collection".into(),
        name: SAVED_NAME_WITH_SEPARATOR.into(),
    }
}

#[test]
fn it_opens_a_saved_playlist_with_a_paragraph_separator_in_its_name() {
    let (mut desktop, _) = setup();
    let playlist = playlist_with_separator();
    let mut data = snapshot(&[]);
    data.selected_playlist = Some(playlist.id.clone());
    data.playlists.push(playlist);
    desktop.page = LibraryPage::Playlists;
    desktop.sources.snapshot = Some(data);

    let labels = rendered_labels(&desktop);

    assert_eq!(
        labels
            .iter()
            .filter(|label| *label == DISPLAYED_SAVED_NAME)
            .count(),
        2
    );
    assert_eq!(
        desktop.sources.snapshot.unwrap().playlists[0].name,
        SAVED_NAME_WITH_SEPARATOR
    );
}

#[test]
fn it_opens_a_saved_source_with_a_paragraph_separator_in_its_name() {
    let (mut desktop, _) = setup();
    let mut data = snapshot(&["first"]);
    data.libraries[0].name = SAVED_NAME_WITH_SEPARATOR.into();
    data.selected = Some("first".into());
    desktop.sources.snapshot = Some(data);

    let labels = rendered_labels(&desktop);

    assert_eq!(
        labels
            .iter()
            .filter(|label| *label == DISPLAYED_SAVED_NAME)
            .count(),
        2
    );
}

#[test]
fn it_shows_a_saved_source_with_a_paragraph_separator_in_settings() {
    let (mut desktop, _) = setup();
    let mut data = snapshot(&["first"]);
    data.libraries[0].name = SAVED_NAME_WITH_SEPARATOR.into();
    data.libraries[0].location = PathBuf::from("/Music\u{2029}حفلة");
    desktop.page = LibraryPage::Settings;
    desktop.sources.snapshot = Some(data);

    let labels = rendered_labels(&desktop);

    assert_eq!(
        labels
            .iter()
            .filter(|label| *label == DISPLAYED_SAVED_NAME)
            .count(),
        2
    );
    assert!(labels.iter().any(|label| label == "/Music حفلة"));
}

#[test]
fn it_displays_a_refresh_warning_for_a_saved_name_with_a_paragraph_separator() {
    let (mut desktop, _) = setup();
    let mut data = snapshot(&["first"]);
    data.libraries[0].name = SAVED_NAME_WITH_SEPARATOR.into();
    let warning = format!("{SAVED_NAME_WITH_SEPARATOR}: Folder is unavailable");
    data.warnings.push(warning.clone());
    let _ = desktop.update(Message::SourcesUpdated(Ok(data)));

    let labels = rendered_labels(&desktop);

    assert!(
        labels
            .iter()
            .any(|label| label == "Studio حفلة: Folder is unavailable")
    );
    assert_eq!(desktop.sources.error.as_deref(), Some(warning.as_str()));
}

#[test]
fn it_displays_dropbox_refresh_for_a_saved_name_with_a_paragraph_separator() {
    let (mut desktop, _) = setup();
    let mut data = snapshot(&["first"]);
    data.libraries[0].name = SAVED_NAME_WITH_SEPARATOR.into();
    desktop.page = LibraryPage::Settings;
    desktop.sources.snapshot = Some(data);
    desktop.dropbox.refreshing = Some("first".into());
    let _ = desktop.update(Message::DropboxLibraryRefreshed(
        "first".into(),
        Ok((SAVED_NAME_WITH_SEPARATOR.into(), 1)),
    ));

    let labels = rendered_labels(&desktop);

    assert!(
        labels
            .iter()
            .any(|label| label == "Refreshed Studio حفلة: 1 track.")
    );
    assert_eq!(
        desktop.sources.snapshot.unwrap().libraries[0].name,
        SAVED_NAME_WITH_SEPARATOR
    );
}

#[test]
fn it_names_a_source_with_a_paragraph_separator_when_confirming_removal() {
    let (mut desktop, _) = setup();
    let mut data = snapshot(&["first"]);
    data.libraries[0].name = SAVED_NAME_WITH_SEPARATOR.into();
    desktop.sources.snapshot = Some(data);
    let _ = desktop.update(Message::RequestSourceRemoval("first".into()));

    let labels = rendered_labels(&desktop);

    assert!(labels.iter().any(|label| label == "Remove Studio حفلة?"));
}

#[test]
fn it_displays_a_saved_playlist_name_with_a_paragraph_separator_in_the_track_picker() {
    let (mut desktop, track) = editing_source();
    let data = desktop.sources.snapshot.as_mut().unwrap();
    let playlist = playlist_with_separator();
    let playlist_id = playlist.id.clone();
    data.playlists.push(playlist);
    let _ = desktop.update(Message::OpenPlaylistPicker(track));
    let _ = desktop.update(Message::ToggleTrackPlaylistPopup);

    let labels = rendered_labels(&desktop);

    assert!(labels.iter().any(|label| label == DISPLAYED_SAVED_NAME));
    assert_eq!(
        desktop.sources.playlist_picker.unwrap().playlist.as_deref(),
        Some(playlist_id.as_str())
    );
}

#[test]
fn it_confirms_adding_a_track_to_a_playlist_with_a_paragraph_separator() {
    let (mut desktop, _) = editing_source();
    desktop.sources.playlist_message = Some(format!("Added Song to {SAVED_NAME_WITH_SEPARATOR}."));

    let labels = rendered_labels(&desktop);

    assert!(
        labels
            .iter()
            .any(|label| label == "Added Song to Studio حفلة.")
    );
}
