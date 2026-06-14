use std::sync::Mutex;

use rusqlite::Connection;

use crate::domain::library::{
    Library, LibraryId, LibraryKind, LibraryLocation, LibraryName, LibraryRepository,
    RepositoryError,
};

/// SQLite-backed [`LibraryRepository`]. Owns a single connection guarded by a
/// mutex because `rusqlite::Connection` is `!Sync`.
pub struct SqliteLibraryRepository {
    connection: Mutex<Connection>,
}

impl SqliteLibraryRepository {
    /// Wrap an existing, already-migrated connection.
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(connection),
        }
    }

    /// Run idempotent schema migrations against the given connection. Safe to
    /// call on every startup.
    pub fn migrate(connection: &Connection) -> Result<(), RepositoryError> {
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS libraries (
                    id       TEXT PRIMARY KEY NOT NULL,
                    name     TEXT NOT NULL,
                    kind     TEXT NOT NULL,
                    location TEXT NOT NULL
                );",
            )
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        Ok(())
    }
}

impl LibraryRepository for SqliteLibraryRepository {
    fn add(&self, library: &Library) -> Result<(), RepositoryError> {
        let conn = self.connection.lock().expect("repository mutex poisoned");
        conn.execute(
            "INSERT INTO libraries (id, name, kind, location) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                library.id().0.as_str(),
                library.name().0.as_str(),
                kind_to_str(library.kind()),
                library.location().0.as_str(),
            ],
        )
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
        Ok(())
    }

    fn find_by_id(&self, id: &LibraryId) -> Result<Option<Library>, RepositoryError> {
        let conn = self.connection.lock().expect("repository mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT id, name, kind, location FROM libraries WHERE id = ?1")
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        let mut rows = stmt
            .query(rusqlite::params![id.0.as_str()])
            .map_err(|e| RepositoryError::Io(e.to_string()))?;

        match rows
            .next()
            .map_err(|e| RepositoryError::Io(e.to_string()))?
        {
            Some(row) => {
                let id: String = row.get(0).map_err(|e| RepositoryError::Io(e.to_string()))?;
                let name: String = row.get(1).map_err(|e| RepositoryError::Io(e.to_string()))?;
                let kind: String = row.get(2).map_err(|e| RepositoryError::Io(e.to_string()))?;
                let location: String =
                    row.get(3).map_err(|e| RepositoryError::Io(e.to_string()))?;

                Ok(Some(Library::new(
                    LibraryId(id),
                    LibraryName(name),
                    kind_from_str(&kind)?,
                    LibraryLocation(location),
                )))
            }
            None => Ok(None),
        }
    }
}

fn kind_to_str(kind: LibraryKind) -> &'static str {
    match kind {
        LibraryKind::Filesystem => "Filesystem",
        LibraryKind::Dropbox => "Dropbox",
    }
}

fn kind_from_str(s: &str) -> Result<LibraryKind, RepositoryError> {
    match s {
        "Filesystem" => Ok(LibraryKind::Filesystem),
        "Dropbox" => Ok(LibraryKind::Dropbox),
        other => Err(RepositoryError::Io(format!(
            "unknown library kind in database: {}",
            other
        ))),
    }
}
