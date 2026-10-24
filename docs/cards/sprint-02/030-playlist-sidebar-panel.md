- Sprint: 02
- Type: Feature
- Status: In progress
- Depends on: 029 (PR #56)

# Card 030 - Consistent Playlists sidebar panel

The user asked for Playlists to use the same panel treatment as Sources. Use the
existing sidebar width/divider and material styling. Show one selected playlist's
tracks in the main workspace instead of rendering every playlist's tracks.

## Acceptance criteria

- [ ] Frame Playlists with the shared header/count, search, scrolling rows and footer.
- [ ] Select a playlist from its compact row and show only its tracks in the main area.
- [ ] Filter names without losing the selected playlist; clear search with Escape/control.
- [ ] Create a playlist from New playlist, select it, and retain drafts on expected failure.
- [ ] Reuse the Sources divider and saved width with the correct accessible label.
- [ ] Preserve entry artwork, playback/queue identity, removal, reordering and unavailable entries.
- [ ] Verify many playlists, light/dark materials and minimum 800 by 600 layout.
- [ ] Complete red/green commits, ordered gates and two review cycles.

## Established patterns

Reuse Muzo's existing Sources list, panel frame and split layout. React's
MIT-licensed portal pattern places the navigator in the existing sidebar while
keeping playlist selection, creation and entries under one state owner:
https://react.dev/reference/react-dom/createPortal
No external source code is copied and no dependency is added. A shared panel
frame has two concrete consumers, Sources and Playlists. Existing Result values
represent expected API failures at the playlist UI boundary.
