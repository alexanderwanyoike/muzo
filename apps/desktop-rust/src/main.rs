#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use muzo_desktop::{desktop::Desktop, fonts};

fn main() -> iced::Result {
    iced::daemon(Desktop::boot, Desktop::update, Desktop::view)
        .settings(fonts::settings())
        .title(Desktop::title)
        .theme(Desktop::theme)
        .subscription(Desktop::subscription)
        .antialiasing(true)
        .run()
}
