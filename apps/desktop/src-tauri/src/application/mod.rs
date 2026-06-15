//! Application layer.
//!
//! Use cases (commands and queries) that orchestrate domain objects with
//! ports defined in the domain layer. Has no knowledge of Tauri, serde on
//! the wire, or any concrete adapter.

pub mod add_library;
pub mod error;
pub mod list_libraries;
pub mod ping;
pub mod scan_library;
