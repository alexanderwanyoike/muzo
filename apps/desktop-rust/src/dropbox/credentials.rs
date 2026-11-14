use super::DropboxError;

const SERVICE: &str = "muzo.dropbox.refresh";

pub struct Credentials;

impl Credentials {
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
