# 001: Play local music in the Rust mini player

- [ ] Play an audio file through the OS output device.
- [ ] Pause, resume, seek and change volume.
- [ ] Queue local files, advance automatically and select queue entries.
- [ ] Show embedded artwork and available title, artist and album.
- [ ] Open the mini player and expand or collapse its queue.
- [ ] Return to the library with its previous bounds and without restarting audio.
- [ ] Test playback failures and recovery.
- [ ] Build on Linux, Windows and macOS. Exercise the native window and capture screenshots.

## Established patterns

Use Iced's documented daemon and window tasks for two views of the same
playback session. Use Rodio's mixer/player with its file decoder and CPAL device
sink. Read track tags and embedded pictures with Lofty's probing API. The
dependencies are used through their public APIs, with no copied source.

- Iced (MIT): https://docs.rs/iced/0.14.0/iced/daemon/index.html
- Rodio (MIT or Apache 2.0): https://docs.rs/rodio/0.22.2/rodio/
- Lofty (MIT or Apache 2.0): https://docs.rs/lofty/0.25.5/lofty/
