- Sprint: 02
- Type: Fix
- Status: Done
- Depends on: 026 (PR #53)

# Card 027 - Preserve Dropbox secure storage identity

The native app-name change caused existing Dropbox credentials to fail decryption.
Electron binds its Linux/macOS OS-crypt key namespace before ready. Earlier builds
used Electron, while the branding update selected Muzo. The original encrypted
credentials are intact and readable under their original identity.

## Acceptance criteria

- [x] Preserve the established keyring identity during Electron initialization.
- [x] Keep Muzo as the visible name after native initialization.
- [x] Keep the existing Muzo renderer/session profile and appearance preferences.
- [x] Read existing credentials without rewriting them or asking for reauthorization.
- [x] Verify fresh profile startup with real Electron.
- [x] Complete ordered checks and two review cycles (286 tests).

## Reproduction and evidence

A read-only native probe against the existing catalogue found gnome_libsecret
available with saved credentials. With identity Muzo both decryptions failed;
with identity Electron both succeeded; @muzo/desktop failed. The corrected
production policy reported the visible name Muzo and successfully decrypted both
tokens twice. Output contained only booleans and backend/name metadata, never
credentials. The catalogue was not modified by the probe.

The identity contract test was committed red before implementation. The native
icon fixture now also verifies a fresh isolated Muzo profile and visible name;
it runs in the Linux suite and native Windows/macOS CI jobs.

## Established pattern

Follow Electron's MIT-licensed initialization order and stable OS-crypt identity:
https://github.com/electron/electron/blob/v43.0.0/shell/browser/electron_browser_main_parts.cc
PostCreateMainMessageLoop derives Linux application/product names and macOS
Keychain service/account names before ready. The visible name can be set after
that initialization. Explicit userData/sessionData paths preserve the current
renderer profile independently of the credential namespace.
https://www.electronjs.org/docs/latest/api/app
https://www.electronjs.org/docs/latest/api/safe-storage
No cryptographic implementation or keyring content is copied.

Review record: [two cycles](../../reviews/2026-10-24-dropbox-storage-identity.md).
