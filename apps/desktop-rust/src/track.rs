use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct Track {
    path: PathBuf,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    artwork: Option<Arc<[u8]>>,
}

impl Track {
    pub fn new(
        path: PathBuf,
        title: String,
        artist: Option<String>,
        album: Option<String>,
        artwork: Option<Arc<[u8]>>,
    ) -> Self {
        Self {
            path,
            title,
            artist,
            album,
            artwork,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }
    pub fn artwork(&self) -> Option<&[u8]> {
        self.artwork.as_deref()
    }
}
