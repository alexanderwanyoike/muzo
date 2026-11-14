# 005: Port desktop appearance

- [ ] Offer Muzo's eight materials, light/dark modes and all classic/glow accents.
- [ ] Restore the appearance on restart and apply it to both player windows.
- [ ] Verify representative appearances in a private desktop session.
- [ ] Preserve legacy preferences and complete native keyboard and layout parity.

The mini player follows the existing compact player's artwork layer and opacity.
It uses [image's Gaussian blur](https://docs.rs/image/0.25.10/image/enum.DynamicImage.html#method.blur)
on a small cover and [Iced's lower stack layer](https://docs.rs/iced/0.14.0/iced/widget/struct.Stack.html#method.push_under)
to keep transport controls above it.
