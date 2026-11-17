---
id: TC-001
title: "Read and evaluate tuple-projection and collection-index nodes"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-009, type: verifies }
---
# TC-001: Read and evaluate tuple-projection and collection-index nodes

## Description

Check the `TupleProject` and `Index` descriptors of FR-009, their borrowed
projections and their evaluation over tuple, sequence, ordered-set and map
values.

## Test Procedure

1. Project a `TupleProject` and an `Index` of each `IndexedKind` through
   `NodeRead::from_stored` with a child map that shifts handles; record the
   visit order and compare the mapped descriptor, `declaration`, `position`,
   `kind` and the borrowed type and location pointers.
2. Evaluate `TupleProject` at positions 0 and 1 over `(3, true)` and a record
   `Field` read of a one-field record holding each value; compare values and
   charge vectors. Evaluate it over a stopped operand.
3. Evaluate `Index` over `[10, 20, 20]` at 1, `{10, 20, 30}` at 2 and
   `{"a": 1, "b": 2}` at `"b"`, recording charges.
4. Under direct kernel evaluation, evaluate `Index` at -1 and 3 over the
   sequence, 3 over the ordered set and `"c"` over the map.

## Expected Results

Projection visits each operand once in argument order and keeps every
descriptor field; the descriptors differ from `Field` and from each other.
Tuple reads complete `3` and `true` with the `Field` charge vector. Index reads
complete `20`, `30` and `2`, each with one `collection.element` and one
`collection.result-retain`, and the map makes no member walk or test. Every
unmet index obligation returns `Undefined::NoElement` at the index node with
no value and no `collection.element` charge.

## Status

Ignored stubs in `crates/filament-checked/tests/tuple_index_nodes.rs`.
