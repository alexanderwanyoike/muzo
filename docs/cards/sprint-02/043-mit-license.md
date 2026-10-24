---
status: in-progress
---

# Card 043 - License Muzo under MIT

The project owner requested that Muzo become MIT licensed. Make that policy
explicit across the repository, all owned workspaces, native app and public site.

## Acceptance criteria

- [x] Root LICENSE uses standard MIT text with copyright 2026 Muzo contributors.
- [x] Root, desktop, distribution and site package metadata declares MIT.
- [x] README and current documentation describe the MIT licence consistently.
- [x] Native payload includes an identical assets/licenses/Muzo-MIT.txt notice.
- [x] The site footer links to an identical public/LICENSE.txt notice.
- [x] Third-party licences and notices remain unchanged.
- [x] Ordered gates and packaged notice comparison pass.
- [ ] Two review cycles and PR handoff complete.

## Source and verification

The MIT text follows GitHub's Choose a License template, replacing only its
copyright placeholders: https://choosealicense.com/licenses/mit/

This is a licence, metadata and documentation change. No runtime logic or test
behavior changes, so no artificial test-first cycle is added. Compare all three
licence copies, verify workspace metadata and public/native packaged output,
and run the existing ordered checks. Immutable ADRs and historical review
records remain unchanged.

Depends on the project site and desktop distribution work in cards 037 and 039.

Verification passed: frozen Yarn install, format, lint, typecheck, 369 existing
tests (351 desktop, 6 distribution, 12 site), desktop build/package/make and site
build. The Forge app, ZIP, DEB and built site include exact copies of the root
licence. Manrope OFL, Bootstrap Icons MIT and Electron/Chromium notices remain
present; committed third-party notice files are unchanged. The root licence
matches the upstream template byte-for-byte after substituting its two copyright
placeholders. No runtime behavior changed.
