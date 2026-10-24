- Sprint: 02
- Type: Licensing
- Status: Done

# Card 043 - License Muzo under MIT

The user explicitly requested changing Muzo from proprietary to MIT.

## Acceptance criteria

- [x] Add the standard MIT licence, copyright 2026 Muzo contributors.
- [x] Set the root and desktop package metadata to MIT.
- [x] Replace the current README proprietary claim with the MIT licence link.
- [x] Include an identical project notice in the packaged native app assets.
- [x] Preserve all third-party licence terms and historical decision records.
- [x] Complete verification and both review cycles, then deliver a feature PR to dev.

This is the independent project licensing PR, based directly on dev. The root
licence applies to all owned project code. The pending site PR #66 separately
synchronizes its website notice, footer and private distribution/site workspace
metadata to this same licence; it does not block adopting MIT for the project.

The standard text was checked against GitHub Choose a License's authoritative
MIT template, substituting the year and project contributor attribution:
https://github.com/github/choosealicense.com/blob/gh-pages/_licenses/mit.txt

Documentation, licence text and package metadata introduce no runtime logic, so
no artificial TDD tests are added. Direct notice comparisons and existing checks
verify this change. Third-party notices keep their own licences.

## Verification and review

All ordered checks passed on independent core head
`d3afb00872953bee605350c03007cb88ca6740b8`: Yarn install, formatting, lint,
typecheck, 337 existing desktop tests, desktop build, package and make.
Root and native licence copies match at SHA-256
`e6d26c00d2f72560e6fb95569c7083c265c573d716267879c15d448dd89c839b`.
The parent verified the native packaged notice. Both initial Standards/Spec
reviews and fresh final re-reviews returned zero actionable findings. No fixes
were required. The evidence is recorded in
[the licensing review](../../reviews/2026-10-24-mit-project-license.md).
