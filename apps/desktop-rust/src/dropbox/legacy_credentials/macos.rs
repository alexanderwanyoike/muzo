use security_framework::passwords::get_generic_password;

use super::{DropboxError, decrypt_cbc};

pub(super) fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, DropboxError> {
    let encrypted = ciphertext
        .strip_prefix(b"v10")
        .ok_or(DropboxError::LegacyCredential)?;
    let password = get_generic_password("Electron Safe Storage", "Electron")
        .map_err(|_| DropboxError::LegacyCredential)?;
    decrypt_cbc(encrypted, &password, 1003)
}
