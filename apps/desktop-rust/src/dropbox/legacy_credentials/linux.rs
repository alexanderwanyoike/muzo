use std::{collections::HashMap, time::Duration};

use dbus::blocking::Connection;
use dbus_secret_service::{EncryptionType, SecretService};

use super::{DropboxError, decrypt_cbc};

const INTERFACE: &str = "org.kde.KWallet";
const MUZO_WALLET_FOLDER: &str = "Muzo";
const MUZO_WALLET_APP: &str = "Muzo";
const ELECTRON_WALLET_FOLDER: &str = "Chromium Keys";
const ELECTRON_WALLET_KEY: &str = "Chromium Safe Storage";
const ELECTRON_WALLET_APP: &str = "Electron";

pub(super) fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, DropboxError> {
    let encrypted = ciphertext
        .strip_prefix(b"v11")
        .ok_or(DropboxError::LegacyCredential)?;
    if let Some(plaintext) = decrypt_secret_service(encrypted) {
        return Ok(plaintext);
    }
    let password = with_wallet(ELECTRON_WALLET_APP, |proxy, handle| {
        read_password(
            proxy,
            handle,
            ELECTRON_WALLET_FOLDER,
            ELECTRON_WALLET_KEY,
            ELECTRON_WALLET_APP,
        )
    })
    .flatten()
    .ok_or(DropboxError::LegacyCredential)?;
    decrypt_cbc(encrypted, password.as_bytes(), 1)
}

fn decrypt_secret_service(encrypted: &[u8]) -> Option<Vec<u8>> {
    let service = SecretService::connect(EncryptionType::Dh).ok()?;
    let matches = service
        .search_items(HashMap::from([("application", "Electron")]))
        .ok()?;
    matches.unlocked.into_iter().find_map(|item| {
        let password = item.get_secret().ok()?;
        let plaintext = decrypt_cbc(encrypted, &password, 1).ok()?;
        valid_token(&plaintext).then_some(plaintext)
    })
}

fn valid_token(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .is_ok_and(|value| !value.is_empty() && !value.chars().any(char::is_whitespace))
}

type WalletProxy<'a> = dbus::blocking::Proxy<'a, &'a Connection>;

fn with_wallet<T>(
    app: &str,
    mut operation: impl FnMut(&WalletProxy<'_>, i32) -> Option<T>,
) -> Option<T> {
    let connection = Connection::new_session().ok()?;
    for daemon in ["kwalletd6", "kwalletd5", "kwalletd"] {
        let proxy = connection.with_proxy(
            format!("org.kde.{daemon}"),
            format!("/modules/{daemon}"),
            Duration::from_secs(3),
        );
        let Ok((wallet,)) = proxy.method_call::<(String,), _, _, _>(INTERFACE, "networkWallet", ())
        else {
            continue;
        };
        let Ok((open,)) = proxy.method_call::<(bool,), _, _, _>(INTERFACE, "isOpen", (&wallet,))
        else {
            continue;
        };
        if !open {
            continue;
        }
        let Ok((handle,)) =
            proxy.method_call::<(i32,), _, _, _>(INTERFACE, "open", (&wallet, 0i64, app))
        else {
            continue;
        };
        if handle < 0 {
            continue;
        }
        let result = operation(&proxy, handle);
        let _ = proxy.method_call::<(i32,), _, _, _>(INTERFACE, "close", (handle, false, app));
        if result.is_some() {
            return result;
        }
    }
    None
}

fn read_password(
    proxy: &WalletProxy<'_>,
    handle: i32,
    folder: &str,
    key: &str,
    app: &str,
) -> Option<Option<String>> {
    let (exists,) = proxy
        .method_call::<(bool,), _, _, _>(INTERFACE, "hasEntry", (handle, folder, key, app))
        .ok()?;
    if !exists {
        return Some(None);
    }
    let (password,) = proxy
        .method_call::<(String,), _, _, _>(INTERFACE, "readPassword", (handle, folder, key, app))
        .ok()?;
    Some((!password.is_empty()).then_some(password))
}

pub(crate) fn read(account: &str) -> Option<Option<String>> {
    with_wallet(MUZO_WALLET_APP, |proxy, handle| {
        read_password(proxy, handle, MUZO_WALLET_FOLDER, account, MUZO_WALLET_APP)
    })
}

pub(crate) fn save(account: &str, token: &str) -> Option<()> {
    with_wallet(MUZO_WALLET_APP, |proxy, handle| {
        let (exists,) = proxy
            .method_call::<(bool,), _, _, _>(
                INTERFACE,
                "hasFolder",
                (handle, MUZO_WALLET_FOLDER, MUZO_WALLET_APP),
            )
            .ok()?;
        if !exists {
            let (created,) = proxy
                .method_call::<(bool,), _, _, _>(
                    INTERFACE,
                    "createFolder",
                    (handle, MUZO_WALLET_FOLDER, MUZO_WALLET_APP),
                )
                .ok()?;
            if !created {
                return None;
            }
        }
        let (written,) = proxy
            .method_call::<(i32,), _, _, _>(
                INTERFACE,
                "writePassword",
                (handle, MUZO_WALLET_FOLDER, account, token, MUZO_WALLET_APP),
            )
            .ok()?;
        (written == 0).then_some(())
    })
}

pub(crate) fn remove(account: &str) -> Option<()> {
    with_wallet(MUZO_WALLET_APP, |proxy, handle| {
        let (exists,) = proxy
            .method_call::<(bool,), _, _, _>(
                INTERFACE,
                "hasEntry",
                (handle, MUZO_WALLET_FOLDER, account, MUZO_WALLET_APP),
            )
            .ok()?;
        if exists {
            let (removed,) = proxy
                .method_call::<(i32,), _, _, _>(
                    INTERFACE,
                    "removeEntry",
                    (handle, MUZO_WALLET_FOLDER, account, MUZO_WALLET_APP),
                )
                .ok()?;
            return (removed == 0).then_some(());
        }
        Some(())
    })
}
