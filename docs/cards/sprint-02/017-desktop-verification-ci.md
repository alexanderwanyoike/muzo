- Sprint: 02
- Type: Tech debt
- Status: In progress
- Blocks: -

# Card 017 - Enforce desktop verification in CI

The completed desktop features have local verification but the repository has no
CI workflow. Run the complete required sequence for pull requests, including the
stacked completion branches, and for updates to dev and main.

## Acceptance criteria

- [x] Use Yarn classic 1.22.22 with a frozen lockfile on Node 22.
- [x] Run install, lint, typecheck, test, build, package, and make in order.
- [x] Use a Linux runner with the tools needed to make deb and zip artifacts.
- [x] Bound Vitest workers and cancel superseded runs for the same PR.
- [x] Pin GitHub Actions revisions and restrict workflow permissions to reading source.
- [x] All local verification commands pass (189 tests; Linux zip and deb).
- [ ] GitHub reports a successful workflow run for this PR.

## Pattern and verification

Use the standard checkout/setup-node GitHub Actions workflow with Yarn caching.
The action revisions were resolved from the official actions/checkout and
actions/setup-node v4 references. No external workflow source is copied.
This is CI configuration only, so no production tests are added. Validate YAML
structure, run the workflow's exact commands locally, then inspect the real
GitHub Actions run.

## Dependencies

Depends on the playlist queue PR and transitively on #42, #41, and #16. Retarget
this PR to dev after its dependencies land. Enabling this workflow does not merge
branches or change repository branch protection.
