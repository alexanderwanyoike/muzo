# ADR 0004 - Dropbox desktop authorization

- Status: Accepted
- Date: 2026-10-24

## Context

Muzo is a public desktop client. It cannot keep an OAuth client secret, and it
already implements Dropbox PKCE URL construction, token exchange, and encrypted
token persistence. No hosted callback or distributed Dropbox app key is configured.

## Decision

Use Dropbox's PKCE S256 authorization-code flow with offline access in the system
browser. Settings accepts the public app key and the authorization code Dropbox
displays when no redirect URI is supplied. Keep the verifier in the main process
for at most ten minutes. Store the app key with encrypted credentials so refresh
can work after restart. Only public connection status crosses IPC.

Require OS-backed encryption. Refuse credential writes when encryption is
unavailable or Electron selects Linux basic_text storage.

## Consequences

Users configure a scoped Dropbox app with files.metadata.read and
files.content.read permissions and paste its public app key. No app secret is
requested. Authorization requires one manual code copy. Disconnect removes the
local credentials; users can also revoke access in Dropbox's connected apps.

## References

- https://dropbox.tech/developers/pkce--what-and-why-
- https://dropbox.tech/developers/using-oauth-2-0-with-offline-access
