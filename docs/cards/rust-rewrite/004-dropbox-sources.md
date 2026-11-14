# 004: Port Dropbox music sources

- [ ] Connect a Dropbox account with PKCE and offline access on all supported OSes.
- [ ] Store refresh credentials using the OS credential store and support reconnect.
- [ ] Browse folders and index account-scoped sources recursively.
- [ ] Reconcile Dropbox sources on startup, on a configured interval and on request.
- [ ] Download audio into a bounded cache and play it through the shared session.
- [ ] Preserve identities and migrated source data, and verify an account upgrade.
- [ ] Disconnect safely and check failed authentication and network recovery.

This port uses Dropbox's documented PKCE and offline token flow, the folder
listing cursor APIs, and the native credential facilities provided by Keyring.

The legacy importer follows Electron 43's synchronous OSCrypt formats and
[desktop identity](../../adr/0006-stable-desktop-storage-identity.md).
[Electron's Chromium patch](https://github.com/electron/electron/blob/v43.0.0/patches/chromium/revert_oscrypt_remove_sync_backend.patch)
documents the Linux Secret Service and KWallet namespaces and CBC derivation.
[Electron's native initialization](https://github.com/electron/electron/blob/v43.0.0/shell/browser/electron_browser_main_parts.cc)
sets the macOS Keychain identity and Windows Local State. The Rust side stores the
migrated token in native secure storage and clears ciphertext from the copied
catalogue only after that write succeeds.
