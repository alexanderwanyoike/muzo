#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use muzo_desktop::desktop::Desktop;

fn main() -> iced::Result {
    iced::daemon(Desktop::boot, Desktop::update, Desktop::view)
        .title(Desktop::title)
        .theme(Desktop::theme)
        .subscription(Desktop::subscription)
        .antialiasing(true)
        .run()
}
