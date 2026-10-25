# 0010: Restart AppImages directly during final shutdown

Status: Accepted. Supersedes the Linux relaunch mechanism in ADR 0009.

## Context

A real packaged update exposed the same restart problem documented in Auto Digger PR #44. Electron 43 launches its restart helper from the currently running executable inside the old temporary AppImage mount. Supplying execPath without args also discards the original launch arguments. Atomic replacement does not fix either issue.

## Decision

Keep electron-updater's download verification and Muzo's same-directory atomic replacement at the existing filename. At will-quit, after normal before-quit cleanup, temporarily defer termination and spawn the outer AppImage directly with the original arguments, its containing directory as cwd, detached true and ignored stdio. Wait for spawn or error before finishing the original process with app.exit. A second app.quit during this handoff left the original native process running in the regression, so it is not used. Expected spawn failures become typed Result values and a native error message explaining how to reopen the installed file.

The MIT-licensed detached-launch pattern from electron-updater 6.8.11 / electron-builder 26.18.0 informs this adapter. No upstream source is copied. Muzo does not acquire a single-instance lock, so there is no application lock to release here.

A Linux packaging regression must run a real AppImage, download another complete AppImage and observe the replacement player. The existing synthetic executable fixture remains useful for corrupt downloads and update UI checks but cannot prove application restart.

## Consequences

The menu entry retains the original filename. Library storage, credentials and other platform installers are unchanged. Muzo 0.0.3 can download the patch, but its old restart code may require reopening the existing AppImage manually once. Future restarts use the fixed adapter.

## References

- [Auto Digger PR #44](https://github.com/alexanderwanyoike/auto-digger/pull/44)
- [Electron 43 relaunch helper](https://github.com/electron/electron/blob/v43.0.0/shell/browser/relauncher.cc)
- [Electron 43 argument handling](https://github.com/electron/electron/blob/v43.0.0/shell/browser/api/electron_api_app.cc)
- [electron-updater detached launch](https://github.com/electron-userland/electron-builder/blob/v26.18.0/packages/electron-updater/src/BaseUpdater.ts)
