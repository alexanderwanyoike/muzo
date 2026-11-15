use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{OptionalExtension, Transaction, TransactionBehavior::Immediate, params};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::dropbox::Entry;

use super::{
    Catalogue, CatalogueError, Library, MUSIC_EXTENSIONS,
    local::{ScannedFile, reconcile},
};

impl Catalogue {
    pub fn save_dropbox_account(
        &self,
        account: &str,
        client_id: &str,
    ) -> Result<(), CatalogueError> {
        self.write_dropbox_account(account, client_id, None)
    }

    pub(crate) fn save_native_dropbox_account(
        &self,
        account: &str,
        client_id: &str,
        store: &str,
    ) -> Result<(), CatalogueError> {
        self.write_dropbox_account(account, client_id, Some(store))
    }

    fn write_dropbox_account(
        &self,
        account: &str,
        client_id: &str,
        store: Option<&str>,
    ) -> Result<(), CatalogueError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64);
        self.database.execute(
            "INSERT INTO dropbox_accounts (account_id, client_id, encrypted_access_token,
              encrypted_refresh_token, access_token_expires_at_unix_seconds, connected_at_unix_seconds, credential_store)
              VALUES (?1, ?2, '', '', 0, ?3, ?4)
              ON CONFLICT(account_id) DO UPDATE SET client_id=excluded.client_id,
              encrypted_access_token='', encrypted_refresh_token='', connected_at_unix_seconds=excluded.connected_at_unix_seconds,
              credential_store=excluded.credential_store",
            params![account, client_id, now, store],
        )?;
        Ok(())
    }

    pub fn latest_dropbox(&self) -> Result<Option<(String, String)>, CatalogueError> {
        Ok(self.database.query_row(
            "SELECT account_id, client_id FROM dropbox_accounts ORDER BY connected_at_unix_seconds DESC, rowid DESC LIMIT 1",
            [], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?)
    }

    pub fn dropbox_client(&self, account: &str) -> Result<Option<String>, CatalogueError> {
        Ok(self
            .database
            .query_row(
                "SELECT client_id FROM dropbox_accounts WHERE account_id=?1",
                [account],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn legacy_dropbox_token(
        &self,
        account: &str,
    ) -> Result<Option<String>, CatalogueError> {
        Ok(self
            .database
            .query_row(
                "SELECT encrypted_refresh_token FROM dropbox_accounts
                 WHERE account_id=?1 AND encrypted_refresh_token<>''",
                [account],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn dropbox_credential_store(
        &self,
        account: &str,
    ) -> Result<Option<String>, CatalogueError> {
        Ok(self
            .database
            .query_row(
                "SELECT credential_store FROM dropbox_accounts WHERE account_id=?1",
                [account],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    pub(crate) fn set_dropbox_credential_store(
        &self,
        account: &str,
        store: &str,
    ) -> Result<(), CatalogueError> {
        self.database.execute(
            "UPDATE dropbox_accounts SET credential_store=?2 WHERE account_id=?1",
            params![account, store],
        )?;
        Ok(())
    }

    pub(crate) fn clear_legacy_dropbox_tokens(&self, account: &str) -> Result<(), CatalogueError> {
        self.database.execute(
            "UPDATE dropbox_accounts SET encrypted_access_token='', encrypted_refresh_token=''
             WHERE account_id=?1",
            [account],
        )?;
        Ok(())
    }

    pub fn dropbox_account_for(&self, library: &str) -> Result<Option<String>, CatalogueError> {
        Ok(self
            .database
            .query_row(
                "SELECT account_id FROM dropbox_library_accounts WHERE library_id=?1",
                [library],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn disconnect_dropbox(&self, account: &str) -> Result<(), CatalogueError> {
        self.database.execute(
            "DELETE FROM dropbox_accounts WHERE account_id=?1",
            [account],
        )?;
        Ok(())
    }

    pub fn dropbox_libraries_for(&self, account: &str) -> Result<Vec<String>, CatalogueError> {
        let mut query = self
            .database
            .prepare("SELECT library_id FROM dropbox_library_accounts WHERE account_id=?1")?;
        Ok(query
            .query_map([account], |row| row.get(0))?
            .collect::<Result<_, _>>()?)
    }

    pub fn add_dropbox(
        &mut self,
        account: &str,
        path: &str,
        name: &str,
    ) -> Result<Library, CatalogueError> {
        let transaction = self.database.transaction_with_behavior(Immediate)?;
        let library = insert_library(&transaction, account, path, name)?;
        transaction.commit()?;
        Ok(library)
    }

    pub(crate) fn add_dropbox_indexed(
        &mut self,
        account: &str,
        path: &str,
        name: &str,
        entries: &[Entry],
    ) -> Result<Library, CatalogueError> {
        let files = scanned_files(entries);
        if files.is_empty() {
            return Err(CatalogueError::NoSupportedAudio);
        }
        let transaction = self.database.transaction_with_behavior(Immediate)?;
        let library = insert_library(&transaction, account, path, name)?;
        reconcile(&transaction, &library.id, &files)?;
        transaction.commit()?;
        Ok(library)
    }

    pub fn sync_dropbox(
        &mut self,
        library: &str,
        entries: &[Entry],
    ) -> Result<usize, CatalogueError> {
        let mut files = scanned_files(entries);
        let transaction = self.database.transaction_with_behavior(Immediate)?;
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM dropbox_library_accounts WHERE library_id=?1)",
            [library],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(CatalogueError::UnknownFolder);
        }
        let mut fields = transaction.prepare(
            "SELECT file_path,title,artist,album,duration_seconds,track_number,disc_number,genre,year
             FROM tracks WHERE library_id=?1"
        )?;
        let prior = fields
            .query_map([library], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    (
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, Option<u32>>(5)?,
                        row.get::<_, Option<u32>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<u32>>(8)?,
                    ),
                ))
            })?
            .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
        drop(fields);
        for file in &mut files {
            if let Some((title, artist, album, seconds, track, disc, genre, year)) =
                prior.get(&file.path)
            {
                file.title.clone_from(title);
                file.artist.clone_from(artist);
                file.album.clone_from(album);
                file.seconds = *seconds;
                file.track = *track;
                file.disc = *disc;
                file.genre.clone_from(genre);
                file.year = *year;
            }
        }
        reconcile(&transaction, library, &files)?;
        transaction.commit()?;
        Ok(files.len())
    }

    pub fn cache_dropbox_metadata(
        &self,
        id: &str,
        title: Option<&str>,
        artist: Option<&str>,
        album: Option<&str>,
    ) -> Result<(), CatalogueError> {
        self.database.execute(
            "UPDATE tracks SET title=COALESCE(?2, title), artist=COALESCE(?3, artist), album=COALESCE(?4, album)
             WHERE id=?1 AND EXISTS (SELECT 1 FROM dropbox_library_accounts WHERE library_id=tracks.library_id)",
            params![id, title, artist, album],
        )?;
        Ok(())
    }

    pub fn dropbox_interval(&self) -> Result<u16, CatalogueError> {
        Ok(self.database.query_row(
            "SELECT interval_minutes FROM dropbox_sync_settings WHERE singleton=1",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn set_dropbox_interval(&self, minutes: u16) -> Result<(), CatalogueError> {
        self.database.execute(
            "UPDATE dropbox_sync_settings SET interval_minutes=?1 WHERE singleton=1",
            [minutes.clamp(1, 60)],
        )?;
        Ok(())
    }
}

fn insert_library(
    transaction: &Transaction<'_>,
    account: &str,
    path: &str,
    name: &str,
) -> Result<Library, CatalogueError> {
    let duplicate: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM libraries l JOIN dropbox_library_accounts a ON a.library_id=l.id
             WHERE a.account_id=?1 AND lower(l.location)=lower(?2))",
            params![account, path], |row| row.get(0),
        )?;
    if duplicate {
        return Err(CatalogueError::DuplicateFolder);
    }
    let library = Library {
        id: Uuid::new_v4().to_string(),
        name: name.to_owned(),
        kind: "dropbox".into(),
        location: PathBuf::from(path),
    };
    transaction.execute(
        "INSERT INTO libraries VALUES (?1, ?2, 'dropbox', ?3)",
        params![library.id, name, path],
    )?;
    transaction.execute(
        "INSERT INTO dropbox_library_accounts VALUES (?1, ?2)",
        params![library.id, account],
    )?;
    Ok(library)
}

fn revision_stamp(revision: &str) -> i64 {
    let digest = Sha256::digest(revision);
    i64::from_le_bytes(digest[..8].try_into().expect("SHA-256 is 32 bytes"))
}

impl Catalogue {
    pub(crate) fn downloaded_artwork_revision(
        library: &str,
        id: &str,
        path: &str,
        size: u64,
        revision: &str,
    ) -> Option<crate::artwork_cache::Revision> {
        let size = i64::try_from(size).ok()?;
        Some(crate::artwork_cache::Revision::new(
            library,
            id,
            path,
            size,
            revision_stamp(revision),
        ))
    }
}

fn scanned_files(entries: &[Entry]) -> Vec<ScannedFile> {
    entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path().filter(|path| path.starts_with('/'))?;
            let name = Path::new(&entry.name);
            if entry.kind != "file"
                || !name.extension().is_some_and(|extension| {
                    MUSIC_EXTENSIONS
                        .iter()
                        .any(|audio| extension.eq_ignore_ascii_case(audio))
                })
            {
                return None;
            }
            Some(ScannedFile {
                path: path.into(),
                size: entry.size.unwrap_or_default(),
                mtime: entry
                    .rev
                    .as_deref()
                    .or(entry.server_modified.as_deref())
                    .map_or(0, revision_stamp),
                title: name
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                artist: String::new(),
                album: None,
                seconds: 0,
                track: None,
                disc: None,
                genre: None,
                year: None,
            })
        })
        .collect()
}
