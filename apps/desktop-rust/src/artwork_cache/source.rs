use std::path::Path;

use lofty::{config::ParseOptions, probe::Probe};

use crate::{catalogue::Catalogue, dropbox::ArtworkAccess, local_track::embedded_cover};

use super::{ArtworkCache, Revision};

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
    library: &str,
    track: &str,
    path: &Path,
    dropbox: Option<&ArtworkAccess>,
) -> Result<(Revision, Option<Vec<u8>>), ReadFailure> {
    let revision = catalogue
        .artwork_revision(library, track)
        .map_err(|_| ReadFailure::Catalogue)?;
    let cover = cache.get_or_try_read(&revision, || {
        if let Some(dropbox) = dropbox {
            return dropbox
                .read(catalogue, library, &path.to_string_lossy())
                .map_err(|_| ReadFailure::Dropbox);
        }
        let file = Probe::open(path)
            .map_err(|_| ReadFailure::File)?
            .options(ParseOptions::new().read_properties(false))
            .read()
            .map_err(|_| ReadFailure::File)?;
        Ok(embedded_cover(&file).map(|picture| picture.data().to_vec()))
    })?;
    Ok((revision, cover))
}
