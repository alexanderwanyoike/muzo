# 005: Port desktop appearance

- [ ] Offer Muzo's eight materials, light/dark modes and all classic/glow accents.
- [ ] Restore the appearance on restart and apply it to both player windows.
- [ ] Verify representative appearances in a private desktop session.
- [ ] Preserve legacy preferences and complete native keyboard and layout parity.

The mini player follows the existing compact player's artwork layer and opacity.
It uses [image's Gaussian blur](https://docs.rs/image/0.25.10/image/enum.DynamicImage.html#method.blur)
on a small cover and [Iced's lower stack layer](https://docs.rs/iced/0.14.0/iced/widget/struct.Stack.html#method.push_under)
to keep transport controls above it.

The visual reference is the Electron Studio theme. Compact and expanded window
sizes, artwork and controls follow `player-window-mode.ts` and
`compact-player.css`. Iced embeds the regular and bold [Manrope
faces](https://github.com/google/fonts/tree/b31870aff700ab7a1d74fa0c6887d95beb9e0037/ofl/manrope),
instanced at weights 400 and 700. The installers include the existing Manrope
SIL Open Font License.

The library track table uses the tabular alternatives from the regular
face. [The FontTools generator](../../../apps/desktop-rust/assets/fonts/generate_table_font.py)
remaps the font's `tnum` glyphs and subsets the characters used by the
numeric cells into the bundled Muzo Table Text face.

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
