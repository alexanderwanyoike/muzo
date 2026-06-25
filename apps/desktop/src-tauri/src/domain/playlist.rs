use crate::domain::library::RepositoryError;
use crate::domain::track::TrackId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaylistId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaylistEntryId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlaylistPosition(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistName(String);

impl PlaylistName {
    pub fn new(value: String) -> Result<Self, PlaylistError> {
        if value.trim().is_empty() {
            return Err(PlaylistError::EmptyName);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistEntry {
    id: PlaylistEntryId,
    track_id: TrackId,
    position: PlaylistPosition,
}

impl PlaylistEntry {
    pub fn new(id: PlaylistEntryId, track_id: TrackId, position: PlaylistPosition) -> Self {
        Self {
            id,
            track_id,
            position,
        }
    }

    pub fn id(&self) -> &PlaylistEntryId {
        &self.id
    }

    pub fn track_id(&self) -> &TrackId {
        &self.track_id
    }

    pub fn position(&self) -> PlaylistPosition {
        self.position
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    id: PlaylistId,
    name: PlaylistName,
    entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn new(id: PlaylistId, name: PlaylistName) -> Self {
        Self {
            id,
            name,
            entries: vec![],
        }
    }

    pub fn from_entries(
        id: PlaylistId,
        name: PlaylistName,
        mut entries: Vec<PlaylistEntry>,
    ) -> Self {
        entries.sort_by_key(|entry| entry.position);
        let mut playlist = Self { id, name, entries };
        playlist.compact_positions();
        playlist
    }

    pub fn id(&self) -> &PlaylistId {
        &self.id
    }

    pub fn name(&self) -> &PlaylistName {
        &self.name
    }

    pub fn entries(&self) -> &[PlaylistEntry] {
        &self.entries
    }

    pub fn add_track(&mut self, entry_id: PlaylistEntryId, track_id: TrackId) -> PlaylistEntry {
        let entry = PlaylistEntry::new(
            entry_id,
            track_id,
            PlaylistPosition(self.entries.len() as u32),
        );
        self.entries.push(entry.clone());
        entry
    }

    pub fn remove_entry(&mut self, entry_id: &PlaylistEntryId) -> Result<(), PlaylistError> {
        let previous_len = self.entries.len();
        self.entries.retain(|entry| entry.id() != entry_id);
        if self.entries.len() == previous_len {
            return Err(PlaylistError::EntryNotFound(entry_id.clone()));
        }
        self.compact_positions();
        Ok(())
    }

    pub fn reorder_entries(
        &mut self,
        ordered_entry_ids: &[PlaylistEntryId],
    ) -> Result<(), PlaylistError> {
        if ordered_entry_ids.len() != self.entries.len() {
            return Err(PlaylistError::InvalidEntryOrder);
        }

        let mut reordered = Vec::with_capacity(self.entries.len());
        for entry_id in ordered_entry_ids {
            let entry = self
                .entries
                .iter()
                .find(|entry| entry.id() == entry_id)
                .cloned()
                .ok_or(PlaylistError::InvalidEntryOrder)?;
            reordered.push(entry);
        }
        self.entries = reordered;
        self.compact_positions();
        Ok(())
    }

    fn compact_positions(&mut self) {
        for (position, entry) in self.entries.iter_mut().enumerate() {
            entry.position = PlaylistPosition(position as u32);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaylistError {
    EmptyName,
    EntryNotFound(PlaylistEntryId),
    InvalidEntryOrder,
}

pub trait PlaylistRepository {
    fn add(&self, playlist: &Playlist) -> Result<(), RepositoryError>;
    fn save(&self, playlist: &Playlist) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &PlaylistId) -> Result<Option<Playlist>, RepositoryError>;
    fn list(&self) -> Result<Vec<Playlist>, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::{Playlist, PlaylistEntryId, PlaylistId, PlaylistName, PlaylistPosition};
    use crate::domain::track::TrackId;

    #[test]
    fn adding_and_removing_tracks_preserves_contiguous_entry_positions() {
        let mut playlist = Playlist::new(
            PlaylistId("pl-1".into()),
            PlaylistName::new("Road Trip".into()).unwrap(),
        );

        let first =
            playlist.add_track(PlaylistEntryId("entry-1".into()), TrackId("track-1".into()));
        let second =
            playlist.add_track(PlaylistEntryId("entry-2".into()), TrackId("track-2".into()));

        assert_eq!(first.position(), PlaylistPosition(0));
        assert_eq!(second.position(), PlaylistPosition(1));

        playlist
            .remove_entry(&PlaylistEntryId("entry-1".into()))
            .unwrap();

        assert_eq!(playlist.entries().len(), 1);
        assert_eq!(
            playlist.entries()[0].id(),
            &PlaylistEntryId("entry-2".into())
        );
        assert_eq!(playlist.entries()[0].position(), PlaylistPosition(0));
    }

    #[test]
    fn reordering_entries_rewrites_positions_to_match_the_requested_order() {
        let mut playlist = Playlist::new(
            PlaylistId("pl-1".into()),
            PlaylistName::new("Road Trip".into()).unwrap(),
        );
        playlist.add_track(PlaylistEntryId("entry-1".into()), TrackId("track-1".into()));
        playlist.add_track(PlaylistEntryId("entry-2".into()), TrackId("track-2".into()));
        playlist.add_track(PlaylistEntryId("entry-3".into()), TrackId("track-3".into()));

        playlist
            .reorder_entries(&[
                PlaylistEntryId("entry-3".into()),
                PlaylistEntryId("entry-1".into()),
                PlaylistEntryId("entry-2".into()),
            ])
            .unwrap();

        assert_eq!(
            playlist.entries()[0].id(),
            &PlaylistEntryId("entry-3".into())
        );
        assert_eq!(playlist.entries()[0].position(), PlaylistPosition(0));
        assert_eq!(
            playlist.entries()[1].id(),
            &PlaylistEntryId("entry-1".into())
        );
        assert_eq!(playlist.entries()[1].position(), PlaylistPosition(1));
        assert_eq!(
            playlist.entries()[2].id(),
            &PlaylistEntryId("entry-2".into())
        );
        assert_eq!(playlist.entries()[2].position(), PlaylistPosition(2));
    }
}
