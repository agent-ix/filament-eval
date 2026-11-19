---
id: FR-013
title: Count live occurrences across a session and the evaluation nested in its frames
type: FR
relationships:
  - target: ix://agent-ix/filament-eval/FR-011
    type: references
  - target: ix://agent-ix/filament-eval/FR-012
    type: references
  - target: ix://agent-ix/filament-spec-language/NFR-130
    type: implements
  - target: ix://agent-ix/filament-spec-language/FR-485
    type: implements
  - target: ix://agent-ix/filament-spec-language/FR-262
    type: implements
---
# FR-013: Count live occurrences across a session and the evaluation nested in its frames

## Description

A `Session` SHALL hand every frame it advances, interpreted or registered
host body, a `LiveContext` that holds the application's live-occurrence
count and limit. A frame that evaluates calls inside one transition, or begins
a session inside one, SHALL count those calls through that context. One
application SHALL have one live-occurrence count, one limit and one peak,
whatever mix of host bodies, interpreted frames, calls evaluated inside a
transition and nested sessions holds its frames. A live-occurrence stop SHALL
end the application wherever it arises.

## Rationale

FR-011 counts the frames on the session's own stack. A closure whose
interpreted frame runs a whole body in one transition holds further frames
that the session never sees. FSL's std reference adapter does this: it keeps
its nested calls on its own task stack. Without the session's count, such a
frame can only count from its own frame upward.

Take a registered host body `P` that requests that kind of frame `C`, where
`C` calls `N` inside its transition. The session can test `P + C`, and the
package's machine can test only `C + N`; neither sees `P + C + N`. With
`P` = 3, `C` = 4, `N` = 4 and a limit of 10, the two partial tests pass (7
and 8) while the application holds 11. This is the target behaviour, not
today's: FSL's std evaluator has no live-occurrence test yet, and its
run-level tests of the limit are still ignored. A `C + N` check would be the
package machine's own count, not an application bound. A memory bound that
depends on whether the root is a host body or an interpreted function is not
a bound, so the count has to cross the frame boundary.

The session owns the stack and the limit. It therefore lends both to the
frame for one transition, and the frame counts its in-transition calls in
that same context. The context also gives a nested session its starting
count and collects every peak. A stop raised inside a transition ends the
application exactly like a stop the session raises itself. The registry path
and the native path then return the same `Incomplete`, the same `location`
and the same `frame_depth`.

## Behavior

### The context a frame runs under

1. `LiveContext` SHALL be a public type with the private `u64` fields `live`,
   `limit` and `peak`, a private count of the frames opened inside the
   current transition and still open, and the read methods `live()`, `limit()` and
   `peak()`. A session makes one for every advance or resume (item 2).
   `LiveContext::new(limit: u64) -> LiveContext` SHALL be a public
   constructor with `live` 0 and `peak` 0, for a caller that runs a body
   outside any session: a generated test helper that calls `Body::resume`
   directly, or a model-checking harness. A context made by `new` belongs to
   no application; whatever a body counts in it bounds only that direct call.
2. `CheckedClosure::advance` SHALL take `context: &mut LiveContext` after
   `delivered`, and `Body::resume` SHALL take `context: &mut LiveContext`
   after `input`: `resume(&mut self, input, context, meter)`. Before each
   advance or resume, the session SHALL make a fresh context whose `live` is
   the session's live count (FR-011, `held` included) with the frame being
   advanced included, whose `limit` is `SessionLimits::live_occurrences` and
   whose `peak` equals `live`. A frame resumed with a child's evaluation sees
   the count after that child's frame was popped. The public `Body`
   implementors that change with this signature are `FailureBody` and
   `FalseBody` (`tests/body_failure.rs`), and `RelayBody`, `ChargingBody`,
   `BadRequestBody`, `YieldingBody`, `EchoBody` and `SequenceBody`
   (`tests/support/mod.rs`) in this crate. In FSL they are `HostBody`,
   `YieldingBody` and `SitedBody` (`fsl-replay/src/application.rs` tests),
   `ActionBody` (`fsl-replay/tests/it/checked_body_failure.rs`), `RequestBody`
   and `ConstantBody` (`fsl-eval/tests/it/reference_adapter.rs`), and
   `GeneratedCall` (`tests/fixtures/checked-closure-consumer`). In
   filament-contract-codegen they are the two generated `impl Body for
   Invocation` templates (`src/oracle/function/owner_render.rs`, with the
   rendered-text assertion that pins them) and the test bodies `EchoBody` and
   `SitedCallBody` (`tests/it/owner_path.rs`). That crate also calls
   `Body::resume` outside a session, in the generated test helper
   `invoke_native_body` (`tests/it/exact_function_agreement.rs`) and the Kani
   harness (`tests/it/kani_function_harness.rs`), which make their context
   with `LiveContext::new`. Its migration is to render the new `resume`
   signature, pass a `LiveContext::new` context in those direct calls, and pass
   `SessionLimits` to FSL's five-argument `ApplicationRegistry::begin_call`
   (FSL FR-485), all in the change that advances its `filament-eval` pin. The
   crate is pre-release, and no compatibility signature is kept.
3. `Session::live_occurrences(&self) -> u64` SHALL return the session's
   current live count, and `Session::limits(&self) -> SessionLimits` the
   limits bound at Begin. Both are reads; neither charges the meter nor
   changes the session.

### How nested calls add

4. A frame that evaluates an application inside one transition, instead of
   returning it as `Transition::Call`, SHALL charge its `function.call` and
   then call `LiveContext::open(&mut self, occurrences: &Integer) ->
   Result<u64, Incomplete>` with the new frame's occurrences (FR-011 item 3)
   before binding. `open` SHALL test `live` plus the occurrences against
   `limit` with checked arithmetic. When the test passes, it adds them to
   `live`, raises `peak` to `live` and returns them as a `u64`. When it fails,
   it changes nothing and returns `Incomplete { limit_kind:
   LimitKind::LiveOccurrences, limit, consumed: live, next_charge:
   occurrences, charge_point: ChargePoint::FunctionCall }`.
   `LiveContext::close(&mut self, occurrences: u64)` SHALL subtract a returned
   frame's occurrences. Frames opened inside a transition therefore add to and
   leave the same count as the session's own frames.
5. `LiveContext::nested(&self) -> SessionLimits` SHALL return
   `SessionLimits { live_occurrences: limit, held: live }` at the moment it is
   called, so it includes every frame opened inside the transition and still
   open. A frame, interpreted or host body, that begins a session inside its
   transition SHALL begin it with these limits. When that session ends, the
   frame SHALL call `LiveContext::reach(&mut self, peak: u64)` with the nested
   evaluation's `peak_live_occurrences`; `reach` raises `peak` to that value
   when it is larger.
6. After every advance or resume, whatever the frame returned
   (`Transition::Continue`, `Transition::Call`, `Transition::Done`, a stop or
   a failure), the session SHALL raise its own peak to the context's `peak`
   before it acts on the transition. A frame's peak therefore covers the calls
   it evaluated inside the transition and the nested sessions it ran. Every
   terminal or delivered `peak_live_occurrences` is the largest count of the
   whole application.
7. A frame that returns `Transition::Continue` or `Transition::Call` while
   frames it opened inside the transition are still open SHALL carry them to
   its next transition, and the session SHALL count them meanwhile. After such
   a return, the session SHALL add the context's `live` minus the `live` it
   lent to that frame's occurrences on its stack, and add the number of frames
   still open (`open` adds one, `close` removes one) to that frame's carried
   frame count. Calls requested meanwhile are tested with the carried
   occurrences included. The next context lent to the frame includes them,
   and they leave the count when the frame closes them or is popped. For
   FR-012, a frame's carried frames count as levels above it. A frame that
   returns `Transition::Done` ends every frame it opened.

### Where a live-occurrence stop ends the application

8. A frame that receives `Err(Incomplete)` from `open` SHALL return
   `Transition::Done` with the located `Evaluation` of FR-011 item 7, carrying
   that `Incomplete`, located at the refused call's site with no losses. Its
   `frame_depth` is the number of frames the frame holds open inside the
   transition at that call (FR-012 item 2). A frame that receives a
   live-occurrence `Incomplete` as the result of a nested session SHALL return
   it the same way.
9. When a frame returns `Transition::Done` whose outcome is
   `Outcome::Incomplete` with `limit_kind: LimitKind::LiveOccurrences`, the
   session SHALL end the application with that evaluation, exactly as it
   ends one at a stop it raises itself (FR-011 item 7). No caller frame is
   resumed with it, and suspended frames are released without executing
   their continuations. The terminal `frame_depth` is the number of session
   frames above the selected frame, the returning frame included, plus the
   returned `frame_depth`, saturating at `u32::MAX`. Neither FSL's
   interpreter nor a host body propagates this stop: the session ends the
   application before either could resume.

### What does not change

10. The stop is incomplete, never a refusal, a `CallFailure` or an
   `InternalFault` (FR-011 item 8). A frame whose calls are all returned as
   `Transition::Call` (requests the session counts) never calls `open`, and
   its context's `peak` stays at `live`. The order at a call stays charge,
   occurrence test, binding.

## Acceptance Criteria

Each criterion uses frames named by their counted occurrences (FR-011): `P`
is the root frame, `C` the frame `P` calls, `N` the frame `C` calls. In the
native path, `P` is an interpreted frame that evaluates `C` and `N` inside its
transitions. In the registry path, `P` is a registered host body that requests
`C` by `Transition::Call`, and `C` is an interpreted frame that evaluates `N`
inside one transition.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-013-AC-1 | With `P` = 3, `C` = 4, `N` = 4 and `live_occurrences` 10, the registry path admits `C` at the session (count 7), stops at `N` inside `C`'s transition and ends the application. The terminal outcome is `Incomplete { limit_kind: LiveOccurrences, limit: 10, consumed: 7, next_charge: 4, charge_point: FunctionCall }`, located at `C`'s call of `N`, with `frame_depth` 1. The host body `P` is resumed once (with `Resume::Start`) and never with the child's evaluation. The native path with the same frames ends with an equal `Incomplete`, `location` and `frame_depth` 1. In both, the meter has spent three `function.call` units and `N` is not bound. | Test (TC-005) |
| FR-013-AC-2 | With `P` = 3, `C` = 3, `N` = 4 and `live_occurrences` 10, the registry path and the native path both complete with equal values and `peak_live_occurrences` 10; with `live_occurrences` 9 both stop at `N` with `consumed` 6 and `next_charge` 4. | Test (TC-005) |
| FR-013-AC-3 | Under the registry path of AC-2, `P`'s resume sees `live` 3, and `C`'s advance sees `live` 6 and `limit` 10. A host body that requests `C` twice in sequence sees `live` 6 at each of `C`'s advances, because the first `C` was popped. During the session, `Session::live_occurrences` reads 3 after `P` is pushed, and `Session::limits` returns the limits given at Begin. | Test (TC-005) |
| FR-013-AC-4 | A session begun inside `C`'s transition in the registry path of AC-2 with `nested()` (`live_occurrences` 10, `held` 6) holds 10 and completes when its root binds a frame of 4. With a root frame of 5 it stops at its root call with `consumed` 6 and `next_charge` 5, and that stop ends the outer application. In the native path, where `P` (3) holds `C` (4) open inside its transition, `nested()` gives `held` 7, not 3: a nested root frame of 3 completes at 10, and one of 4 stops with `consumed` 7. A host body `P` (3) that begins a session with `nested()` gives `held` 3. A session begun with `SessionLimits::default()` has `held` 0, and FR-011-AC-1's results are unchanged. | Test (TC-005) |
| FR-013-AC-5 | An interpreted root `P` (3) under `live_occurrences` 20 opens `C` (4) and `N` (4) inside its transition, closes both, and then returns `Transition::Call` for `D` (2) on a meter that grants three `function.call` charges. The application ends `Incomplete` at `D`'s `function.call` with `peak_live_occurrences` 11. A host body whose nested session reaches a peak of 9 under `nested()` reports, through `reach`, a terminal peak of 9. | Test (TC-005) |
| FR-013-AC-6 | No stop of AC-1, AC-2, AC-4, AC-5 or AC-7 is a `CallFailure`, a refusal or an `InternalFault`, and each outcome is identical under the std and the no-std build. | Test (TC-005) |
| FR-013-AC-7 | An interpreted root `P` (3) under `live_occurrences` 20 opens `C` (4) and `N` (4) inside its transition and returns `Transition::Call` for `D` (2) with both still open. `Session::live_occurrences` reads 11 before `D` is entered, `D` is admitted at a count of 13, and `D`'s advance sees `live` 13. After `D` returns, `P`'s next context has `live` 11; `P` closes both frames and completes with `peak_live_occurrences` 13. Under `live_occurrences` 12, `D` stops with `consumed` 11, `next_charge` 2 and `frame_depth` 2, and the application ends. | Test (TC-005) |
| FR-013-AC-8 | `LiveContext::new(10)` has `live` 0, `peak` 0 and `limit` 10. A host body resumed directly with it, outside any session, that opens frames of 4 and 4 gets `Ok` for both (`live` 8, `peak` 8). A third `open` of 4 returns `Incomplete { limit_kind: LiveOccurrences, limit: 10, consumed: 8, next_charge: 4, charge_point: FunctionCall }` and leaves `live` 8. | Test (TC-005) |

## Dependencies

- FR-011 owns the per-frame count, the limit, the stop's shape and the call
  order, and FR-012 owns `frame_depth`. This requirement extends both across a
  frame's transition and across a nested session.
- FSL NFR-130 states the bound over every call frame in progress. FSL FR-485
  passes the caller's `SessionLimits` through `ApplicationRegistry::begin_call`,
  and FSL's std reference adapter counts its task stack through `LiveContext`.
- `filament-exact` owns `LimitKind::LiveOccurrences`, `Incomplete`,
  `ChargePoint::FunctionCall`, `Integer` and `occ`.
