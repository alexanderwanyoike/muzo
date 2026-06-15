//! Domain layer.
//!
//! Pure Rust. No Tauri, no serde on the wire, no filesystem, no Tokio.
//! Anything that needs to be persisted or sent across a layer boundary is
//! mapped at the boundary in `application` or `infrastructure`.

pub mod library;
pub mod track;
