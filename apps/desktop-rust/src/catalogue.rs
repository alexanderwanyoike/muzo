mod dropbox;
mod local;
pub use local::MUSIC_EXTENSIONS;

mod metadata;
mod migrations;
mod playlists;

pub use metadata::MetadataOverride;
pub use playlists::{Playlist, PlaylistEntry};

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior, backup::Backup};

#[derive(Debug, thiserror::Error)]
pub enum CatalogueError {
    #[error("Could not access the music catalogue: {0}")]
    Io(#[from] io::Error),
    #[error("Could not read or update the music catalogue: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("This music catalogue's schema is unsupported.")]
    UnsupportedSchema,
    #[error("The selected location is not a folder.")]
    NotDirectory,
    #[error("This folder is already indexed.")]
    DuplicateFolder,
    #[error("This folder source no longer exists.")]
    UnknownFolder,
    #[error("A folder or file name cannot be stored as text.")]
    UnsupportedPath,
    #[error("Enter a playlist name.")]
    EmptyPlaylistName,
    #[error("This playlist entry is no longer available.")]
    UnknownPlaylistEntry,
    #[error("This track is no longer available.")]
    UnknownTrack,
    #[error("Could not read the music folder: {0}")]
    Walk(#[from] walkdir::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub location: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub path: PathBuf,
    pub plays: u32,
    pub duration_seconds: u32,
    pub library_id: String,
}

pub struct Catalogue {
    database: Connection,
}

impl Catalogue {
    pub fn open(path: &Path, legacy: Option<&Path>) -> Result<Self, CatalogueError> {
        let mut database =
            match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
                Ok(existing) => existing,
                Err(_) if !path.try_exists()? => initialise(path, legacy)?,
                Err(error) => return Err(error.into()),
            };
        database.busy_timeout(Duration::from_secs(10))?;
        database.pragma_update(None, "foreign_keys", true)?;
        migrations::run(&mut database)?;
        Ok(Self { database })
    }

    pub(crate) fn begin_write(&self) -> Result<Transaction<'_>, CatalogueError> {
        Ok(Transaction::new_unchecked(
            &self.database,
            TransactionBehavior::Immediate,
        )?)
    }

    pub fn libraries(&self) -> Result<Vec<Library>, CatalogueError> {
        let mut statement = self
            .database
            .prepare("SELECT id, name, lower(kind), location FROM libraries ORDER BY rowid")?;
        let libraries = statement
            .query_map([], |row| {
                Ok(Library {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    location: PathBuf::from(row.get::<_, String>(3)?),
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(libraries)
    }

    pub(crate) fn track_counts(
        &self,
    ) -> Result<std::collections::HashMap<String, u64>, CatalogueError> {
        let mut query = self.database.prepare(
            "SELECT l.id, COUNT(t.id) FROM libraries l LEFT JOIN tracks t ON t.library_id=l.id GROUP BY l.id",
        )?;
        Ok(query
            .query_map([], |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u64)))?
            .collect::<Result<_, _>>()?)
    }

    pub(crate) fn playlist_counts(
        &self,
    ) -> Result<std::collections::HashMap<String, u64>, CatalogueError> {
        let mut query = self.database.prepare(
            "SELECT p.id, COUNT(e.id) FROM playlists p LEFT JOIN playlist_entries e ON e.playlist_id=p.id GROUP BY p.id",
        )?;
        Ok(query
            .query_map([], |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u64)))?
            .collect::<Result<_, _>>()?)
    }

    pub fn tracks(&self, library_id: &str) -> Result<Vec<IndexedTrack>, CatalogueError> {
        let mut statement = self.database.prepare(
            "SELECT id, COALESCE(override_title, title), COALESCE(override_artist, artist),
             COALESCE(override_album, album), file_path,
             (SELECT COUNT(*) FROM play_history h WHERE h.library_id=tracks.library_id AND h.track_id=tracks.id),
             library_id, MAX(duration_seconds, 0) FROM tracks WHERE library_id = ?1 ORDER BY rowid"
        )?;
        let tracks = statement
            .query_map([library_id], |row| {
                Ok(IndexedTrack {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album: row.get(3)?,
                    path: PathBuf::from(row.get::<_, String>(4)?),
                    plays: row.get(5)?,
                    library_id: row.get(6)?,
                    duration_seconds: row.get(7)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(tracks)
    }

    pub(crate) fn artwork_revision(
        &self,
        library: &str,
        id: &str,
    ) -> Result<crate::artwork_cache::Revision, CatalogueError> {
        Ok(self.database.query_row(
            "SELECT file_path, file_size, file_mtime FROM tracks WHERE id=?1 AND library_id=?2",
            rusqlite::params![id, library],
            |row| {
                let path = row.get::<_, String>(0)?;
                Ok(crate::artwork_cache::Revision::new(
                    library,
                    id,
                    &path,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )?)
    }
}

fn initialise(path: &Path, legacy: Option<&Path>) -> Result<Connection, CatalogueError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temporary = tempfile::NamedTempFile::new_in(parent)?;
    {
        let mut staging = Connection::open(temporary.path())?;
        if let Some(source) = legacy
            && source.try_exists()?
        {
            let origin = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            Backup::new(&origin, &mut staging)?.run_to_completion(
                64,
                Duration::from_millis(10),
                None,
            )?;
        }
        migrations::run(&mut staging)?;
    }
    temporary.as_file().sync_all()?;
    match temporary.persist_noclobber(path) {
        Ok(_) => {}
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.error.into()),
    }
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?)
}
