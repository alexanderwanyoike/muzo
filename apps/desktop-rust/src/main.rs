#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use muzo_desktop::desktop::Desktop;

fn main() -> iced::Result {
    iced::daemon(Desktop::boot, Desktop::update, Desktop::view)
        .font(include_bytes!("../assets/fonts/Manrope-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Manrope-Bold.ttf").as_slice())
        .default_font(iced::Font::with_name("Manrope"))
        .title(Desktop::title)
        .theme(Desktop::theme)
        .subscription(Desktop::subscription)
        .antialiasing(true)
        .run()
}
