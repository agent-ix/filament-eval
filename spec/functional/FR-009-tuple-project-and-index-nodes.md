---
id: FR-009
title: Read the checked tuple-projection and collection-index operations
type: FR
relationships:
  - target: ix://agent-ix/filament-specification/FR-143
    type: implements
  - target: ix://agent-ix/filament-specification/FR-144
    type: implements
  - target: ix://agent-ix/filament-specification/FR-322
    type: implements
---
# FR-009: Read the checked tuple-projection and collection-index operations

## Description

The checked node vocabulary SHALL represent a tuple position read as
`TupleProject { operand, declaration: NodeKey, position: usize }` and an
ordered-collection or map element read as
`Index { collection, index, kind: IndexedKind }`, where `IndexedKind` is
exactly `Sequence`, `OrderedSet` or `Map`. The two descriptors are distinct
from each other, from record `Field` reads and from every collection query.

## Rationale

The standard reads a tuple position and a collection element through one
source form, `c[i]`, but records them as two operations: a tuple read has a
static position and no obligation, while an element read is partial and
carries the index obligation. Keeping two descriptors lets an evaluator and a
code generator read which one an admitted node is without re-deriving it from
operand types.

## Behavior

### Tuple projection

1. `declaration` is the declaration key of the tuple type `P` and `position`
   is the zero-based position `n` that the admitted
   `filament.op.tuple.project` application names in its member
   `{kind: "position", declaration: P, position: n}`. Together they are the
   position identity of FSpec FR-143.
2. A borrowed projection SHALL visit the operand exactly once, retain its
   mapped handle, keep `declaration` and `position` unchanged, and borrow the
   supplied checked result type and location unchanged.
3. The descriptor carries no obligation, presence, optional, unit, law or mode
   field: every value of `P` holds all of its positions, so the read cannot
   fail once the operand completes.
4. The descriptor does not check `position` against the tuple's arity;
   Contract-IR admission establishes `position < k`.

### Collection index

5. `kind` names the checked type of the `collection` operand: `Sequence` for a
   bounded or unbounded `Sequence<T>`, `OrderedSet` for an `OrderedSet<T>` and
   `Map` for a `Map<K, V>`. There is no kind for `Set`, `Bag` or any other
   operand type, because FSpec FR-144 refuses an index over them at check time.
6. A borrowed projection SHALL visit `collection` before `index`, each exactly
   once, retain both mapped handles in that order, keep `kind` unchanged, and
   borrow the supplied checked result type and location unchanged.
7. The node carries FSpec FR-144's index obligation (`0 <= index` and
   `index < size(collection)` for `Sequence` and `OrderedSet`;
   `contains(collection, index)` for `Map`), which a linked body has proved.
   The descriptor holds no obligation field: `kind` alone tells a consumer
   which form of the obligation applies.

### Evaluation

8. This crate does not evaluate either node: no evaluation entry of
   `filament-eval` interprets a `NodeKind`. FSL's checked closure evaluates
   both under FSL FR-680, with the operand order, selection, map canonical-key
   lookup, `Field`-equal tuple charge, `collection.element` and
   `collection.result-retain` charges, and unmet-obligation outcome of FSpec
   FR-143 and FR-144.

### Projection from Contract-IR

9. Contract-IR projects each admitted `filament.op.tuple.project` application
    (FSpec FR-143) into one `TupleProject` node whose operand is the
    application's one argument handle, and each admitted
    `filament.op.collection.index` application (FSpec FR-144, arguments
    `[ref(c), ref(i)]`) into one `Index` node whose `collection` and `index`
    are the argument handles in argument order. A tuple position read is
    never projected as an `Index`, and a collection index never as a
    `TupleProject`.

The descriptors grant no admission or evaluation authority and do not infer
admission from their shape.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-009-AC-1 | A `TupleProject` read retains a distinct tuple-projection operation, maps its one operand exactly once even when the mapped handle differs from the stored handle, keeps `declaration` and `position` unchanged, and borrows the supplied checked type and location; it compares unequal to a `Field` read and to an `Index` over the same operand. | Test (TC-001) |
| FR-009-AC-2 | An `Index` read maps `collection` before `index`, each exactly once, including when the stored handles have descending indices, keeps `kind` unchanged and borrows the supplied checked type and location; `IndexedKind` has exactly the three kinds `Sequence`, `OrderedSet` and `Map`. | Test (TC-001) |

## Dependencies

- QSpec FR-143 "Tuple position read" owns the read's typing, refusals, absence of an obligation and its `r.f` charge.
- QSpec FR-144 "Indexing" owns element selection, the index obligation, the `collection.element` and `collection.result-retain` charges and the map's canonical-key lookup.
- QSpec FR-322 owns the `filament.op.tuple.project` and `filament.op.collection.index` application shapes.
- Contract-IR owns admission and the projection of both applications into these nodes.
- FSL FR-680 owns lowering of `p[n]` and `c[i]` to the applications and the evaluation of both nodes, with its values, charges and outcomes.
- `filament-exact` owns canonical keys and map values used by that evaluation.
