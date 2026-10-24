- Sprint: 01
- Status: Done (this PR)
- Blocks: 002

# Card 001 - Scaffold desktop shell

## What to build

Get the project from empty directory to a runnable, tested, linted, typechecked
monorepo with one Tauri + React + TypeScript app inside it. No domain features.
This card is the foundation every later card stands on.

The Rust backend uses a single crate with module-level Clean Architecture
split (`domain`, `application`, `infrastructure`, `commands`). The split is
visible now so PRs that add domain logic have an obvious home.

A smoke test ("ping") cuts through every Rust layer and one React test proves
the frontend test harness works.

## Acceptance criteria

- [ ] Yarn workspace root at `package.json` with `apps/*` and `packages/*` workspaces.
- [ ] `apps/desktop` is a Tauri 2 + React 18 + TypeScript app (`@muzo/desktop`).
- [ ] Rust backend is a single crate at `apps/desktop/src-tauri` with `domain`, `application`, `infrastructure`, `commands` modules and a `lib.rs` + `main.rs` split.
- [ ] Domain module is empty of entities (those land in 002); its doc comment states the layer rules.
- [ ] A `ping` smoke test cuts through `commands -> application` and is covered by a Rust unit test.
- [ ] React frontend renders a heading; that heading is covered by a Vitest test using Testing Library + jsdom.
- [ ] Tooling configured: ESLint (flat config), TypeScript strict, Prettier not required (editorconfig + rustfmt instead), Vitest, cargo fmt + clippy.
- [ ] `README.md` describes the project and quickstart.
- [ ] `AGENTS.md` captures the house rules (TDD, DDD, Clean Architecture, KISS, GRASP, SOLID), the GitOps flow, the toolchain, and the "before you push" verification commands.
- [ ] `docs/cards/sprint-01/` and `docs/adr/` exist; ADR 0001 records the stack decision.
- [ ] App icon generated via `cargo tauri icon` so the bundle step does not fail on missing icons later.
- [ ] `main` and `dev` branches exist; this PR targets `dev`.

## Blocked by

None - this is the start of the project.

## Verification

All of these must be green on `dev` before the card closes:

```bash
yarn install
yarn lint
yarn typecheck
yarn test
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --all
yarn desktop:build
```

## Notes

- The brotli 8.0.3 / alloc-no-stdlib conflict is recorded in ADR 0001 with the
  workaround in `apps/desktop/src-tauri/Cargo.toml`. Remove the workaround
  when a fixed brotli is published.
