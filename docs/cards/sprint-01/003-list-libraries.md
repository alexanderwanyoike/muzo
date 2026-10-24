- Sprint: 01
- Status: Done
- Blocks: -

# Card 003 - List libraries in the UI

## What to build

A read-side counterpart to card 002: when the user opens the app, they see the
libraries they have already registered. No library is silently invisible.

Vertical slice:

- **Domain**: extend `LibraryRepository` with `list(&self) -> Result<Vec<Library>, ...>`.
- **Application**: a `ListLibraries` use case (query) that returns the libraries.
- **Infrastructure**: SQLite implementation of `list`.
- **Commands**: a `list_libraries` Tauri command returning a list of DTOs.
- **UI**: the home screen shows every persisted library. Each row shows the library name, kind, and location. Empty state is friendly ("No libraries yet").

This is a thin slice but it proves the read path works end-to-end and
exercises the persistence from card 002 across app restarts.

## Acceptance criteria

- [x] `LibraryRepository::list` is part of the trait, with a unit-tested fake in the application tests.
- [x] `ListLibraries` use case in `application` with a unit test using a fake repository.
- [x] SQLite implementation of `list` with an integration test that persists two libraries and reads them back.
- [x] `list_libraries` Tauri command returns `Vec<LibraryDto>`.
- [x] React UI renders the list on mount and handles the empty state.
- [x] After adding a library (card 002) and restarting the app, the library appears in the list without any extra action.
- [x] DTO shape is stable enough to drive a basic list UI; future cards (004, sprint 02) extend it.
- [x] Verification covered by the current test suites.

## Blocked by

- 002 - Add a filesystem library.

## Notes for the implementer

- ListLibraries is a query, not a command. Consider whether to distinguish
  queries from commands in the module structure now (CQRS-lite) or leave them
  together. Pick the simpler one (KISS) and revisit if a card forces it.
- The UI does not need to be pretty. A flat list is fine. Styling is a later card.
