- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 003 - Dropbox library adapter

## What to build

Add the backend adapter that maps Dropbox folder metadata into Muzo's library
scan model. Card 002 added OAuth and a Dropbox API client; this card consumes
that client through an application-level port so Muzo can treat a Dropbox
folder as a library source.

This first adapter slice should not download audio files or make Dropbox
tracks playable yet. It should establish the scan boundary, filter supported
audio files, persist Dropbox-backed tracks, and remove stale tracks when files
disappear from Dropbox.

## What the user can do after this card

1. Have a Dropbox library whose location is a Dropbox folder path.
2. Scan that Dropbox folder through backend application code using a Dropbox
   access token supplied by the caller.
3. See supported audio files from Dropbox represented as tracks in Muzo.
4. See deleted remote files removed on the next Dropbox scan.

UI wiring, account connection, token persistence, downloads, and playback are
separate follow-ups.

## Vertical slice

- **Domain**:
  - No new Dropbox-specific domain types. `LibraryKind::Dropbox` already
    identifies the source kind.
- **Application**:
  - Add a Dropbox scan use case that loads a persisted Dropbox library,
    asks an application port for remote audio files, upserts tracks, and
    removes stale tracks.
  - Reject non-Dropbox libraries with a clear error.
- **Infrastructure**:
  - Add a Dropbox catalog adapter that uses the Dropbox API client from card
    002 and filters to Muzo-supported audio extensions.
  - Map Dropbox file metadata into the application port's source-file type.
- **Commands / UI**:
  - Out of scope until token persistence and account connection exist.

## Acceptance criteria

- [x] Dropbox scan use case rejects missing libraries.
- [x] Dropbox scan use case rejects non-Dropbox libraries.
- [x] Dropbox scan asks the remote catalog for files under the library
      location using the caller's access token.
- [x] Dropbox scan persists supported audio files as tracks for the library.
- [x] Dropbox scan removes tracks whose remote paths are no longer returned.
- [x] Dropbox adapter filters out unsupported remote file extensions.
- [x] Dropbox adapter maps Dropbox path, size, and modified timestamp into
      application source-file metadata.
- [x] Domain layer has no Dropbox API, OAuth, HTTP, or serde dependency.
- [x] Tests cover the scan use case with a fake catalog and repository.
- [x] Tests cover the infrastructure adapter with a fake Dropbox API client.
- [x] Verification covered by the current test suites.

## Blocked by

- Sprint 02 card 002 - Dropbox OAuth and API client.

## Notes for the implementer

- Keep the access token outside the domain. The use case can accept it as an
  application input until token storage lands.
- The existing `TrackFilePath` can temporarily store a Dropbox path as an
  opaque source path. A later playback/download card should split local file
  paths from remote source paths if needed.
- Use the existing audio extension list from the filesystem walker so Dropbox
  and filesystem scanning agree on supported formats.
- Without downloading files, embedded metadata and duration are not available.
  Use filename fallback metadata and duration `0` for this slice.

## Out of scope

- OAuth commands and UI.
- Token persistence.
- Dropbox file downloads.
- Playback for Dropbox tracks.
- Dropbox polling loop.
- Dropbox cursor persistence.
