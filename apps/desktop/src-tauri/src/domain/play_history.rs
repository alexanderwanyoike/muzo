use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::TrackId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayHistoryId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayedAtUnixSeconds(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayHistoryEntry {
    id: PlayHistoryId,
    library_id: LibraryId,
    track_id: TrackId,
    played_at: PlayedAtUnixSeconds,
}

impl PlayHistoryEntry {
    pub fn new(
        id: PlayHistoryId,
        library_id: LibraryId,
        track_id: TrackId,
        played_at: PlayedAtUnixSeconds,
    ) -> Self {
        Self {
            id,
            library_id,
            track_id,
            played_at,
        }
    }

    pub fn id(&self) -> &PlayHistoryId {
        &self.id
    }

    pub fn library_id(&self) -> &LibraryId {
        &self.library_id
    }

    pub fn track_id(&self) -> &TrackId {
        &self.track_id
    }

    pub fn played_at(&self) -> PlayedAtUnixSeconds {
        self.played_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayCount {
    library_id: LibraryId,
    track_id: TrackId,
    count: u32,
    last_played_at: Option<PlayedAtUnixSeconds>,
}

impl PlayCount {
    pub fn new(
        library_id: LibraryId,
        track_id: TrackId,
        count: u32,
        last_played_at: Option<PlayedAtUnixSeconds>,
    ) -> Self {
        Self {
            library_id,
            track_id,
            count,
            last_played_at,
        }
    }

    pub fn library_id(&self) -> &LibraryId {
        &self.library_id
    }

    pub fn track_id(&self) -> &TrackId {
        &self.track_id
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn last_played_at(&self) -> Option<PlayedAtUnixSeconds> {
        self.last_played_at
    }
}

pub trait PlayHistoryRepository {
    fn record_play(&self, entry: &PlayHistoryEntry) -> Result<(), RepositoryError>;
    fn play_counts_for_library(
        &self,
        library_id: &LibraryId,
    ) -> Result<Vec<PlayCount>, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::{PlayCount, PlayHistoryEntry, PlayHistoryId, PlayedAtUnixSeconds};
    use crate::domain::library::LibraryId;
    use crate::domain::track::TrackId;

    #[test]
    fn play_history_entry_records_the_track_library_and_time() {
        let entry = PlayHistoryEntry::new(
            PlayHistoryId("play-1".into()),
            LibraryId("lib-1".into()),
            TrackId("track-1".into()),
            PlayedAtUnixSeconds(1_719_000_000),
        );

        assert_eq!(entry.id(), &PlayHistoryId("play-1".into()));
        assert_eq!(entry.library_id(), &LibraryId("lib-1".into()));
        assert_eq!(entry.track_id(), &TrackId("track-1".into()));
        assert_eq!(entry.played_at(), PlayedAtUnixSeconds(1_719_000_000));
    }

    #[test]
    fn play_count_tracks_the_latest_play_time_for_a_track() {
        let count = PlayCount::new(
            LibraryId("lib-1".into()),
            TrackId("track-1".into()),
            2,
            Some(PlayedAtUnixSeconds(1_719_000_100)),
        );

        assert_eq!(count.library_id(), &LibraryId("lib-1".into()));
        assert_eq!(count.track_id(), &TrackId("track-1".into()));
        assert_eq!(count.count(), 2);
        assert_eq!(
            count.last_played_at(),
            Some(PlayedAtUnixSeconds(1_719_000_100))
        );
    }
}
