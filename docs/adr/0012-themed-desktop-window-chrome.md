# ADR 0012 - Themed desktop window chrome

## Status

Accepted. Supersedes the window frame decision in [ADR 0011](0011-rust-iced-desktop.md).

## Context

The library and mini player need a frame that fits Muzo's theme. Showbiz's inset
panels and the compact Codex window controls are the visual references.

## Decision

Draw the same 32 logical pixel title bar in both windows. Center the small
caption symbols inside 32 pixel controls, and derive the title bar and solid
rounded panels from Muzo's appearance palette. The artwork remains visible in
its cover and thumbnail slots.

Delegate window movement, resizing and state changes to the native window
manager through Iced's window API. The mini player keeps its shared playback
session when it expands the queue or returns to the library.

## Consequences

Muzo owns the rendering and hit targets for its frame. Verify both windows and
their controls on Linux, Windows and macOS as part of the desktop release checks.
