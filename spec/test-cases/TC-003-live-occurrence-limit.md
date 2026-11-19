---
id: TC-003
title: "Stop a session at its live-occurrence limit"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-011, type: verifies }
---
# TC-003: Stop a session at its live-occurrence limit

## Description

Check FR-011's live-occurrence count, its limit and the incomplete outcome
through the toy closure of the scheduler tests.

## Test Procedure

1. Run `count(600000)` (600,001 calls) and `count(1000)` (1,001 calls) under
   `SessionLimits::default()` on a meter with `work_units` `u64::MAX`; read
   the outcome, its `Incomplete` fields, location and the spent
   `function.call` units.
2. Run `count(600000)` under `live_occurrences` 4,000,000 and a
   one-argument call under 0; read `peak_live_occurrences`.
3. Run a body calling a one-argument callee ten times in sequence under 4,
   with interpreted and with host-body callees.
4. Call with an argument of `occ` 5, and run an expression root with
   parameters of total `occ` 3 that calls a one-argument callee under 5.
5. Repeat step 1 under the no-std build.

## Expected Results

Step 1 stops `Incomplete` with `LiveOccurrences`, `limit` 1,048,576,
`consumed` 1,048,576, `next_charge` 2 and `FunctionCall` at the 524,289th call,
with 524,289 `function.call` units spent; `count(1000)` completes. Step 2
completes with peak 1,200,002 and stops at the root with `consumed` 0 and
`next_charge` 2. Step 3 completes with peak 4. Step 4 counts 6 for the frame
and stops with `consumed` 4, `next_charge` 2. No outcome is a `CallFailure`,
refusal or fault, and both builds agree.

## Status

Implemented in `crates/filament-eval/tests/live_occurrences.rs`.
