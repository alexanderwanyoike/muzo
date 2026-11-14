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
