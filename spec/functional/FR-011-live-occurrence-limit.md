---
id: FR-011
title: Stop a session incomplete at its live-occurrence limit
type: FR
relationships:
  - target: ix://agent-ix/filament-spec-language/NFR-130
    type: implements
  - target: ix://agent-ix/filament-spec-language/FR-262
    type: implements
  - target: ix://agent-ix/filament-spec-language/FR-255
    type: implements
---
# FR-011: Stop a session incomplete at its live-occurrence limit

## Description

A `Session` SHALL count the live occurrences of the call frames it holds
against a live-occurrence limit set on the session, and SHALL end the
application with `Outcome::Incomplete` of the limit kind
`LimitKind::LiveOccurrences` at the call that would exceed it, before that
call's frame is pushed.

## Rationale

The work budget bounds memory only through the work charged so far, which is
cumulative, so a caller who raises `work_units` also lifts that bound. The
session owns the frame stack and binds each call's arguments, so it is the
only place that knows how many frames and argument values are held at one
moment. A count of occurrences is defined on values, falls when calls return,
and gives the same result on every platform, unlike a byte or depth count.

## Behavior

### The limit

1. `SessionLimits { live_occurrences: u64, held: u64 }` SHALL be a public
   `Copy` type whose `Default` sets `live_occurrences` to
   `DEFAULT_LIVE_OCCURRENCES`, a public constant equal to 1,048,576 (the
   default of FSL's `evaluation.live_occurrences` setting), and `held` to 0.
   `held` is the count an enclosing application already holds when a session
   begins inside one of its frames (FR-013).
2. `Registry::begin_call` and `Registry::begin_expression` SHALL take a
   `SessionLimits` and bind it to the returned session for its life. Every
   `u64` is a valid limit: 0 is a real limit that the first call exceeds, and
   `u64::MAX` is the largest. The caller supplies the bound; this crate reads
   no ambient setting.

### What counts

3. A frame's live occurrences are one for the frame plus `occ(v)`
   (`filament.value.accounting/v1`'s occurrence count, as `filament-exact`
   computes it) of each value bound in it at entry: the arguments of a call
   frame, or the parameters of an expression-root frame. Interpreted frames and
   registered host-body frames count alike.
4. The session's live count is `held` plus the sum over every frame on its
   stack; it starts at `held`. A frame that evaluates calls inside one
   transition counts them in the `LiveContext` the session lends it, against
   the same count and limit; frames it carries open from one transition to the
   next stay in the count (FR-013 item 7). A
   frame's occurrences enter the count when it is pushed and leave it when its
   body finishes and the frame is popped, and a dropped session holds none.
   Values a body computes or holds after entry are not counted; the work
   budget bounds them.

### When the session checks

5. When the session enters a call, at the root of `begin_call` or at a body's
   saved request, it SHALL first charge `function.call` on the meter (a denied
   charge ends the application as today). After that charge is granted and
   before it binds the arguments, it SHALL compute the new frame's
   occurrences and test whether the live count plus them exceeds
   `live_occurrences`. The arithmetic is checked: a sum past `u64::MAX`
   exceeds every limit.
6. An expression root's frame is added to the count when it is pushed and is
   not tested on its own, because an expression root makes no call; the test
   happens at each call it reaches, with the root frame included.

### The incomplete result

7. When the count would exceed the limit, the session SHALL end the
   application with the terminal `Evaluation` whose outcome is
   `FamilyOutcome::Evaluated(Outcome::Incomplete(Incomplete { .. }))` with
   `limit_kind: LimitKind::LiveOccurrences`, `limit` the configured
   `live_occurrences`, `consumed` the live count before the call,
   `next_charge` the new frame's occurrences and `charge_point:
   ChargePoint::FunctionCall`, located at the call site, with no losses. The
   callee is not bound and no frame is pushed. The granted `function.call`
   charge stays spent; earlier spend is kept. Suspended frames are released
   without executing their continuations. A live-occurrence `Incomplete`
   that a frame returns from inside its transition ends the application in the
   same way (FR-013 item 9).
8. Reaching the limit is incomplete, never a refusal, a `CallFailure` or an
   `InternalFault`, and names no depth or frame count.
9. The terminal `Evaluation` of every application SHALL carry
   `peak_live_occurrences: u64`, the largest live count the application
   reached: `held`, the session's frames, the frames opened inside its frames'
   transitions and the peaks of sessions nested in them (FR-013 item 6), the
   frame of a call stopped by this limit excluded.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-011-AC-1 | The interpreted recursion `count(600000)`, which makes 600,001 calls (the root and 600,000 nested), each binding one `Integer` argument (two occurrences per frame), begun with `SessionLimits::default()` on a meter with `work_units` `u64::MAX`, ends `Incomplete` with `LimitKind::LiveOccurrences`, `limit` 1,048,576, `consumed` 1,048,576, `next_charge` 2 and `charge_point` `FunctionCall`, at the request site of the 524,289th call, and the meter has spent exactly 524,289 `function.call` units; `count(1000)` (1,001 calls) completes. | Test (TC-003) |
| FR-011-AC-2 | With `live_occurrences` 4,000,000 `count(600000)` completes all 600,001 calls and its `peak_live_occurrences` is 1,200,002 (600,001 frames of two occurrences); with `live_occurrences` 0 a one-argument call stops `Incomplete` at its root call with `consumed` 0 and `next_charge` 2, binding nothing. | Test (TC-003) |
| FR-011-AC-3 | Frames leave the count when they return: a body that calls a one-argument callee 10 times in sequence under `live_occurrences` 4 completes, and its `peak_live_occurrences` is 4. A host-body frame counts like an interpreted frame. | Test (TC-003) |
| FR-011-AC-4 | An argument of `occ` 5 makes its frame count 6; an expression root binding parameters of total `occ` 3 that calls a one-argument callee under `live_occurrences` 5 stops `Incomplete` at that call with `consumed` 4 and `next_charge` 2. | Test (TC-003) |
| FR-011-AC-5 | No outcome of the limit is a `CallFailure`, a refusal or an `InternalFault`; the stopped call's continuation is never resumed, including when the stop arises inside a frame's transition (FR-013-AC-1), and the outcome is identical under the std and the no-std build. | Test (TC-003) |

## Dependencies

- FSL NFR-130 owns the bound, its count and its default; FR-255 names the `evaluation.live_occurrences` setting that raises it; FR-262 owns the session's explicit task stack.
- `filament-exact` owns `LimitKind`, including `LiveOccurrences` (normative name `live_occurrences`, outside the ten `ScalarLimitsV1` counters of `LimitKind::ALL`), `Incomplete`, `ChargePoint::FunctionCall` and `occ`.
- FSL decodes `call.evaluation.live_occurrences`, passes it as `SessionLimits`, and maps the incomplete outcome to `stage_limit_exceeded` naming `evaluation.live_occurrences`.
