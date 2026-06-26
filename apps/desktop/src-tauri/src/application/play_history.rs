use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::play_history::{
    PlayCount, PlayHistoryEntry, PlayHistoryId, PlayHistoryRepository, PlayedAtUnixSeconds,
};
use crate::domain::track::TrackId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTrackPlayInput {
    pub library_id: LibraryId,
    pub track_id: TrackId,
    pub played_at: PlayedAtUnixSeconds,
}

pub fn record_track_play(
    repository: &dyn PlayHistoryRepository,
    input: RecordTrackPlayInput,
) -> Result<PlayHistoryEntry, RepositoryError> {
    let entry = PlayHistoryEntry::new(
        generate_play_history_id(),
        input.library_id,
        input.track_id,
        input.played_at,
    );
    repository.record_play(&entry)?;
    Ok(entry)
}

pub fn list_play_counts(
    repository: &dyn PlayHistoryRepository,
    library_id: &LibraryId,
) -> Result<Vec<PlayCount>, RepositoryError> {
    repository.play_counts_for_library(library_id)
}

fn generate_play_history_id() -> PlayHistoryId {
    PlayHistoryId(ulid::Ulid::new().to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::domain::library::{LibraryId, RepositoryError};
    use crate::domain::play_history::{
        PlayCount, PlayHistoryEntry, PlayHistoryRepository, PlayedAtUnixSeconds,
    };
    use crate::domain::track::TrackId;

    use super::{list_play_counts, record_track_play, RecordTrackPlayInput};

    #[derive(Default, Clone)]
    struct FakePlayHistoryRepository {
        entries: Arc<Mutex<Vec<PlayHistoryEntry>>>,
    }

    impl PlayHistoryRepository for FakePlayHistoryRepository {
        fn record_play(&self, entry: &PlayHistoryEntry) -> Result<(), RepositoryError> {
            self.entries.lock().unwrap().push(entry.clone());
            Ok(())
        }

        fn play_counts_for_library(
            &self,
            library_id: &LibraryId,
        ) -> Result<Vec<PlayCount>, RepositoryError> {
            let count = self
                .entries
                .lock()
                .unwrap()
                .iter()
                .filter(|entry| entry.library_id() == library_id)
                .count() as u32;

            Ok(vec![PlayCount::new(
                library_id.clone(),
                TrackId("track-1".into()),
                count,
                Some(PlayedAtUnixSeconds(1_719_000_000)),
            )])
        }
    }

    #[test]
    fn records_a_track_play_with_the_requested_library_track_and_time() {
        let repository = FakePlayHistoryRepository::default();

        let entry = record_track_play(
            &repository,
            RecordTrackPlayInput {
                library_id: LibraryId("lib-1".into()),
                track_id: TrackId("track-1".into()),
                played_at: PlayedAtUnixSeconds(1_719_000_000),
            },
        )
        .unwrap();

        assert_eq!(entry.library_id(), &LibraryId("lib-1".into()));
        assert_eq!(entry.track_id(), &TrackId("track-1".into()));
        assert_eq!(entry.played_at(), PlayedAtUnixSeconds(1_719_000_000));
        assert_eq!(repository.entries.lock().unwrap().as_slice(), &[entry]);
    }

    #[test]
    fn lists_play_counts_for_a_library() {
        let repository = FakePlayHistoryRepository::default();
        record_track_play(
            &repository,
            RecordTrackPlayInput {
                library_id: LibraryId("lib-1".into()),
                track_id: TrackId("track-1".into()),
                played_at: PlayedAtUnixSeconds(1_719_000_000),
            },
        )
        .unwrap();

        let counts = list_play_counts(&repository, &LibraryId("lib-1".into())).unwrap();

        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].track_id(), &TrackId("track-1".into()));
        assert_eq!(counts[0].count(), 1);
    }
}
