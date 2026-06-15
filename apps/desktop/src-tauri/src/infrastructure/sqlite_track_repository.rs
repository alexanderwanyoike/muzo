use std::sync::Mutex;

use rusqlite::Connection;

use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::{
    FileMtime, FileSize, Track, TrackArtist, TrackDuration, TrackFilePath, TrackId,
    TrackRepository, TrackTitle,
};

pub struct SqliteTrackRepository {
    connection: Mutex<Connection>,
}

impl SqliteTrackRepository {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(connection),
        }
    }

    pub fn migrate(connection: &Connection) -> Result<(), RepositoryError> {
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS tracks (
                    id               TEXT PRIMARY KEY NOT NULL,
                    library_id       TEXT NOT NULL,
                    title            TEXT NOT NULL,
                    artist           TEXT NOT NULL,
                    duration_seconds INTEGER NOT NULL,
                    file_path        TEXT NOT NULL,
                    file_size        INTEGER NOT NULL,
                    file_mtime       INTEGER NOT NULL,
                    UNIQUE(library_id, file_path)
                );",
            )
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        Ok(())
    }
}

impl TrackRepository for SqliteTrackRepository {
    fn upsert(&self, track: &Track) -> Result<(), RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("track repository mutex poisoned");
        conn.execute(
            "INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(library_id, file_path) DO UPDATE SET
                 title            = excluded.title,
                 artist           = excluded.artist,
                 duration_seconds = excluded.duration_seconds,
                 file_size        = excluded.file_size,
                 file_mtime       = excluded.file_mtime",
            rusqlite::params![
                track.id().0.as_str(),
                track.library_id().0.as_str(),
                track.title().0.as_str(),
                track.artist().0.as_str(),
                track.duration().0,
                track.file_path().0.display().to_string(),
                track.file_size().0,
                track.file_mtime().0,
            ],
        )
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
        Ok(())
    }

    fn list_for_library(&self, library_id: &LibraryId) -> Result<Vec<Track>, RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("track repository mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime
                 FROM tracks WHERE library_id = ?1 ORDER BY rowid",
            )
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        let mut rows = stmt
            .query(rusqlite::params![library_id.0.as_str()])
            .map_err(|e| RepositoryError::Io(e.to_string()))?;

        let mut result = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| RepositoryError::Io(e.to_string()))?
        {
            result.push(read_track(row)?);
        }
        Ok(result)
    }

    fn delete_by_library_and_path(
        &self,
        library_id: &LibraryId,
        file_path: &TrackFilePath,
    ) -> Result<(), RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("track repository mutex poisoned");
        conn.execute(
            "DELETE FROM tracks WHERE library_id = ?1 AND file_path = ?2",
            rusqlite::params![library_id.0.as_str(), file_path.0.display().to_string()],
        )
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
        Ok(())
    }
}

fn read_track(row: &rusqlite::Row<'_>) -> Result<Track, RepositoryError> {
    let id: String = row.get(0).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let library_id: String = row.get(1).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let title: String = row.get(2).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let artist: String = row.get(3).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let duration_seconds: u64 = row.get(4).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let file_path: String = row.get(5).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let file_size: u64 = row.get(6).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let file_mtime: u64 = row.get(7).map_err(|e| RepositoryError::Io(e.to_string()))?;

    Ok(Track::new(
        TrackId(id),
        LibraryId(library_id),
        TrackTitle(title),
        TrackArtist(artist),
        TrackDuration(duration_seconds),
        TrackFilePath(std::path::PathBuf::from(file_path)),
        FileSize(file_size),
        FileMtime(file_mtime),
    ))
}
