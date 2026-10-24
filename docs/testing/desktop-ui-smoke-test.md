# Desktop UI smoke test

The local branch `codex/test-all-desktop-work` combines the completed desktop
stack for manual testing. Feature PRs target dev and remain separate; their
descriptions give the prerequisite merge order.

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
2. Try all six materials in light and dark, then the ten accents. Check Songs,
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

9. Add several sources, search by a name fragment and folder path, then clear
   by the button and Escape. Selection and playback should stay unchanged.
   Scroll the source list to the end; navigation, search and artwork stay visible.
10. Check the native window after restart: no File/Edit/View/Window menus on
    Windows/Linux. On macOS check the Muzo app controls and editing shortcuts.

11. Check the flat blue/black Muzo icon in the native title/taskbar or Dock.
    On Linux install the generated DEB in a test environment and check the Muzo
    launcher entry. On Windows/macOS check the packaged executable/app bundle.
    Inspect small icons and a high-density display as well as the large icon.

12. View several album covers, quit and reopen the same build. Existing covers
    should come from the local thumbnail cache. Cached Dropbox covers remain
    visible offline. The first view after installing this change populates it.
    Modify an embedded cover, refresh its source and check the updated image.
13. Restart the packaged build and browse Dropbox using the existing account.
    The secure-storage identity fix should restore the saved connection.

14. Drag the Sources divider, use arrows/Home/End, double-click to reset and Escape
    to cancel a drag. Reopen to check the saved width. Temporarily narrow the
    window, cancel a drag, then widen it; the wider preference should return.
15. Open Playlists. Its panel should share the Sources frame, search and divider.
    With many playlists, scroll rows while the header/footer and artwork stay
    visible. Select one playlist and check that only its entries appear in the
    main area. Search should preserve selection; Escape and Clear restore rows.
16. Use New playlist, cancel with Escape, then create with a trimmed name. The new
    playlist should be selected and focus should return to New playlist. Switch
    to Songs and back; search/selection should survive. Check entry reordering,
    removal, duplicate-track queue playback and unavailable entries after source
    removal. Repeat in light/dark and at 800 by 600.

17. Open Settings with a narrow and wide sidebar at 800x600 and a larger window.
    Appearance, Dropbox, Library folders and Indexed sources should share their
    left/right edges, heading alignment and section gaps. Name should end at the
    same edge as the Location/Browse row. Check connected/disconnected Dropbox,
    folder selection, long source paths/actions, all materials and both modes.
    Errors should remain visible in the danger colour and keyboard focus clear.

18. Check Carbon fibre, Walnut and Porcelain in light/dark, plus Teal, Rose, Slate,
    Violet and Ice accents. Restart to confirm persistence. All ten swatches should
    fit or wrap cleanly with a wide sidebar at the minimum size. In Cinnamon,
    restart the packaged player and check the waveform has distinct bars at the
    panel's 24/32px icon size.


Automated verification covers typed failures, account mismatches, late scans,
refresh races, pending audio, watcher close events, range-read budgets and native
thumbnail bounds. Live Dropbox behavior still depends on the account, connection
and file metadata used during manual testing.
