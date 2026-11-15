use iced::{Color, color};

#[derive(Clone, Copy)]
pub(crate) struct Surfaces {
    pub(crate) raised: Color,
    pub(crate) sidebar: Color,
    pub(crate) chrome: Color,
    pub(crate) line: Color,
    pub(crate) muted: Color,
}

impl Surfaces {
    const fn new(raised: Color, sidebar: Color, chrome: Color, line: Color, muted: Color) -> Self {
        Self {
            raised,
            sidebar,
            chrome,
            line,
            muted,
        }
    }
}

macro_rules! surfaces {
    ($raised:literal, $sidebar:literal, $chrome:literal, $line:literal, $muted:literal) => {
        Surfaces::new(
            color!($raised),
            color!($sidebar),
            color!($chrome),
            color!($line),
            color!($muted),
        )
    };
}

pub(super) fn for_material(material: &str, dark: bool) -> Surfaces {
    match (material, dark) {
        ("aluminium", false) => surfaces!(0xfafbfc, 0xd9dde1, 0xe1e5e8, 0xbcc3ca, 0x505c68),
        ("paper", false) => surfaces!(0xfffdf7, 0xebe5d9, 0xefe9dd, 0xd6cdbd, 0x6b604f),
        ("aluminium", true) => surfaces!(0x32373c, 0x2b3035, 0x383e44, 0x4b535c, 0xbcc5ce),
        ("paper", true) => surfaces!(0x302c27, 0x2e2a25, 0x38322c, 0x514a40, 0xb9ae9b),
        ("carbon", false) => surfaces!(0xf5f6f7, 0xd9dde1, 0xd4d9de, 0xb9c0c8, 0x4f5a66),
        ("carbon", true) => surfaces!(0x202429, 0x191d22, 0x282e35, 0x3b454f, 0xa4b2bf),
        ("walnut", false) => surfaces!(0xfcf7ef, 0xe4d8c8, 0xe8dccb, 0xcfc0ad, 0x665343),
        ("walnut", true) => surfaces!(0x2e251e, 0x291f19, 0x382a20, 0x514033, 0xbfad98),
        ("porcelain", false) => surfaces!(0xffffff, 0xe6edeb, 0xedf2f0, 0xccd7d2, 0x51675b),
        ("porcelain", true) => surfaces!(0x222c28, 0x1d2722, 0x2b3730, 0x3e5146, 0xa6bcb0),
        ("glass", false) => surfaces!(0xf6f9fc, 0xdde6ee, 0xe2eaf1, 0xb5c5d3, 0x43515f),
        ("glass", true) => surfaces!(0x25323e, 0x1c2935, 0x22303e, 0x455a6c, 0xb9c9d5),
        ("pulse", false) => surfaces!(0xffffff, 0xe7e7e7, 0xfafafa, 0xd4d4d4, 0x555555),
        ("pulse", true) => surfaces!(0x1f1f1f, 0x080808, 0x181818, 0x303030, 0xb3b3b3),
        (_, false) => surfaces!(0xffffff, 0xeceef2, 0xf0f1f4, 0xd5d9e0, 0x616b78),
        (_, true) => surfaces!(0x252931, 0x20242b, 0x292e37, 0x3e4551, 0xa5afbd),
    }
}
