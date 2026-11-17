---
id: TC-001
title: "Read tuple-projection and collection-index nodes"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-009, type: verifies }
---
# TC-001: Read tuple-projection and collection-index nodes

## Description

Check the `TupleProject` and `Index` descriptors of FR-009 and their borrowed
projections. Their evaluation is FSL's (FR-680) and is not run here.

## Test Procedure

1. Project a `TupleProject` through `NodeRead::from_stored` with a child map
   that shifts handles; record the visit order and compare the mapped
   descriptor, `declaration`, `position` and the borrowed type and location
   pointers. Compare it with a `Field` read and an `Index` over the same
   operand.
2. Project an `Index` of each `IndexedKind` with descending stored handles;
   record the visit order and compare `kind` and the borrowed pointers. Match
   `IndexedKind` exhaustively over its three kinds.

## Expected Results

Each projection visits its operands once, in argument order, and keeps every
descriptor field and borrowed pointer; the descriptors differ from `Field` and
from each other, and `IndexedKind` has exactly `Sequence`, `OrderedSet` and
`Map`.

## Status

Ignored stubs in `crates/filament-checked/tests/tuple_index_nodes.rs`.
