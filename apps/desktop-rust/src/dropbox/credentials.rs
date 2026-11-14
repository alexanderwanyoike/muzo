use super::{DropboxError, legacy_credentials};
use crate::catalogue::Catalogue;

const SERVICE: &str = "muzo.dropbox.refresh";

pub struct Credentials;

impl Credentials {
    pub fn read_or_migrate(
        catalogue: &Catalogue,
        account: &str,
    ) -> Result<Option<String>, DropboxError> {
        Self::migrate_with(
            catalogue,
            account,
            || Self::read(account),
            legacy_credentials::decrypt,
            |token| Self::save(account, token),
        )
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

    pub fn read(account: &str) -> Result<Option<String>, DropboxError> {
        match keyring::Entry::new(SERVICE, account)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(account: &str, refresh_token: &str) -> Result<(), DropboxError> {
        keyring::Entry::new(SERVICE, account)?.set_password(refresh_token)?;
        Ok(())
    }

    pub fn remove(account: &str) -> Result<(), DropboxError> {
        match keyring::Entry::new(SERVICE, account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}
