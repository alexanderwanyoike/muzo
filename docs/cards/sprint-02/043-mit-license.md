- Sprint: 02
- Type: Licensing
- Status: In progress

# Card 043 - License Muzo under MIT

The user explicitly requested changing Muzo from proprietary to MIT.

## Acceptance criteria

- [x] Add the standard MIT licence, copyright 2026 Muzo contributors.
- [x] Set the root and desktop package metadata to MIT.
- [x] Replace the current README proprietary claim with the MIT licence link.
- [x] Include an identical project notice in the packaged native app assets.
- [x] Preserve all third-party licence terms and historical decision records.
- [ ] Complete verification and both review cycles, then deliver a feature PR to dev.

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
