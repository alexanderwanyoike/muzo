- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: 014

# Card 013 - Dropbox token persistence

## What to build

Persist a connected Dropbox account's access and refresh tokens behind an
encryption boundary. This lets the later connection, polling, and download
flows reuse a durable refresh token without keeping secrets in plaintext.

## What the user can do after this card

Nothing visible changes yet. This is a backend persistence slice that unlocks
the user-facing Dropbox connection flow.

## Vertical slice

- **Application**: define a Dropbox account repository interface and token
  cipher boundary.
- **Infrastructure**: add a SQLite table for a connected Dropbox account.
- **Infrastructure**: add a repository that stores only protected token
  strings.
- **Infrastructure**: add an Electron safe-storage cipher adapter.

## Acceptance criteria

- [x] SQLite migrations create `dropbox_accounts`.
- [x] Migration version advances without data loss.
- [x] Repository saves and loads a connected Dropbox account.
- [x] Repository deletes a connected Dropbox account.
- [x] Stored access and refresh tokens are protected before they reach SQLite.
- [x] Electron safe-storage adapter round-trips protected token strings.
- [x] Tests cover migration, repository, and token cipher behavior.

## Out of scope

- Starting or completing OAuth from the UI.
- Refreshing expired access tokens.
- Dropbox polling.
- Dropbox file downloads.
- Dropbox playback.
