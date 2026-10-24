- Sprint: 02
- Type: Feature
- Status: Done
- Blocks: -

# Card 020 - Choose Dropbox folders visually

Replace manual Dropbox path and library-name entry with a folder browser. Users
can navigate their connected account, select one or several folders, and add them
without knowing paths. Each chosen folder becomes a library named after the folder.

## Acceptance criteria

- [x] Browse root and nested folders with breadcrumbs and loading/error/retry states.
- [x] Retrieve direct child folders across Dropbox pagination, without exposing tokens.
- [x] Select folders across locations, review/remove selections and add them together.
- [x] Include subfolders through the existing recursive library sync.
- [x] Show already-added folders for the connected account and prevent accidental repeats.
- [x] Preserve failed selections and report partial success without losing successful additions.
- [x] Bind browsing and addition to the same connected account identity.
- [x] Stay in Settings while adding folders and update the source sidebar immediately.
- [x] Style connection, browsing, sync and disconnect controls consistently and responsively.
- [x] All repository checks pass (242 tests, Linux zip and deb).
- [x] Both review cycles pass, with no remaining findings.

## Established patterns

Use Dropbox's non-recursive list_folder operation and list_folder/continue cursor,
as demonstrated by the official MIT-licensed JavaScript SDK's basic example:
https://github.com/dropbox/dropbox-sdk-js/blob/main/examples/javascript/basic/index.html
Endpoint contract: https://docs.dropboxapi.com/dropbox-api/api-reference/user-endpoints/files/list-folder
No external implementation is copied. The existing authenticated main-process HTTP
adapter keeps credentials out of the renderer. Reuse the existing folder-per-library
model, account bindings and recursive scanner.

The picker follows a conventional breadcrumb browser with independent checkboxes,
so navigating does not lose selections. Keep Muzo's quiet cream/charcoal palette,
clear spacing and restrained accent. No new UI framework or Dropbox SDK is needed.

## Verification

Use separate red/green commits for each behavior, with a real SQLite/IPC integration
tracer and user-facing component tests. Check the rendered UI at desktop and narrow
widths, including multi-selection and failure recovery. Live consent is already
confirmed by the user; test fixtures must not alter their Dropbox account.

Visual verification used the rendered app with a controlled runtime fixture. Navigation,
cross-folder selection, successful multi-add, source-sidebar updates and the 800px
minimum desktop width were checked without changing the user's Dropbox contents.

Review evidence: [folder picker review record](../../reviews/2026-10-24-dropbox-folder-picker.md).
