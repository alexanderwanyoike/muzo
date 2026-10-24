- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: 015

# Card 014 - Connect a Dropbox account

Connect from Settings using a public Dropbox app key and the system browser.
Paste Dropbox's authorization code into Muzo to complete the PKCE offline flow.
No client secret or hosted callback is required.

## Acceptance criteria

- [x] Settings starts authorization in the system browser using PKCE S256.
- [x] Completing authorization persists the app key and encrypted credentials.
- [x] UI and IPC expose account status without returning access or refresh tokens.
- [x] Expired or cancelled sessions cannot connect an account.
- [x] Disconnect deletes stored credentials and invalidates pending authorization.
- [x] Credential storage refuses unavailable encryption and Linux basic_text.
- [x] All repository verification commands pass (163 tests; Linux zip and deb).

## Pattern

Use Dropbox's documented public desktop client flow with PKCE, offline access,
and a manually copied code. This extends the existing API client rather than
adding an OAuth framework. No external implementation code is copied.

- https://dropbox.tech/developers/pkce--what-and-why-
- https://dropbox.tech/developers/using-oauth-2-0-with-offline-access

## Dependencies and verification

Depends on PR #16, which includes token persistence PR #40. Merge #16 first,
then retarget this card's PR to dev. New tests cover the durable public app key,
authorization service, cancellation, IPC composition, OS storage policy, and
Settings connection controls.
