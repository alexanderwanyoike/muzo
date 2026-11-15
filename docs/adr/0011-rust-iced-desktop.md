# ADR 0011 - Rust and Iced desktop

## Status

Accepted.

Supersedes the desktop runtime decisions in [ADR 0002](0002-electron-desktop-shell.md)
and [ADR 0003](0003-remove-tauri-rust-backend.md) at cutover.

## Context

Muzo needs a native desktop experience, especially for its mini player, on
Linux, Windows and macOS. The existing Electron release provides the baseline
for product behaviour and installed-user upgrades.

## Decision

Implement the desktop application in Rust with Iced. Keep domain and use-case
logic independent of the UI and OS adapters. Use a single playback session
across Iced windows. Port the behaviour on focused feature branches integrated
through the rewrite epic, keeping the released desktop available until the
Rust build passes the parity and upgrade checks.

Iced's state and message model drives the app; its documented daemon and window
APIs manage the windows. Use the operating system's title bar and frame. The
mini player follows native compositor rules and keeps playback independent of
window order, visibility and geometry.

## Consequences

The rewrite retains Linux x64, Windows x64 and both macOS architectures. It
needs native package, update and credential adapters on those platforms. The
existing website and build tooling retain their current toolchain. The
[rewrite cards](../cards/rust-rewrite/README.md) track the parity checks.
