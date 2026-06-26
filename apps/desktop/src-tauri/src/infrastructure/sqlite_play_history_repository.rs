use std::sync::Mutex;

use rusqlite::Connection;

use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::play_history::{
    PlayCount, PlayHistoryEntry, PlayHistoryRepository, PlayedAtUnixSeconds,
};
use crate::domain::track::TrackId;

pub struct SqlitePlayHistoryRepository {
    connection: Mutex<Connection>,
}

impl SqlitePlayHistoryRepository {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(connection),
        }
    }
}

impl PlayHistoryRepository for SqlitePlayHistoryRepository {
    fn record_play(&self, entry: &PlayHistoryEntry) -> Result<(), RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("play history repository mutex poisoned");
        conn.execute(
            "INSERT INTO play_history (id, library_id, track_id, played_at_unix_seconds)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                entry.id().0.as_str(),
                entry.library_id().0.as_str(),
                entry.track_id().0.as_str(),
                entry.played_at().0,
            ],
        )
        .map_err(|error| RepositoryError::Io(error.to_string()))?;
        Ok(())
    }

    fn play_counts_for_library(
        &self,
        library_id: &LibraryId,
    ) -> Result<Vec<PlayCount>, RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("play history repository mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT library_id, track_id, COUNT(*), MAX(played_at_unix_seconds)
                 FROM play_history
                 WHERE library_id = ?1
                 GROUP BY library_id, track_id
                 ORDER BY MAX(played_at_unix_seconds) DESC, track_id ASC",
            )
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![library_id.0.as_str()], |row| {
                Ok(PlayCount::new(
                    LibraryId(row.get::<_, String>(0)?),
                    TrackId(row.get::<_, String>(1)?),
                    row.get::<_, u32>(2)?,
                    row.get::<_, Option<i64>>(3)?.map(PlayedAtUnixSeconds),
                ))
            })
            .map_err(|error| RepositoryError::Io(error.to_string()))?;

        let mut counts = Vec::new();
        for row in rows {
            counts.push(row.map_err(|error| RepositoryError::Io(error.to_string()))?);
        }
        Ok(counts)
    }
}
