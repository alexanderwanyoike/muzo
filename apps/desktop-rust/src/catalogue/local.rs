use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use lofty::{
    file::{AudioFile, TaggedFileExt},
    tag::Accessor,
};
use rusqlite::{OptionalExtension, Transaction, params};
use uuid::Uuid;
use walkdir::WalkDir;

use super::{Catalogue, CatalogueError, Library};

pub const MUSIC_EXTENSIONS: [&str; 6] = ["mp3", "flac", "m4a", "ogg", "opus", "wav"];

impl Catalogue {
    pub fn add_folder(&mut self, path: &Path) -> Result<Library, CatalogueError> {
        let location = path.canonicalize()?;
        if !location.is_dir() {
            return Err(CatalogueError::NotDirectory);
        }
        let library = Library {
            id: Uuid::new_v4().to_string(),
            name: location
                .file_name()
                .unwrap_or(location.as_os_str())
                .to_string_lossy()
                .into_owned(),
            kind: "filesystem".into(),
            location,
        };
        let location = printable(&library.location)?;
        let files = walk(&library.location)?;
        let transaction = self
            .database
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut roots = transaction
            .prepare("SELECT location FROM libraries WHERE lower(kind) = 'filesystem'")?;
        let existing = roots
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let duplicate = existing.iter().any(|existing| {
            let existing = Path::new(existing);
            existing == library.location
                || existing
                    .canonicalize()
                    .is_ok_and(|canonical| canonical == library.location)
        });
        drop(roots);
        if duplicate {
            return Err(CatalogueError::DuplicateFolder);
        }
        transaction.execute(
            "INSERT INTO libraries (id, name, kind, location) VALUES (?1, ?2, 'filesystem', ?3)",
            params![library.id, library.name, location],
        )?;
        reconcile(&transaction, &library.id, &files)?;
        transaction.commit()?;
        Ok(library)
    }

    pub fn refresh(&mut self, library_id: &str) -> Result<usize, CatalogueError> {
        let location: String = self
            .database
            .query_row(
                "SELECT location FROM libraries WHERE id = ?1 AND lower(kind) = 'filesystem'",
                [library_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(CatalogueError::UnknownFolder)?;
        let files = walk(Path::new(&location))?;
        let transaction = self
            .database
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let still_exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM libraries WHERE id = ?1 AND location = ?2)",
            params![library_id, location],
            |row| row.get(0),
        )?;
        if !still_exists {
            return Err(CatalogueError::UnknownFolder);
        }
        reconcile(&transaction, library_id, &files)?;
        transaction.commit()?;
        Ok(files.len())
    }

    pub fn remove(&mut self, library_id: &str) -> Result<(), CatalogueError> {
        let transaction = self
            .database
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        transaction.execute(
            "DELETE FROM play_history WHERE library_id = ?1",
            [library_id],
        )?;
        transaction.execute("DELETE FROM tracks WHERE library_id = ?1", [library_id])?;
        let removed = transaction.execute("DELETE FROM libraries WHERE id = ?1", [library_id])?;
        if removed == 0 {
            return Err(CatalogueError::UnknownFolder);
        }
        transaction.commit()?;
        Ok(())
    }
}

struct ScannedFile {
    path: String,
    size: i64,
    mtime: i64,
    title: String,
    artist: String,
    album: Option<String>,
    seconds: i64,
    track: Option<u32>,
    disc: Option<u32>,
    genre: Option<String>,
    year: Option<u32>,
}

fn walk(folder: &Path) -> Result<Vec<ScannedFile>, CatalogueError> {
    let mut files = Vec::new();
    for entry in WalkDir::new(folder)
        .follow_root_links(true)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|extension| {
                MUSIC_EXTENSIONS
                    .iter()
                    .any(|audio| extension.eq_ignore_ascii_case(audio))
            })
        {
            files.push(read_file(entry.into_path())?);
        }
    }
    Ok(files)
}

fn read_file(path: PathBuf) -> Result<ScannedFile, CatalogueError> {
    let stat = path.metadata()?;
    let mtime = stat
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let tags = lofty::probe::Probe::open(&path)
        .and_then(|probe| {
            probe
                .options(lofty::config::ParseOptions::new().read_cover_art(false))
                .read()
        })
        .ok();
    let tag = tags
        .as_ref()
        .and_then(|file| file.primary_tag().or_else(|| file.first_tag()));
    Ok(ScannedFile {
        path: printable(&path)?.to_owned(),
        size: stat.len().min(i64::MAX as u64) as i64,
        mtime: mtime.min(i64::MAX as u64) as i64,
        title: tag.and_then(Accessor::title).map_or_else(
            || {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            },
            |title| title.into_owned(),
        ),
        artist: tag
            .and_then(Accessor::artist)
            .map_or_else(String::new, |artist| artist.into_owned()),
        album: tag
            .and_then(Accessor::album)
            .map(|album| album.into_owned()),
        seconds: tags
            .as_ref()
            .map_or(0, |file| file.properties().duration().as_secs() as i64),
        track: tag.and_then(Accessor::track),
        disc: tag.and_then(Accessor::disk),
        genre: tag
            .and_then(Accessor::genre)
            .map(|genre| genre.into_owned()),
        year: tag
            .and_then(|tag| tag.get_string(lofty::tag::ItemKey::Year))
            .and_then(|year| year.parse().ok()),
    })
}

fn reconcile(
    transaction: &Transaction<'_>,
    library: &str,
    files: &[ScannedFile],
) -> Result<(), CatalogueError> {
    let mut upsert = transaction.prepare(
        "INSERT INTO tracks (id, library_id, title, artist, album, duration_seconds, file_path, file_size, file_mtime, track_number, disc_number, genre, year)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(library_id, file_path) DO UPDATE SET title=excluded.title, artist=excluded.artist, album=excluded.album,
         duration_seconds=excluded.duration_seconds, file_size=excluded.file_size, file_mtime=excluded.file_mtime,
         track_number=excluded.track_number, disc_number=excluded.disc_number, genre=excluded.genre, year=excluded.year"
    )?;
    for file in files {
        upsert.execute(params![
            Uuid::new_v4().to_string(),
            library,
            file.title,
            file.artist,
            file.album,
            file.seconds,
            file.path,
            file.size,
            file.mtime,
            file.track,
            file.disc,
            file.genre,
            file.year,
        ])?;
    }
    let paths: HashSet<_> = files.iter().map(|file| file.path.as_str()).collect();
    let mut query = transaction.prepare("SELECT file_path FROM tracks WHERE library_id = ?1")?;
    let stale = query
        .query_map([library], |row| row.get::<_, String>(0))?
        .filter_map(|path| match path {
            Ok(path) if paths.contains(path.as_str()) => None,
            other => Some(other),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut delete =
        transaction.prepare("DELETE FROM tracks WHERE library_id = ?1 AND file_path = ?2")?;
    for path in stale {
        delete.execute(params![library, path])?;
    }
    Ok(())
}

fn printable(path: &Path) -> Result<&str, CatalogueError> {
    path.to_str().ok_or(CatalogueError::UnsupportedPath)
}
