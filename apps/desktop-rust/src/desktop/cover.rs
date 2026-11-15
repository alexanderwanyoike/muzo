use iced::widget::image::Handle;
use image::{DynamicImage, GrayImage, Luma, Rgba, RgbaImage, imageops};

use crate::{artwork_cache::decode_cover, track::Track};

pub(super) const SLEEVE_PAD: f32 = 36.0;
const SLEEVE_RADIUS: u32 = 8;
const SLEEVE_SHADOW_OFFSET: u32 = 12;
const SLEEVE_SHADOW_SIGMA: f32 = 13.0;
const SLEEVE_SHADOW_ALPHA: f32 = 38.0;
pub(super) const SLEEVE_EDGE: u32 = 240;

#[derive(Default)]
pub(super) struct Cover {
    thumbnail: Option<Handle>,
    sleeve: Option<Handle>,
}

impl Cover {
    pub(super) fn from_track(track: Option<&Track>) -> Self {
        let Some(artwork) = track.and_then(Track::artwork).and_then(decode_cover) else {
            return Self::default();
        };
        let square = centered_square(artwork);
        let sleeve = sleeve_pixels(&square, SLEEVE_EDGE, SLEEVE_RADIUS);
        Self {
            thumbnail: Some(handle(square.into_rgba8())),
            sleeve: Some(handle(sleeve)),
        }
    }

    pub(super) fn sleeve(&self) -> Option<&Handle> {
        self.sleeve.as_ref()
    }

    pub(super) fn thumbnail(&self) -> Option<&Handle> {
        self.thumbnail.as_ref()
    }
}

fn handle(image: RgbaImage) -> Handle {
    Handle::from_rgba(image.width(), image.height(), image.into_raw())
}

pub(super) fn centered_square(image: DynamicImage) -> DynamicImage {
    let (width, height) = (image.width(), image.height());
    if width == height {
        return image;
    }
    let side = width.min(height);
    image.crop_imm((width - side) / 2, (height - side) / 2, side, side)
}

fn sleeve_pixels(cover: &DynamicImage, edge: u32, radius: u32) -> RgbaImage {
    let pad = SLEEVE_PAD as u32;
    let total = edge + pad * 2;
    let mut sleeve = cover
        .resize_exact(edge, edge, imageops::FilterType::Triangle)
        .into_rgba8();

    let mut shadow = GrayImage::new(total, total);
    for (x, y, pixel) in sleeve.enumerate_pixels_mut() {
        let corner_x = radius.saturating_sub(x.min(edge - 1 - x)) as f32 - 0.5;
        let corner_y = radius.saturating_sub(y.min(edge - 1 - y)) as f32 - 0.5;
        let distance = corner_x.max(0.0).hypot(corner_y.max(0.0));
        let coverage = (radius as f32 + 0.5 - distance).clamp(0.0, 1.0);
        pixel[3] = (f32::from(pixel[3]) * coverage).round() as u8;
        shadow.put_pixel(
            x + pad,
            y + pad + SLEEVE_SHADOW_OFFSET,
            Luma([(coverage * SLEEVE_SHADOW_ALPHA).round() as u8]),
        );
    }

    let shadow = imageops::blur(&shadow, SLEEVE_SHADOW_SIGMA);
    let mut result = RgbaImage::from_fn(total, total, |x, y| {
        Rgba([0, 0, 0, shadow.get_pixel(x, y)[0]])
    });
    imageops::overlay(&mut result, &sleeve, i64::from(pad), i64::from(pad));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_masks_cover_corners_and_keeps_the_soft_sleeve_shadow() {
        let pixels = ::image::RgbaImage::from_pixel(64, 64, ::image::Rgba([194, 105, 84, 255]));
        let sleeve = sleeve_pixels(&::image::DynamicImage::ImageRgba8(pixels), 240, 8);
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
