use iced::{Color, Theme, color, theme::Palette};

pub(super) fn palette(id: &str, dark: bool, accent: &str) -> Theme {
    let (background, text) = match (id, dark) {
        ("studio", false) => (color!(0xf7f8fa), color!(0x252c36)),
        ("aluminium", false) => (color!(0xeef0f2), color!(0x252c33)),
        ("paper", false) => (color!(0xf7f3ea), color!(0x352f27)),
        ("studio", true) => (color!(0x1b1e24), color!(0xe6eaf0)),
        ("aluminium", true) => (color!(0x272b2f), color!(0xedf1f4)),
        ("paper", true) => (color!(0x26231f), color!(0xeee8dc)),
        ("carbon", false) => (color!(0xe7e9eb), color!(0x252b32)),
        ("carbon", true) => (color!(0x16191c), color!(0xe3e9ef)),
        ("walnut", false) => (color!(0xf2ece2), color!(0x3c3028)),
        ("walnut", true) => (color!(0x231c17), color!(0xefe5d8)),
        ("porcelain", false) => (color!(0xf3f6f5), color!(0x26352f)),
        ("porcelain", true) => (color!(0x18201e), color!(0xe4eee8)),
        ("glass", false) => (color!(0xe7edf2), color!(0x192b3a)),
        ("glass", true) => (color!(0x19232d), color!(0xedf5fb)),
        ("pulse", false) => (color!(0xf3f3f3), color!(0x171717)),
        ("pulse", true) => (color!(0x121212), color!(0xf5f5f5)),
        (_, true) => (color!(0x1b1e24), color!(0xe6eaf0)),
        _ => (color!(0xf7f8fa), color!(0x252c36)),
    };
    let primary = accent_color(accent, dark);
    Theme::custom(
        id.to_owned(),
        Palette {
            background,
            text,
            primary,
            success: color!(0x5b8d61),
            warning: color!(0xb87c39),
            danger: color!(0xb14742),
        },
    )
}

fn accent_color(id: &str, dark: bool) -> Color {
    match (id, dark) {
        ("cobalt", false) => color!(0x3563a7),
        ("copper", false) => color!(0xa44e32),
        ("sage", false) => color!(0x466c52),
        ("plum", false) => color!(0x825277),
        ("amber", false) => color!(0x906415),
        ("cobalt", true) => color!(0x91b5ea),
        ("copper", true) => color!(0xe5a18a),
        ("sage", true) => color!(0xa2c7aa),
        ("plum", true) => color!(0xc6a0bd),
        ("amber", true) => color!(0xdec088),
        ("teal", false) => color!(0x256c70),
        ("rose", false) => color!(0xa24966),
        ("slate", false) => color!(0x526276),
        ("violet", false) => color!(0x6853a0),
        ("ice", false) => color!(0x39718b),
        ("teal", true) => color!(0x8dc9c9),
        ("rose", true) => color!(0xe5a5b8),
        ("slate", true) => color!(0xb6c3d6),
        ("violet", true) => color!(0xbdb0e4),
        ("ice", true) => color!(0x9dc8dd),
        ("glow-green", _) => color!(0x1ed760),
        ("glow-cyan", _) => color!(0x00d9ed),
        ("glow-blue", _) => color!(0x4b9eff),
        ("glow-violet", _) => color!(0xb48aff),
        ("glow-pink", _) => color!(0xff61bc),
        ("glow-orange", _) => color!(0xff9842),
        (_, true) => color!(0x91b5ea),
        _ => color!(0x3563a7),
    }
}
