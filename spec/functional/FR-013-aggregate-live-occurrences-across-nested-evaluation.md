---
id: FR-013
title: Count live occurrences across a session and the evaluation nested in its frames
type: FR
relationships:
  - target: ix://agent-ix/filament-eval/FR-011
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

A `Session` SHALL tell every interpreted frame it advances how many live
occurrences the application holds and what its limit is, through a public
`LiveContext`, so that an interpreted frame that evaluates calls inside one
transition, and a session begun inside a frame, test each such call against
the whole application's live count. One application SHALL have one
live-occurrence count and one limit, whatever mix of host bodies,
interpreted frames and nested sessions holds its frames.

## Rationale

FR-011 counts the frames on the session's own stack. A closure whose
interpreted frame runs a whole body in one transition (the std reference
adapter of FSL, which keeps its nested calls on its own task stack) holds
further frames the session never sees. Without the session's count, such a
frame can only count from its own frame upward. When a registered host body
`P` requests that frame `C`, and `C` calls `N` inside its transition, the
session tests `P + C` and the frame tests `C + N`, and neither sees `P + C + N`.
With `P` = 3, `C` = 4, `N` = 4 and a limit of 10 both tests pass (7 and 8)
while the application holds 11. The same three frames reached through a root
whose own frame is interpreted are counted in one sum and stop at `N`. A
memory bound that depends on whether the caller is a host body or an
interpreted function is not a bound, so the count has to cross the boundary.

The session owns the stack and the limit, so it supplies both to the frame;
the frame reports back the largest count it reached. A nested session takes
the count already held as its starting point. The stop then carries the
same `consumed`, `limit` and `next_charge` whichever path reached it.

## Behavior

### The context a frame runs under

1. `LiveContext { live: u64, limit: u64 }` SHALL be a public `Copy` type.
   `live` is the session's live count (FR-011) with the frame being advanced
   included; `limit` is the session's `SessionLimits::live_occurrences`.
2. `CheckedClosure::advance` SHALL take a `LiveContext` argument, after
   `delivered`. The session SHALL pass, at every advance of an interpreted
   frame, the context computed at that moment. The context of one frame
   changes between its transitions only through the frames above it, so a
   frame resumed with a child's evaluation sees the count after that child's
   frame was popped.
3. `Session::live_occurrences(&self) -> u64` SHALL return the session's
   current live count, and `Session::limits(&self) -> SessionLimits` the
   limits bound at Begin. Both are reads; neither charges the meter nor
   changes the session.

### How nested calls add

4. A closure that evaluates an application inside one `advance`, instead of
   returning it as `Transition::Call`, SHALL count that application's frame by
   FR-011's rule (one plus `occ` of each bound argument) and SHALL test it,
   after its `function.call` charge is granted and before binding, against
   `context.limit` with the count `context.live` plus the occurrences of the
   frames it already holds open inside this transition. Frames it opens add
   to that count and leave it when their calls return, as on the session's
   own stack.
5. When that test fails, the closure SHALL return `Transition::Done` with the
   located `Evaluation` FR-011 item 7 specifies, with `limit` equal to
   `context.limit`, `consumed` equal to `context.live` plus the occurrences of
   the frames it holds open inside the transition, `next_charge` the refused
   frame's occurrences and `charge_point: ChargePoint::FunctionCall`, located
   at the refused call's site. The session SHALL deliver it to the caller
   frame unchanged except for FR-012's `frame_depth`, exactly as it delivers
   a stop it raised itself.
6. The `Evaluation` an `advance` returns in `Transition::Done` SHALL carry in
   `peak_live_occurrences` the largest count the frame reached, as an
   application-wide count (at least `context.live`). Before it pops the
   frame, the session SHALL raise its own peak to that value when it is
   larger. A terminal or delivered evaluation's `peak_live_occurrences` is
   therefore the largest count of the whole application, the frames held
   inside interpreted transitions included.
7. `SessionLimits` SHALL gain the public field `held: u64`, the live
   occurrences an enclosing application already holds when this session
   begins; `Default` sets it to 0. A session SHALL start its live count and its
   peak at `held`, and FR-011's tests, `consumed` and `peak_live_occurrences`
   SHALL include it. `LiveContext::nested(&self) -> SessionLimits` SHALL return
   `SessionLimits { live_occurrences: self.limit, held: self.live }`; a closure
   or host body that begins a session inside a frame SHALL begin it with these
   limits, so the nested session's frames add to the enclosing count. A
   `held` greater than `live_occurrences` is a valid limit set: the first call
   exceeds it.
8. A closure that returns every application as `Transition::Call` reads the
   context and needs nothing more: the session counts every frame. A frame
   that evaluates no application inside a transition reports
   `peak_live_occurrences` equal to `context.live`.

### What does not change

9. The stop is incomplete, never a refusal, a `CallFailure` or an
   `InternalFault` (FR-011 item 8). The registered host-body contract
   (`Body::resume`) is unchanged: a host body's calls are requests the session
   counts. The order at a call stays charge, then the occurrence test, then
   binding.

## Acceptance Criteria

Each criterion uses frames named by their counted occurrences (FR-011): `P`
is the root frame, `C` the frame `P` calls, `N` the frame `C` calls. "Native"
means `P` is an interpreted frame and `C` and `N` are evaluated inside its
transitions; "registry" means `P` is a registered host body that requests `C`
by `Transition::Call`, and `C` is an interpreted frame that evaluates `N`
inside one transition.

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-013-AC-1 | With `P` = 3, `C` = 4, `N` = 4 and `live_occurrences` 10, the registry path admits `C` at the session (count 7) and stops at `N` inside `C`'s transition with `Incomplete { limit_kind: LiveOccurrences, limit: 10, consumed: 7, next_charge: 4, charge_point: FunctionCall }` located at `C`'s call of `N`; the host body `P`, propagating it, ends the application with that same `Incomplete`, `location` and `frame_depth` 1, and the native path with the same frames ends with an equal `Incomplete`, `location` and `frame_depth`. In both, the meter has spent three `function.call` units and `N` is not bound. | Test (TC-005) |
| FR-013-AC-2 | With `P` = 3, `C` = 3, `N` = 4 and `live_occurrences` 10, the registry path and the native path both complete with equal values and `peak_live_occurrences` 10; with `live_occurrences` 9 both stop at `N` with `consumed` 6 and `next_charge` 4. | Test (TC-005) |
| FR-013-AC-3 | A closure records the `LiveContext` of every `advance`: under the registry path of AC-2, `C`'s first advance sees `live` 6 and `limit` 10. A host body that requests `C` twice in sequence sees, at each of `C`'s advances, `live` 6 (the first `C` was popped). During the session `Session::live_occurrences` reads 3 after `P` is pushed and `Session::limits` returns the limits given at Begin. | Test (TC-005) |
| FR-013-AC-4 | A session begun inside `C`'s transition with `LiveContext::nested()` (here `live_occurrences` 10, `held` 6), whose root binds a frame of 4, holds 10 and completes; with a root frame of 5 it stops at its root call with `consumed` 6 and `next_charge` 5. A session begun with `SessionLimits::default()` reports `held` 0 and FR-011-AC-1's results unchanged. | Test (TC-005) |
| FR-013-AC-5 | No stop of AC-1, AC-2 or AC-4 is a `CallFailure`, a refusal or an `InternalFault`, and each outcome is identical under the std and the no-std build. | Test (TC-005) |

## Dependencies

- FR-011 owns the per-frame count, the limit, the stop's shape and the call
  order; FR-012 owns `frame_depth`. This requirement extends that count
  across a frame's transition and across a nested session.
- FSL NFR-130 states the bound over every call frame in progress; FSL FR-485
  passes the caller's `SessionLimits` through `ApplicationRegistry::begin_call`;
  FSL's std reference adapter reads `LiveContext` to seed its own task stack's
  count and reports its peak.
- `filament-exact` owns `LimitKind::LiveOccurrences`, `Incomplete`,
  `ChargePoint::FunctionCall` and `occ`.
