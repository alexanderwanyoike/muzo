# Muzo

A desktop music player built with [Tauri](https://tauri.app), React and TypeScript.

Muzo treats your music collection as a set of **libraries** and **playlists**. A library is a source of truth for tracks - either a folder on your local filesystem or a Dropbox location. Libraries are scanned recursively and kept in sync with their source as files are added, changed, or removed.

## Status

Pre-alpha, but no longer just scaffolded. The local desktop player slice is
usable: filesystem libraries can be added, scanned, kept in sync, browsed, and
played with metadata editing and play counts. Sprint card status is tracked in
[`docs/cards/`](./docs/cards); the next architectural step is the Electron
migration card under [`docs/cards/migration/`](./docs/cards/migration/).

## Repository layout

```
muzo/
├── apps/
│   └── desktop/         # Tauri + React desktop app (@muzo/desktop)
│       ├── src/         # React frontend
│       └── src-tauri/   # Rust backend (single crate, modular inside)
├── packages/            # Shared TS packages (future)
├── docs/
│   ├── cards/           # Sprint cards: docs/cards/<sprint>/<card-title>.md
│   └── adr/             # Architecture Decision Records
├── AGENTS.md            # How agents work in this repo
└── package.json         # Yarn workspace root
```

A React Native / PWA mobile app will live under `apps/mobile` in a future sprint. The workspace is structured now so that addition is friction-free.

## Requirements

- Node `>= 20.10`
- Yarn `>= 1.22.22`
- Rust (stable)
- Tauri 2.x [system dependencies](https://v2.tauri.app/start/prerequisites/)

## Quick start

```bash
yarn install                 # install all workspace deps
yarn desktop:dev             # run the desktop app in dev mode
```

## Common scripts

| Script | Description |
| --- | --- |
| `yarn desktop:dev` | Run the desktop Tauri app with HMR |
| `yarn desktop:build` | Build the desktop frontend (Vite) |
| `yarn desktop:tauri build` | Build a distributable desktop bundle |
| `yarn lint` | Lint every workspace |
| `yarn typecheck` | Typecheck every workspace |
| `yarn test` | Run tests in every workspace |

## Engineering principles

These are house rules. See [`AGENTS.md`](./AGENTS.md) for the full version.

- **TDD is a requirement.** Red, green, refactor. No production code without a failing test first.
- **Domain-Driven Design.** The domain model is at the centre; framework code points inward at it, never the reverse.
- **Clean Architecture.** Dependencies point inward: UI -> application -> domain. Domain depends on nothing.
- **KISS, GRASP, SOLID.** Boring, readable code wins.

## GitOps flow

```
main        <- stable, tagged releases. Always buildable.
  ^
  |
dev         <- integration branch for the next release.
  ^
  |
feature/*   <- one branch per card. PRs target dev.
```

- Branch from `dev`.
- Open PRs against `dev`.
- `dev` merges into `main` via release PRs.
- See [`AGENTS.md`](./AGENTS.md) for branch and PR naming.

## License

UNLICENSED (proprietary).
