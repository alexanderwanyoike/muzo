- Sprint: 01
- Status: Done
- Blocks: -

# Card 006 - Play tracks from a library

## What to build

Close the loop on sprint 01's "usable music player" promise. After this card,
a user can browse a library's tracks, click one, and hear it play through
their speakers. Play/pause, seek, and volume controls are included.

This card is intentionally backend-light. The audio element is a UI concern:
there is no domain or application change. The only backend work is enabling
Tauri's asset protocol so the webview can read local file paths. Play counts,
history, queueing, and playlists all belong to sprint 02 - this card exists
to make sprint 01 a complete vertical slice.

## What the user can do after this card

1. Open Muzo.
2. See their libraries with track counts (cards 002-004).
3. Click a library to expand / open it and see the list of tracks.
4. Click a track (or a per-row play button). It plays.
5. Pause, resume, scrub, change volume.
6. Click another track. Playback switches.

## Vertical slice

- **Domain**: no change. `Track` already has every field the UI needs.
- **Application**: no change. `list_tracks` from card 004 returns everything.
- **Infrastructure**: no change.
- **Commands**: no change. `list_tracks` already exists.
- **Tauri config**: enable the `asset:` protocol (or `tauri://`/`http://asset.localhost`)
  so the webview can load `file://` paths via `convertFileSrc`.
- **UI**:
  - `LibraryList` rows expand to show that library's tracks (fetched lazily via `list_tracks`).
  - Clicking a track loads it into a single shared `<audio>` element and plays it.
  - A now-playing bar (fixed at the bottom of the window) shows the current
    track's title + artist, with play/pause, seek, and volume controls.
  - The currently-playing track is visually marked in its track list.

## Acceptance criteria

- [x] Selecting a library shows its tracks (title, artist, duration) in a list.
- [x] Tracks are loaded lazily (only fetched when the library is expanded), not eagerly on app mount.
- [x] Clicking a track starts playback within a reasonable time (no perceptible delay beyond audio engine spin-up).
- [x] Play/pause button toggles playback; the button label reflects state.
- [x] Seek bar shows current position; user can scrub by clicking / dragging.
- [x] Volume control adjusts output and persists across tracks within a session.
- [x] Now-playing bar shows the active track's title and artist.
- [x] The currently-playing track is visually distinct in its list.
- [x] Clicking a different track switches playback to it (no overlap, no queue).
- [x] Reaching the end of a track leaves the UI in a sane state (paused, seek at end).
- [x] Tauri `asset:` protocol or equivalent configured; playback uses a local range-capable HTTP stream server.
- [x] File paths from `list_tracks` are resolved through `prepare_track_audio_source` before being passed to `<audio>`.
- [x] No framework types leak into the domain. (No change expected here; listed for completeness.)
- [x] Verification covered by the current test suites.

## Out of scope

- **Queue or playback list.** The user clicks tracks one at a time. Auto-advance to the next track is debatable: include it only if it is one line of effort; otherwise defer.
- **Playlists.** Sprint 02.
- **Play counts / history / "last played" timestamps.** Sprint 02; would require a domain event and a new table.
- **Gapless playback, crossfade, EQ, replay gain.** Not now.
- **Media key support** (play/pause on the keyboard's media key). Sprint 02+; nice-to-have.
- **Audio normalisation.** Not now.

## Blocked by

- 004 - Scan filesystem library recursively. The track list this card plays
  comes from card 004's scan output.

## Notes for the implementer

- **Use the HTML5 `<audio>` element**, not a Rust audio library. The webview
  already has one; a Rust-side library (rodio, etc.) would add a crate, a
  thread, and a control surface that has to be wired through Tauri commands
  for no benefit on a desktop app.
- **`convertFileSrc`** from `@tauri-apps/api/core` is the supported way to
  turn a filesystem path into something the webview can load. It picks the
  right protocol per platform (`asset://` on macOS/Linux, `https://asset.localhost/`
  on Windows).
- **Asset protocol capability.** Tauri 2 needs the asset protocol scope
  declared in `capabilities/default.json`. Scope it to the user's library
  roots; do not grant blanket filesystem access.
- **State ownership.** A single `useAudioPlayer` hook (or context) owns the
  `<audio>` element and exposes `play(track)`, `toggle()`, `seek(position)`,
  `setVolume(v)`. `App` renders the now-playing bar from this hook's state.
- **Lazy track loading.** `listTracks(libraryId)` is called when a library
  row is expanded, not on app mount. Caching the result in App state is
  fine; re-fetching on every expand is also acceptable for sprint 01.
- **Duration display.** Format seconds as `m:ss` (or `h:mm:ss` for long
  files). A small `formatDuration` helper next to `types.ts` is fine.
- **Switching tracks.** Setting `<audio>.src` and calling `.play()` is
  enough. No need to manage a queue abstraction yet; the hook just holds
  the current track and the element.
- **Test coverage.** The hook should be unit-tested with a mocked audio
  element (jsdom doesn't have a real audio element; stub `HTMLAudioElement`).
  Cover: `play(track)` sets the src and calls play; `toggle` flips state;
  `seek` updates the element's currentTime; `setVolume` updates volume.
  Skip end-to-end "actual sound came out" testing; that's manual.
- **Why no domain changes.** Adding a `PlayEvent` or `PlayCount` now would
  be speculative - we don't yet have a use case that consumes them. The
  YAGNI cost of adding them prematurely is higher than the cost of adding
  them later when a sprint 02 card actually needs them.

## Out of scope for this card (recap)

- Anything that requires a new domain entity, value object, or repository.
- Anything that requires a new SQL table.
- Anything that requires a new Tauri command.
- Anything that crosses more than one library (e.g. "all tracks" view).
