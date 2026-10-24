# Card 027 review evidence

Base: 0f962dcdfcd78bffe6990fabd93ebd51eadf33ad.
Code head: 3ed6284e255269a084d25e2991d14853c71d9e61.

# initial

## Standards

No documented-standard violations or actionable baseline smells found in the pinned Card 027 diff from 0f962dcdfcd78bffe6990fabd93ebd51eadf33ad to 3ed6284e255269a084d25e2991d14853c71d9e61.

Desktop identity policy is isolated in a small Electron-boundary module with a narrow lifecycle port. Named identity constants and the initialization comment explain the compatibility constraint; explicit renderer/session paths keep credential and presentation identity responsibilities clear. The application/domain layers and typed credential failure handling remain unchanged.

The history separates the failing lifecycle contract, minimal implementation, narrower-port refactoring, formatting and supplementary real-Electron profile verification. ADR 0006 records the new compatibility decision without rewriting earlier ADRs. Card 027 cites the established Electron initialization pattern and records sanitized read-only evidence rather than copying keyring or cryptographic implementation.

The twelve-smell baseline raises no actionable findings. Tool-enforced concerns are excluded. Final ordered gates and target-platform CI remain with the parent review.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: Dropbox storage identity, initial cycle

Pinned diff: `0f962dcdfcd78bffe6990fabd93ebd51eadf33ad...3ed6284e255269a084d25e2991d14853c71d9e61`. Reviewed the exact diff and its six commits against card 027 and ADR 0006.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. Visible naming, explicit renderer/session paths and the native fixture directly support restoring existing credentials while retaining the current application identity and preferences.

## (c) Apparently implemented requirements done incorrectly

No actionable findings. `desktop-identity.ts` sets Electron before native initialization and changes only the visible name on ready, meeting "Preserve the established keyring identity during Electron initialization" and "Keep Muzo as the visible name after native initialization." The synchronous entrypoint call precedes container construction and startup work. Explicit Muzo userData/sessionData paths preserve the current renderer profile independently of the credential namespace.

Electron 43's [initialization implementation](https://github.com/electron/electron/blob/v43.0.0/shell/browser/electron_browser_main_parts.cc#L478-L520) independently supports the timing: Linux OS-crypt configuration and macOS Keychain service/account names derive from the application name before ready.

The diff does not rewrite encrypted tokens or replace account state. Existing cipher behavior and its OS-backed-keyring requirement remain intact, consistent with "Read existing credentials without rewriting them or asking for reauthorization." The parent reports the production policy decrypted both original tokens twice while exposing only booleans and metadata.

The unit contract covers lifecycle ordering and explicit profile paths. The isolated real Electron fixture verifies visible name/profile/session together with window creation and runs in the existing platform CI coverage. This review did not access credentials or independently rerun native probes or ordered gates.

Findings: Standards 0, Spec 0.

# rereview

## Standards

No documented-standard violations or actionable baseline smells found on the second review of the pinned Card 027 diff from 0f962dcdfcd78bffe6990fabd93ebd51eadf33ad to 3ed6284e255269a084d25e2991d14853c71d9e61.

Rechecked lifecycle ownership: the entry point configures identity synchronously, the established credential name is selected before native initialization, and one ready listener restores the visible Muzo name. Explicit Muzo userData/sessionData paths preserve the renderer profile independently. The narrow port and named constants express the compatibility policy without adding a new credential service or changing encryption behavior.

The test-first lifecycle regression, separate focused implementation/refactoring/formatting commits and isolated native profile verification remain consistent with repository requirements. ADR 0006 preserves the rationale and migration constraint. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports all ordered gates passed with 286 tests and real fresh-profile verification. Native target-platform CI remains separately tracked.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: Dropbox storage identity, second cycle

Pinned diff: `0f962dcdfcd78bffe6990fabd93ebd51eadf33ad...3ed6284e255269a084d25e2991d14853c71d9e61`.

## (a) Missing or partial requirements

No actionable findings.

## (b) Unasked scope

No actionable findings. The lifecycle identity helper, retained Muzo profile paths and expanded native fixture remain focused on the reported credential decryption regression.

## (c) Apparently implemented requirements done incorrectly

No actionable findings after fresh inspection of the pinned helper, production entrypoint and real Electron fixture. The helper runs synchronously before container construction, holds the established Electron identity through native initialization and restores visible Muzo naming on ready. This meets card 027's "Preserve the established keyring identity during Electron initialization" and "Keep Muzo as the visible name after native initialization."

Explicit userData/sessionData paths retain the current Muzo renderer profile rather than selecting a profile through the encryption identity. The native fixture isolates appData and verifies name/profile/session plus window creation. The parent confirms fresh-profile startup passed and no filesystem workaround was required.

No token persistence, reauthorization or cipher/keyring protection logic is changed, matching "Read existing credentials without rewriting them or asking for reauthorization." The previously inspected [Electron 43 initialization code](https://github.com/electron/electron/blob/v43.0.0/shell/browser/electron_browser_main_parts.cc#L478-L520) supports setting the credential identity before native Linux/macOS key-provider setup while applying visible naming later.

The parent reports all ordered local gates green with 286 tests, including the real fresh Muzo profile. This read-only review did not access credentials, modify the repository or rerun those gates. Native Windows/macOS CI remains the parent task's verification responsibility.

Findings: Standards 0, Spec 0.

All ordered checks passed with 286 tests. A read-only real-keyring probe
reproduced the original failure and verified both tokens decrypt with the fixed
policy while the visible name stays Muzo. No credential values were logged.
