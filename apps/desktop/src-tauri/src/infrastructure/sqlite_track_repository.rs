use std::sync::Mutex;

use rusqlite::Connection;

use crate::domain::library::{LibraryId, RepositoryError};
use crate::domain::track::{
    DiscNumber, FileMtime, FileSize, Track, TrackAlbum, TrackArtist, TrackDuration, TrackFilePath,
    TrackGenre, TrackId, TrackMetadata, TrackMetadataOverride, TrackNumber, TrackRepository,
    TrackTitle, TrackYear,
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
}

impl TrackRepository for SqliteTrackRepository {
    fn upsert(&self, track: &Track) -> Result<(), RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("track repository mutex poisoned");
        conn.execute(
            "INSERT INTO tracks (
                 id, library_id, title, artist, album, track_number, disc_number, genre, year,
                 duration_seconds, file_path, file_size, file_mtime
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(library_id, file_path) DO UPDATE SET
                 title            = excluded.title,
                 artist           = excluded.artist,
                 album            = excluded.album,
                 track_number     = excluded.track_number,
                 disc_number      = excluded.disc_number,
                 genre            = excluded.genre,
                 year             = excluded.year,
                 duration_seconds = excluded.duration_seconds,
                 file_size        = excluded.file_size,
                 file_mtime       = excluded.file_mtime",
            rusqlite::params![
                track.id().0.as_str(),
                track.library_id().0.as_str(),
                track.file_metadata().title().0.as_str(),
                track.file_metadata().artist().0.as_str(),
                track.file_metadata().album().map(|album| album.0.as_str()),
                track
                    .file_metadata()
                    .track_number()
                    .map(|track_number| track_number.0),
                track
                    .file_metadata()
                    .disc_number()
                    .map(|disc_number| disc_number.0),
                track.file_metadata().genre().map(|genre| genre.0.as_str()),
                track.file_metadata().year().map(|year| year.0),
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
                "SELECT
                     id, library_id, title, artist, album, track_number, disc_number, genre, year,
                     override_title, override_artist, override_album, override_track_number,
                     override_disc_number, override_genre, override_year,
                     duration_seconds, file_path, file_size, file_mtime
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

    fn update_metadata_override(
        &self,
        library_id: &LibraryId,
        track_id: &TrackId,
        metadata_override: &TrackMetadataOverride,
    ) -> Result<(), RepositoryError> {
        let conn = self
            .connection
            .lock()
            .expect("track repository mutex poisoned");
        let affected = conn
            .execute(
                "UPDATE tracks SET
                    override_title = ?1,
                    override_artist = ?2,
                    override_album = ?3,
                    override_track_number = ?4,
                    override_disc_number = ?5,
                    override_genre = ?6,
                    override_year = ?7
                 WHERE library_id = ?8 AND id = ?9",
                rusqlite::params![
                    metadata_override.title().map(|title| title.0.as_str()),
                    metadata_override.artist().map(|artist| artist.0.as_str()),
                    metadata_override.album().map(|album| album.0.as_str()),
                    metadata_override
                        .track_number()
                        .map(|track_number| track_number.0),
                    metadata_override
                        .disc_number()
                        .map(|disc_number| disc_number.0),
                    metadata_override.genre().map(|genre| genre.0.as_str()),
                    metadata_override.year().map(|year| year.0),
                    library_id.0.as_str(),
                    track_id.0.as_str(),
                ],
            )
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        if affected == 0 {
            return Err(RepositoryError::NotFound(library_id.clone()));
        }
        Ok(())
    }
}

fn read_track(row: &rusqlite::Row<'_>) -> Result<Track, RepositoryError> {
    let id: String = row.get(0).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let library_id: String = row.get(1).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let title: String = row.get(2).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let artist: String = row.get(3).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let album: Option<String> = row.get(4).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let track_number: Option<u32> = row.get(5).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let disc_number: Option<u32> = row.get(6).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let genre: Option<String> = row.get(7).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let year: Option<i32> = row.get(8).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_title: Option<String> =
        row.get(9).map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_artist: Option<String> = row
        .get(10)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_album: Option<String> = row
        .get(11)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_track_number: Option<u32> = row
        .get(12)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_disc_number: Option<u32> = row
        .get(13)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_genre: Option<String> = row
        .get(14)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let override_year: Option<i32> = row
        .get(15)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let duration_seconds: u64 = row
        .get(16)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let file_path: String = row
        .get(17)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let file_size: u64 = row
        .get(18)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;
    let file_mtime: u64 = row
        .get(19)
        .map_err(|e| RepositoryError::Io(e.to_string()))?;

    Ok(Track::new_with_metadata(
        TrackId(id),
        LibraryId(library_id),
        TrackMetadata::new(
            TrackTitle(title),
            TrackArtist(artist),
            album.map(TrackAlbum),
            track_number.map(TrackNumber),
            disc_number.map(DiscNumber),
            genre.map(TrackGenre),
            year.map(TrackYear),
        ),
        TrackMetadataOverride::new(
            override_title.map(TrackTitle),
            override_artist.map(TrackArtist),
            override_album.map(TrackAlbum),
            override_track_number.map(TrackNumber),
            override_disc_number.map(DiscNumber),
            override_genre.map(TrackGenre),
            override_year.map(TrackYear),
        ),
        TrackDuration(duration_seconds),
        TrackFilePath(std::path::PathBuf::from(file_path)),
        FileSize(file_size),
        FileMtime(file_mtime),
    ))
}
