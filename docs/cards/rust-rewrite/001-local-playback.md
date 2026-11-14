# 001: Play local music in the Rust mini player

- [x] Play an audio file through the OS output device.
- [x] Pause, resume, seek and change volume.
- [x] Queue local files, advance automatically and select queue entries.
- [x] Show embedded artwork and available title, artist and album.
- [x] Open the mini player and expand or collapse its queue.
- [x] Return to the library with its previous bounds and without restarting audio.
- [x] Test playback failures and recovery.
- [x] Exercise the native Linux window and capture screenshots.
- [ ] Verify Windows and macOS builds and run native window checks.

## Established patterns

Use Iced's documented daemon and window tasks for two views of the same
playback session. Use Rodio's mixer/player with its file decoder and CPAL device
sink. Read track tags and embedded pictures with Lofty's probing API. The
dependencies are used through their public APIs, with no copied source.

- Iced (MIT): https://docs.rs/iced/0.14.0/iced/daemon/index.html
- Rodio (MIT or Apache 2.0): https://docs.rs/rodio/0.22.2/rodio/
- Lofty (MIT or Apache 2.0): https://docs.rs/lofty/0.25.5/lofty/

## Run the first slice

From `apps/desktop-rust`, start the desktop with
`yarn exec cargo -- run --locked`. Pick audio files in the native dialog, or pass
the paths as arguments after `--`. On Linux, install the ALSA development headers
before compiling.

The mixer tests drive real decoded samples, including pause, seek, resume, queue
advance, recovery from a missing file and playback continuity when switching
windows. The X11 smoke test uses a disposable Xvfb display and Metacity to check
the moved library window's geometry before and after compact mode. It also opens
the queue, selects another track and seeks in it.

Ogg Opus uses Symphonia's demuxer and the MIT or Apache 2.0
[Opus decoder](https://docs.rs/opus-decoder/0.1.1/opus_decoder/) for packet
decoding. A real Opus fixture exercises playback and seek.
