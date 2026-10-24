- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 015 - Use Dropbox libraries

After connecting an account, add a Dropbox folder, keep its track catalog in sync,
and play its tracks through the existing player and playlists.

## Acceptance criteria

- [x] Settings registers Dropbox folders and supports the root folder.
- [x] Libraries are bound to the connected account that created them.
- [x] Main-process credentials refresh automatically, including concurrent callers.
- [x] Refresh cannot restore an account after disconnect or account replacement.
- [x] Automatic sync starts with the app and retries after failures.
- [x] The polling interval is configurable and persists across restarts.
- [x] Manual refresh and automatic polling share in-flight library scans.
- [x] Visible Dropbox track lists refresh without reopening the library.
- [x] Playback downloads binary audio to private files without exposing tokens.
- [x] Downloads publish atomically and local storage is capped at 1 GiB.
- [x] Download failures and evicted local audio sources recover cleanly.
- [x] Opus files accepted by scanning can be played.
- [x] All repository verification commands pass (179 tests; Linux zip and deb).

## Design and patterns

Extend the existing Dropbox recursive-listing adapter and PKCE refresh client.
Use a completion-based polling loop, rather than setInterval, so slow syncs do not
stack. Poll every five minutes by default, configurable from one to sixty minutes.

Use staged file writes followed by rename for complete audio downloads, then serve
through the existing loopback audio adapter with range support. Downloads are fetched
on demand on each new track start, so remote edits are not hidden by a stale cache.
Only the most recent files fitting the storage limit remain. Offline playback is not
promised. Audio metadata from Dropbox uses the existing filename fallback.

Dropbox's official API guidance informs authorization, paginated listing, and content
downloads. The staged-file pattern is also used by the installed @electron/get
implementation (MIT); no external implementation code is copied.

## Dependencies and verification

Depends on card 014 and PR #41. Merge the existing integration PR #16, then #41,
then this PR. The integration test connects an account, registers a library, lists
remote files into real SQLite, downloads audio, and reads it through the real local
HTTP audio server. Network and OS credential storage are replaced at their boundaries.
Live Dropbox consent and playback require a scoped app key and an authorized account.
