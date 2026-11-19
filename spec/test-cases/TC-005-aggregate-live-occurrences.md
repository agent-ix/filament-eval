---
id: TC-005
title: "Count live occurrences across a frame's transition and a nested session"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-013, type: verifies }
---
# TC-005: Count live occurrences across a frame's transition and a nested session

## Description

Check FR-013's `LiveContext`, the aggregate count across an interpreted
frame that evaluates a call inside one transition, the reported peak and
`SessionLimits::held`, through the toy closure of the scheduler tests. The
toy closure gains an interpreted frame kind that evaluates one nested call
inside its own transition by FR-013 item 4, so the registry path and the
native path can be run over the same frames.

Scope: FR-013-AC-1 to FR-013-AC-5.

## Test Procedure

1. Registry path: register a host body `P` (two `occ`-1 arguments, frame 3)
   that requests the interpreted `C` (frame 4) and propagates its result; `C`
   evaluates `N` (frame 4) inside its transition. Begin `P` with
   `live_occurrences` 10 and run it. Run the native path (interpreted `P`
   evaluating `C` and `N` inside its transitions) with the same frames and
   limit. Read the outcome, its `Incomplete` fields, `location`,
   `frame_depth` and the spent `function.call` units.
2. Repeat both paths with `C` = 3 under `live_occurrences` 10 and under 9; read
   the values and `peak_live_occurrences`.
3. Record each `LiveContext` the closure receives under step 2's limit 10,
   with `P` requesting `C` once and twice in sequence; read
   `Session::live_occurrences` after `P` is pushed and `Session::limits`.
4. Inside `C`'s transition, begin a session with `LiveContext::nested()` whose
   root binds a frame of 4, then of 5; run `count(600000)` under
   `SessionLimits::default()`.
5. Repeat steps 1, 2 and 4 under the no-std build.

## Expected Results

- Step 1: both paths end `Incomplete` with `LiveOccurrences`, `limit` 10,
  `consumed` 7, `next_charge` 4, `FunctionCall`, at `C`'s call of `N`, with
  `frame_depth` 1; three `function.call` units spent; the two outcomes are
  equal.
- Step 2: both complete with equal values and peak 10 under 10; both stop with
  `consumed` 6, `next_charge` 4 under 9.
- Step 3: `C` sees `live` 6, `limit` 10 at each advance, in both runs of `P`;
  `live_occurrences` reads 3; `limits` equals the Begin limits.
- Step 4: the nested session completes holding 10; with a root frame of 5 it
  stops at its root call with `consumed` 6 and `next_charge` 5; `held` defaults
  to 0 and `count(600000)` gives FR-011-AC-1's results.
- No outcome is a `CallFailure`, refusal or fault, and both builds agree.

## Status

Not implemented. Planned in `crates/filament-eval/tests/live_occurrences.rs`.
