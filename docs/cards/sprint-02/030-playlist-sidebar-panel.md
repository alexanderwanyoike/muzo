- Sprint: 02
- Type: Feature
- Status: In review
- Depends on: 029 (PR #56)

# Card 030 - Consistent Playlists sidebar panel

The user asked for Playlists to use the same panel treatment as Sources. Use the
existing sidebar width/divider and material styling. Show one selected playlist's
tracks in the main workspace instead of rendering every playlist's tracks.

## Acceptance criteria

- [x] Frame Playlists with the shared header/count, search, scrolling rows and footer.
- [x] Select a playlist from its compact row and show only its tracks in the main area.
- [x] Filter names without losing the selected playlist; clear search with Escape/control.
- [x] Create a playlist from New playlist, select it, and retain drafts on expected failure.
- [x] Reuse the Sources divider and saved width with the correct accessible label.
- [x] Preserve entry artwork, playback/queue identity, removal, reordering and unavailable entries.
- [x] Verify many playlists, light/dark materials and minimum 800 by 600 layout.
- [x] Complete red/green commits, ordered gates and two review cycles.

## Established patterns

Reuse Muzo's existing Sources list, panel frame and split layout. React's
MIT-licensed portal pattern places the navigator in the existing sidebar while
keeping playlist selection, creation and entries under one state owner:
https://react.dev/reference/react-dom/createPortal
No external source code is copied and no dependency is added. A shared panel
frame has two concrete consumers, Sources and Playlists. Existing Result values
represent expected API failures at the playlist UI boundary.

## Verification

All ordered gates passed with 327 tests. New red-first tests cover selection,
name filtering and Escape clearing, creation with selection/focus restoration,
shared divider placement/width, retained search/selection across navigation, and
both orderings of concurrent refresh/creation. A further regression covers source
availability changes during creation. Existing queue identity, playback, entry
removal/reorder and artwork tests remain green.

Browser checks used 40 playlists, created a 41st, filtered to a distant row,
selected it, cleared search, created a playlist, reordered/removed entries and
switched between Sources and Playlists with the same divider width. Light/dark
brushed aluminium and minimum 800 by 600 were inspected with playback and art
visible. At normal size, list height was 129px with 1312px scrolling content;
the sidebar stayed within its 638px height and the page had no horizontal overflow.

Initial review found an overlapping refresh could drop/duplicate a new playlist.
The first fix re-review identified a related availability-update loss. Both were
fixed with red-first regressions, followed by a third two-axis review with zero
remaining findings. See [review record](../../reviews/2026-10-24-playlist-sidebar-panel.md).
