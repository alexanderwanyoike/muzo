use std::{collections::HashMap, ops::Range, path::PathBuf};

use iced::{
    Element, Size,
    widget::{image::Handle, sensor},
};

use crate::{
    artwork_cache::{decode_cover, thumbnail},
    catalogue::IndexedTrack,
    local_track::read_track,
    track::Track,
};

use super::{Message, Snapshot, cover::centered_square};

mod loader;
mod queue;

pub(super) use loader::{Complete, Loader, Target, viewport_rows};
pub(super) use queue::{QueueArtwork, QueueReconciled};

const ROW_EDGE: u32 = 48;
pub(super) const ROW_LIMIT: usize = 512;

pub type Covers = HashMap<String, Handle>;
pub type DirectCovers = HashMap<PathBuf, Handle>;

pub(in crate::desktop) fn watch_fit<'a>(
    list: impl Into<Element<'a, Message>>,
    identity: Option<&str>,
    count: usize,
    content_height: f32,
    message: impl Fn(Range<usize>) -> Message + Copy + 'a,
) -> Element<'a, Message> {
    // Iced does not report a scroll viewport when its contents already fit.
    let report = move |size: Size| {
        let rows = if content_height <= size.height {
            count
        } else {
            0
        };
        message(0..rows)
    };
    sensor(list)
        .key((identity.map(str::to_owned), count))
        .on_show(report)
        .on_resize(report)
        .into()
}

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

pub(super) fn fill_snapshot_cover(snapshot: &mut Snapshot, id: &str, cover: Handle) {
    if (snapshot.covers.len() < ROW_LIMIT || snapshot.covers.contains_key(id))
        && snapshot_tracks(snapshot).any(|track| track.id == id)
    {
        snapshot.covers.insert(id.to_owned(), cover);
    }
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
