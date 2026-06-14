- Sprint: 01
- Status: Pending
- Blocks: 005

# Card 004 - Scan filesystem library recursively

## What to build

A registered library is useless if Muzo cannot see what is inside it. This card
adds the recursive scan: given a library of kind `Filesystem`, walk the folder
tree, identify audio files, read their metadata, and persist them as **tracks**
belonging to the library.

Vertical slice:

- **Domain**: `Track`, `TrackId`, `TrackTitle`, `TrackArtist`, `TrackDuration`, plus a relationship to `LibraryId`. A `TrackRepository` port.
- **Application**: a `ScanLibrary` use case. Given a `LibraryId`, finds the library, walks its location, and upserts tracks. The use case depends on a `FilesystemWalker` port, not on `std::fs` directly.
- **Infrastructure**: a recursive filesystem walker using the `walkdir` crate, an audio metadata reader (e.g. `lofty`) implementing a port, and a SQLite `TrackRepository`.
- **Commands**: a `scan_library` Tauri command.
- **UI**: a "Scan" button on each library row. Shows progress or, at minimum, a final track count.

Which file extensions count as audio is a named constant (house rule: no magic
strings). Start with `mp3`, `flac`, `m4a`, `ogg`, `opus`, `wav`.

## Ubiquitous language

- **Track**: a single playable audio file belonging to exactly one library.
- **TrackId**: stable identifier for a track within its library.
- **Scan**: a one-shot recursive walk of a library's location that upserts tracks.
- **Upsert**: insert if new, update if metadata changed, leave alone if unchanged.

## Acceptance criteria

- [ ] `Track` and its value objects live in `domain` with unit tests; no framework imports in the module.
- [ ] `TrackRepository` trait in `domain` with `upsert`, `list_for_library`, and at least one delete-on-disappear hook (used fully in 005).
- [ ] `ScanLibrary` use case in `application` is unit-tested with a fake walker and a fake `TrackRepository`.
- [ ] A `FilesystemWalker` port is defined in the domain (or `application`); the `walkdir`-based implementation lives in `infrastructure` with an integration test using a temp dir.
- [ ] An `AudioMetadataReader` port; `lofty`-based (or equivalent) implementation in `infrastructure`. Tested against committed fixture files under `src-tauri/tests/fixtures/`.
- [ ] SQLite `TrackRepository` implementation with an integration test.
- [ ] The supported-extensions list is a named constant, not inline strings.
- [ ] `scan_library` Tauri command wired up; DTOs at the boundary, never on the domain entity.
- [ ] UI shows per-library scan result (track count). Full progress bar is optional; a spinner or final count is enough.
- [ ] Re-scanning the same library is idempotent: no duplicate rows.
- [ ] All verification commands green.

## Blocked by

- 002 - Add a filesystem library.

## Notes for the implementer

- Walk + read + persist is a lot for one card. If it gets heavy, the natural
  split is: (a) define `Track` + repository + scan use case with a fake walker,
  (b) wire the real walker + metadata reader. Prefer to keep this one card if
  the test-first approach keeps it honest; split only if review says so.
- The use case must be the place where "what is an audio file" lives, via the
  walker. The walker's contract is "given a path, return the audio files",
  not "return every file".
- Persist file size and a content hash or mtime now so 005 can detect changes
  cheaply. Hashing is more correct; mtime is faster. Pick one and note the
  trade-off in an ADR if it is non-obvious.
- Symlinks: decide explicitly (follow / skip). Record the decision in code as
  a named constant or ADR.
