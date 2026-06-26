use crate::domain::library::RepositoryError;
use crate::domain::playlist::{
    Playlist, PlaylistEntryId, PlaylistError, PlaylistId, PlaylistName, PlaylistRepository,
};
use crate::domain::track::TrackId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePlaylistInput {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddTrackToPlaylistInput {
    pub playlist_id: PlaylistId,
    pub track_id: TrackId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovePlaylistEntryInput {
    pub playlist_id: PlaylistId,
    pub entry_id: PlaylistEntryId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReorderPlaylistEntriesInput {
    pub playlist_id: PlaylistId,
    pub ordered_entry_ids: Vec<PlaylistEntryId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaylistApplicationError {
    EmptyName,
    PlaylistNotFound(PlaylistId),
    Repository(RepositoryError),
    InvalidEntryOrder,
}

impl From<RepositoryError> for PlaylistApplicationError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<PlaylistError> for PlaylistApplicationError {
    fn from(error: PlaylistError) -> Self {
        match error {
            PlaylistError::EmptyName => PlaylistApplicationError::EmptyName,
            PlaylistError::EntryNotFound(_) => PlaylistApplicationError::InvalidEntryOrder,
            PlaylistError::InvalidEntryOrder => PlaylistApplicationError::InvalidEntryOrder,
        }
    }
}

pub fn create_playlist(
    repository: &dyn PlaylistRepository,
    input: CreatePlaylistInput,
) -> Result<Playlist, PlaylistApplicationError> {
    let playlist = Playlist::new(
        generate_playlist_id(),
        PlaylistName::new(input.name).map_err(PlaylistApplicationError::from)?,
    );
    repository.add(&playlist)?;
    Ok(playlist)
}

pub fn list_playlists(
    repository: &dyn PlaylistRepository,
) -> Result<Vec<Playlist>, PlaylistApplicationError> {
    repository.list().map_err(PlaylistApplicationError::from)
}

pub fn add_track_to_playlist(
    repository: &dyn PlaylistRepository,
    input: AddTrackToPlaylistInput,
) -> Result<Playlist, PlaylistApplicationError> {
    let mut playlist = find_required_playlist(repository, &input.playlist_id)?;
    playlist.add_track(generate_playlist_entry_id(), input.track_id);
    repository.save(&playlist)?;
    Ok(playlist)
}

pub fn remove_playlist_entry(
    repository: &dyn PlaylistRepository,
    input: RemovePlaylistEntryInput,
) -> Result<Playlist, PlaylistApplicationError> {
    let mut playlist = find_required_playlist(repository, &input.playlist_id)?;
    playlist.remove_entry(&input.entry_id)?;
    repository.save(&playlist)?;
    Ok(playlist)
}

pub fn reorder_playlist_entries(
    repository: &dyn PlaylistRepository,
    input: ReorderPlaylistEntriesInput,
) -> Result<Playlist, PlaylistApplicationError> {
    let mut playlist = find_required_playlist(repository, &input.playlist_id)?;
    playlist.reorder_entries(&input.ordered_entry_ids)?;
    repository.save(&playlist)?;
    Ok(playlist)
}

fn find_required_playlist(
    repository: &dyn PlaylistRepository,
    playlist_id: &PlaylistId,
) -> Result<Playlist, PlaylistApplicationError> {
    repository
        .find_by_id(playlist_id)?
        .ok_or_else(|| PlaylistApplicationError::PlaylistNotFound(playlist_id.clone()))
}

fn generate_playlist_id() -> PlaylistId {
    PlaylistId(ulid::Ulid::new().to_string())
}

fn generate_playlist_entry_id() -> PlaylistEntryId {
    PlaylistEntryId(ulid::Ulid::new().to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{
        add_track_to_playlist, create_playlist, list_playlists, remove_playlist_entry,
        reorder_playlist_entries, AddTrackToPlaylistInput, CreatePlaylistInput,
        PlaylistApplicationError, RemovePlaylistEntryInput, ReorderPlaylistEntriesInput,
    };
    use crate::domain::library::RepositoryError;
    use crate::domain::playlist::{Playlist, PlaylistId, PlaylistRepository};

    #[derive(Default, Clone)]
    struct FakePlaylistRepository {
        playlists: Arc<Mutex<Vec<Playlist>>>,
    }

    impl PlaylistRepository for FakePlaylistRepository {
        fn add(&self, playlist: &Playlist) -> Result<(), RepositoryError> {
            self.playlists.lock().unwrap().push(playlist.clone());
            Ok(())
        }

        fn save(&self, playlist: &Playlist) -> Result<(), RepositoryError> {
            let mut guard = self.playlists.lock().unwrap();
            if let Some(existing) = guard.iter_mut().find(|item| item.id() == playlist.id()) {
                *existing = playlist.clone();
            } else {
                guard.push(playlist.clone());
            }
            Ok(())
        }

        fn find_by_id(&self, id: &PlaylistId) -> Result<Option<Playlist>, RepositoryError> {
            Ok(self
                .playlists
                .lock()
                .unwrap()
                .iter()
                .find(|playlist| playlist.id() == id)
                .cloned())
        }

        fn list(&self) -> Result<Vec<Playlist>, RepositoryError> {
            Ok(self.playlists.lock().unwrap().clone())
        }
    }

    #[test]
    fn creates_a_named_playlist() {
        let repository = FakePlaylistRepository::default();

        let playlist = create_playlist(
            &repository,
            CreatePlaylistInput {
                name: "Road Trip".into(),
            },
        )
        .expect("playlist should be created");

        assert_eq!(playlist.name().as_str(), "Road Trip");
        assert!(playlist.entries().is_empty());
        assert_eq!(repository.list().unwrap(), vec![playlist]);
    }

    #[test]
    fn creating_a_playlist_rejects_an_empty_name() {
        let repository = FakePlaylistRepository::default();

        let result = create_playlist(&repository, CreatePlaylistInput { name: " ".into() });

        assert_eq!(result, Err(PlaylistApplicationError::EmptyName));
        assert!(repository.list().unwrap().is_empty());
    }

    #[test]
    fn lists_persisted_playlists() {
        let repository = FakePlaylistRepository::default();
        create_playlist(
            &repository,
            CreatePlaylistInput {
                name: "Road Trip".into(),
            },
        )
        .unwrap();

        let playlists = list_playlists(&repository).unwrap();

        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].name().as_str(), "Road Trip");
    }

    #[test]
    fn adds_removes_and_reorders_playlist_entries() {
        let repository = FakePlaylistRepository::default();
        let playlist = create_playlist(
            &repository,
            CreatePlaylistInput {
                name: "Road Trip".into(),
            },
        )
        .unwrap();

        let playlist = add_track_to_playlist(
            &repository,
            AddTrackToPlaylistInput {
                playlist_id: playlist.id().clone(),
                track_id: crate::domain::track::TrackId("track-1".into()),
            },
        )
        .unwrap();
        let playlist = add_track_to_playlist(
            &repository,
            AddTrackToPlaylistInput {
                playlist_id: playlist.id().clone(),
                track_id: crate::domain::track::TrackId("track-2".into()),
            },
        )
        .unwrap();
        let first_entry = playlist.entries()[0].id().clone();
        let second_entry = playlist.entries()[1].id().clone();

        let playlist = reorder_playlist_entries(
            &repository,
            ReorderPlaylistEntriesInput {
                playlist_id: playlist.id().clone(),
                ordered_entry_ids: vec![second_entry.clone(), first_entry.clone()],
            },
        )
        .unwrap();
        assert_eq!(playlist.entries()[0].id(), &second_entry);

        let playlist = remove_playlist_entry(
            &repository,
            RemovePlaylistEntryInput {
                playlist_id: playlist.id().clone(),
                entry_id: second_entry,
            },
        )
        .unwrap();

        assert_eq!(playlist.entries().len(), 1);
        assert_eq!(playlist.entries()[0].id(), &first_entry);
        assert_eq!(playlist.entries()[0].position().0, 0);
    }
}
