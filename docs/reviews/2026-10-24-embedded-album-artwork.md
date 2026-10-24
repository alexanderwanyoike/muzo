# Embedded artwork review record

Card 023 depends on PR #49. Both cycles use base
c88d89ccff37fe89c88764775e67472bd305b34a. Initial review head:
6f73bc8dff9fc1a991820cd33b24d4c1b72265d8. Fix re-review head:
462abd1a5fa2d765e4231db80a0beb476efa06ef. Later documentation records
verification and manual testing instructions without changing behavior.

## Standards

Both cycles found no documented-standard violations or actionable Fowler smells.
Artwork sources and reader ports are defined inward, expected failures use typed
Results, and main-process account credentials/content URLs do not enter renderer
responses. Existing parser/tokenizer/native-image patterns are documented with
redistribution notices. Tests, implementations, refactors and formatting have
focused commits. The native decoder fixture is kept separate and readable.

## Spec

Initial review found one P2: manual source refresh updated song-row covers but
did not reload the persistent sleeve/transport covers. A public App regression
now plays a track, refreshes its source in Settings, and verifies both images
update. The source refresh version is passed to both persistent views. Fix
re-review found that issue resolved with no further actionable findings.

## Verification

All ordered repository gates passed on the reviewed code: installation,
formatting, lint, strict types, 272 tests across 64 files, desktop build, Linux
package, zip and deb. Real SQLite/IPC tests parse original MP3/FLAC fixtures;
real Electron nativeImage tests under Xvfb verify decoding, scaling, invalid/SVG
rejection and a smaller fallback for highly detailed covers. The range reader
retrieves a cover from an 8 MiB audio fixture using less than 128 KiB of data.

The rendered desktop was checked with original, controlled artwork/runtime
fixtures. Covers appear in song rows, playlists, transport and the larger sleeve.
The 800 by 600 desktop minimum has no horizontal window overflow. Real user
music and Dropbox contents were not changed. Live-account file/network behavior
remains part of the manual smoke-test guide.

Remaining findings: Standards 0; Spec 0.
