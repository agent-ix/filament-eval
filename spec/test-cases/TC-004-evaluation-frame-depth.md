---
id: TC-004
title: "Report the frame depth of an evaluation's outcome"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-012, type: verifies }
---
# TC-004: Report the frame depth of an evaluation's outcome

## Description

Check FR-012's `Evaluation.frame_depth` for completed, nested, recursive and
session-produced outcomes through the toy closure.

## Test Procedure

1. Run a completed root call and read `frame_depth`.
2. Register host bodies `A` relaying directly to `B`, relaying directly to
   `boom`; begin calls of `boom`, `B` and `A`.
3. Run a recursive `d` undefined at one site for argument 1, with arguments
   0 to 3, and an expression root calling `d(1)`.
4. Deny `function.call` at the root and at a call made by the third nested
   frame; stop a call at the live-occurrence limit at the same place.

## Expected Results

Completed evaluations report 0. Step 2 reports 0, 1 and 2 at the same
location. Step 3 reports 0, 1 and 2 for `d(1)` to `d(3)` at one location, 0
for the completed `d(0)` and 1 for the expression root. Step 4 reports 0 and 2.

## Status

Ignored stubs in `crates/filament-eval/tests/frame_depth.rs`.
