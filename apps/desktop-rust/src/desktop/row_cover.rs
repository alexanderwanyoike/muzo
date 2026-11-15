use std::{collections::HashMap, path::PathBuf};

use iced::widget::image::Handle;

use crate::{
    artwork_cache::{decode_cover, thumbnail},
    local_track::read_track,
    track::Track,
};

use super::cover::centered_square;

mod loader;
mod remote;

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
