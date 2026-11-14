use std::sync::LazyLock;

use iced::{
    Color, Element, Fill, Length, Point, Radians, Rectangle, Renderer, Size, Theme,
    advanced::{Layout, Widget, layout, mouse, renderer, svg, widget::Tree},
    gradient,
    widget::{Space, container, stack},
};

use crate::appearance::{Appearance, Surfaces};

use super::Message;

static ALUMINIUM: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/aluminium.svg").as_slice())
});
static CARBON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/carbon.svg").as_slice())
});
static CARBON_DARK: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/carbon-dark.svg").as_slice())
});
static WALNUT: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/walnut.svg").as_slice())
});
static WALNUT_DARK: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/walnut-dark.svg").as_slice())
});

fn texture(material: &str, dark: bool) -> Option<&'static svg::Handle> {
    match (material, dark) {
        ("aluminium", _) => Some(&ALUMINIUM),
        ("carbon", false) => Some(&CARBON),
        ("carbon", true) => Some(&CARBON_DARK),
        ("walnut", false) => Some(&WALNUT),
        ("walnut", true) => Some(&WALNUT_DARK),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn has_texture(material: &str) -> bool {
    texture(material, false).is_some()
}

pub(super) fn chrome(
    appearance: &Appearance,
    surfaces: Surfaces,
    height: impl Into<Length>,
) -> Element<'static, Message> {
    let height = height.into();
    let base = surface(appearance, surfaces.chrome, height);
    if let Some(grain) = texture(&appearance.theme, appearance.mode == "dark") {
        stack![base, Element::new(Grain(grain))].clip(true).into()
    } else {
        base
    }
}

fn surface(appearance: &Appearance, base: Color, height: Length) -> Element<'static, Message> {
    let dark = appearance.mode == "dark";
    let amount = match (appearance.theme.as_str(), dark) {
        ("pulse" | "carbon", _) => 0.0,
        ("studio", false) => 0.5,
        ("porcelain" | "glass", false) => 0.45,
        ("paper", false) => 0.25,
        ("aluminium" | "walnut", false) => 0.125,
        ("walnut", true) => 0.016,
        (_, false) => 0.10,
        (_, true) => 0.03,
    };
    let material = gradient::Linear::new(Radians(std::f32::consts::PI))
        .add_stop(0.0, mix(base, Color::WHITE, amount))
        .add_stop(1.0, base);
    container(Space::new())
        .height(height)
        .width(Fill)
        .style(move |_| container::Style::default().background(material))
        .into()
}

fn mix(base: Color, overlay: Color, amount: f32) -> Color {
    Color::from_rgb(
        base.r + (overlay.r - base.r) * amount,
        base.g + (overlay.g - base.g) * amount,
        base.b + (overlay.b - base.b) * amount,
    )
}

struct Grain(&'static svg::Handle);

impl Widget<Message, Theme, Renderer> for Grain {
    fn size(&self) -> Size<Length> {
        Size::new(Fill, Fill)
    }

    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(Fill, Fill, Size::ZERO))
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use svg::Renderer as _;

        let frame = layout.bounds();
        let Some(clip) = frame.intersection(viewport) else {
            return;
        };
        let tile = renderer.measure_svg(self.0);
        let tile = Size::new(tile.width.max(1) as f32, tile.height.max(1) as f32);
        let first_column = ((clip.x - frame.x) / tile.width).floor() as u32;
        let last_column = ((clip.x + clip.width - frame.x) / tile.width).ceil() as u32;
        let first_row = ((clip.y - frame.y) / tile.height).floor() as u32;
        let last_row = ((clip.y + clip.height - frame.y) / tile.height).ceil() as u32;
        for row in first_row..last_row {
            for column in first_column..last_column {
                renderer.draw_svg(
                    self.0.into(),
                    Rectangle::new(
                        Point::new(
                            frame.x + column as f32 * tile.width,
                            frame.y + row as f32 * tile.height,
                        ),
                        tile,
                    ),
                    clip,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::advanced::renderer::Headless;
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size, Theme,
        advanced::{Layout, layout, mouse, renderer, widget::Tree},
    };

    use super::*;

    fn chrome_pixels(width: u32, mode: &str) -> Vec<u8> {
        let appearance = Appearance {
            theme: "walnut".into(),
            mode: mode.into(),
            ..Appearance::default()
        };
        let size = Size::new(width, 64);
        let logical_size = Size::new(width as f32, 64.0);
        let mut chrome = chrome(&appearance, appearance.surfaces(), 64);
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut tree = Tree::new(&chrome);
        let widget = chrome.as_widget_mut();
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(logical_size, logical_size),
        );
        widget.draw(
            &tree,
            &mut renderer,
            &Theme::Light,
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &Rectangle::with_size(logical_size),
        );
        let pixels = renderer.screenshot(size, 1.0, Color::WHITE);
        pixels
            .chunks_exact(width as usize * 4)
            .nth(9)
            .unwrap()
            .to_vec()
    }

    #[test]
    fn it_paints_walnut_grain_at_the_same_scale_after_resizing() {
        for mode in ["light", "dark"] {
            let narrow = chrome_pixels(180, mode);
            let wide = chrome_pixels(720, mode);
            assert!(narrow.chunks_exact(4).any(|pixel| pixel != &narrow[..4]));
            assert!(
                narrow == wide[..narrow.len()],
                "walnut grain moved when the {mode} window resized"
            );
        }
    }
}
