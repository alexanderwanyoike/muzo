# 005: Port desktop appearance

- [ ] Offer Muzo's eight materials, light/dark modes and all classic/glow accents.
- [ ] Restore the appearance on restart and apply it to both player windows.
- [ ] Verify representative appearances in a private desktop session.
- [ ] Preserve legacy preferences and complete native keyboard and layout parity.

The window shell follows Showbiz's inset workspace and solid rounded panels.
Muzo's appearance palette supplies the panel and accent colors. The library,
mini player and expanded queue share a 32 logical pixel title bar, with small
caption symbols centered in full-size controls. The mini player keeps the cover
and shadows inside the playback panel. Queue rows retain their cover thumbnails.

The visual reference is the Electron Studio theme. Compact and expanded window
sizes, artwork and controls follow `player-window-mode.ts` and
`compact-player.css`. Iced embeds the regular, semibold and bold [Manrope
faces](https://github.com/google/fonts/tree/b31870aff700ab7a1d74fa0c6887d95beb9e0037/ofl/manrope),
instanced at weights 400, 600 and 700 with FontTools. The installers include the existing Manrope
SIL Open Font License.

The library track table uses the tabular alternatives from the regular
face. [The FontTools generator](../../../apps/desktop-rust/assets/fonts/generate_table_font.py)
remaps the font's `tnum` glyphs and subsets the characters used by the
numeric cells into the bundled Muzo Table Text face.

Source rows follow `library-list.css` and `sidebar-panels.css` for the
selection stripe, the name and track-count grid, and the separate remove
target. The count uses the same tabular face as the track table. The cloud
and remove glyphs come from the existing desktop's Lucide icons. The
installers include the Lucide and Feather licences.

The track table follows the original `music-views.css` grid at wide and compact
widths, including the `themes.css` and `pulse-material.css` row colors. It
reuses the sidebar's width-aware text widget for track titles and artist details
and follows `formatDuration.ts` for tracks that last an hour or more.

The library matches the Electron window dimensions and the
`styles.css` and `transport.css` arrangement of the top transport,
resizable navigation sidebar, content toolbar and bottom update status.
The transport uses the original 950px breakpoint for its columns, volume
control and spacing. The display keeps its 610px maximum width and reuses
the library label widget to abbreviate long now-playing metadata.
The transport uses the five-percent volume steps from `NowPlayingBar.tsx`
and measures pointer travel between the thumb centers. Mouse presses on the
thumb keep the current level until the pointer moves, following
[Chromium's range control](https://chromium.googlesource.com/chromium/src/+/259286fb5706438088c858361ff942239598ea3b/third_party/blink/renderer/core/html/forms/slider_thumb_element.cc#226).
The transport follows `glow-accents.css` for the playback and volume shadows
and paints the thin seek rail's softer halo over its full redraw bounds.
