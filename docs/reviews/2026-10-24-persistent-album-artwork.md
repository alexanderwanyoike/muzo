# Card 028 review evidence

Base: 1b687870d472126ea67117ee5b2fb73e268a3ea3.
Initial code head: 4aa1251f8b5fe75c2d321241ea0b4ac53bf717da.
Final code head: ff21bf75655c66204e7d959487b7a55a55d8fd57.

# initial

## Standards

No documented-standard violations or actionable baseline smells found in the pinned Card 028 diff from 1b687870d472126ea67117ee5b2fb73e268a3ea3 to 4aa1251f8b5fe75c2d321241ea0b4ac53bf717da.

The application owns cache-aside policy through a typed Result port; the filesystem adapter owns serialization, validation, integrity and eviction. Expected storage failures are explicit Results, handled deliberately through logging and normal metadata fallback. Serial queues recover after rejection, file handles close in finally, and ordinary write failures clean up their temporary file. The cohesive adapter has one class and does not leak Node types inward.

The history separates meaningful restart, offline, eviction, corruption, size and queue regressions from focused implementations and later refactoring/formatting. Card 028 records established cache/thumbnail patterns, distinguishes the GPL reference from this independent implementation and preserves application-private format scope. ADR 0007 records the compatibility policy.

The twelve-smell baseline raises no actionable findings. Tool-enforced concerns are excluded. Full ordered gates remain with the parent review; crash-abandoned temporary-file behavior is a separate Spec-axis consideration.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: persistent artwork cache, initial cycle

Pinned diff: `1b687870d472126ea67117ee5b2fb73e268a3ea3...4aa1251f8b5fe75c2d321241ea0b4ac53bf717da`. Reviewed the exact diff and its 19 commits against card 028 and ADR 0007.

## (a) Missing or partial requirements

No additional actionable findings.

## (b) Unasked scope

No actionable findings. Serving cached Dropbox thumbnails before credential access is explicitly required by the updated offline catalogue policy; fresh media reads retain owning-account access checks.

## (c) Apparently implemented requirements done incorrectly

- [P2] Reclaim interrupted atomic-write files when opening the cache. `apps/desktop/electron/infrastructure/metadata/node-artwork-cache.ts:105-111` writes UUID `.tmp` records before renaming them. A force quit or crash between those operations bypasses `finally`. `loadIndex` at lines 160-172 subsequently ignores every non-hashed-JSON file, so those thumbnails are never removed or counted by eviction. This violates card line 19: "Bound the cache to 128 MiB or 4,096 records, evicting older covers." Repeated interrupted launches can grow the cache directory without either limit applying. Reproduction with a leftover 1,024-byte UUID temp file and a 100-byte/one-entry configured budget: saving a known-absence record succeeds, leaves both files, and retains 1,112 bytes. Add startup recovery for cache-owned stale temp files and a restart regression proving abandoned write data cannot remain outside the storage budget. Account for concurrent application instances when selecting what is safe to remove.

The remaining implementation covers restart reuse, stored source/revision validation, known absence, bounded record reads, content digests, expected storage failures and queue recovery. The read-only review used a disposable `/tmp` directory for the budget reproduction; no catalogue or repository files were changed. Parent-owned full gates were not duplicated.

# rereview

## Standards

No documented-standard violations or actionable baseline smells found in the final pinned Card 028 diff from 1b687870d472126ea67117ee5b2fb73e268a3ea3 to ff21bf75655c66204e7d959487b7a55a55d8fd57.

The final delta adds filesystem-cache and application-service suites to the existing Windows/macOS matrix and broadens its display label to Native desktop. It follows existing Yarn, pinned-action and package-verification conventions. No production code changed after the reviewed cleanup fix.

Abandoned-write cleanup remains cohesive with the filesystem adapter: PID-tagged names, a process-local active registry and conservative foreign-process liveness checks protect active writes. Expected storage failures retain typed Results, while finally blocks clean temporary writes and registry entries. The application port and metadata fallback policy remain focused and inward-facing.

The history separates the orphan regression from its implementation, adds a real child-process ownership test, and keeps later formatting/documentation distinct. Established pattern/licence records, named bounds, read caps and atomic private writes remain preserved. The twelve-smell baseline raises no actionable findings; tool-enforced concerns are excluded.

The parent reports all ordered local gates passing with 302 tests. Final target-platform CI remains separately tracked.

Findings: 0 hard violations; 0 judgement-call smells.

## Spec

# Spec review: persistent artwork cache, fix cycle

Pinned diff: `1b687870d472126ea67117ee5b2fb73e268a3ea3...ff21bf75655c66204e7d959487b7a55a55d8fd57`.

## Initial P2 verification

Resolved. Card 028 requires "Bound the cache to 128 MiB or 4,096 records, evicting older covers" and now explicitly requires "Remove abandoned atomic writes while preserving another active writer."

`node-artwork-cache.ts` now associates new temporary files with their owner PID and records active same-process writes. Startup inventory removes legacy, abandoned same-process and demonstrably exited-process temporary files before enforcing record budgets. It preserves active in-process writes and foreign writes whose owner is alive or cannot safely be classified. The cleanup therefore addresses the original orphan reproduction without deleting another active instance's write.

The new regression cases require physical directory bytes and file count to stay within a small configured budget after recovery. A real child-process test separately verifies preservation while that process lives and cleanup after it exits.

## (a) Missing or partial requirements

No actionable findings in the pinned implementation.

## (b) Unasked scope

No actionable findings. Owner-aware atomic-write recovery directly supports bounded persistent storage.

## (c) Apparently implemented requirements done incorrectly

No additional actionable findings. Fresh inspection confirms retained source/revision validation before cache lookup, offline Dropbox hits before credential access, owning-account checks on misses, bounded PNG records, integrity verification and expected storage-failure fallback remain intact. The serial artwork queue still recovers after unexpected rejection, and successful cached absence remains distinct from a miss.

The final additional diff expands the existing Windows/macOS native CI job to run the cache adapter and artwork service tests, directly covering cross-platform persistence and owner-aware recovery. Production code remains identical to the reviewed fix. The parent reports all ordered local gates passed with 302 tests; these runs were not duplicated by this read-only review. Native platform CI remains pending. No repository or catalogue files were changed.

Initial: Standards 0, Spec 1 P2. Final: Standards 0, Spec 0.
The abandoned-write issue was reproduced red, fixed, and verified against both
exited and active writer processes. All ordered local gates passed with 302 tests.
