//! Tauri command handlers (the outermost edge).
//!
//! These translate between the JS frontend and the application layer. They
//! must stay thin: validate input, call a use case, map the result to a
//! serialisable DTO. No domain logic lives here.

pub mod ping;
