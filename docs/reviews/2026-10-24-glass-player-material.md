# Card 042 review record

Base: `12e1fe68e4dbfd4b50f6587f1057b49f892b424d` (compact player and satin aluminium).
Initial head: `bad099a03997e5f23b6f0843ea3ca8b1b1a976f0`.
Final head: `50870946527d6058c8279660c4c8c45179f9be1e`.
Both cycles inspected the full three-dot diff and commit list.

## Initial Standards

Zero hard violations and zero actionable judgement-call smells.

## Initial Spec

One P2: the queue surface remained translucent when backdrop filtering was not
supported. The existing CSS fallback therefore did not meet the opaque fallback
criterion. The native regression in ec325db removes backdrop support rules and
measures actual canvas alpha. It failed with 224 before the fix. Commit 5087094
adds the opaque queue background before supported-filter overrides; alpha is 255.

## Final Standards

Zero hard violations and zero actionable judgement-call smells.

## Final Spec

Zero actionable findings. The native opaque fallback regression passes.

## Recovery audit

The host restarted before the original temporary reports were copied here.
The summaries above retain the completed review results from the conversation.
Two independent reviewers freshly audited the complete final pin after restart:

### Standards

# Frosted glass Standards recovery review

Base `12e1fe68e4dbfd4b50f6587f1057b49f892b424d`; reviewed head `50870946527d6058c8279660c4c8c45179f9be1e`.
Freshly reviewed the full three-dot diff and twelve-commit list after the reboot, using AGENTS.md and the twelve Fowler smell heuristics.

Hard documented violations: none found.

Judgement-call smells: none actionable.

The material extends the existing typed appearance catalogue and persistence validation without a second storage mechanism or domain dependency. Renderer-only styling is cohesive in glass-material.css, with semantic opaque defaults, supported backdrop filtering and reduced-transparency overrides. Queue styling retains Chromium's native scrollbar, keyboard and wheel behavior, including forced-colour fallback.

The history separates red persistence coverage from its implementation, and the later opaque-fallback regression from its minimal CSS correction. Native integration coverage extends the existing fixture instead of introducing a parallel harness. It checks sixteen compact layouts, queue access and conservative colour-compositing bounds, plus the opaque fallback when backdrop support rules are removed. No production application-error boundary is added or bypassed.

Card 042 records the established CSS pattern and dependencies on the compact player and satin material work. Parent-owned ordered gates and PR delivery remain outstanding and are not certified by this read-only review.

Findings: 0 hard violations; 0 judgement-call smells.


### Spec

# Spec review: Frosted glass recovery audit

Pinned diff: `12e1fe68e4dbfd4b50f6587f1057b49f892b424d...50870946527d6058c8279660c4c8c45179f9be1e`. Reviewed the complete ten-file diff and twelve commits against Card 042 after the reboot.

## Missing or partial requirements

No actionable findings. The seventh material enters the existing appearance model without renaming prior IDs or accents, with light/dark persistence coverage (Card 042 lines 13, 19-20). Glass surfaces use translucent semantic colours, blurred artwork, wash/highlight layers and readable text (line 14).

## Unasked scope

No actionable findings. The implementation stays within renderer material styling, native scrollbar styling, documentation and relevant maintained appearance/native regressions. It adds no custom scrolling engine or native transparent-window behavior.

## Apparently implemented requirements done incorrectly

No actionable findings. The previously identified opaque-queue fallback defect is corrected: glass explicitly sets the queue background to the opaque sidebar token before supported backdrop overrides. The actual renderer regression removes backdrop-support rules and checks alpha 255, satisfying line 15. Reduced transparency removes atmosphere, translucency and wash overlays (line 16).

The queue retains native scrolling with a thin thumb, transparent track, clearer hover/focus state and forced-colour fallback (line 17). The native fixture exercises wheel input, Tab reaching the final row and Enter selecting it (line 18), while sixteen theme/mode/queue/size combinations check geometry, restore behavior and conservative normal/muted contrast bounds including specular wash. Existing Settings coverage derives all seven materials for 56 scenarios.

Findings: 0. This read-only recovery audit did not rerun suites or certify the still pending ordered-gate/PR handoff criterion at line 21.


## Verification

On the final source pin: frozen Yarn install, format check, lint, strict types,
350 tests in 83 files, desktop build, package and Linux ZIP/DEB make all passed.
Native checks cover 16 compact layouts, wheel/Tab/Enter access to the last track,
opaque unsupported-filter fallback, 56 Settings layouts and conservative glass
text contrast over black/white artwork extremes. No dev/main merge or public
release was performed.
