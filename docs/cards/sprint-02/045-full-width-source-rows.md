- Sprint: 02
- Type: UI fix
- Status: In review

# Card 045 - Fill the source panel with its rows

The source list reserves an unused 15px scrollbar gutter even when its contents
fit, leaving source selection and removal controls short of the panel edge.
The user requested full-width entries. Use the native overflow scrollbar when
needed, without reserving empty space in either Sources or Playlists.

## Acceptance criteria

- [x] Source rows extend to the panel's inner right edge when no scrollbar is needed.
- [x] Source names still truncate, and counts/removal controls remain inside the row.
- [x] Shared playlist list styling receives the same gutter correction.
- [x] Search, selection and removal behavior remain unchanged.
- [ ] Complete ordered checks and two review cycles; open a feature PR to dev.

The existing native Electron Settings fixture measures actual source row edges
at desktop/minimum viewport sizes across light/dark materials and connected/
disconnected states. It failed with the intended 15px gap before the CSS fix
and passes after it. Existing LibraryList interaction tests also pass.
The pattern uses standard native CSS overflow behavior, not a custom scroller:
https://developer.mozilla.org/en-US/docs/Web/CSS/scrollbar-gutter
