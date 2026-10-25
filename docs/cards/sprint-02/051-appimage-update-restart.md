# 051: Restart updated AppImages reliably

## Background

The user identified Auto Digger PR #44 as a possible Muzo updater defect and approved fixing and releasing it. Muzo 0.0.3 replaces the AppImage correctly but calls Electron's relaunch helper inside the old temporary mount, without preserving launch arguments. The existing HTTP update fixture launches a shell script directly and cannot cover that lifecycle.

## Acceptance criteria

- [x] Reproduce the defect with a real packaged AppImage before implementation.
- [x] Start the retained outer AppImage directly during final shutdown, preserving launch arguments and using its containing directory.
- [x] Wait for the operating system's spawn acknowledgement, finish the original process, and report launch errors through Result and a readable native message.
- [x] Keep the atomic replacement, original filename, menu entry and existing library intact.
- [x] Run the maintained packaged regression in Linux installer CI, with extraction mode for CI and optional FUSE mounting locally.
- [ ] Complete two review cycles and all ordered checks before promoting a 0.0.4 patch through dev to main.

## Verification

The native regression builds an older copy of the complete player with a local network provider, downloads a real replacement AppImage, requests installation through the renderer bridge and verifies the original exits, replacement window opens, version advances, library survives and the menu still targets the retained executable. Only its remote feed and confirmation dialog are fixture substitutions. Normal unit checks also cover launch ordering and original arguments.

The initial regression failed in both extraction and FUSE modes after successful replacement because the replacement debugging endpoint never appeared. The unit test separately confirmed the unsafe relaunch call. Failing tests were committed before implementation.

After the fix, both FUSE and extraction-mode packaged regressions pass. The native library, user-agent version, new debugging endpoint, retained filename and menu entry are verified. Expected spawn errors also pass dedicated adapter tests.
