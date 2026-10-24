# Muzo app icon

The flat blue and black waveform mark is shared by all desktop platforms.
Nine separate bars form two peaks and a central trough, suggesting an M.
`muzo.png` is the transparent high-resolution master and Linux window/launcher
icon. `muzo.ico` contains Windows executable sizes, including BMP entries at
16, 24, 32 and 48 pixels and PNG entries at larger sizes. `muzo.icns` contains
macOS sizes through 1024 pixels. The icon is independent of the selected theme.

Regenerate native containers from the committed master:

```sh
yarn workspace @muzo/desktop icons:generate
```

Exports use the public API/CLI of [png2icons 2.0.1](https://github.com/idesis-gmbh/png2icons),
with its Windows-executable format and bicubic scaling. The MIT licence is
preserved in `png2icons-license.txt`. No converter implementation is copied.
Forge follows its [custom app icon guide](https://www.electronforge.io/guides/create-and-add-icons).

## Artwork provenance

Created with the built-in `image_gen` tool. No third-party artwork was copied.
The generated PNG is kept as the canonical master. Native format conversion
preserves its transparency. The discarded metallic/sage design is not shipped.

Generation prompt:

> Use case: logo-brand. Asset type: final square desktop app icon for Muzo, a music player, usable on Windows, Linux and macOS. Primary request: a flat blue and black icon. Center a bold geometric blue M monogram that subtly resembles an audio waveform inside a solid black rounded-square tile. Use only solid black #101216 and solid restrained cobalt blue #3478D4, with antialiasing at shape edges. Broad clean strokes, balanced negative space, simple silhouette legible at 16 and 32 pixels. Front-facing flat vector-like graphic, no depth, no metallic material, no texture, no gradients, no shine, no bevels, no outlines, no shadows, no white, no other colors, no letters beyond the single M, no wordmark. The tile fits within the canvas with even modest padding. Truly transparent alpha outside the rounded tile. One finished icon, not a mockup, presentation sheet or collection.

Earlier flat-fill edit prompt:

> Edit target: the attached Muzo blue-and-black app icon. Keep its exact M/audio-wave silhouette, rounded black tile, transparent outside, scale, placement and proportions. Change only the surface treatment to completely flat, uniform solid color fills: one single blue #3478D4 for the entire M and one single black #101216 for the entire tile. Remove all texture, speckle, grain, lighting, gradients and shading. This should look like a clean two-color SVG logo exported as PNG, with antialiased shape edges only. No added objects or text. Actual transparent alpha outside the tile.

Final waveform edit prompt:

> Edit target: the Muzo app icon. Replace the entire blue continuous ribbon M with a clear audio waveform built from nine separate, evenly spaced vertical blue rounded bars. The heights of the bars create two high peaks with a low center, suggesting the letter M through the waveform silhouette. All bars centered on a common horizontal centerline, symmetrical arrangement, broad enough to remain crisp at 16 and 32 pixels. The symbol should read first as an audio waveform or equalizer, with an understated M shape, not a conventional typographic letter. Keep the black rounded-square tile, its proportions, generous padding, front-facing composition and actual transparent alpha outside. Flat solid blue #3478D4 and black #101216 only. No continuous letter outline, no connecting stroke, no gradients, no lighting, no metallic material, no grain, no texture, no bevels, no shadows, no other colors, no wordmark.
