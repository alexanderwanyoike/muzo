# Muzo

A desktop music player built with [Electron](https://www.electronjs.org/), React and TypeScript.

Muzo treats your music collection as a set of **libraries** and **playlists**. A library is a source of truth for tracks - either a folder on your local filesystem or a Dropbox location. Libraries are scanned recursively and kept in sync with their source as files are added, changed, or removed.

## Status

Pre-alpha, but no longer just scaffolded. The local desktop player slice is
usable: filesystem libraries can be added, scanned, kept in sync, browsed, and
played with metadata editing and play counts. Dropbox account connection, library
sync and playback, and playlist queues with auto-advance are also implemented. Sprint card status is tracked in
[`docs/cards/`](./docs/cards). The desktop shell and backend now run on Electron
and TypeScript.

## Repository layout

```
muzo/
├── apps/
│   └── desktop/         # Electron + React desktop app (@muzo/desktop)
│       ├── src/         # React frontend
│       └── electron/    # Electron main process backend
├── packages/            # Shared TS packages (future)
├── docs/
│   ├── cards/           # Sprint cards: docs/cards/<sprint>/<card-title>.md
│   └── adr/             # Architecture Decision Records
├── AGENTS.md            # How agents work in this repo
└── package.json         # Yarn workspace root
```

A React Native / PWA mobile app will live under `apps/mobile` in a future sprint. The workspace is structured now so that addition is friction-free.

## Requirements

- Node `>= 22.12`
- Yarn `>= 1.22.22`
- Electron runtime dependencies for your OS

## Quick start

```bash
yarn install                 # install all workspace deps
yarn desktop:dev             # run the desktop app in dev mode
```

## Dropbox setup

Create a scoped app in the [Dropbox App Console](https://www.dropbox.com/developers/apps)
with Full Dropbox access and the `files.metadata.read` and `files.content.read`
permissions. In Muzo Settings, enter its public app key, choose Connect Dropbox,
approve access in your system browser, then paste the displayed authorization code.
No client secret or redirect URI is needed. Enable your OS keyring for encrypted
credential storage. Muzo refuses Linux basic_text storage.

Choose Browse Dropbox folders, navigate with the breadcrumbs, and select one or
several folders. Selections stay with you as you browse. Add the selected folders
to create libraries using their folder names; subfolders are included automatically.
Already-added folders are marked, and failed additions remain selected for retry.
Choose Entire Dropbox if you want the account root. Each library syncs on startup
and at the interval set in Settings. Refresh under Indexed sources runs a
sync immediately. Each library stays bound to the account that created it.

Dropbox tracks download when selected, then use the normal playback controls.
Downloads require an internet connection, and retained audio is capped at 1 GiB.
Disconnect removes local account credentials. Access can also be revoked through
Dropbox's connected-app settings. Live Dropbox access requires your own app key
and consent; automated tests exercise the API boundary with fixtures.

## Playlist playback

Use Play playlist to start its available entries, or play an entry to start there.
Previous and next controls move within the queue. Playback stops at its end.
Repeated tracks remain separate entries; unavailable entries are skipped when the
queue starts. Selecting a song directly replaces the queue. Edits to a playlist
affect the next queue started, leaving the current listening order intact.

## Desktop appearance and artwork

Settings includes Studio, Brushed aluminium, Warm paper, Carbon fibre, Walnut,
Porcelain and Frosted glass materials, each in light and dark. Ten accents include Cobalt,
Copper, Sage, Plum, Amber, Teal, Rose, Slate, Violet and Ice. Preferences
are saved locally and applied when Muzo starts. The listening workspace keeps
its transport and source sidebar in place while Songs, playlists and Settings
scroll inside the window.

Embedded album covers appear in Songs, playlists, the transport and the larger
current-track sleeve. Local MP3/FLAC covers are covered by real parser tests.
Dropbox artwork uses private, bounded range reads of metadata without downloading
the whole song. Missing or unsupported embedded art uses a disc placeholder.
Directory images and online cover searches are not included. Refresh a source
in Settings after changing its embedded covers.

Sources have removal controls beside their sidebar entries and in Settings.
Confirmation explains that source files and Dropbox contents stay in place.
Removing a source clears its Muzo tracks, metadata overrides and play counts;
playlist entries remain visibly unavailable. Storage failures leave the source
and watcher in place. If catalogue removal succeeds but watcher cleanup fails,
Muzo reports a restart warning.

On Linux, the native artwork test also needs Xvfb and Electron's GTK/audio
runtime libraries. Desktop verification CI installs those dependencies.
The [desktop smoke-test guide](./docs/testing/desktop-ui-smoke-test.md) covers
manual testing of the integrated feature stack.

## Common scripts

| Script | Description |
| --- | --- |
| `yarn desktop:dev` | Run the Electron desktop app with HMR |
| `yarn desktop:build` | Build the desktop frontend for Electron |
| `yarn desktop:package` | Build an unpacked Electron app bundle |
| `yarn desktop:make` | Build Electron distributable artifacts |
| `yarn format` | Automatically format maintained code and configuration |
| `yarn format:check` | Check formatting without writing |
| `yarn lint` | Run the Airbnb TypeScript and React lint baseline |
| `yarn typecheck` | Typecheck every workspace |
| `yarn test` | Run tests in every workspace |

Formatting is checked in CI before lint, type checking, tests and desktop packaging.
Run `yarn format` before pushing. Lint rejects explicit any and unsafe operations in
production TypeScript; test fixtures have narrowly scoped exceptions.

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
- Open feature PRs against `dev` and use merge commits to preserve their history.
- `dev` merges into `main` via release PRs, also using merge commits.
- Squash and rebase merging are disabled in GitHub.
- Tag the merged `main` release commit with an annotated `vX.Y.Z` tag.
- See [`AGENTS.md`](./AGENTS.md) for branch and PR naming.

## License

UNLICENSED (proprietary).
