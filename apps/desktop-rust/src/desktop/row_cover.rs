use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use iced::widget::image::Handle;

use crate::{
    artwork_cache::{ArtworkCache, decode_cover, thumbnail},
    catalogue::{Catalogue, IndexedTrack, Library},
    local_track::read_track,
    track::Track,
};

use super::CataloguePaths;

const ROW_EDGE: u32 = 48;
pub(super) const ROW_LIMIT: usize = 512;
const PROBE_LIMIT: usize = 768;

pub type Covers = HashMap<String, Handle>;
pub type DirectCovers = HashMap<PathBuf, Handle>;

#[derive(Debug, Clone)]
pub struct OpenedFiles {
    pub tracks: Vec<Track>,
    pub covers: DirectCovers,
}

pub(super) fn open_files(paths: Vec<PathBuf>) -> OpenedFiles {
    let mut tracks = Vec::with_capacity(paths.len());
    let mut covers = DirectCovers::new();
    for path in paths {
        let track = read_track(path);
        if let Some(art) = track.artwork().and_then(thumbnail) {
            if covers.len() < ROW_LIMIT
                && let Some(handle) = row_handle(&art)
            {
                covers.insert(track.path().into(), handle);
            }
            tracks.push(track.with_artwork(art));
        } else {
            tracks.push(track.without_artwork());
        }
    }
    OpenedFiles { tracks, covers }
}

pub(super) fn indexed<'a>(
    paths: &CataloguePaths,
    catalogue: &Catalogue,
    libraries: &[Library],
    tracks: impl IntoIterator<Item = &'a IndexedTrack>,
) -> Covers {
    let local: HashSet<_> = libraries
        .iter()
        .filter(|library| library.kind == "filesystem")
        .map(|library| library.id.as_str())
        .collect();
    let cache = ArtworkCache::new(paths.artwork_cache());
    let mut covers = Covers::new();
    let mut visited = HashSet::new();
    let candidates = tracks
        .into_iter()
        .filter(|track| visited.insert(&track.id))
        .take(PROBE_LIMIT);
    for track in candidates {
        if covers.len() >= ROW_LIMIT {
            break;
        }
        let Ok(revision) = catalogue.artwork_revision(&track.library_id, &track.id) else {
            continue;
        };
        let art = if local.contains(track.library_id.as_str()) {
            cache.get_or_read(&revision, || {
                read_track(track.path.clone()).artwork().map(<[u8]>::to_vec)
            })
        } else {
            cache.lookup(&revision)
        };
        if let Some(handle) = art.as_deref().and_then(row_handle) {
            covers.insert(track.id.clone(), handle);
        }
    }
    covers
}

pub(super) fn row_handle(bytes: &[u8]) -> Option<Handle> {
    let pixels = decode_cover(bytes)?
        .thumbnail(ROW_EDGE, ROW_EDGE)
        .into_rgba8();
    Some(Handle::from_rgba(
        pixels.width(),
        pixels.height(),
        pixels.into_raw(),
    ))
}
