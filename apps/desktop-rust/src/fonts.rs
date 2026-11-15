use std::borrow::Cow;

use iced::{Font, Settings};

pub(crate) const TABLE_FIGURES: Font = Font::with_name("Muzo Table Text");

pub fn settings() -> Settings {
    let fonts = [
        include_bytes!("../assets/fonts/Manrope-Regular.ttf").as_slice(),
        include_bytes!("../assets/fonts/Manrope-Bold.ttf").as_slice(),
        include_bytes!("../assets/fonts/MuzoTableText-Regular.ttf").as_slice(),
    ];
    Settings {
        default_font: Font::with_name("Manrope"),
        fonts: fonts.into_iter().map(Cow::Borrowed).collect(),
        ..Settings::default()
    }
}
