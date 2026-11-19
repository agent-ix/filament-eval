---
id: FR-012
title: Report the call depth at which an evaluation's outcome arose
type: FR
relationships:
  - target: ix://agent-ix/filament-spec-language/FR-100
    type: implements
  - target: ix://agent-ix/filament-spec-language/FR-098
    type: implements
  - target: ix://agent-ix/filament-spec-language/FR-262
    type: implements
---
# FR-012: Report the call depth at which an evaluation's outcome arose

## Description

`Evaluation` SHALL carry, beside `location`, the public field
`frame_depth: u32`: the number of call frames open above the selected
application's own frame when its outcome arose. It is 0 for an outcome of the
selected function's own body (or the expression root's own expression), 1 for
one inside a function that body calls, and one more for each further nested
call, a recursive call of the same function included. A completed
evaluation's `frame_depth` is 0.

## Rationale

An `ExpressionLocation` names a place in a body and no frame, so in a
recursive function the same location is reached at every depth. A replay that
must tell the selected call's own stop at a site from a nested call's stop at
that same site needs the depth with the location. The session owns the frame
stack, so it is the one producer that can state the depth without a consumer
re-running the call.

## Behavior

1. The selected application's frame is the root call's frame for
   `begin_call` and the expression root's frame for `begin_expression`. Every
   call frame the session pushes above it, interpreted or registered host
   body, counts one level, and so does every frame a body opens inside one
   transition (FR-013 item 4).
2. An evaluation a body returns with `Transition::Done` carries a
   `frame_depth` relative to that body's frame. A body that originates a
   non-completed outcome returns it with `frame_depth` equal to the number of
   frames it holds open inside the current transition where the outcome arose:
   0 for an outcome of the body's own code, 1 for one arising in a frame it
   opened inside the transition, and one more for each further such frame. The `Evaluation`
   constructors of this crate set 0. A body that propagates a
   non-completed child evaluation it was delivered SHALL return it with its
   `frame_depth` unchanged.
3. When the session pops a finished callee frame and delivers its evaluation
   to the caller, it SHALL add one to a non-completed evaluation's
   `frame_depth` and set a completed evaluation's `frame_depth` to 0. The
   evaluation the selected frame finishes with is the terminal evaluation,
   with no further adjustment, so its `frame_depth` counts the frames above the
   selected frame. A live-occurrence stop a frame returns is not
   delivered: it ends the application with the `frame_depth` FR-013 item 9 states.
4. An outcome the session itself produces at a call site, a denied
   `function.call` charge or a reached live-occurrence limit (FR-011), arises
   in the calling frame: its `frame_depth` is the number of call frames open
   above the selected frame at that moment (0 when the denied call is the root
   call or a call made by the selected frame's body).
5. Addition saturates at `u32::MAX`; a deeper outcome reports `u32::MAX`.
6. `frame_depth` belongs to the in-process `Evaluation` only; it adds no
   member to any outcome document and does not change `location`, `losses` or
   the outcome.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-012-AC-1 | A completed root call's terminal `Evaluation` has `frame_depth` 0, as does a completed child evaluation delivered to its caller. | Test (TC-004) |
| FR-012-AC-2 | With host body `A` relaying directly to host body `B`, which relays directly to the interpreted `boom` that is undefined at its own body, `begin_call(boom)` returns `frame_depth` 0, `begin_call(B)` 1 and `begin_call(A)` 2, each with the same `location` of `boom`'s undefined site. | Test (TC-004) |
| FR-012-AC-3 | For a recursive interpreted function `d` that is undefined at one site when its argument is 1 and otherwise calls `d(n - 1)` and propagates, `d(1)`, `d(2)` and `d(3)` return `Undefined` at that same `location` with `frame_depth` 0, 1 and 2, and `d(0)` completes with `frame_depth` 0; an expression root that calls `d(1)` returns `frame_depth` 1. | Test (TC-004) |
| FR-012-AC-4 | A `function.call` charge denied at the root reports `frame_depth` 0, and one denied at the call the third nested frame makes (the selected frame and two callees open) reports `frame_depth` 2; a live-occurrence stop is reported the same way. | Test (TC-004) |

## Dependencies

- FSL FR-100 "The call depth of an outcome" owns the field's meaning, and FR-098 its use by the replay executor, which observes a claim-site stop only at `frame_depth` 0.
- FSL FR-262 owns the session's frame stack.
- FSL's checked closure (`fsl-eval`) originates and propagates body evaluations under rule 2.
