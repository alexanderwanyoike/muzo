//! Muzo desktop entrypoint.
//!
//! Layered per Clean Architecture (inner -> outer):
//!   domain -> application -> infrastructure -> commands
//!
//! See AGENTS.md for the full layering rules.

mod application;
mod commands;
mod domain;
mod infrastructure;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::ping::ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
