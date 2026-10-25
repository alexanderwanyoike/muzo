# Desktop distribution

Muzo uses Electron Forge to produce the existing application bundle, then
`electron-builder` 26.18.0 packages that exact bundle through its documented
`--prepackaged` mode. Forge remains the app packager; the renderer, bundled main
process, profile, native icon and credential-storage identity stay in place.

The build follows the installer and validated-release-set pattern in sibling
Showbiz and Jolt: build native artifacts, verify them, assemble one complete set,
then create a draft for a maintainer to test and publish. Version 0.0.3 adds explicit
download and restart updates; see [ADR 0009](adr/0009-explicit-desktop-updates.md).

## Installer contract

| Target | Published installer |
| --- | --- |
| Linux x64 | `Muzo-X.Y.Z-linux-x64.AppImage` |
| Windows x64 | `Muzo-X.Y.Z-windows-x64-setup.exe` |
| macOS Apple silicon | `Muzo-X.Y.Z-macos-arm64.dmg` |
| macOS Intel | `Muzo-X.Y.Z-macos-x64.dmg` |

Every complete release also contains `muzo-X.Y.Z-release.json`, `SHA256SUMS`,
`latest-linux.yml` and `latest.yml`. The update YAML files name the Linux and
Windows installers and include their exact sizes and SHA-512 checksums.
The private `@muzo/distribution` build-tool workspace owns filenames and manifest
validation. Its public contract has no Node or application-infrastructure imports,
so the website can validate the same contract without copying it.

```json
{
  "schemaVersion": 1,
  "tag": "v0.0.3",
  "version": "0.0.3",
  "sourceCommit": "<exact 40-character lowercase commit SHA>",
  "artifacts": [
    {
      "platform": "linux",
      "arch": "x64",
      "name": "Muzo-0.0.3-linux-x64.AppImage",
      "size": 123,
      "sha256": "<64-character lowercase SHA-256>"
    }
  ]
}
```

The example abbreviates the artifact list; a valid manifest must contain all four
supported platform/architecture pairs exactly once, with version-correct names,
positive file sizes and valid hashes. URLs come from matching published GitHub
release assets, not from untrusted manifest fields. SHA256SUMS covers the four
installers, release manifest and both update YAML files.

## Local packaging

Run the command on the installer's native operating system:

```sh
yarn install
yarn desktop:distribute --target linux-x64
```

Windows uses `windows-x64`; macOS uses `macos-arm64` or `macos-x64`. Both macOS
architectures can be packaged on a macOS runner. Installers are written to
`apps/desktop/out/distribution/`. The packager is explicitly configured never to
publish. Electron is pinned to the already-used 43.0.0 for reproducible packaging.
Existing `desktop:package` and `desktop:make` commands remain available.

## CI and release promotion

The Desktop distribution workflow runs on relevant PRs to dev/main, annotated
version-tag pushes, and manual workflow runs. It verifies package-version equality
and the exact source SHA, then runs formatting, lint, strict types and tests.
Four native jobs build the installers and upload reviewer artifacts.

Linux extracts the real AppImage and checks its launcher and packaged resources.
macOS verifies and mounts each DMG, checks the executable architecture and compares
Info.plist against the Forge bundle. Windows installs the real NSIS package in the
runner's temporary directory. Each job compares the installed/extracted main.cjs
and sql-wasm.wasm with Forge's originals and checks the renderer exists. The complete
Forge payload retains Electron/Chromium notices plus the Manrope font OFL under
`assets/licenses/`.

Assembly validates native installer containers, rejects missing/corrupt files and
wrong-CPU AppImages, streams SHA-256 hashes, writes the manifest and checksums, and
refuses to overwrite an existing output directory. Standard sha256sum verification
runs against the assembled set before upload.

PR/manual artifacts are unpublished release candidates. Their manifest tag names
the intended release, and sourceCommit identifies that exact candidate checkout.
Only a real annotated tag whose commit is already on main can create a draft GitHub
Release. Both root and desktop package versions must match that tag. Existing
releases and published tags are never replaced or moved. Version 0.0.3 is
prepared through a feature PR before promotion to main.

Merge feature PRs into dev with merge commits, then promote dev to main through a
release PR. After all checks pass, tag the merged main release commit with the
matching annotated version. Review the resulting four installers and SHA256SUMS,
then publish the draft. The separate website workflow consumes published releases.
Preview installers are currently unsigned; macOS builds are not notarized. Present
that status alongside downloads. Windows and Linux AppImage builds offer explicit
download and Restart to install. Unsigned macOS builds open the release page for
manual installation. Users on v0.0.2 must install v0.0.3 manually once to enable
future updates.
Linux users mark the downloaded AppImage executable before opening it.

## Project licence

Muzo is MIT licensed. The complete project licence ships as
`assets/licenses/Muzo-MIT.txt` inside the app, matching the root `LICENSE`.
Third-party notices, including Manrope OFL and the Electron/Chromium runtime
notices, remain under their original terms and are retained in the payload.

## Established tools and licences

- electron-builder, MIT, used unmodified as a build dependency:
  https://github.com/electron-userland/electron-builder
- Its documented prepackaged native formats:
  https://www.electron.build/v26/docs/cli/
- AppImage format and execution:
  https://github.com/electron-userland/electron-builder/blob/master/website/docs/appimage.md

No third-party implementation is copied into Muzo. Existing native container and
runtime notices remain in the standard tool-generated payload.

## AppImage desktop menus

Packaged AppImage launches register Muzo in the per-user XDG application directory
(`$XDG_DATA_HOME/applications`, or `~/.local/share/applications`). The menu entry
uses the outer AppImage filename and a durable copy of the native Linux icon,
not a file inside the temporary mount. Desktop environments can then discover,
launch, group and pin Muzo through their normal controls.

The stable launcher is refreshed when the AppImage is opened from a new location.
Existing externally managed menu integrations are preserved. A known Cinnamon
shortcut to `/tmp/.mount_Muzo-*/muzo` is repaired under its existing filename to
retain pins. This is a migration for that old shortcut, while the normal
registration uses freedesktop/XDG standards for Linux desktops.

This does not move the AppImage, require root, alter desktop preferences or change
Muzo's profile and credential identity. If registration cannot be saved, Muzo
continues running and reports the problem with a retry instruction.

The Linux distribution job runs `appimage:verify-menu` against the real packaged
AppImage in isolated XDG directories, with clean and broken-shortcut scenarios.
The test validates desktop syntax and Gio discovery, checks the icon and launch
path, and matches the actual X11 window class. The startup guard skips other
platforms and non-AppImage launches. Normal unit checks also exercise native menu
launching and desktop argument escaping. Linux CI installs desktop-file-utils and
Python Gio as verification tools, not bundled runtime requirements.

## Installed updater configuration

Forge's prepackaged payload skips electron-builder's general afterPack hook. The
distribution command writes a fixed GitHub feed and updater cache configuration
into `resources/app-update.yml` before native installer creation. The AppImage
target may regenerate that file using its standard cache identity. Native installer
checks verify the shipped configuration; release assembly verifies both update
YAML files against the installers before publication.
