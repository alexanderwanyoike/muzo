- Sprint: 01
- Status: Done
- Blocks: 003, 004

# Card 002 - Add a filesystem library

## What to build

The first real feature: a user can register a local folder as a Muzo **library**
and have it persisted, so that on the next launch the library is still there.

This is the first vertical slice through every layer:

- **Domain**: `Library`, `LibraryId`, `LibraryName`, `LibraryKind` (`Filesystem` only, for now), `LibraryLocation`. Value objects over primitives - no `String` for an id, no `String` for a path.
- **Application**: an `AddLibrary` use case (command) that takes the inputs, constructs the `Library`, and persists it through a `LibraryRepository` port.
- **Infrastructure**: a `LibraryRepository` trait in the domain; a SQLite-backed implementation in infrastructure. Schema migration included.
- **Commands**: a `add_library` Tauri command that the frontend can `invoke`.
- **UI**: a minimal form (path input + "Add library" button) wired through `invoke`. No styling polish yet - that is a later card.

Sync, recursive scanning, and listing libraries are out of scope for this card.
This card only persists the library record.

## Ubiquitous language

- **Library**: a named source of tracks. Aggregate root.
- **LibraryId**: stable identifier for a library. Survives renames.
- **LibraryName**: human-friendly, mutable.
- **LibraryKind**: `Filesystem` or `Dropbox`. Only `Filesystem` is implemented this sprint.
- **LibraryLocation**: where the library lives on its source. For `Filesystem`, an absolute path.

## Acceptance criteria

- [x] `Library`, `LibraryId`, `LibraryName`, `LibraryKind`, `LibraryLocation` live in `domain/library.rs` with no framework imports. Each has unit tests for construction, equality, and display.
- [x] `LibraryRepository` is a trait in `domain` with at least `add(&self, library: &Library) -> Result<...>` and `find_by_id`.
- [x] `AddLibrary` use case lives in `application` and is unit-tested with a fake repository (no real DB in unit tests).
- [x] SQLite `LibraryRepository` implementation lives in `infrastructure`. Uses sqlx or rusqlite. Schema is created on first run via a migration or `CREATE TABLE IF NOT EXISTS`.
- [x] DB file location is platform-appropriate (use Tauri's `app_data_dir`).
- [x] `add_library` Tauri command in `commands` validates input, calls the use case, returns a DTO (not the domain entity).
- [x] DTO types live in `commands` or a sibling boundary module - never on the domain entity itself. No `#[derive(Serialize)]` on `Library`.
- [x] React UI has a form to add a library and shows success/error. Path is picked via Tauri's `dialog` plugin, not typed by hand, where reasonable.
- [x] Restarting the app and querying the DB returns the library that was added. (Manual check; automated in 003.)
- [x] Fakes, not mocks, for repository tests.
- [x] Verification covered by the current test suites.

## Blocked by

- 001 - Scaffold desktop shell.

## Notes for the implementer

- Start TDD at the domain layer. Define `Library` first, then `LibraryRepository`,
  then the SQLite impl, then the command, then the UI. Each layer is tested
  before the next outer layer starts.
- The SQLite impl is an integration test, not a unit test. Use a temp file or
  in-memory SQLite and clean up after.
- The `LibraryKind::Dropbox` variant exists in the enum now but has no adapter.
  005 / sprint 02 will add the Dropbox adapter.
- Pick a stable on-disk ID format now (e.g. ULID or UUIDv7) so libraries
  survive renames of the human-friendly name.
