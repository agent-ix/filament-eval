---
id: TC-005
title: "Count live occurrences across a frame's transition and a nested session"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-013, type: verifies }
---
# TC-005: Count live occurrences across a frame's transition and a nested session

## Description

Check FR-013's `LiveContext`, the aggregate count across a frame that
evaluates calls inside one transition, the end of the application at an
in-transition stop, the reported peak and `SessionLimits::held`. The tests
use the toy closure of the scheduler tests. That closure gains an interpreted
frame kind that evaluates nested calls inside its own transition through
`LiveContext::open` and `close`, and can begin a nested session with
`nested()`, so the registry path and the native path run over the same
frames.

Scope: FR-013-AC-1 to FR-013-AC-8.

## Test Procedure

1. Registry path: register a host body `P` (two `occ`-1 arguments, frame 3)
   that requests the interpreted `C` (frame 4) and records every resume; `C`
   evaluates `N` (frame 4) inside its transition. Begin `P` with
   `live_occurrences` 10 and run it. Run the native path (interpreted `P`
   evaluating `C` and `N` inside its transitions) with the same frames and
   limit. Read the outcome, its `Incomplete` fields, `location`,
   `frame_depth`, the spent `function.call` units and `P`'s recorded resumes.
2. Repeat both paths with `C` = 3 under `live_occurrences` 10 and under 9; read
   the values and `peak_live_occurrences`.
3. Record each `LiveContext` that `P` and `C` receive under step 2's limit
   10, with `P` requesting `C` once and then twice in sequence. Read
   `Session::live_occurrences` after `P` is pushed, and `Session::limits`.
4. Inside `C`'s transition in the registry path, begin a session with
   `nested()` whose root binds a frame of 4, then of 5. In the native path,
   with `C` open inside `P`'s transition, begin one with `nested()` whose
   root binds 3, then 4. From host body `P`, begin one with `nested()`. Run
   `count(600000)` under `SessionLimits::default()`.
5. Interpreted root `P` (3) under `live_occurrences` 20 opens `C` (4) and
   `N` (4) inside its transition, closes both, and returns `Transition::Call`
   for `D` (2) on a meter granting three `function.call` charges. Separately,
   a host body runs a nested session that peaks at 9 and calls `reach`.
6. Interpreted root `P` (3) under `live_occurrences` 20 opens `C` (4) and
   `N` (4) inside its transition and returns `Transition::Call` for `D` (2)
   with both open; read `Session::live_occurrences` before `D` is entered,
   `D`'s context, `P`'s next context and the peak. Repeat under 12.
7. Resume a host body directly with `LiveContext::new(10)`, outside any
   session, opening frames of 4, 4 and 4.
8. Repeat steps 1, 2, 4, 5 and 6 under the no-std build.

## Expected Results

- Step 1: both paths end `Incomplete` with `LiveOccurrences`, `limit` 10,
  `consumed` 7, `next_charge` 4 and `FunctionCall`, at `C`'s call of `N`,
  with `frame_depth` 1. Three `function.call` units are spent, and the two
  outcomes are equal. `P` was resumed only once, with `Resume::Start`.
- Step 2: under 10, both complete with equal values and peak 10; under 9,
  both stop with `consumed` 6 and `next_charge` 4.
- Step 3: `P` sees `live` 3. `C` sees `live` 6 and `limit` 10 at each advance,
  in both runs of `P`. `live_occurrences` reads 3, and `limits` equals the
  Begin limits.
- Step 4: registry nested session: `held` 6; root 4 completes at 10; root 5
  stops with `consumed` 6 and `next_charge` 5 and ends the outer application.
  Native nested session: `held` 7; root 3 completes at 10; root 4 stops with
  `consumed` 7. Host-body nested session: `held` 3. Default limits: `held` 0,
  and `count(600000)` gives FR-011-AC-1's results.
- Step 5: the application ends `Incomplete` at `D`'s `function.call` with
  `peak_live_occurrences` 11; the host body's application reports peak 9.
- Step 6: the count reads 11, `D` is admitted at 13 and sees `live` 13,
  `P`'s next context has `live` 11, and the application completes with peak
  13. Under 12, `D` stops with `consumed` 11, `next_charge` 2 and
  `frame_depth` 2.
- Step 7: the first two opens succeed (`live` 8, `peak` 8). The third
  returns `Incomplete` with `limit` 10, `consumed` 8 and `next_charge` 4,
  and leaves `live` 8.
- No outcome is a `CallFailure`, refusal or fault, and both builds agree.

## Status

Not implemented. Planned in `crates/filament-eval/tests/live_occurrences.rs`.
