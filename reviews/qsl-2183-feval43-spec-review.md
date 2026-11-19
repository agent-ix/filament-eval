---
id: SR-2026111921471500000001
title: "spec-review of QSL-2183 FR-013 aggregate live-occurrence count across nested evaluation"
type: SpecReview
analysis: integrity
scope: "agent-ix/filament-eval; PR #43 at 993fde4a; spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md, spec/test-cases/TC-005-aggregate-live-occurrences.md, spec/spec.md; context: FR-011, FR-012, crates/filament-eval/src/session.rs, closure.rs, body.rs, registry.rs; FSL fsl-eval/src/value/expression/closure.rs, fsl-replay/src/application.rs, tests/it/spine_run.rs, FR-401 item 4"
review_set: subset
---

## Summary

Spec review of filament-eval PR #43. Ticket: QSL-2183. The PR adds FR-013 (a public `LiveContext { live, limit }` passed to `CheckedClosure::advance`, in-transition counting from `context.live`, peak reporting, `SessionLimits::held`, `LiveContext::nested()`, `Session::live_occurrences()` and `Session::limits()`) and TC-005. `quire validate` passes on all three files. FR-013 and TC-005 are free on origin/main and in the other open PR (#42).

## Examined scope

- Current-behaviour claims. `Session.live` and `peak` are private fields with no accessor (session.rs:107-109); `SessionLimits` has only `live_occurrences` with default 1,048,576; `advance` has exactly one caller (session.rs:185) and two implementors (the test toy in tests/support/mod.rs and FSL's `PackageClosure`); FSL's std adapter runs a whole body in one `advance` (closure.rs:289-330, FR-401 item 4). All hold. (examined)
- Arithmetic. Registry AC-1: P pushed (3), C admitted 3+4=7 and pushed, C's context live 7, N tested 7+4=11>10, so consumed 7, next_charge 4. Native AC-1: 3 + C 4 = 7 in-transition, N 11>10, consumed 7. AC-2: 6+4=10 completes at peak 10; under 9, consumed 6. AC-3: C sees live 6. AC-4: held 6, 6+4=10 completes, 6+5=11 stops consumed 6 next_charge 5. Three `function.call` units in AC-1. All correct, and "live includes the frame being advanced" gives exactly 7 consumed at N. (examined)
- Failing power. An adapter ignoring the context completes AC-1 (4+4=8); double-counting C gives consumed 11; passing live without C is pinned by AC-3's `live` 6; a session ignoring the reported peak fails AC-2's peak 10. (examined)
- Stop delivery and `frame_depth` (FND-001, FND-002), host-body and in-transition nested sessions (FND-003, FND-004), peak coverage (FND-005), FR-011 consistency (FND-006), rationale accuracy (FND-007). (examined)
- API break. Pre-release, no shim needed; the only `SessionLimits { .. }` literal is tests/live_occurrences.rs:25. (examined)

## Verdict

Not mergeable yet: four medium findings. The arithmetic, the ACs' failing power and the claims about today's code are right. The stop's delivery wording and the native `frame_depth` rule must be made consistent with FR-011/FR-012, and the nested-session rule must be reachable by host bodies and count frames held open inside a transition. Merge before FSL #3109.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Item 5 says the session delivers an in-transition stop to the caller frame "exactly as it delivers a stop it raised itself", but a session-raised live-occurrence stop is never delivered: it ends the application and releases suspended frames without running their continuations (FR-011 item 7, FR-011-AC-5; session.rs `enter` returns `Ok(Some(..))`, which `step` makes terminal). Under FR-013 the registry path resumes P's continuation with the stop, so path equality depends on P propagating it. State one rule: either the session ends the application on a `LiveOccurrences` incomplete returned by an interpreted frame, or the stop is delivered and the claim is removed | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:75-82 |
| FND-002 | medium | FR-013 never says how frames opened inside a transition count for `frame_depth`. FR-012 item 2 says a body that originates a non-completed outcome returns `frame_depth` 0, which gives the native path of AC-1 depth 0, yet AC-1 requires it to equal the registry path's 1. Specify that frames held open inside the transition count as FR-012 levels (the stop's depth is the number of such frames above the advanced frame) | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:75-82 |
| FND-003 | medium | Item 7 requires a host body that begins a session inside a frame to begin it with `LiveContext::nested()`, but `Body::resume(input, meter)` receives no `LiveContext` and item 9 keeps that contract unchanged, so a host body cannot comply and its nested sessions stay uncounted | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:90-99 |
| FND-004 | medium | `LiveContext::nested()` sets `held` to `context.live`, which excludes the frames the closure holds open inside its transition (item 4). A session begun inside such a frame (native P evaluating C inline, C begins a session) starts at 3 instead of 7 and undercounts. AC-4 covers only the case with no open in-transition frames | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:90-99 |
| FND-005 | low | Item 6 reports a frame's in-transition peak only in `Transition::Done`, yet concludes that every terminal evaluation carries the whole application's peak. A frame that evaluates calls inline and then returns `Transition::Call`, followed by a terminal session stop or failure before its `Done`, loses that peak. Folding a nested session's `peak_live_occurrences` into the frame's peak is also unstated | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:83-89 |
| FND-006 | low | FR-011 items 1, 4 and 9 still define `SessionLimits { live_occurrences: u64 }` and the live count as the sum over the stack's frames. FR-013 adds `held` and in-transition frames without amending FR-011, so the two FRs describe the type and the count differently | spec/functional/FR-011-live-occurrence-limit.md |
| FND-007 | low | The rationale presents "the frame tests C + N" and the native stop at N as today's behaviour, but FSL's std evaluator has no live-occurrence test yet: NFR-130's TC-2164 and TC-2165 are `#[ignore = "QSL-2183: ... not implemented yet"]`. This is the target behaviour, not the current one | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:27-40 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | The implementor list in item 2 misses agent-ix/filament-contract-codegen, which consumes `filament_eval::Body`. Its generator emits two `impl Body for Invocation` templates (src/oracle/function/owner_render.rs:361 and 440, with a source assertion at 542), and its tests define `EchoBody` and `SitedCallBody` (owner_path.rs:856 and 890). Two of its callers run a body outside any session: the generated test helper `invoke_native_body` (tests/it/exact_function_agreement.rs:1727) and the Kani harness (tests/it/kani_function_harness.rs:285). Item 1's "No public constructor exists; only a session makes one" leaves those callers no way to supply the new `&mut LiveContext`. Name CG in the list and give out-of-session callers a way to build a context (for example a public `LiveContext::new(limit)`), or state CG's migration | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:61-80 |
| FND-009 | low | Item 6's "frames a transition opened end with that transition" is a description, not an obligation. Each advance gets a fresh context, so a closure that returns `Continue` or `Call` with in-transition frames still open undercounts them on its next transition. Make it a SHALL on the frame (close every opened frame before returning), or carry the open frames' count across transitions | spec/functional/FR-013-aggregate-live-occurrences-across-nested-evaluation.md:109-116 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Item 8 now ends the application on a live-occurrence `Incomplete` returned by a frame; FR-011 item 7, FR-011-AC-5 and FR-012 item 3 were amended to match; AC-1 asserts that P is never resumed with the child |
| FND-002 | fixed | Item 7 and FR-012 items 1-2 count frames opened inside a transition as levels. Item 8 adds the session frames above the selected frame. Native AC-1 gives 0+1=1 and registry gives 1+0=1, matching FR-485-AC-19 |
| FND-003 | fixed | `Body::resume` takes `&mut LiveContext` (item 2), and item 5 binds host bodies to `nested()` and `reach`; AC-4 covers a host body (`held` 3) |
| FND-004 | fixed | `nested()` reads `live` at call time, including open in-transition frames; AC-4 native gives held 7 (3+4), root 3 completes at 10, root 4 stops with consumed 7 |
| FND-005 | fixed | Item 6 raises the session peak after every advance or resume, whatever the frame returned, and `reach` folds nested peaks in. AC-5 gives 3+4+4=11 through a later `Call` and a denied charge, plus a host body nested peak of 9 |
| FND-006 | fixed | FR-011 items 1, 4, 7 and 9 and AC-5 now carry `held`, in-transition frames and the in-transition stop |
| FND-007 | fixed | The rationale now says this is the target behaviour and that FSL's std evaluator has no live-occurrence test yet |
