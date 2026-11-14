use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::params;
use uuid::Uuid;

use super::{Catalogue, CatalogueError};

#[derive(Debug, Clone, Default)]
pub struct MetadataOverride {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genre: Option<String>,
    pub year: Option<u32>,
}

pub(crate) struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
    pub genre: Option<String>,
    pub year: Option<i64>,
    pub overridden: bool,
}

impl Catalogue {
    pub(crate) fn track_metadata(
        &self,
        id: &str,
        library_id: &str,
    ) -> Result<TrackMetadata, CatalogueError> {
        Ok(self.database.query_row(
            "SELECT COALESCE(override_title, title), COALESCE(override_artist, artist),
                    COALESCE(override_album, album), COALESCE(override_track_number, track_number),
                    COALESCE(override_disc_number, disc_number), COALESCE(override_genre, genre),
                    COALESCE(override_year, year),
                    (override_title IS NOT NULL OR override_artist IS NOT NULL
                        OR override_album IS NOT NULL OR override_track_number IS NOT NULL
                        OR override_disc_number IS NOT NULL OR override_genre IS NOT NULL
                        OR override_year IS NOT NULL)
             FROM tracks WHERE id=?1 AND library_id=?2",
            [id, library_id],
            |row| {
                Ok(TrackMetadata {
                    title: row.get(0)?,
                    artist: row.get(1)?,
                    album: row.get(2)?,
                    track_number: row.get(3)?,
                    disc_number: row.get(4)?,
                    genre: row.get(5)?,
                    year: row.get(6)?,
                    overridden: row.get(7)?,
                })
            },
        )?)
    }

    pub fn track_overrides(&self, id: &str) -> Result<MetadataOverride, CatalogueError> {
        Ok(self.database.query_row(
            "SELECT override_title, override_artist, override_album, override_track_number,
                    override_disc_number, override_genre, override_year FROM tracks WHERE id=?1",
            [id],
            |row| {
                Ok(MetadataOverride {
                    title: row.get(0)?,
                    artist: row.get(1)?,
                    album: row.get(2)?,
                    track_number: row.get(3)?,
                    disc_number: row.get(4)?,
                    genre: row.get(5)?,
                    year: row.get(6)?,
                })
            },
        )?)
    }

    pub fn override_track(
        &self,
        id: &str,
        change: &MetadataOverride,
    ) -> Result<(), CatalogueError> {
        let affected = self.database.execute(
            "UPDATE tracks SET override_title=?1, override_artist=?2, override_album=?3,
             override_track_number=?4, override_disc_number=?5, override_genre=?6, override_year=?7 WHERE id=?8",
            params![change.title, change.artist, change.album, change.track_number, change.disc_number, change.genre, change.year, id],
        )?;
        if affected == 0 {
            return Err(CatalogueError::UnknownTrack);
        }
        Ok(())
    }

    pub fn record_play(&self, track_id: &str) -> Result<(), CatalogueError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64);
        let inserted = self.database.execute(
            "INSERT INTO play_history (id, library_id, track_id, played_at_unix_seconds)
             SELECT ?1, library_id, id, ?2 FROM tracks WHERE id=?3",
            params![Uuid::new_v4().to_string(), now, track_id],
        )?;
        if inserted == 0 {
            return Err(CatalogueError::UnknownTrack);
        }
        Ok(())
    }

    pub fn record_origin_play(
        &self,
        event: &str,
        library: &str,
        track: &str,
        started: i64,
    ) -> Result<u32, CatalogueError> {
        self.database.execute(
            "INSERT OR IGNORE INTO play_history (id, library_id, track_id, played_at_unix_seconds)
             VALUES (?1, ?2, ?3, ?4)",
            params![event, library, track, started],
        )?;
        Ok(self.database.query_row(
            "SELECT count(*) FROM play_history WHERE library_id=?1 AND track_id=?2",
            params![library, track],
            |row| row.get(0),
        )?)
    }
}
