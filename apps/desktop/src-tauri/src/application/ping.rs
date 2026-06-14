//! Smoke-test use case used to wire the layers together at scaffold time.
//!
//! Real use cases (e.g. `AddLibrary`, `ScanLibrary`, `SyncLibrary`) will
//! replace this in sprint 01.

/// Returns a greeting. Proves the application layer compiles and is callable.
pub fn ping() -> String {
    "muzo: ok".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_a_greeting() {
        assert_eq!(ping(), "muzo: ok");
    }
}
