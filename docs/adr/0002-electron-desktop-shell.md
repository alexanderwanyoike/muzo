# ADR 0002 - Electron Desktop Shell

## Status

Accepted.

Supersedes the desktop-shell choice in [ADR 0001](./0001-monorepo-and-stack.md).

## Context

Muzo started as a Tauri app with a Rust backend. The current product behavior is
filesystem scanning, SQLite persistence, metadata reading, Dropbox HTTP,
filesystem watching, and local audio streaming. These responsibilities now live
cleanly in the Electron main process with TypeScript application and
infrastructure modules.

Keeping Tauri and Rust as the default shell would keep two implementation stacks
for one desktop product and slow feature work.

## Decision

Muzo's desktop shell is Electron. The React frontend talks to the desktop shell
through the local runtime bridge, which is injected by Electron preload code.
React must not import Electron or Tauri APIs directly.

The legacy `apps/desktop/src-tauri` crate remains only until the final deletion
PR. New desktop backend work belongs under `apps/desktop/electron`.

## Consequences

- `yarn desktop:dev` starts Electron.
- Tauri JavaScript packages are no longer frontend dependencies.
- Rust/Tauri verification remains temporarily while `src-tauri` exists, then
  should be removed with the crate.
- Electron distributable packaging is the next release-engineering step.
