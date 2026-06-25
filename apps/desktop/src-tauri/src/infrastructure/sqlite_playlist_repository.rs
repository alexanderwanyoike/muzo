use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};

use crate::domain::library::RepositoryError;
use crate::domain::playlist::{
    Playlist, PlaylistEntry, PlaylistEntryId, PlaylistId, PlaylistName, PlaylistPosition,
    PlaylistRepository,
};
use crate::domain::track::TrackId;

pub struct SqlitePlaylistRepository {
    connection: Mutex<Connection>,
}

impl SqlitePlaylistRepository {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(connection),
        }
    }
}

impl PlaylistRepository for SqlitePlaylistRepository {
    fn add(&self, playlist: &Playlist) -> Result<(), RepositoryError> {
        let mut conn = self
            .connection
            .lock()
            .expect("playlist repository mutex poisoned");
        let tx = conn
            .transaction()
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        tx.execute(
            "INSERT INTO playlists (id, name) VALUES (?1, ?2)",
            rusqlite::params![playlist.id().0.as_str(), playlist.name().as_str()],
        )
        .map_err(|error| RepositoryError::Io(error.to_string()))?;
        insert_entries(&tx, playlist)?;
        tx.commit()
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        Ok(())
    }

    fn save(&self, playlist: &Playlist) -> Result<(), RepositoryError> {
        let mut conn = self
            .connection
            .lock()
            .expect("playlist repository mutex poisoned");
        let tx = conn
            .transaction()
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        tx.execute(
            "INSERT INTO playlists (id, name) VALUES (?1, ?2)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
            rusqlite::params![playlist.id().0.as_str(), playlist.name().as_str()],
        )
        .map_err(|error| RepositoryError::Io(error.to_string()))?;
        tx.execute(
            "DELETE FROM playlist_entries WHERE playlist_id = ?1",
            rusqlite::params![playlist.id().0.as_str()],
        )
        .map_err(|error| RepositoryError::Io(error.to_string()))?;
        insert_entries(&tx, playlist)?;
        tx.commit()
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        Ok(())
    }

    fn find_by_id(&self, id: &PlaylistId) -> Result<Option<Playlist>, RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("playlist repository mutex poisoned");
        let row = conn
            .query_row(
                "SELECT id, name FROM playlists WHERE id = ?1",
                rusqlite::params![id.0.as_str()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|error| RepositoryError::Io(error.to_string()))?;

        match row {
            Some((id, name)) => read_playlist(&conn, id, name).map(Some),
            None => Ok(None),
        }
    }

    fn list(&self) -> Result<Vec<Playlist>, RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("playlist repository mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT id, name FROM playlists ORDER BY rowid")
            .map_err(|error| RepositoryError::Io(error.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| RepositoryError::Io(error.to_string()))?;

        let mut playlists = Vec::new();
        for row in rows {
            let (id, name) = row.map_err(|error| RepositoryError::Io(error.to_string()))?;
            playlists.push(read_playlist(&conn, id, name)?);
        }
        Ok(playlists)
    }
}

fn insert_entries(
    tx: &rusqlite::Transaction<'_>,
    playlist: &Playlist,
) -> Result<(), RepositoryError> {
    for entry in playlist.entries() {
        tx.execute(
            "INSERT INTO playlist_entries (id, playlist_id, track_id, position)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                entry.id().0.as_str(),
                playlist.id().0.as_str(),
                entry.track_id().0.as_str(),
                entry.position().0,
            ],
        )
        .map_err(|error| RepositoryError::Io(error.to_string()))?;
    }
    Ok(())
}

fn read_playlist(
    conn: &rusqlite::Connection,
    id: String,
    name: String,
) -> Result<Playlist, RepositoryError> {
    let name = PlaylistName::new(name)
        .map_err(|_| RepositoryError::Io("invalid playlist name in database".into()))?;
    let entries = read_entries(conn, &id)?;

    Ok(Playlist::from_entries(PlaylistId(id), name, entries))
}

fn read_entries(
    conn: &rusqlite::Connection,
    playlist_id: &str,
) -> Result<Vec<PlaylistEntry>, RepositoryError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, track_id, position FROM playlist_entries
             WHERE playlist_id = ?1 ORDER BY position",
        )
        .map_err(|error| RepositoryError::Io(error.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params![playlist_id], |row| {
            Ok(PlaylistEntry::new(
                PlaylistEntryId(row.get::<_, String>(0)?),
                TrackId(row.get::<_, String>(1)?),
                PlaylistPosition(row.get::<_, u32>(2)?),
            ))
        })
        .map_err(|error| RepositoryError::Io(error.to_string()))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|error| RepositoryError::Io(error.to_string()))?);
    }
    Ok(entries)
}
