use std::sync::LazyLock;

use iced::{
    Alignment, Color, ContentFit, Element, Fill, Length, Radians, gradient,
    widget::{Space, container, stack, svg},
};

use crate::appearance::{Appearance, Surfaces};

use super::Message;

static ALUMINIUM: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/aluminium.svg").as_slice())
});
static CARBON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/carbon.svg").as_slice())
});
static WALNUT: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!("../../assets/materials/walnut.svg").as_slice())
});

fn texture(material: &str) -> Option<&'static svg::Handle> {
    match material {
        "aluminium" => Some(&ALUMINIUM),
        "carbon" => Some(&CARBON),
        "walnut" => Some(&WALNUT),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn has_texture(material: &str) -> bool {
    texture(material).is_some()
}

pub(super) fn chrome(
    appearance: &Appearance,
    surfaces: Surfaces,
    height: impl Into<Length>,
) -> Element<'static, Message> {
    let height = height.into();
    let base = surface(appearance, surfaces.chrome, height);
    if let Some(grain) = texture(&appearance.theme) {
        stack![
            base,
            svg(grain.clone())
                .content_fit(ContentFit::Cover)
                .width(Fill)
                .height(Fill),
        ]
        .clip(true)
        .into()
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
        ("aluminium" | "walnut", false) => 0.15,
        (_, false) => 0.10,
        (_, true) => 0.03,
    };
    let material = gradient::Linear::new(Radians(std::f32::consts::PI))
        .add_stop(0.0, mix(base, Color::WHITE, amount))
        .add_stop(1.0, base);
    container(Space::new())
        .height(height)
        .width(Fill)
        .align_x(Alignment::Center)
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
