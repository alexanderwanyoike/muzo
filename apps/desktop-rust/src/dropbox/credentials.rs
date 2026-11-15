use super::{DropboxError, legacy_credentials};
use crate::catalogue::Catalogue;

const SERVICE: &str = "muzo.dropbox.refresh";
static LIFECYCLE: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub struct Credentials;

impl Credentials {
    pub(crate) fn account(
        catalogue: &Catalogue,
        account: &str,
    ) -> Result<(String, String), DropboxError> {
        let key = catalogue
            .dropbox_client(account)?
            .ok_or(DropboxError::Reconnect)?;
        let refresh = Self::read_or_migrate(catalogue, account)?.ok_or(DropboxError::Reconnect)?;
        Ok((key, refresh))
    }

    pub fn read_or_migrate(
        catalogue: &Catalogue,
        account: &str,
    ) -> Result<Option<String>, DropboxError> {
        let _guard = lifecycle_guard()?;
        if catalogue.dropbox_client(account)?.is_none() {
            return Ok(None);
        }
        let (store, secret) = resolve_store(catalogue, account)?;
        catalogue.set_dropbox_credential_store(account, store.label())?;
        Self::migrate_with(
            catalogue,
            account,
            || Ok(secret),
            legacy_credentials::decrypt,
            |token| store.save(account, token),
        )
    }

    pub fn connect(
        catalogue: &Catalogue,
        account: &str,
        client: &str,
        token: &str,
    ) -> Result<(), DropboxError> {
        let _guard = lifecycle_guard()?;
        let (store, previous) = resolve_store(catalogue, account)?;
        store.save(account, token)?;
        if let Err(error) = catalogue.save_native_dropbox_account(account, client, store.label()) {
            if let Some(previous) = previous {
                store.save(account, &previous)?;
            } else {
                store.remove(account)?;
            }
            return Err(error.into());
        }
        Ok(())
    }

    pub fn disconnect(catalogue: &Catalogue, account: &str) -> Result<Vec<String>, DropboxError> {
        let _guard = lifecycle_guard()?;
        let libraries = catalogue.dropbox_libraries_for(account)?;
        let (store, previous) = resolve_store(catalogue, account)?;
        store.remove(account)?;
        if let Err(error) = catalogue.disconnect_dropbox(account) {
            if let Some(previous) = previous {
                store.save(account, &previous)?;
            }
            return Err(error.into());
        }
        Ok(libraries)
    }

    pub(crate) fn migrate_with(
        catalogue: &Catalogue,
        account: &str,
        read: impl FnOnce() -> Result<Option<String>, DropboxError>,
        decrypt: impl FnOnce(&str) -> Result<String, DropboxError>,
        save: impl FnOnce(&str) -> Result<(), DropboxError>,
    ) -> Result<Option<String>, DropboxError> {
        let stored = read()?;
        let Some(encrypted) = catalogue.legacy_dropbox_token(account)? else {
            return Ok(stored);
        };
        let token = if let Some(stored) = stored {
            stored
        } else {
            let token = decrypt(&encrypted)?;
            save(&token)?;
            token
        };
        catalogue.clear_legacy_dropbox_tokens(account)?;
        Ok(Some(token))
    }
}

#[derive(Clone, Copy)]
enum Store {
    Keyring,
    #[cfg(target_os = "linux")]
    KWallet,
}

impl Store {
    fn label(self) -> &'static str {
        match self {
            Self::Keyring => "keyring",
            #[cfg(target_os = "linux")]
            Self::KWallet => "kwallet",
        }
    }

    fn read(self, account: &str) -> Result<Option<String>, DropboxError> {
        match self {
            Self::Keyring => match keyring::Entry::new(SERVICE, account)?.get_password() {
                Ok(secret) => Ok(Some(secret)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(error) => Err(error.into()),
            },
            #[cfg(target_os = "linux")]
            Self::KWallet => {
                legacy_credentials::platform::read(account).ok_or(DropboxError::WalletUnavailable)
            }
        }
    }

    fn save(self, account: &str, token: &str) -> Result<(), DropboxError> {
        match self {
            Self::Keyring => Ok(keyring::Entry::new(SERVICE, account)?.set_password(token)?),
            #[cfg(target_os = "linux")]
            Self::KWallet => legacy_credentials::platform::save(account, token)
                .ok_or(DropboxError::WalletUnavailable),
        }
    }

    fn remove(self, account: &str) -> Result<(), DropboxError> {
        match self {
            Self::Keyring => match keyring::Entry::new(SERVICE, account)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(error) => Err(error.into()),
            },
            #[cfg(target_os = "linux")]
            Self::KWallet => {
                legacy_credentials::platform::remove(account).ok_or(DropboxError::WalletUnavailable)
            }
        }
    }
}

fn resolve_store(
    catalogue: &Catalogue,
    account: &str,
) -> Result<(Store, Option<String>), DropboxError> {
    let configured = catalogue.dropbox_credential_store(account)?;
    let store = match configured.as_deref() {
        Some("keyring") | None => Store::Keyring,
        #[cfg(target_os = "linux")]
        Some("kwallet") => Store::KWallet,
        Some(_) => return Err(DropboxError::WalletUnavailable),
    };
    let result = store.read(account);
    #[cfg(target_os = "linux")]
    if configured.is_none()
        && matches!(&result, Err(DropboxError::Credential(error)) if secret_service_unavailable(error))
    {
        let store = Store::KWallet;
        return Ok((store, store.read(account)?));
    }
    Ok((store, result?))
}

#[cfg(target_os = "linux")]
fn secret_service_unavailable(error: &keyring::Error) -> bool {
    use std::error::Error;
    error
        .source()
        .and_then(|source| source.downcast_ref::<dbus_secret_service::Error>())
        .is_some_and(|error| match error {
            dbus_secret_service::Error::Unavailable => true,
            dbus_secret_service::Error::Dbus(error) => matches!(
                error.name(),
                Some(
                    "org.freedesktop.DBus.Error.ServiceUnknown"
                        | "org.freedesktop.DBus.Error.NameHasNoOwner"
                )
            ),
            _ => false,
        })
}

fn lifecycle_guard() -> Result<std::sync::MutexGuard<'static, ()>, DropboxError> {
    LIFECYCLE.lock().map_err(|_| DropboxError::LegacyCredential)
}
