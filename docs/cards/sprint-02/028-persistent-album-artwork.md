- Sprint: 02
- Type: Fix
- Status: In review
- Depends on: 027 (PR #54)

# Card 028 - Persistent album artwork

The old 128-entry process-local cache disappears on exit, making every launch
extract or fetch covers again. Keep bounded thumbnail records on disk and reuse
them before requesting media or Dropbox credentials.

## Acceptance criteria

- [x] Reuse MP3/FLAC thumbnails across new application instances.
- [x] Reuse cached Dropbox covers while offline or disconnected.
- [x] Validate the retained library/track before looking up its cached art.
- [x] Regenerate art when the stored file size/mtime revision or path changes.
- [x] Retain known missing covers; retry failed media/cache reads normally.
- [x] Bound the cache to 128 MiB or 4,096 records, evicting older covers.
- [x] Use private directories/files, hashed filenames and atomic record replacement.
- [x] Verify content integrity and bounded PNG payload/record reads.
- [x] Recover from truncated/corrupt records and unavailable cache storage.
- [x] Remove abandoned atomic writes while preserving another active writer.
- [ ] Complete ordered checks and two review cycles.

## Design

The application owns cache-aside policy through ArtworkCache, while
NodeArtworkCache owns storage, serialization, integrity and eviction. Records
live in artwork-cache beside the catalogue, consistently with the existing
Dropbox audio cache. They contain only a bounded PNG data URI or known absence,
plus a digest. Tokens and source paths are not stored in records or filenames.
The existing memory/in-flight cache and serial extraction remain in place.

Cached covers are retained catalogue metadata, so offline display does not need
an active cloud token. A cache miss still uses the existing account-bound access
checks before any Dropbox request. Removing a source makes its tracks fail the
catalogue lookup before cache access; old disk entries are disposable and bounded.

## Established patterns

Studied Strawberry/Clementine's disk-backed thumbnail/network cache pattern:
https://github.com/strawberrymusicplayer/strawberry/blob/master/src/core/threadsafenetworkdiskcache.cpp
That project is GPLv3-or-later. No code is copied; this implementation uses Node's
built-in filesystem and crypto APIs. The Freedesktop Thumbnail Managing Standard
informs hashed identities, modification/size validation and private thumbnail
storage: https://specifications.freedesktop.org/thumbnail/latest-single/
This is an application-private cache rather than the shared Freedesktop format.

## Verification

Red-first restart tests use real MP3/FLAC fixtures and real IPC composition, then
remove the source audio and create a new container. The result must be identical
without invoking the decoder again. Other tests cover offline Dropbox hits,
persisted revision invalidation, bounded LRU eviction, corrupt bytes, oversized
payloads/reads, private storage, concurrent writes, storage failures and queue
recovery. Cache failures preserve normal artwork reads and never block playback.

Initial Spec review found abandoned temporary files outside the budget. A
red-first regression now covers legacy, current-process orphan and exited-process
files. A real child-process test verifies that active foreign writes are preserved
and cleaned after the writer exits. New temporary names include their owner PID.
