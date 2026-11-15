use std::io::Cursor;

use iced::{
    Color, Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, layout, mouse,
        renderer::{Headless, Style},
        widget::Tree,
    },
    window,
};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

use super::{Desktop, Message, queue_tests::setup, row_cover};

const CENTRE: Rgba<u8> = Rgba([243, 40, 109, 255]);
const FLANKS: Rgba<u8> = Rgba([12, 232, 45, 255]);
const COVER_SHORT_SIDE: u32 = 64;
const COVER_LONG_SIDE: u32 = 128;
const MINI_FRAME: Size<u32> = Size::new(740, 560);
const LIBRARY_FRAME: Size<u32> = Size::new(1200, 800);
const QUEUE_PANE_WIDTH: usize = 260;
const LIBRARY_TRANSPORT_HEIGHT: usize = 82;
const QUEUE_THUMBNAIL_EDGE: usize = 36;
const LIBRARY_TRANSPORT_THUMBNAIL_EDGE: usize = 41;

fn rectangular_artwork(width: u32, height: u32) -> Vec<u8> {
    let side = width.min(height);
    let left = (width - side) / 2;
    let top = (height - side) / 2;
    let painting = RgbaImage::from_fn(width, height, |x, y| {
        if (left..left + side).contains(&x) && (top..top + side).contains(&y) {
            CENTRE
        } else {
            FLANKS
        }
    });
    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(painting)
        .write_to(&mut png, ImageFormat::Png)
        .unwrap();
    png.into_inner()
}

fn play_with_artwork(width: u32, height: u32) -> Desktop {
    let (mut desktop, local) = setup();
    desktop
        .session
        .as_mut()
        .unwrap()
        .replace_track(0, local.with_artwork(rectangular_artwork(width, height)))
        .unwrap();
    let _ = desktop.update(Message::Select(0));
    assert!(desktop.now_playing().is_some());
    desktop
}

fn show_queue(desktop: &mut Desktop) -> window::Id {
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::MiniResized(
        mini,
        (MINI_FRAME.width, MINI_FRAME.height).into(),
    ));
    assert!(desktop.windows.queue_open());
    mini
}

fn paint(desktop: &Desktop, id: window::Id, size: Size<u32>) -> Vec<u8> {
    let mut view = desktop.view(id);
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut tree = Tree::new(&view);
    let bounds = Rectangle::with_size((size.width, size.height).into());
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(bounds.size(), bounds.size()),
    );
    widget.draw(
        &tree,
        &mut renderer,
        &desktop.theme(id),
        &Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &bounds,
    );
    renderer.screenshot(size, 1.0, Color::WHITE)
}

fn assert_centred_square(pixels: &[u8], stride: usize, area_width: usize, edge: usize) {
    let cover: Vec<_> = pixels
        .chunks_exact(4)
        .enumerate()
        .filter(|(position, pixel)| {
            position % stride < area_width && (*pixel == CENTRE.0 || *pixel == FLANKS.0)
        })
        .map(|(position, pixel)| (position % stride, position / stride, pixel))
        .collect();
    let min_x = cover
        .iter()
        .map(|(x, _, _)| *x)
        .min()
        .expect("visible cover");
    let max_x = cover.iter().map(|(x, _, _)| *x).max().unwrap();
    let min_y = cover.iter().map(|(_, y, _)| *y).min().unwrap();
    let max_y = cover.iter().map(|(_, y, _)| *y).max().unwrap();
    assert_eq!((max_x - min_x + 1, max_y - min_y + 1), (edge, edge));
    assert!(cover.iter().all(|(_, _, pixel)| *pixel == CENTRE.0));
}

#[test]
fn it_fills_the_playing_queue_thumbnail_with_the_centre_of_wide_artwork() {
    let mut desktop = play_with_artwork(COVER_LONG_SIDE, COVER_SHORT_SIDE);
    let mini = show_queue(&mut desktop);
    let pixels = paint(&desktop, mini, MINI_FRAME);

    assert_centred_square(
        &pixels,
        MINI_FRAME.width as usize,
        QUEUE_PANE_WIDTH,
        QUEUE_THUMBNAIL_EDGE,
    );
}

#[test]
fn it_fills_the_library_transport_thumbnail_with_the_centre_of_tall_artwork() {
    let desktop = play_with_artwork(COVER_SHORT_SIDE, COVER_LONG_SIDE);
    let pixels = paint(&desktop, desktop.windows.library_id(), LIBRARY_FRAME);
    let width = LIBRARY_FRAME.width as usize;
    let transport = &pixels[..width * LIBRARY_TRANSPORT_HEIGHT * 4];

    assert_centred_square(transport, width, width, LIBRARY_TRANSPORT_THUMBNAIL_EDGE);
}

#[test]
fn it_keeps_tall_indexed_artwork_in_its_queue_row_before_playback() {
    const TRACK_ID: &str = "portrait";
    let (mut desktop, local) = setup();
    desktop
        .session
        .as_mut()
        .unwrap()
        .replace_track(1, local.with_indexed_id(TRACK_ID.into()))
        .unwrap();
    let portrait = rectangular_artwork(COVER_SHORT_SIDE, COVER_LONG_SIDE);
    desktop
        .queue_covers
        .insert(TRACK_ID.into(), row_cover::row_handle(&portrait).unwrap());
    let mini = show_queue(&mut desktop);
    let pixels = paint(&desktop, mini, MINI_FRAME);

    assert_centred_square(
        &pixels,
        MINI_FRAME.width as usize,
        QUEUE_PANE_WIDTH,
        QUEUE_THUMBNAIL_EDGE,
    );
}
