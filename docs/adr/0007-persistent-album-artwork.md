# ADR 0007 - Persistent album artwork

- Status: Accepted
- Date: 2026-10-24

## Context

Process-local thumbnail caching loses all covers on exit. Repeated launches then
read embedded metadata again or request Dropbox media ranges, even for artwork
already viewed. Covers should behave like the other retained catalogue metadata.

## Decision

Add an application ArtworkCache port with an app-private filesystem adapter.
Keys include the stored library, track, media path and size/mtime revision.
Resolve that stored source before using cached data. Keep only bounded PNG
thumbnails or known absence, protected with a content digest and atomic writes.
Use a 128 MiB / 4,096-entry LRU budget and keep cache failures non-fatal.

Read retained thumbnails before cloud credentials. Cache misses still require
account-bound Dropbox access. Cached covers remain available offline alongside
titles/artists in the local catalogue. Removing a source prevents further lookup
of its cached tracks; bounded orphan entries can be evicted normally.

## Consequences

First viewing a cover populates the persistent cache. Later launches reuse it,
including without the original audio being accessible. File revision changes
produce a new cache identity. Existing process-local deduplication continues to
avoid repeated extraction in a session. Cache contents are disposable, private
and independent of credential storage.
