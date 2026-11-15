use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    catalogue::Catalogue,
    dropbox::{Client, Credentials, DropboxError},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const TOKEN_EXPIRY_MARGIN: u64 = 30;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub(crate) struct ArtworkAccess(Mutex<Authorizations>);

#[derive(Default)]
struct Authorizations {
    client: Option<Arc<Client>>,
    tokens: HashMap<String, Access>,
}

struct Access {
    key: String,
    token: Arc<str>,
    expires: Instant,
}

impl ArtworkAccess {
    pub(crate) fn read(
        &self,
        catalogue: &Catalogue,
        library: &str,
        path: &str,
    ) -> Result<Option<Vec<u8>>, DropboxError> {
        let account = catalogue
            .dropbox_account_for(library)?
            .ok_or(DropboxError::Reconnect)?;
        let (client, token) = self.authorize(catalogue, &account)?;
        let result = client.read_artwork(&token, path);
        if matches!(result, Err(DropboxError::Reconnect))
            && let Ok(mut state) = self.0.lock()
            && state
                .tokens
                .get(&account)
                .is_some_and(|access| Arc::ptr_eq(&access.token, &token))
        {
            state.tokens.remove(&account);
        }
        result
    }

    fn authorize(
        &self,
        catalogue: &Catalogue,
        account: &str,
    ) -> Result<(Arc<Client>, Arc<str>), DropboxError> {
        let mut state = self.0.lock().map_err(|_| DropboxError::Reconnect)?;
        let key = catalogue
            .dropbox_client(account)?
            .ok_or(DropboxError::Reconnect)?;
        if state.client.is_none() {
            state.client = Some(Arc::new(Client::for_artwork(REQUEST_TIMEOUT)?));
        }
        let client = Arc::clone(state.client.as_ref().ok_or(DropboxError::Reconnect)?);
        if let Some(access) = state.tokens.get(account)
            && access.key == key
            && access.expires > Instant::now()
        {
            return Ok((client, Arc::clone(&access.token)));
        }
        let (key, refresh) = Credentials::account(catalogue, account)?;
        let token = client.refresh(&key, &refresh)?;
        let expires = Instant::now()
            + Duration::from_secs(token.expires_in.saturating_sub(TOKEN_EXPIRY_MARGIN));
        let token: Arc<str> = token.access_token.into();
        state.tokens.insert(
            account.to_owned(),
            Access {
                key,
                token: Arc::clone(&token),
                expires,
            },
        );
        Ok((client, token))
    }
}
