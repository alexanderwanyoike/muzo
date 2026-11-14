use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use lofty::{file::TaggedFileExt, picture::PictureType, tag::Accessor};

pub struct Track {
    path: PathBuf,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    artwork: Option<Arc<[u8]>>,
}

impl Track {
    pub fn read(path: PathBuf) -> Self {
        let file = lofty::read_from_path(&path).ok();
        let tag = file
            .as_ref()
            .and_then(|file| file.primary_tag().or_else(|| file.first_tag()));
        let title = tag.and_then(Accessor::title).map_or_else(
            || {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            },
            |title| title.into_owned(),
        );
        let artwork = tag
            .and_then(|tag| {
                let pictures = tag.pictures();
                pictures
                    .iter()
                    .find(|picture| picture.pic_type() == PictureType::CoverFront)
                    .or_else(|| pictures.first())
            })
            .map(|picture| Arc::from(picture.data()));

        Self {
            path,
            title,
            artist: tag
                .and_then(Accessor::artist)
                .map(|value| value.into_owned()),
            album: tag
                .and_then(Accessor::album)
                .map(|value| value.into_owned()),
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
