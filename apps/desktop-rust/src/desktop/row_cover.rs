use std::{collections::HashMap, path::PathBuf};

use iced::widget::image::Handle;

use crate::{
    artwork_cache::{decode_cover, thumbnail},
    catalogue::IndexedTrack,
    local_track::read_track,
    track::Track,
};

use super::{Snapshot, cover::centered_square};

mod loader;

pub(super) use loader::{Complete, Loader, viewport_rows};

const ROW_EDGE: u32 = 48;
pub(super) const ROW_LIMIT: usize = 512;

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

pub(super) fn retain_unchanged(previous: &Snapshot, next: &mut Snapshot) {
    if previous.covers.is_empty() {
        return;
    }
    let covered: HashMap<_, _> = snapshot_tracks(previous)
        .filter_map(|track| Some((track.id.as_str(), (track, previous.covers.get(&track.id)?))))
        .collect();
    let unchanged: Covers = snapshot_tracks(next)
        .filter_map(|track| {
            let (before, handle) = covered.get(track.id.as_str())?;
            (before.library_id == track.library_id
                && before.path == track.path
                && before.file_size == track.file_size
                && before.file_mtime == track.file_mtime)
                .then(|| (track.id.clone(), (*handle).clone()))
        })
        .collect();
    next.covers.extend(unchanged);
}

fn snapshot_tracks(snapshot: &Snapshot) -> impl Iterator<Item = &IndexedTrack> {
    snapshot.tracks.iter().chain(
        snapshot
            .playlist_entries
            .iter()
            .filter_map(|entry| entry.track.as_ref()),
    )
}

pub(super) fn row_handle(bytes: &[u8]) -> Option<Handle> {
    let pixels = centered_square(decode_cover(bytes)?)
        .thumbnail(ROW_EDGE, ROW_EDGE)
        .into_rgba8();
    Some(Handle::from_rgba(
        pixels.width(),
        pixels.height(),
        pixels.into_raw(),
    ))
}
