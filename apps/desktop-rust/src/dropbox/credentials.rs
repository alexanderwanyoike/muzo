use super::{DropboxError, legacy_credentials};
use crate::catalogue::Catalogue;

const SERVICE: &str = "muzo.dropbox.refresh";
static LIFECYCLE: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub struct Credentials;

impl Credentials {
    pub fn read_or_migrate(
        catalogue: &Catalogue,
        account: &str,
    ) -> Result<Option<String>, DropboxError> {
        let _guard = lifecycle_guard()?;
        if catalogue.dropbox_client(account)?.is_none() {
            return Ok(None);
        }
        Self::migrate_with(
            catalogue,
            account,
            || Self::read(account),
            legacy_credentials::decrypt,
            |token| Self::save(account, token),
        )
    }

    pub fn connect(
        catalogue: &Catalogue,
        account: &str,
        client: &str,
        token: &str,
    ) -> Result<(), DropboxError> {
        let _guard = lifecycle_guard()?;
        let previous = Self::read(account)?;
        Self::save(account, token)?;
        if let Err(error) = catalogue.save_dropbox_account(account, client) {
            if let Some(previous) = previous {
                Self::save(account, &previous)?;
            } else {
                Self::remove(account)?;
            }
            return Err(error.into());
        }
        Ok(())
    }

    pub fn disconnect(catalogue: &Catalogue, account: &str) -> Result<Vec<String>, DropboxError> {
        let _guard = lifecycle_guard()?;
        let libraries = catalogue.dropbox_libraries_for(account)?;
        let previous = Self::read(account)?;
        Self::remove(account)?;
        if let Err(error) = catalogue.disconnect_dropbox(account) {
            if let Some(previous) = previous {
                Self::save(account, &previous)?;
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

    fn read(account: &str) -> Result<Option<String>, DropboxError> {
        let result = keyring::Entry::new(SERVICE, account).and_then(|entry| entry.get_password());
        #[cfg(target_os = "linux")]
        if result.is_err()
            && let Some(value) = legacy_credentials::platform::read(account)
        {
            return Ok(value);
        }
        match result {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn save(account: &str, refresh_token: &str) -> Result<(), DropboxError> {
        let result = keyring::Entry::new(SERVICE, account)
            .and_then(|entry| entry.set_password(refresh_token));
        #[cfg(target_os = "linux")]
        if result.is_err() && legacy_credentials::platform::save(account, refresh_token).is_some() {
            return Ok(());
        }
        Ok(result?)
    }

    fn remove(account: &str) -> Result<(), DropboxError> {
        let result =
            keyring::Entry::new(SERVICE, account).and_then(|entry| entry.delete_credential());
        #[cfg(target_os = "linux")]
        if legacy_credentials::platform::remove(account).is_some() && result.is_err() {
            return Ok(());
        }
        match result {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

fn lifecycle_guard() -> Result<std::sync::MutexGuard<'static, ()>, DropboxError> {
    LIFECYCLE.lock().map_err(|_| DropboxError::LegacyCredential)
}
