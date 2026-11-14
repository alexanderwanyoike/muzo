use std::{collections::HashMap, time::Duration};

use dbus::blocking::Connection;
use dbus_secret_service::{EncryptionType, SecretService};

use super::{DropboxError, decrypt_cbc};

pub(super) fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, DropboxError> {
    let encrypted = ciphertext
        .strip_prefix(b"v11")
        .ok_or(DropboxError::LegacyCredential)?;
    let password = secret_service_password()
        .or_else(kwallet_password)
        .ok_or(DropboxError::LegacyCredential)?;
    decrypt_cbc(encrypted, &password, 1)
}

fn secret_service_password() -> Option<Vec<u8>> {
    let service = SecretService::connect(EncryptionType::Dh).ok()?;
    let matches = service
        .search_items(HashMap::from([("application", "Electron")]))
        .ok()?;
    matches
        .unlocked
        .into_iter()
        .find_map(|item| item.get_secret().ok())
}

fn kwallet_password() -> Option<Vec<u8>> {
    let connection = Connection::new_session().ok()?;
    for daemon in ["kwalletd6", "kwalletd5", "kwalletd"] {
        let proxy = connection.with_proxy(
            format!("org.kde.{daemon}"),
            format!("/{daemon}"),
            Duration::from_secs(3),
        );
        let Ok((wallet,)) =
            proxy.method_call::<(String,), _, _, _>("org.kde.KWallet", "networkWallet", ())
        else {
            continue;
        };
        let Ok((open,)) =
            proxy.method_call::<(bool,), _, _, _>("org.kde.KWallet", "isOpen", (&wallet,))
        else {
            continue;
        };
        if !open {
            continue;
        }
        let Ok((handle,)) = proxy.method_call::<(i32,), _, _, _>(
            "org.kde.KWallet",
            "open",
            (&wallet, 0i64, "Electron"),
        ) else {
            continue;
        };
        if handle < 0 {
            continue;
        }
        let result = proxy.method_call::<(String,), _, _, _>(
            "org.kde.KWallet",
            "readPassword",
            (handle, "Chromium Keys", "Chromium Safe Storage", "Electron"),
        );
        let _ = proxy.method_call::<(i32,), _, _, _>(
            "org.kde.KWallet",
            "close",
            (handle, false, "Electron"),
        );
        if let Ok((password,)) = result
            && !password.is_empty()
        {
            return Some(password.into_bytes());
        }
    }
    None
}
