//! Track aggregate root and related value objects.
//!
//! A [`Track`] is a single playable audio file belonging to exactly one
//! [`Library`](super::library::Library). Pure domain types - no framework
//! imports, no serde on the wire.

use std::fmt;
use std::path::PathBuf;

use super::library::{LibraryId, RepositoryError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackId(pub String);

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackTitle(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackArtist(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackDuration(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackFilePath(pub PathBuf);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSize(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMtime(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    id: TrackId,
    library_id: LibraryId,
    title: TrackTitle,
    artist: TrackArtist,
    duration: TrackDuration,
    file_path: TrackFilePath,
    file_size: FileSize,
    file_mtime: FileMtime,
}

impl Track {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: TrackId,
        library_id: LibraryId,
        title: TrackTitle,
        artist: TrackArtist,
        duration: TrackDuration,
        file_path: TrackFilePath,
        file_size: FileSize,
        file_mtime: FileMtime,
    ) -> Self {
        Self {
            id,
            library_id,
            title,
            artist,
            duration,
            file_path,
            file_size,
            file_mtime,
        }
    }

    pub fn id(&self) -> &TrackId {
        &self.id
    }

    pub fn library_id(&self) -> &LibraryId {
        &self.library_id
    }

    pub fn title(&self) -> &TrackTitle {
        &self.title
    }

    pub fn artist(&self) -> &TrackArtist {
        &self.artist
    }

    pub fn duration(&self) -> TrackDuration {
        self.duration
    }

    pub fn file_path(&self) -> &TrackFilePath {
        &self.file_path
    }

    pub fn file_size(&self) -> FileSize {
        self.file_size
    }

    pub fn file_mtime(&self) -> FileMtime {
        self.file_mtime
    }
}

pub fn generate_track_id() -> TrackId {
    TrackId(ulid::Ulid::new().to_string())
}

/// Persists tracks. Implemented by infrastructure (SQLite in sprint 01).
/// Defined in the domain so the application layer depends on the
/// abstraction, not the concrete adapter.
pub trait TrackRepository {
    /// Insert or update a track keyed on (library_id, file_path).
    fn upsert(&self, track: &Track) -> Result<(), RepositoryError>;

    /// Every track belonging to a library, in unspecified order unless the
    /// implementation documents otherwise.
    fn list_for_library(&self, library_id: &LibraryId) -> Result<Vec<Track>, RepositoryError>;

    /// Remove a track from a library by its file path. Used by sync (card 005)
    /// when a file disappears from the backing source.
    fn delete_by_library_and_path(
        &self,
        library_id: &LibraryId,
        file_path: &TrackFilePath,
    ) -> Result<(), RepositoryError>;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn a_track_exposes_the_values_it_was_constructed_with() {
        let track = Track::new(
            TrackId("trk-1".into()),
            LibraryId("lib-1".into()),
            TrackTitle("Hotel California".into()),
            TrackArtist("Eagles".into()),
            TrackDuration(391),
            TrackFilePath(PathBuf::from("/lib/Eagles/Hotel California.mp3")),
            FileSize(9_000_000),
            FileMtime(1_700_000_000),
        );

        assert_eq!(track.id(), &TrackId("trk-1".into()));
        assert_eq!(track.library_id(), &LibraryId("lib-1".into()));
        assert_eq!(track.title(), &TrackTitle("Hotel California".into()));
        assert_eq!(track.artist(), &TrackArtist("Eagles".into()));
        assert_eq!(track.duration(), TrackDuration(391));
        assert_eq!(
            track.file_path(),
            &TrackFilePath(PathBuf::from("/lib/Eagles/Hotel California.mp3"))
        );
        assert_eq!(track.file_size(), FileSize(9_000_000));
        assert_eq!(track.file_mtime(), FileMtime(1_700_000_000));
    }
}
