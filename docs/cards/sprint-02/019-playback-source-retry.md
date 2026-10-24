- Sprint: 02
- Type: Fix
- Status: In progress
- Blocks: -

# Card 019 - Retry the selected playback source

After a queued source fails, Play must retry that track instead of resuming the
previous audio source. Preserve the queue and the rule that only a successful
start records a play. This follow-up was found during final source-retry verification.

## Acceptance criteria

- [x] Retrying a preparation/download failure prepares the selected track again.
- [x] Retrying an audio-engine rejection prepares a new source and records its successful start.
- [x] A native media error invalidates the failed source without discarding the queue.
- [x] A stale playing event from paused old audio cannot bypass the preparation guard.
- [x] Failed starts do not record a play; a successful retry records the selected track once.
- [x] Rejected resumes through either control invalidate only their matching request.
- [x] Late resume rejection cannot change a newer track's state.
- [ ] All verification commands and Linux packaging pass.
- [ ] Initial review and fix re-review are complete.

## Pattern and scope

Reuse the player's existing source preparation and request-invalidation path.
Howler's MIT-licensed core play implementation waits for a loaded source before
starting playback: https://github.com/goldfire/howler.js/blob/master/src/howler.core.js
That loading-state pattern informs this guard; no external code is copied and no
new player abstraction or dependency is added.

Eight regression cases cover preparation rejection, engine rejection, native error,
delayed playing-event delivery, both resume controls, and late resume rejection. They were committed red before their focused
implementations. Explicit pause/resume of a valid source retains its current position.

Depends on PR #45 and the desktop completion stack. Merge after its base, then rebase
and retarget the dependent branch as needed. No merge is authorized by this card.

Initial Spec review found the resume-rejection path still marked its source valid.
Separate red/green corrections cover both controls and newer-track isolation; a
subsequent refactor shares the guarded resume operation between its two callers.
