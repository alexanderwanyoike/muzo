# ADR 0006 - Stable desktop credential identity

- Status: Accepted
- Date: 2026-10-24

## Context

Earlier Muzo development builds encrypted Dropbox tokens in Electron's OS-keyring
namespace named Electron. Setting the app name to Muzo before ready switched the
Linux/macOS key provider, making those existing credentials unreadable. A native
read-only probe confirmed that the original credentials still decrypt correctly.

## Decision

Keep the established Electron credential identity before native initialization.
Set the visible app name to Muzo on ready, after Electron binds OS-crypt keys.
Explicitly retain the Muzo userData and sessionData paths for current renderer
preferences. Configure this policy once, synchronously at the entry point.

## Consequences

Existing credentials remain usable without reauthorization or rewriting them.
Branding changes must not implicitly select a different encryption namespace.
Changing that namespace in future requires an explicit decrypt/re-encrypt
migration. Credential protection continues to require an OS-backed keyring, as
specified in ADR 0004.
