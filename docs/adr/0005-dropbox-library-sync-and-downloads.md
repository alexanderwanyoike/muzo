# ADR 0005 - Dropbox library sync and playback

- Status: Accepted
- Date: 2026-10-24

## Context

A desktop application cannot reliably receive Dropbox webhooks. Local playback
already uses a private loopback HTTP adapter for byte-range audio requests.
Dropbox libraries need background reconciliation and a local playback source.

## Decision

Poll Dropbox libraries at startup and on a persisted interval, five minutes by
default. Schedule the next poll only after the previous one completes. Share scans
between manual refresh and background polling, and expose sanitized failures in
Settings. Bind libraries to their creating Dropbox account. Refresh credentials
inside the main process and invalidate refreshes on disconnect or account changes.

Download selected audio to a private staged file, then rename it into local storage.
Fetch on each track start to avoid stale remote content. Cap retained audio at 1 GiB
and reject individual downloads above that limit. Serve completed files using Muzo's
existing loopback audio server. Never send Dropbox credentials to the renderer.

## Consequences

Remote changes appear on the next poll, with manual refresh available in Settings.
Playback waits for the selected file to download and supports seeking afterwards.
This provides online playback, without promising offline availability. Catalog
metadata continues to use filename fallback. Embedded metadata enrichment and
incremental cursor-based sync remain future improvements.

## References

- https://www.dropbox.com/developers/documentation/http/documentation
- https://github.com/electron/get
