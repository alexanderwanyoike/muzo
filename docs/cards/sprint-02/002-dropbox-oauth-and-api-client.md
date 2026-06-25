- Sprint: 02
- Type: Feature
- Status: Pending
- Blocks: 003

# Card 002 - Dropbox OAuth and API client

## What to build

Add the backend foundation for connecting Muzo to Dropbox. This card should
not scan Dropbox libraries yet; it should make Dropbox authorization and API
calls a first-class infrastructure capability that card 003 can use as an
adapter.

Muzo is a desktop app, so Dropbox authorization should use OAuth code flow with
PKCE and request offline access so future background polling can refresh
short-lived access tokens without prompting the user each time.

## What the user can do after this card

1. Start a Dropbox authorization flow from Muzo.
2. Complete authorization in Dropbox.
3. Exchange the returned authorization code for a Dropbox token payload.
4. Refresh an access token with a stored refresh token.
5. List Dropbox folder contents recursively through the backend client.

This card does not add Dropbox libraries to the library scanner yet. That is
card 003.

## Vertical slice

- **Application**:
  - Build Dropbox OAuth authorization URLs using PKCE.
  - Keep token exchange inputs explicit: authorization code, code verifier,
    redirect URI, and client ID.
- **Infrastructure**:
  - Add a Dropbox API client that can exchange authorization codes, refresh
    access tokens, and list folders recursively.
  - Keep HTTP behind a testable transport seam.
  - Parse Dropbox file and folder metadata needed by the later library adapter.
- **Commands**:
  - Add command DTOs only if this PR wires the OAuth flow through Tauri state.
- **UI**:
  - Out of scope for this backend foundation slice unless commands are added.

## Acceptance criteria

- [ ] OAuth authorization URL uses Dropbox's authorize endpoint.
- [ ] OAuth authorization URL includes `response_type=code`.
- [ ] OAuth authorization URL includes `token_access_type=offline`.
- [ ] OAuth authorization URL uses PKCE `S256` code challenge.
- [ ] Token exchange posts to Dropbox's token endpoint with
      `grant_type=authorization_code`, `client_id`, authorization code, and
      code verifier.
- [ ] Token refresh posts to Dropbox's token endpoint with
      `grant_type=refresh_token`, `client_id`, and refresh token.
- [ ] Dropbox API client sends bearer-authenticated requests.
- [ ] Recursive folder listing calls `/2/files/list_folder` and follows
      `/2/files/list_folder/continue` while `has_more` is true.
- [ ] Folder listing returns file and folder metadata separately enough for
      card 003 to filter playable files later.
- [ ] Domain layer has no Dropbox, OAuth, HTTP, or serde dependency.
- [ ] Tests cover PKCE URL construction, token exchange, refresh, and paged
      folder listing with a fake transport.
- [ ] All verification commands green.

## Blocked by

- Sprint 02 card 001 - Formal SQLite migrations.

## Notes for the implementer

- Use Dropbox's official OAuth guidance for desktop apps: code flow with PKCE,
  `S256`, and `token_access_type=offline`.
- The redirect URI may be omitted for a copy-paste code flow in early desktop
  builds, but the API should support one because Dropbox validates registered
  redirect URIs when provided.
- Do not store tokens in plaintext in this card. A later storage card should
  decide whether to use OS keychain support or another local secret store.
- Do not add a Dropbox scanner here. Card 003 should consume the client and map
  Dropbox metadata into the existing library scan concepts.

## Out of scope

- Dropbox library scanning.
- Token persistence.
- OS keychain integration.
- OAuth callback listener.
- Dropbox file downloads.
- UI for connecting an account.
