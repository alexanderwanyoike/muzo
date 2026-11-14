use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct Track {
    path: PathBuf,
    indexed_id: Option<String>,
    library: Option<String>,
    remote: Option<(String, String)>,
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
            indexed_id: None,
            library: None,
            remote: None,
            title,
            artist,
            album,
            artwork,
        }
    }

    pub fn remote(&self) -> Option<(&str, &str)> {
        self.remote
            .as_ref()
            .map(|(library, path)| (library.as_str(), path.as_str()))
    }

    pub fn with_remote(mut self, library: String, path: String) -> Self {
        self.remote = Some((library, path));
        self
    }

    pub fn with_audio(mut self, local: PathBuf, metadata: &Track) -> Self {
        self.path = local;
        self.artwork = metadata.artwork.clone();
        if self.artist.as_deref().is_none_or(str::is_empty) {
            self.artist = metadata.artist.clone();
        }
        if self.album.is_none() {
            self.album = metadata.album.clone();
        }
        self
    }

    pub(crate) fn with_artwork(mut self, bytes: Vec<u8>) -> Self {
        self.artwork = Some(Arc::from(bytes));
        self
    }

    pub(crate) fn without_artwork(mut self) -> Self {
        self.artwork = None;
        self
    }

    pub fn library(&self) -> Option<&str> {
        self.library.as_deref()
    }

    pub fn with_library(mut self, library: String) -> Self {
        self.library = Some(library);
        self
    }

    pub fn indexed_id(&self) -> Option<&str> {
        self.indexed_id.as_deref()
    }

    pub fn with_indexed_id(mut self, id: String) -> Self {
        self.indexed_id = Some(id);
        self
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
