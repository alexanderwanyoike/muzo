# AGENTS.md

Read this before doing anything in this repo. It applies to humans and to AI agents alike.

## Project at a glance

**Muzo** is an Electron-based desktop music player. Users organise music through **libraries** (filesystem folders or Dropbox locations) and **playlists**. Libraries are scanned recursively and kept in sync with their backing source. A React Native / PWA mobile client is planned for later; this repo is set up as a Yarn workspace to host both.

## House rules

These are not negotiable.

### 1. TDD is a requirement

- Write a failing test first. Watch it fail. Make it pass. Refactor.
- No production code without a failing test that justifies it.
- Tests live next to the code they exercise. TS tests as `*.test.ts(x)` next to the unit under test.
- Unit tests for domain logic. Integration tests for adapters (filesystem, Dropbox, Electron IPC commands). End-to-end tests only where the value justifies the cost.
- Tests are behaviour names, not implementation names: `it_adds_a_track_to_the_library`, not `test1`.

### 2. Domain-Driven Design

- The **domain layer** holds the core model: `Library`, `Track`, `Playlist`, `LibraryId`, etc. It has zero dependencies on Electron, Node, filesystem, HTTP, or wire DTOs.
- Frameworks point **inward** at the domain. The domain never imports them.
- Define **repositories** as interfaces in the inner layer; implement them in infrastructure.
- Value objects over primitives. A `LibraryPath` is not a `String`.
- Language matters - use the [Ubiquitous Language](./docs/cards) consistently. If a term is contested, add it to the glossary and stop arguing.

### 3. Clean Architecture

Layering, inner to outer:

```
domain        <- entities, value objects, domain services, repository interfaces
application   <- use cases (commands/queries), orchestrates domain + ports
infrastructure <- repository implementations, filesystem/Dropbox adapters, persistence
ui / electron <- React frontend + Electron IPC; the outermost edge
```

- Dependencies only point inward.
- Crossing a layer boundary uses an interface defined on the inner side.

### 4. KISS

- Pick the simplest design that satisfies the test.
- Resist speculative abstraction. If you cannot name two concrete places that need a generic thing, do not make it generic yet.

### 5. GRASP

- Apply **Information Expert**, **Creator**, **Controller**, **Pure Fabrication** when assigning responsibilities.
- If a class is doing too many things, it is violating **SRP** (below) - split it.

### 6. SOLID

- **SRP**: one reason to change per module.
- **OCP**: extend by adding, not by editing.
- **LSP**: subtypes are substitutable.
- **ISP**: depend only on the methods you actually call.
- **DIP**: depend on abstractions defined on the inner side.

## Repository layout

```
apps/
  desktop/                 @muzo/desktop  - Electron + React app
    src/                   React frontend (TS)
    electron/              Electron main process backend:
      application/         use cases, commands, interfaces
      infrastructure/      filesystem, Dropbox, SQLite, metadata, audio
      ipc/                 Electron IPC dispatch
      composition/         Awilix IoC container
packages/                  future shared TS packages
docs/
  cards/<sprint>/          sprint cards, one file per card
  adr/                     ADRs, numbered, immutable once merged
```

## GitOps flow

```
main  <-  dev  <-  feature/<card-slug>
```

- `main`: stable. Always green, always buildable, always tagged for release. Never commit to `main` directly.
- `dev`: integration branch. PRs land here. Trunk for the next release.
- `feature/<card-slug>`: one branch per card. Examples: `feature/library-domain-model`, `feature/fs-adapter`.

### Branch rules

- Branch from `dev`. Rebase onto `dev` before opening the PR.
- One card per PR. If the PR does two things, split the card.
- PR title: `<card-id>: <imperative summary>`, e.g. `001: scaffold desktop shell`.

### Commit messages

Conventional Commits, scoped to the workspace:

```
feat(desktop): scan filesystem library recursively
fix(desktop-ui): playlist row reorders by one
test(domain): library rejects duplicate track ids
docs(cards): add sprint-01 cards
chore(repo): init yarn workspaces
```

### Merging

- Squash-and-merge feature -> `dev`.
- Release PRs `dev` -> `main` are merge commits, with a release notes section.

## Toolchain

- **Package manager:** Yarn (classic) workspaces. Always. No npm, no pnpm, no bun.
- **Node:** `>= 20.10`.
- **Frontend:** React + TypeScript + Vite.
- **Desktop shell:** Electron.

### Before you push

Run, in this order, and they must all be green:

```bash
yarn install
yarn lint
yarn typecheck
yarn test
yarn desktop:build
yarn desktop:package
yarn desktop:make
```

If any of those fails, the card is not done.

## Cards and sprints

- One markdown card per unit of work under `docs/cards/<sprint>/<NNN>-<slug>.md`.
- Each card is a vertical slice through every layer it touches (domain, application, infrastructure, UI). Not a horizontal layer slice.
- Acceptance criteria are a checklist. A card is done when every box is ticked and the verification commands above are green.
- A PR is the card made real. The PR description links the card.

## ADRs

Numbered, immutable once merged. New decision? Write a new ADR superseding the old one. File them under `docs/adr/NNNN-<slug>.md`.

## Things this repo does NOT do

- No direct commits to `main` or `dev`.
- No production code without a failing test first.
- No framework or wire DTO types leaking into the domain; wrap or map at the boundary.
- No magic numbers - name them as value objects or constants.
- No emojis in source files or commit messages unless explicitly requested.
- No em dashes in any output. Use a regular hyphen or reword.

## When in doubt

Read the relevant card. Then the ADRs. Then ask in the PR. Do not guess - state the assumption you are making and let a reviewer catch it.
