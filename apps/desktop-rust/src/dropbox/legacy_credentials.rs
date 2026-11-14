use base64::{Engine, engine::general_purpose::STANDARD};

use super::DropboxError;

pub(super) fn decrypt(encoded: &str) -> Result<String, DropboxError> {
    let ciphertext = STANDARD
        .decode(encoded)
        .map_err(|_| DropboxError::LegacyCredential)?;
    let bytes = platform::decrypt(&ciphertext)?;
    let token = String::from_utf8(bytes).map_err(|_| DropboxError::LegacyCredential)?;
    if token.is_empty() || token.chars().any(char::is_whitespace) {
        return Err(DropboxError::LegacyCredential);
    }
    Ok(token)
}

#[cfg(unix)]
fn decrypt_cbc(
    ciphertext: &[u8],
    password: &[u8],
    iterations: u32,
) -> Result<Vec<u8>, DropboxError> {
    use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};

    // These are the parameters of Chromium's synchronous OSCrypt format.
    let mut key = [0; 16];
    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, b"saltysalt", iterations, &mut key);
    let mut buffer = ciphertext.to_vec();
    let decrypted = cbc::Decryptor::<aes::Aes128>::new((&key).into(), (&[b' '; 16]).into())
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| DropboxError::LegacyCredential)?;
    Ok(decrypted.to_vec())
}

#[cfg(target_os = "linux")]
#[path = "legacy_credentials/linux.rs"]
pub(super) mod platform;

#[cfg(target_os = "macos")]
#[path = "legacy_credentials/macos.rs"]
pub(super) mod platform;

#[cfg(target_os = "windows")]
#[path = "legacy_credentials/windows.rs"]
pub(super) mod platform;

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "run only with a disposable D-Bus Secret Service"]
    fn it_decrypts_a_token_created_by_the_electron_safe_storage_backend() {
        let ciphertext = std::env::var("MUZO_TEST_ENCRYPTED_REFRESH").unwrap();
        assert_eq!(decrypt(&ciphertext).unwrap(), "muzo-fixture-refresh-7");
    }
}
