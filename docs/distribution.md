# Desktop distribution

Muzo uses Electron Forge to produce the existing application bundle, then
`electron-builder` 26.18.0 packages that exact bundle through its documented
`--prepackaged` mode. Forge remains the app packager; the renderer, bundled main
process, profile, native icon and credential-storage identity stay in place.

The build follows the installer and validated-release-set pattern in sibling
Showbiz and Jolt: build native artifacts, verify them, assemble one complete set,
then create a draft for a maintainer to test and publish. No updater is added.

## Installer contract

| Target | Published installer |
| --- | --- |
| Linux x64 | `Muzo-X.Y.Z-linux-x64.AppImage` |
| Windows x64 | `Muzo-X.Y.Z-windows-x64-setup.exe` |
| macOS Apple silicon | `Muzo-X.Y.Z-macos-arm64.dmg` |
| macOS Intel | `Muzo-X.Y.Z-macos-x64.dmg` |

Every complete release also contains `muzo-X.Y.Z-release.json` and `SHA256SUMS`.
The private `@muzo/distribution` build-tool workspace owns filenames and manifest
validation. Its public contract has no Node or application-infrastructure imports,
so the website can validate the same contract without copying it.

```json
{
  "schemaVersion": 1,
  "tag": "v0.0.2",
  "version": "0.0.2",
  "sourceCommit": "<exact 40-character lowercase commit SHA>",
  "artifacts": [
    {
      "platform": "linux",
      "arch": "x64",
      "name": "Muzo-0.0.2-linux-x64.AppImage",
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
installers and the manifest itself.

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
releases are not replaced, and existing v0.0.1 is never moved. Version 0.0.2 is
prepared for the next release; no new tag is created by this feature PR.

Merge feature PRs into dev with merge commits, then promote dev to main through a
release PR. After all checks pass, tag the merged main release commit with the
matching annotated version. Review the resulting four installers and SHA256SUMS,
then publish the draft. The separate website workflow consumes published releases.
Preview installers are currently unsigned; macOS builds are not notarized. Present
that status alongside downloads rather than claiming trusted signing or updates.
Linux users mark the downloaded AppImage executable before opening it.

## Established tools and licences

- electron-builder, MIT, used unmodified as a build dependency:
  https://github.com/electron-userland/electron-builder
- Its documented prepackaged native formats:
  https://www.electron.build/v26/docs/cli/
- AppImage format and execution:
  https://github.com/electron-userland/electron-builder/blob/master/website/docs/appimage.md

No third-party implementation is copied into Muzo. Existing native container and
runtime notices remain in the standard tool-generated payload.
