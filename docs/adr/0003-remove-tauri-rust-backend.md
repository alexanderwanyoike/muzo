# ADR 0003 - Remove the Tauri and Rust backend

## Status

Accepted

## Context

ADR 0002 moved the desktop shell to Electron while keeping the legacy
`apps/desktop/src-tauri` crate temporarily during parity work. The Electron
backend now covers local libraries, scanning, playback sources, startup
reconciliation, filesystem watching, metadata overrides, playlists, play
history, SQLite migrations, Dropbox backend foundations, and distributable
packaging.

Keeping the old backend after parity creates two implementations for the same
behavior and keeps Cargo verification in the normal development path.

## Decision

Remove `apps/desktop/src-tauri` and make Electron plus TypeScript the only
desktop product backend. Repository verification now uses Yarn, Vitest,
TypeScript, Electron packaging, and packaged-app smoke tests.

## Consequences

- Product code no longer depends on Rust or Tauri.
- Future desktop backend work happens under `apps/desktop/electron`.
- Historical sprint cards and ADRs may still describe the old Tauri design.
  They remain historical records and are superseded by ADR 0002 and this ADR.
- SQLite compatibility remains protected by the Electron migration tests.
