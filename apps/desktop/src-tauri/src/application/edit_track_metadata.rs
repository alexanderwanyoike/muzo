use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::{TrackId, TrackMetadataOverride, TrackRepository};

pub struct EditTrackMetadataInput {
    pub library_id: LibraryId,
    pub track_id: TrackId,
    pub metadata_override: TrackMetadataOverride,
}

pub fn edit_track_metadata(
    repository: &dyn TrackRepository,
    input: EditTrackMetadataInput,
) -> Result<(), RepositoryError> {
    repository.update_metadata_override(
        &input.library_id,
        &input.track_id,
        &input.metadata_override,
    )
}

pub fn clear_track_metadata_override(
    repository: &dyn TrackRepository,
    library_id: &LibraryId,
    track_id: &TrackId,
) -> Result<(), RepositoryError> {
    repository.update_metadata_override(library_id, track_id, &TrackMetadataOverride::empty())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::domain::library::{LibraryId, RepositoryError};
    use crate::domain::track::{
        Track, TrackAlbum, TrackFilePath, TrackId, TrackMetadataOverride, TrackRepository,
        TrackTitle,
    };

    use super::{clear_track_metadata_override, edit_track_metadata, EditTrackMetadataInput};

    #[derive(Default, Clone)]
    struct FakeTrackRepository {
        updated: Arc<Mutex<Option<(LibraryId, TrackId, TrackMetadataOverride)>>>,
    }

    impl FakeTrackRepository {
        fn updated(&self) -> Option<(LibraryId, TrackId, TrackMetadataOverride)> {
            self.updated.lock().unwrap().clone()
        }
    }

    impl TrackRepository for FakeTrackRepository {
        fn upsert(&self, _track: &Track) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn list_for_library(&self, _library_id: &LibraryId) -> Result<Vec<Track>, RepositoryError> {
            Ok(vec![])
        }

        fn delete_by_library_and_path(
            &self,
            _library_id: &LibraryId,
            _file_path: &TrackFilePath,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn update_metadata_override(
            &self,
            library_id: &LibraryId,
            track_id: &TrackId,
            metadata_override: &TrackMetadataOverride,
        ) -> Result<(), RepositoryError> {
            *self.updated.lock().unwrap() = Some((
                library_id.clone(),
                track_id.clone(),
                metadata_override.clone(),
            ));
            Ok(())
        }
    }

    #[test]
    fn edit_track_metadata_persists_a_user_override() {
        let repository = FakeTrackRepository::default();
        let metadata_override = TrackMetadataOverride::new(
            Some(TrackTitle("Edited".into())),
            None,
            Some(TrackAlbum("Edited Album".into())),
            None,
            None,
            None,
            None,
        );

        edit_track_metadata(
            &repository,
            EditTrackMetadataInput {
                library_id: LibraryId("lib-1".into()),
                track_id: TrackId("trk-1".into()),
                metadata_override: metadata_override.clone(),
            },
        )
        .unwrap();

        assert_eq!(
            repository.updated(),
            Some((
                LibraryId("lib-1".into()),
                TrackId("trk-1".into()),
                metadata_override
            ))
        );
    }

    #[test]
    fn clear_track_metadata_override_removes_user_overrides() {
        let repository = FakeTrackRepository::default();

        clear_track_metadata_override(
            &repository,
            &LibraryId("lib-1".into()),
            &TrackId("trk-1".into()),
        )
        .unwrap();

        assert_eq!(
            repository.updated(),
            Some((
                LibraryId("lib-1".into()),
                TrackId("trk-1".into()),
                TrackMetadataOverride::empty()
            ))
        );
    }
}
