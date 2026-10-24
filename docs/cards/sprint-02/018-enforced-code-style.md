- Sprint: 02
- Type: Tech debt
- Status: In progress
- Blocks: -

# Card 018 - Enforce code style and typed lint

Make formatting and the documented TypeScript lint baseline runnable locally and
required in desktop CI. This closes the inherited gate debt identified during the
initial Standards review of the Electron integration.

## Acceptance criteria

- [x] Yarn scripts automatically format maintained code/configuration and check it without writing.
- [x] CI checks formatting before lint and retains the required verification sequence.
- [x] Airbnb's base, TypeScript and React rules provide the lint baseline with Prettier compatibility.
- [x] Production TypeScript rejects explicit any and unsafe assignments, arguments, calls, member access and returns.
- [x] Strict TypeScript checks include shared contracts and configuration.
- [x] Existing modules conform without lint suppression comments.
- [x] Unknown Dropbox token data is validated before mapping it into application credentials.
- [x] All required checks and Linux packaging succeed on the final branch (225 tests; zip and deb).
- [ ] Initial review and fix re-review are complete.

## Established patterns and scope

Use [ESLint Airbnb Extended](https://github.com/eslint-config/airbnb-extended)'s
maintained ESLint 9 flat configurations, TypeScript-aware rules and React rules,
then [eslint-config-prettier](https://github.com/prettier/eslint-config-prettier)
to disable formatter conflicts. Prettier, the compatibility config and Airbnb
Extended are MIT licensed dependencies. No third-party implementation is copied.

Keep named exports for existing ports/adapters and React's automatic JSX runtime.
Use TypeScript union exhaustiveness in place of unreachable default branches.
Allow sequential I/O and early continues in scanning, polling and migrations.
Permit mutations only for named imperative audio/ref/watcher handles. Adapter
methods implementing an interface may be stateless. The legacy outer IPC adapter
retains its structured rejection contract; application failures remain typed Results.
React lifecycle effects reset loading/editor state; the React Compiler is not enabled.
Conditional JSX views and local event handlers retain the existing component structure.

Test-only exemptions accommodate Vitest matchers, fixtures and controlled doubles.
They do not weaken production unsafe-operation rules. Build outputs, dependencies,
lockfiles and generated Vite configuration are excluded from formatting. Maintained
TypeScript, JavaScript, JSON, CSS, HTML and YAML are covered; immutable ADR prose is
not rewritten. No git hook interferes with separately committed TDD red phases.

## Verification and dependency

Depends on PR #44 and the desktop completion stack. A temporary negative CLI probe
confirmed that explicit any and every required unsafe-operation category fail lint;
the probe was removed. Existing tests cover the mechanical lint refactors. Separate
red and green commits add malformed token and unsuccessful HTTP response coverage.
