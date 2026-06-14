use crate::application::ping as ping_use_case;

/// Tauri command that delegates to the application-layer smoke test.
///
/// Frontend can call `invoke("ping")` to verify the bridge is wired up.
#[tauri::command]
pub fn ping() -> String {
    ping_use_case::ping()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_delegates_to_the_application_layer() {
        assert_eq!(ping(), "muzo: ok");
    }
}
