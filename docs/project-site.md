# Muzo project site

The site is a static HTML/CSS page in `site/`, built with the existing Vite
version. It follows the static site and official GitHub Pages Actions pattern
used by sibling projects Vibez and Showbiz. No sibling implementation, assets or
copy were copied. GitHub documents the workflow at:
https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages

The page has no client JavaScript, analytics or remote fonts. The existing Muzo
window icon and canonical screenshots are served alongside a local Manrope Latin
font. Its SIL Open Font License is retained in `site/public/fonts/OFL.txt`.
Screenshot provenance is recorded in [screenshots.md](./screenshots.md).
The material swatches are illustrative CSS samples; the app captures are real.

## Local checks and preview

```sh
yarn install
yarn site:verify
yarn site:build
yarn site:preview
```

Open the preview URL printed by Vite, including its `/muzo/` project base.
`yarn workspace @muzo/site dev` starts the source preview instead.
Root formatting, lint, typecheck and test commands include the site workspace.

The HTML integration tests verify section anchors, source/release availability,
canonical and sharing metadata, local stylesheet/font/icon references, and the
current demo captures. Public assets keep stable URLs so Open Graph images do
not depend on Vite's hashed stylesheet names. The build contains only public
assets and compiled HTML/CSS, with no test code or tool configuration.

## Publication

This feature PR prepares the site. It does not merge dev/main or enable public
hosting. After approval, merge dependencies using merge commits, then promote dev
to main through a release PR. In repository Settings > Pages, select GitHub Actions
as the publishing source. Keep the `github-pages` environment limited to main.
Run Project site on main if Pages was configured after the promotion.

The Pages workflow verifies relevant pull requests to dev/main. Deployment is
limited to main pushes and manual runs explicitly selected on main; PRs and
feature-branch manual runs cannot publish. The intended URL is:
https://alexanderwanyoike.github.io/muzo/

## Release information

The current page offers source installation from the immutable v0.0.1 tag and a
link to GitHub release history. There are no published binary assets yet. It does
not claim an open-source licence: the repository remains UNLICENSED/proprietary.

When publishing packaged downloads, update the availability text, source version
and links through the release PR, then adjust the matching availability test.
Use real release asset URLs only after those files exist. Do not move an existing
release tag to match a later screenshot or website revision. The screenshots show
the development interface, including the seek-focus correction after v0.0.1.
