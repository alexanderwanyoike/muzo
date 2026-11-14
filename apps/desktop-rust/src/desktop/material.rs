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
