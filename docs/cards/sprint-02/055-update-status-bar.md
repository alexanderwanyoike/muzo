# 055: Move updates into the application status bar

## Background

The sidebar update icon and two-line notice are too prominent. The user requested a small facelift: a proper application footer with a quiet update pill at the bottom right, showing checks, availability and download percentage before restart.

## Acceptance criteria

- [x] Add a slim footer spanning the full library window, outside the sidebar and content scrolling areas.
- [x] Replace the sidebar update block with a single-line text pill at the bottom right, without an icon.
- [x] Show Checking for updates..., Update available to x.x.x, percentage download progress, restart readiness and retryable failures.
- [x] Preserve explicit download, restart confirmation, manual platform downloads and the installed updater boundary.
- [x] Keep light/dark materials, keyboard focus and minimum-size layout readable and unclipped.
- [x] Complete both review cycles and ordered repository checks, then deliver a focused PR to dev.

## Implementation pattern

The established desktop status-bar pattern in [VS Code](https://github.com/microsoft/vscode/blob/main/src/vs/workbench/browser/parts/statusbar/media/statusbarpart.css), [MIT licensed](https://github.com/microsoft/vscode/blob/main/LICENSE.txt), informed the full-width footer and compact right-aligned action. No source or assets were copied and no dependencies were added. The existing Muzo update bridge and explicit restart flow from ADR 0009 remain responsible for updating.

## Verification

TDD first reproduced the hidden checking state, then the missing application footer and old availability label. Separate implementation commits made those tests pass. The native regression checks all eight materials in light and dark mode at 800 and 1200 pixels, including footer bounds, separate content/sidebar bounds, small pill height and real Tab focus. The existing Electron flow still rejects corrupt downloads, reports progress, preserves the executable path and waits for restart confirmation.

Local ordered gates passed with one worker: install, lint, strict types, 444 tests, desktop build, package, make and formatting. Both review axes reported zero findings in both cycles. [Review record](../../reviews/2026-10-25-update-status-bar.md). Native screenshots were inspected in Studio dark and Pulse light.
