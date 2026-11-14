use std::{fs, ptr, slice};

use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use base64::{Engine, engine::general_purpose::STANDARD};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData},
};

use super::DropboxError;

pub(super) fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, DropboxError> {
    let profile = directories::BaseDirs::new().ok_or(DropboxError::LegacyCredential)?;
    let local_state = fs::read(profile.config_dir().join("Muzo").join("Local State"))
        .map_err(|_| DropboxError::LegacyCredential)?;
    let state = serde_json::from_slice::<serde_json::Value>(&local_state)
        .map_err(|_| DropboxError::LegacyCredential)?;
    let encoded_key = state
        .pointer("/os_crypt/encrypted_key")
        .and_then(serde_json::Value::as_str)
        .ok_or(DropboxError::LegacyCredential)?;
    let encrypted_key = STANDARD
        .decode(encoded_key)
        .map_err(|_| DropboxError::LegacyCredential)?;
    let mut wrapped_key = encrypted_key
        .strip_prefix(b"DPAPI")
        .ok_or(DropboxError::LegacyCredential)?
        .to_vec();
    let key = unlock_key(&mut wrapped_key)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| DropboxError::LegacyCredential)?;
    let body = ciphertext
        .strip_prefix(b"v10")
        .ok_or(DropboxError::LegacyCredential)?;
    let (nonce, payload) = body
        .split_at_checked(12)
        .ok_or(DropboxError::LegacyCredential)?;
    cipher
        .decrypt(Nonce::from_slice(nonce), payload)
        .map_err(|_| DropboxError::LegacyCredential)
}

fn unlock_key(wrapped: &mut [u8]) -> Result<Vec<u8>, DropboxError> {
    let input = CRYPT_INTEGER_BLOB {
        cbData: wrapped
            .len()
            .try_into()
            .map_err(|_| DropboxError::LegacyCredential)?,
        pbData: wrapped.as_mut_ptr(),
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    // DPAPI owns the output until LocalFree. Input stays live for the call.
    unsafe {
        if CryptUnprotectData(
            &input,
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
            ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        ) == 0
            || output.pbData.is_null()
        {
            return Err(DropboxError::LegacyCredential);
        }
        let key = slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData.cast());
        Ok(key)
    }
}
