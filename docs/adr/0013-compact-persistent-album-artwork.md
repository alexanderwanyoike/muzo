# 0013: Compact persistent album artwork

## Status

Accepted. Updates the thumbnail format in [ADR 0007](0007-persistent-album-artwork.md).

## Context

A large library can exceed the artwork cache budget before all of its covers
have been viewed. Detailed opaque thumbnails can take several hundred kilobytes
each as PNG, which means revisiting a cover can require another Dropbox read.

## Decision

Encode opaque persistent thumbnails as JPEG at quality 84. Use PNG where
transparency is needed, and continue reading existing PNG records. Keep the
384-pixel limit, integrity digest, revision key and 128 MiB / 4,096-entry cache
budget. Direct playback thumbnails use lossless encoding.

## Consequences

More opaque covers fit in the existing disk budget. Reopening a cover from the
persistent cache still works without Dropbox credentials. Existing cache entries
remain usable and are evicted under the same least-recently-used policy.
