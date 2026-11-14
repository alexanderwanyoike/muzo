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
