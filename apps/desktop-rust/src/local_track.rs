use std::{path::PathBuf, sync::Arc};

use lofty::{file::TaggedFileExt, picture::PictureType, tag::Accessor};

use crate::track::Track;

pub fn read_track(path: PathBuf) -> Track {
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

    Track::new(
        path,
        title,
        tag.and_then(Accessor::artist)
            .map(|value| value.into_owned()),
        tag.and_then(Accessor::album)
            .map(|value| value.into_owned()),
        artwork,
    )
}
