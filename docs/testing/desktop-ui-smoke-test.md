# Desktop UI smoke test

The local branch `codex/test-all-desktop-work` combines the completed desktop
stack for manual testing. The feature PRs remain separate: #47 folder selection,
#48 desktop materials, #49 source removal, then #50 embedded artwork.

Close any running Muzo development process before switching. From the repository:

```sh
git switch codex/test-all-desktop-work
yarn install
yarn desktop:dev
```

A full restart is required for new main-process IPC commands. The browser-only
Vite preview cannot connect to real Dropbox or local music files.

1. Open Settings and browse the connected Dropbox account. Select folders in
   different locations, review the selection and add them together. Settings
   should stay open; Sources should update and added folders should be marked.
2. Try all three materials in light and dark, then the five accents. Check Songs,
   playlists, folder selection, fields, focus rings and disabled controls.
   Restart Muzo and verify the chosen appearance is restored.
3. Select a source by clicking its name. Play local MP3/FLAC tracks containing
   embedded PNG/JPEG covers. Check the row, transport and larger sleeve. Check a
   track with no cover as well; playback should work with the disc placeholder.
4. Check artwork on Dropbox tracks while connected to the owning account.
   Playback downloads audio when selected; viewing artwork reads metadata only.
   Missing, unsupported or temporarily unavailable art should not block playback.
5. After changing a file's embedded cover, refresh its source in Settings.
   Song rows and any persistent current-track covers should reload.
6. Create a playlist with repeated tracks and try entry playback, previous/next,
   pause/resume, reordering and removal. Start a queue and let an entry finish.
7. Use a disposable source to test removal. Add one of its tracks to a playlist,
   then cancel the removal dialog and try Escape. Reopen and confirm. Its music
   files should remain on disk; its Muzo catalogue data should be gone. Visible
   playlist entries should become unavailable immediately. Playback from another
   source should continue, with removed entries filtered out of its queue.
8. Resize to the minimum 800 by 600 window. The transport and sidebar should stay
   in their desktop arrangement without horizontal window scrolling.

Automated verification covers typed failures, account mismatches, late scans,
refresh races, pending audio, watcher close events, range-read budgets and native
thumbnail bounds. Live Dropbox behavior still depends on the account, connection
and file metadata used during manual testing.
