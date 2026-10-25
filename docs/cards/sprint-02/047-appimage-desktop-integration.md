# 047: Register AppImages with Linux desktop menus

## Request

Opening Muzo's AppImage for the first time should add it to the application menu
with its real icon, so users can pin it through their desktop's normal controls.
Use Linux desktop standards rather than a Cinnamon-only installation path.

## Diagnosis

The released 0.0.2 AppImage starts with WM_CLASS muzo and bundles a matching
muzo.desktop, but does not register it in the user's XDG application directory.
The user's manually generated Cinnamon entry had no icon and launched
/tmp/.mount_Muzo-*/muzo, which disappears when the AppImage unmounts.
A disposable profile reproduced missing Gio desktop discovery. Registering the
launcher with the outer AppImage path made it discoverable immediately.

## Acceptance criteria

- [x] Register packaged Linux AppImages automatically on launch using XDG data paths.
- [x] Keep the launcher executable outside the temporary mount and retain a stable icon.
- [x] Match existing muzo window identity without changing profile or credential identity.
- [x] Refresh the same launcher after relocation or replacement.
- [x] Preserve existing external integrations and repair the known temporary Cinnamon launcher in place.
- [x] Handle invalid paths, concurrent launches and storage failures explicitly.
- [x] Leave development, unpacked, Windows and macOS launches unchanged.
- [x] Pass ordered repository gates and real AppImage first-launch smoke tests.
- [x] Add real AppImage first-launch verification to Linux CI.
- [x] Complete initial review and fix re-review.

## Established patterns

Follow the freedesktop Desktop Entry specification and the AppImage desktop
integration pattern of registering a launcher and durable icon in the user's
XDG data directory. No third-party implementation is copied and no dependency
is added to the application runtime.

- https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
- https://specifications.freedesktop.org/basedir-spec/latest/
- https://docs.appimage.org/reference/desktop-integration.html
- https://docs.appimage.org/user-guide/run-appimages.html

## Verification

Native Gio discovery and actual launcher execution exercise spaces, quoting and
literal percent characters. Other regressions cover relocation, overlapping
registration, foreign launchers, malformed paths and failed icon reads.
The packaged smoke test launches a real AppImage with disposable XDG directories,
checks menu discovery, icon, persistent executable and native WM_CLASS, then repeats
with a broken Cinnamon launcher. It uses Xvfb and does not alter the real desktop.
This verifies the standard integration boundary, not every distribution's shell UI.
