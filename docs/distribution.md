# Desktop distribution

Muzo builds the Rust desktop executable for each installer. Cargo Packager builds
the Linux AppImage and macOS application bundle. `hdiutil` builds the macOS disk
image. On Windows, `electron-builder` 26.18.0 packages the Rust executable and
license files through its documented `--prepackaged` mode, preserving the
existing installer identity.

The build follows the installer and validated-release-set pattern in sibling
Showbiz and Jolt: build native artifacts, verify them, assemble one complete set,
then create a draft for a maintainer to test and publish. [ADR 0009](adr/0009-explicit-desktop-updates.md) describes explicit download and restart updates.

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
  "tag": "v1.0.0",
  "version": "1.0.0",
  "sourceCommit": "<exact 40-character lowercase commit SHA>",
  "artifacts": [
    {
      "platform": "linux",
      "arch": "x64",
      "name": "Muzo-1.0.0-linux-x64.AppImage",
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

Run the command on the installer's native operating system and architecture.
Install [Cargo Packager](https://docs.crabnebula.dev/packager/) 0.11.8 on Linux or
macOS. The Windows installer uses the build-time NSIS dependency installed by Yarn.

```sh
yarn install
yarn desktop:distribute --target linux-x64
```

Windows uses `windows-x64` and macOS uses `macos-arm64` or `macos-x64`.
`desktop:package`, `desktop:make`, and `desktop:distribute` all build the host
installer. Omit `--target` to select it automatically. The default output is
`apps/desktop-rust/out/rust-distribution/`. Pass `--output /path/to/installers`
to use another filesystem. Packaging reuses Cargo's build directory for
intermediate files and stages final executables privately below Cargo's target
directory. Installer output is staged separately on the selected filesystem,
which can be mounted with execution disabled. Packaging removes both temporary
directories when it ends. These commands produce local files only.

## CI and release promotion

The Rust desktop installers workflow runs on relevant pull requests, annotated
version-tag pushes and manual runs. It validates the root version, the Rust
workspace version, Cargo and Packager against the exact checkout before launching four
native installer jobs. It builds the Linux AppImage, Windows NSIS installer and
both macOS DMGs.

The Rust workflow extracts the AppImage and checks for its launcher and binary.
It mounts each DMG and verifies the application identity, executable architecture
and binary contents. Both macOS jobs exercise a copy of the installed profile.
The Windows job installs the native NSIS package over v0.0.6 in a temporary
directory and verifies the executable and profile.
The upgrade tests use checksum-verified copies of the published v0.0.6
installers.

Assembly validates native installer containers, rejects missing/corrupt files and
wrong-CPU AppImages, streams SHA-256 hashes, writes the manifest and checksums, and
refuses to overwrite an existing output directory. Standard sha256sum verification
runs against the assembled set before upload.

PR/manual artifacts are unpublished release candidates. Their manifest tag names
the intended release, and sourceCommit identifies that exact candidate checkout.
Pushing an annotated tag whose commit is already on main can create a draft
GitHub Release. Its native package versions must match the version in the tag.
The release job rechecks the remote tag identity before creating the draft.
Existing releases and published tags retain their original assets and commits.

Merge feature PRs into dev with merge commits, then promote dev to main through a
release PR. After all checks pass, tag the merged main release commit with the
matching annotated version. Review the resulting four installers and SHA256SUMS,
then publish the draft. The separate website workflow consumes published releases.
The installers are unsigned, and macOS builds are unnotarized. Windows and Linux
AppImage builds offer explicit download and restart. macOS builds open the release
page for manual installation. Linux users mark the downloaded AppImage executable
before opening it.


## Project licence

Muzo is MIT licensed. The complete project licence ships as
`assets/licenses/Muzo-MIT.txt` inside the app, matching the root `LICENSE`.
Third-party notices under `apps/desktop-rust/assets`, including Manrope OFL,
are retained under their original terms.

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

The Linux installer check starts the packaged app under Xvfb with isolated
XDG directories. Rust tests exercise native menu registration, migration and
desktop argument escaping. Run the isolated startup check after packaging with:

```sh
bash apps/desktop-rust/tests/manual/linux-appimage-start.sh /path/to/Muzo-X.Y.Z-linux-x64.AppImage
```

The Rust updater consumes the published release manifest. Release assembly
validates both update YAML files against their installers so existing releases
can use their established update feed to install the Rust build.
The restart adapter spawns the retained outer AppImage during final shutdown and
preserves launch arguments. See ADR 0010. On the first upgrade from 1.0.0, its old
restart code may require reopening the existing AppImage after replacement.
