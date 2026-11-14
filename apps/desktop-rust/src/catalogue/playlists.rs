use rusqlite::{Transaction, TransactionBehavior::Immediate, params};
use uuid::Uuid;

use super::{Catalogue, CatalogueError, IndexedTrack};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub id: String,
    pub name: String,
}

impl std::fmt::Display for Playlist {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(formatter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistEntry {
    pub id: String,
    pub track: Option<IndexedTrack>,
}

impl Catalogue {
    pub fn playlists(&self) -> Result<Vec<Playlist>, CatalogueError> {
        let mut statement = self
            .database
            .prepare("SELECT id, name FROM playlists ORDER BY rowid")?;
        Ok(statement
            .query_map([], |row| {
                Ok(Playlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<_, _>>()?)
    }

    pub fn create_playlist(&self, name: &str) -> Result<Playlist, CatalogueError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CatalogueError::EmptyPlaylistName);
        }
        let playlist = Playlist {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
        };
        self.database.execute(
            "INSERT INTO playlists (id, name) VALUES (?1, ?2)",
            params![playlist.id, playlist.name],
        )?;
        Ok(playlist)
    }

    pub fn playlist_entries(&self, playlist: &str) -> Result<Vec<PlaylistEntry>, CatalogueError> {
        let mut statement = self.database.prepare(
            "SELECT e.id, t.id, COALESCE(t.override_title, t.title), COALESCE(t.override_artist, t.artist),
                    COALESCE(t.override_album, t.album), t.file_path, COUNT(h.id), t.library_id
             FROM playlist_entries e LEFT JOIN tracks t ON t.id=e.track_id
             LEFT JOIN play_history h ON h.library_id=t.library_id AND h.track_id=t.id
             WHERE e.playlist_id=?1 GROUP BY e.id ORDER BY e.position"
        )?;
        Ok(statement
            .query_map([playlist], |row| {
                let id = row.get(0)?;
                let track = row
                    .get::<_, Option<String>>(1)?
                    .map(|track_id| {
                        Ok::<_, rusqlite::Error>(IndexedTrack {
                            id: track_id,
                            title: row.get(2)?,
                            artist: row.get(3)?,
                            album: row.get(4)?,
                            path: std::path::PathBuf::from(row.get::<_, String>(5)?),
                            plays: row.get(6)?,
                            library_id: row.get(7)?,
                        })
                    })
                    .transpose()?;
                Ok(PlaylistEntry { id, track })
            })?
            .collect::<Result<_, _>>()?)
    }

    pub fn add_to_playlist(
        &mut self,
        playlist: &str,
        track: &str,
    ) -> Result<String, CatalogueError> {
        let entry = Uuid::new_v4().to_string();
        let transaction = self.database.transaction_with_behavior(Immediate)?;
        let inserted = transaction.execute(
            "INSERT INTO playlist_entries (id, playlist_id, track_id, position)
             SELECT ?1, ?2, ?3, (SELECT COALESCE(MAX(position) + 1, 0) FROM playlist_entries WHERE playlist_id=?2)
             WHERE EXISTS (SELECT 1 FROM tracks WHERE id=?3) AND EXISTS (SELECT 1 FROM playlists WHERE id=?2)",
            params![entry, playlist, track],
        )?;
        if inserted == 0 {
            return Err(CatalogueError::UnknownPlaylistEntry);
        }
        transaction.commit()?;
        Ok(entry)
    }

    pub fn move_playlist_entry(
        &mut self,
        playlist: &str,
        entry: &str,
        delta: i8,
    ) -> Result<(), CatalogueError> {
        let transaction = self.database.transaction_with_behavior(Immediate)?;
        let mut ids = entry_ids(&transaction, playlist)?;
        let index = ids
            .iter()
            .position(|id| id == entry)
            .ok_or(CatalogueError::UnknownPlaylistEntry)?;
        if let Some(destination) = index
            .checked_add_signed(isize::from(delta))
            .filter(|next| *next < ids.len())
        {
            ids.swap(index, destination);
            apply_order(&transaction, playlist, &ids)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn remove_playlist_entry(
        &mut self,
        playlist: &str,
        entry: &str,
    ) -> Result<(), CatalogueError> {
        let transaction = self.database.transaction_with_behavior(Immediate)?;
        let removed = transaction.execute(
            "DELETE FROM playlist_entries WHERE playlist_id=?1 AND id=?2",
            params![playlist, entry],
        )?;
        if removed == 0 {
            return Err(CatalogueError::UnknownPlaylistEntry);
        }
        apply_order(&transaction, playlist, &entry_ids(&transaction, playlist)?)?;
        transaction.commit()?;
        Ok(())
    }
}

fn entry_ids(transaction: &Transaction<'_>, playlist: &str) -> rusqlite::Result<Vec<String>> {
    transaction
        .prepare("SELECT id FROM playlist_entries WHERE playlist_id=?1 ORDER BY position")?
        .query_map([playlist], |row| row.get(0))?
        .collect()
}

fn apply_order(
    transaction: &Transaction<'_>,
    playlist: &str,
    entries: &[String],
) -> rusqlite::Result<()> {
    // Move every position outside the occupied range before reassigning the unique positions.
    transaction.execute(
        "UPDATE playlist_entries SET position = -position - 1 WHERE playlist_id=?1",
        [playlist],
    )?;
    let mut update = transaction.prepare("UPDATE playlist_entries SET position=?1 WHERE id=?2")?;
    for (position, entry) in entries.iter().enumerate() {
        update.execute(params![position as i64, entry])?;
    }
    Ok(())
}
