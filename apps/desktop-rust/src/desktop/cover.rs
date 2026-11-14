use iced::widget::image::Handle;

use crate::{artwork_cache::decode_cover, track::Track};

const BACKGROUND_EDGE: u32 = 64;
const BACKGROUND_SIGMA: f32 = 10.0;

#[derive(Default)]
pub(super) struct Cover {
    thumbnail: Option<Handle>,
    background: Option<Handle>,
}

impl Cover {
    pub(super) fn from_track(track: Option<&Track>) -> Self {
        let Some(bytes) = track.and_then(Track::artwork) else {
            return Self::default();
        };
        let background = decode_cover(bytes).map(|image| {
            let blur = image
                .thumbnail(BACKGROUND_EDGE, BACKGROUND_EDGE)
                .blur(BACKGROUND_SIGMA)
                .into_rgba8();
            Handle::from_rgba(blur.width(), blur.height(), blur.into_raw())
        });
        Self {
            thumbnail: Some(Handle::from_bytes(bytes.to_vec())),
            background,
        }
    }

    pub(super) fn thumbnail(&self) -> Option<&Handle> {
        self.thumbnail.as_ref()
    }

    pub(super) fn background(&self) -> Option<&Handle> {
        self.background.as_ref()
    }
}
