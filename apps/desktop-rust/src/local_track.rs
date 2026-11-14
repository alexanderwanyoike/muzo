use std::{path::PathBuf, sync::Arc};

use lofty::{file::TaggedFileExt, picture::PictureType, tag::Accessor};

use crate::track::Track;

pub fn read_track(path: PathBuf) -> Track {
    read_track_with_embedded_title(path).0
}

pub(crate) fn read_track_with_embedded_title(path: PathBuf) -> (Track, Option<String>) {
    let path = path.canonicalize().unwrap_or(path);
    let file = lofty::read_from_path(&path).ok();
    let tag = file
        .as_ref()
        .and_then(|file| file.primary_tag().or_else(|| file.first_tag()));
    let embedded_title = tag
        .and_then(Accessor::title)
        .filter(|title| !title.trim().is_empty())
        .map(|title| title.into_owned());
    let title = embedded_title.clone().unwrap_or_else(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    let artwork = tag
        .and_then(|tag| {
            let pictures = tag.pictures();
            pictures
                .iter()
                .find(|picture| picture.pic_type() == PictureType::CoverFront)
                .or_else(|| pictures.first())
        })
        .map(|picture| Arc::from(picture.data()));

    let track = Track::new(
        path,
        title,
        tag.and_then(Accessor::artist)
            .map(|value| value.into_owned()),
        tag.and_then(Accessor::album)
            .map(|value| value.into_owned()),
        artwork,
    );
    (track, embedded_title)
}
