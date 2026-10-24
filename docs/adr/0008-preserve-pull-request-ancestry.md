# ADR 0008: Preserve pull request ancestry

- Status: Accepted
- Date: 2026-10-24

## Context

The first release PR was squash-merged into main. Its files match dev, but the
release commit does not retain dev as a parent. The user requested that squash
merging stop and that the release be tagged. Atomic TDD commits and shared branch
ancestry should survive feature integration and release promotion.

## Decision

Enable merge commits as the only GitHub PR merge method. Disable both squash and
rebase merging. Feature PRs target dev; release PRs promote dev to main. Rebasing
an unpublished feature branch before opening its PR remains permitted, subject
to the existing branch rules.

Create annotated release tags on the actual merged main commit. Version 0.0.1
is tagged as v0.0.1 at f626742e1e8d48af6ec665b69fd448ca46ff4bdb. Published tags
are immutable. This decision does not rewrite the existing release or publish a
GitHub Release.

The policy PR includes a merge of the existing main release commit into its dev
based feature branch. The trees were identical before that merge, so it changes
only ancestry. Merging the policy PR into dev records the release as an ancestor
without changing released application files.

## Consequences

PR merges retain atomic test, implementation and refactoring history. GitHub no
longer offers flattening merge methods. Existing historical squash commits remain
unchanged. The dev-to-main Git flow and review requirements still apply.
