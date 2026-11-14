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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_masks_cover_corners_and_keeps_the_soft_sleeve_shadow() {
        let pixels = ::image::RgbaImage::from_pixel(64, 64, ::image::Rgba([194, 105, 84, 255]));
        let sleeve = sleeve_pixels(
            Some(::image::DynamicImage::ImageRgba8(pixels)),
            [236, 238, 242, 255],
            240,
            8,
        );
        let pad = 36;
        assert_eq!(sleeve.get_pixel(pad, pad)[0], 0);
        assert_eq!(
            sleeve.get_pixel(pad + 120, pad + 120).0,
            [194, 105, 84, 255]
        );
        assert!(sleeve.get_pixel(pad + 120, pad + 248)[3] > 0);
        assert_eq!(sleeve.get_pixel(0, 0)[3], 0);
    }
}
