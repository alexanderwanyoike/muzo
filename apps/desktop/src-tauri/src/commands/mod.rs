//! Tauri command handlers (the outermost edge).
//!
//! These translate between the JS frontend and the application layer. They
//! must stay thin: validate input, call a use case, map the result to a
//! serialisable DTO. No domain logic lives here.

pub mod add_library;
pub mod list_libraries;
pub mod load_track_audio_source;
pub mod ping;
pub mod scan_library;
