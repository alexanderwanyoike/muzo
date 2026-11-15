use lofty::{config::ParseOptions, probe::Probe};

use crate::{
    catalogue::{Catalogue, IndexedTrack},
    dropbox::ArtworkAccess,
    local_track::embedded_cover,
};

use super::ArtworkCache;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ReadFailure {
    Catalogue,
    File,
    Dropbox,
    Worker,
}

pub(crate) fn load_indexed(
    cache: &ArtworkCache,
    catalogue: &Catalogue,
    track: &IndexedTrack,
    dropbox: Option<&ArtworkAccess>,
) -> Result<Option<Vec<u8>>, ReadFailure> {
    let revision = catalogue
        .artwork_revision(&track.library_id, &track.id)
        .map_err(|_| ReadFailure::Catalogue)?;
    cache.get_or_try_read(&revision, || {
        if let Some(dropbox) = dropbox {
            return dropbox
                .read(catalogue, &track.library_id, &track.path.to_string_lossy())
                .map_err(|_| ReadFailure::Dropbox);
        }
        let file = Probe::open(&track.path)
            .map_err(|_| ReadFailure::File)?
            .options(ParseOptions::new().read_properties(false))
            .read()
            .map_err(|_| ReadFailure::File)?;
        Ok(embedded_cover(&file).map(|picture| picture.data().to_vec()))
    })
}
