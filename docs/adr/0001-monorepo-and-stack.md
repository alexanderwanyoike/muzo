- Status: Accepted
- Date: 2026-06-14

# ADR 0001: Monorepo layout and stack

## Context

Muzo is starting from nothing. It will be a Tauri desktop music player today,
with a React Native / PWA mobile client later. The codebase must support both
without rework, and must give the domain layer (libraries, tracks, playlists)
a stable home that does not depend on any framework.

House rules require: TDD on every change; Domain-Driven Design with frameworks
pointing inward at the domain; Clean Architecture layering; KISS, GRASP, SOLID.

## Decision

Adopt a **Yarn workspace monorepo** with the following layout:

```
apps/
  desktop/                 @muzo/desktop - Tauri + React + TypeScript
    src/                   React frontend
    src-tauri/             Rust backend, single crate, modules inside:
      src/
        domain/            entities, value objects, repository traits
        application/       use cases (commands/queries)
        infrastructure/    fs adapter, dropbox adapter, sqlite, persistence
        commands/          Tauri command handlers (the outermost edge)
packages/                  future shared TS packages
docs/
  cards/<sprint>/          sprint cards
  adr/                     this directory
```

Stack:

- **Package manager:** Yarn classic (1.22) workspaces. No npm/pnpm/bun.
- **Frontend:** React 18 + TypeScript + Vite. Vitest + Testing Library.
- **Desktop shell:** Tauri v2.
- **Backend:** Rust, stable. One crate now. Split into workspace crates only when a card justifies it (KISS).
- **Persistence:** SQLite, added when card 002 needs it.

## Rationale

- **Workspace over single app.** The mobile client is on the roadmap. A workspace now costs nothing and avoids a painful split later.
- **React over Solid/Svelte.** The mobile client is likely React Native. Sharing patterns and mental model across desktop and mobile outweighs framework novelty.
- **Yarn classic over Berry.** House rule already specifies Yarn; classic workspaces are simple and well-understood. KISS.
- **Single Rust crate with modules over multi-crate workspace.** Cards 002+ can split crates when there is a concrete reason. Premature splitting violates KISS.
- **Layered modules inside the crate.** Clean Architecture is a house rule. The `domain / application / infrastructure / commands` split makes the layering visible at a glance and keeps the domain free of framework imports.

## Consequences

- Adding the mobile app later is a new entry under `apps/`, not a restructure.
- Shared TS code (e.g. types for IPC DTOs) lives under `packages/` and is consumed by both apps.
- The Rust domain module must be reviewed on every PR for accidental framework leakage. `serde::Serialize` on a domain entity is a smell; mapping happens at the boundary.

## Alternatives considered

- **Multi-crate Rust workspace from day one.** Rejected - no card justifies it yet.
- **Tauri v1.** Rejected - v2 is stable and is the supported line going forward.
- **SolidJS / Svelte frontend.** Rejected - less overlap with a future React Native app.
- **Nx / Turbo on top of yarn workspaces.** Rejected at scaffold stage - KISS. Revisit when CI times or task graph complexity justifies it.

## Worked around

- `brotli 8.0.3` published on crates.io pins `alloc-no-stdlib` to 2.x while its
  sibling `brotli-decompressor 5.x` uses 3.x, producing ambiguous trait
  resolutions. Tauri's `tauri-utils` pulls brotli in for asset compression.
  Workaround in `apps/desktop/src-tauri/Cargo.toml`:
  - `[patch.crates-io] brotli = { git = "...master" }` (master widens the range).
  - Add `alloc-no-stdlib = "3"` as a direct dep so cargo unifies on 3.0.0.
  - Remove the patch once a fixed brotli is published.
