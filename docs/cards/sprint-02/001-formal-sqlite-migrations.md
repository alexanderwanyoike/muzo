- Sprint: 02
- Type: Tech debt
- Status: Pending
- Blocks: 002, 003, 004 (sprint 02 feature cards add new tables)

# Card 001 - Formal SQLite migrations

## What to build

Replace the inline `CREATE TABLE IF NOT EXISTS` strings inside
`SqliteLibraryRepository::migrate()` and `SqliteTrackRepository::migrate()`
with a real migration framework. Migrations should be first-class, versioned,
discoverable, and run centrally - not scattered across repository impls.

Pull this forward before sprint 02 feature work, because every new table
(playlists, play counts, dropbox cursor, sync state) makes the current
approach harder to refactor out.

## Problem with the current approach

Card 002 introduced `SqliteLibraryRepository::migrate(&Connection)` with an
inline SQL string. Card 004 copied the pattern for `SqliteTrackRepository`.
The result is:

- **No version tracking.** Every startup runs every `CREATE TABLE IF NOT
  EXISTS` again. Cheap because of `IF NOT EXISTS`, but doesn't scale to
  schema changes (adding a column, renaming, etc.).
- **No history table.** We cannot tell which migrations have been applied.
- **No rollback.** If a bad migration ships, there is no down path.
- **No ordering.** Two `migrate()` calls happen to work because they touch
  different tables; nothing enforces order when migrations start touching
  shared state.
- **Schema knowledge leaks across modules.** The library repository knows
  about the libraries table; the track repository knows about the tracks
  table; the app entry point has to remember to call both. Cross-table
  migrations have no home.
- **Pain compounds with every new table.** Card 005 (sync state), sprint 02
  (playlists, dropbox cursor, play counts) will all add tables. Each one
  would copy the pattern.

## Proposed solution

Adopt the [`rusqlite_migration`](https://crates.io/crates/rusqlite_migration)
crate. Reasons:

- Lightweight; works against an existing `rusqlite::Connection`.
- Maintains a `migration_status` tracking table automatically.
- Migrations are first-class `Migration` structs with explicit names.
- An ordered `Migrations::new(vec![...])` makes ordering explicit.
- No runtime SQL parser, no async runtime, no extra daemon.

Alternatives considered and rejected:

- **`refinery`** - more features (multiple backends, embed macros) but
  heavier than we need; we are committed to SQLite for the desktop app.
- **`sqlx::migrate!`** - would force a switch from `rusqlite` to `sqlx`,
  which is async and a much bigger refactor.
- **Hand-rolled** - a `schema_migrations` table plus hand-rolled version
  numbers. Works, but reinvents what `rusqlite_migration` already does
  correctly.

## Layout

```
apps/desktop/src-tauri/
├── Cargo.toml                       (add rusqlite_migration dep)
└── src/
    └── infrastructure/
        └── migrations.rs            (new module; owns the Migrations struct)
            - MIGRATIONS: lazy_static Migrations::new(vec![
                Migration::new("2026-06-15-create-libraries"),
                Migration::new("2026-06-15-create-tracks"),
                ...future migrations appended here...
              ])
            - pub fn current_schema_sql() -> &str  (handy for tests/docs)
```

`lib.rs` calls `MIGRATIONS.to_latest(&mut conn)?` once on startup. The two
repository impls lose their `migrate()` methods.

## Acceptance criteria

- [ ] `rusqlite_migration` added to `Cargo.toml`.
- [ ] New module `infrastructure::migrations` owns an ordered `Migrations`
      struct with one `Migration` per currently-existing table:
      - `2026-06-15-create-libraries`
      - `2026-06-15-create-tracks`
- [ ] Each migration's SQL is identical to the current `CREATE TABLE IF NOT
      EXISTS` body, so an existing dev/prod database with data is migrated
      cleanly to the tracked state.
- [ ] `lib.rs` runs `MIGRATIONS.to_latest(&mut conn)` once on startup; the
      per-repository `migrate()` calls are removed.
- [ ] `SqliteLibraryRepository::migrate` and `SqliteTrackRepository::migrate`
      are deleted. Their construction (`new(conn)`) no longer assumes
      migrations have been run by the caller - `lib.rs` is the single place
      that owns migrations.
- [ ] A `schema_migrations` (or `migration_status` - whatever the crate
      uses) table exists after the first migration run.
- [ ] Re-running `to_latest` against an already-migrated database is a
      no-op (integration test).
- [ ] A fresh database reaches the same schema as an existing one
      (integration test: open in-memory, migrate, assert both tables exist
      and match expected column lists).
- [ ] Existing repository integration tests still pass - no behaviour
      change from the user's perspective.
- [ ] A `DELETEME` comment is **not** left behind. The migration history
      starts clean from this card; there is no "v0 baseline" hack.
- [ ] The `migrations` module has a doc comment explaining how to add a
      future migration (one-line recipe).
- [ ] All verification commands green.

## Blocked by

None. Should land before sprint 02 card 002 (Dropbox OAuth) and ideally
before sprint 01 card 005 (sync), since 005 will want a `sync_state` table
and that is the natural first new migration.

## Notes for the implementer

- **Do not change table schemas in this card.** The first two migrations
  must reproduce the current `libraries` and `tracks` tables verbatim.
  Schema changes (e.g. dropping `duration_seconds` in favour of
  `duration_ms`) belong in their own follow-up migration cards.
- **Migration naming.** Use `YYYY-MM-DD-<short-slug>` for ordering and
  readability. The crate also accepts a numeric version; date prefixes are
  more informative for humans reading `schema_migrations`.
- **Testing applied state.** The rusqlite_migration crate exposes
  `Migrations::current_version()`; assert against it in the integration
  test.
- **The current schema is small enough** that you can hand-write the two
  `Migration::new(...)` entries from the existing `migrate()` SQL. No need
  for a state-reconstruction sub-card.
- **Consider documenting this as ADR 0002** ("SQLite migrations via
  rusqlite_migration"). Not strictly required, but the choice between
  rusqlite_migration / refinery / sqlx is exactly the kind of decision
  worth recording.

## Out of scope for this card

- Schema changes to existing tables.
- Data backfills.
- Migration-driven seed data.
- Adding the `sync_state` table itself (that lands with card 005).
- Splitting migrations into per-table files. The single `migrations.rs`
  module is fine until it isn't.
