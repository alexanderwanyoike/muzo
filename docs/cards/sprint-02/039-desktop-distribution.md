- Sprint: 02
- Type: Distribution
- Status: In progress

# Card 039 - Ship native desktop installers

The user requested a proper user download experience like Showbiz and Jolt, with
Windows, macOS and Linux AppImage delivery instead of build-from-source CTAs.

## Acceptance criteria

- [x] Build Windows x64 NSIS, macOS arm64/Intel DMG, Linux x64 AppImage from the existing Forge payload.
- [x] Preserve app executable/profile/credential identity, icon, runtime assets and bundled notices.
- [x] Define one shared typed manifest/filename contract for packaging and website validation.
- [x] Validate complete native artifacts, source/version identity and SHA-256 hashes before assembling a release.
- [x] Build PR artifacts on native runners; verify real extraction/mounting/installation and unchanged packaged main/WASM.
- [x] Permit draft release creation only for annotated main tags matching both package versions; never replace tags/releases.
- [x] Prepare 0.0.2 in a separate version-only commit, leaving v0.0.1 unchanged.
- [ ] Complete ordered gates, real local AppImage verification and two review cycles before feature PR handoff.

Depends on PR #63 (merge policy) and #65 (seek focus). This branch starts at dev
with those pending prerequisites included. The website PR consumes this contract
and published release artifacts separately. The miniplayer is another feature PR.
No dev/main merge, new tag or public release publication is performed here.

See [distribution guide](../../distribution.md) for the established tool pattern,
manifest, installer checks, candidate/release distinction and unsigned-build status.
