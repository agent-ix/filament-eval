---
id: SR-17940486709564847047966
title: QSL-2418 checked TupleProject declaration gap-analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/filament-eval; QSL-2418; crates/filament-checked/src/node.rs; crates/filament-checked/tests/node_projection.rs; crates/filament-checked/tests/tuple_index_nodes.rs'
review_set: subset
relationships:
- target: ix://agent-ix/filament-eval/FR-009
  type: references
---
## Summary

Scoped the computed requirement-to-test-to-code chain to the changed tuple-projection declaration and reader path. FR-009-AC-1 remains bound to the tuple projection test; the implementation now constrains the declaration to `NodeKey` and the projection preserves it. No code-to-spec gap or hollow test was introduced. The external FR-680-AC-7 source/model emission behavior is owned by FSL PR #2892; this repository has no separate tuple-project wire/vector surface to update.

## Verdict

**PASS** — no gaps introduced by this PR in the scoped checked-node behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

The computed matrix reports FR-009-AC-1 and FR-009-AC-2 as tagged by active tests. The changed implementation provides the checked descriptor type and generic projection only; no evaluation, admission, serialization, or source/model producer exists in this repository for the external emitted-operation assertion. Repository-wide matrix context includes 15 untagged criteria, five criteria tagged only by ignored tests, and two inspection criteria without symbols; these are outside this PR's changed paths and none is FR-009-AC-1 or FR-009-AC-2. Optional semantic fan-out was not run. The direct changed-criterion review is recorded in the code-review artifact.

## Examined Scope

```yaml
scope:
- id: FR-009-AC-1
  path: spec/functional/FR-009-tuple-project-and-index-nodes.md
  role: examined
  excerpt: A `TupleProject` read retains a distinct tuple-projection operation, maps its one operand exactly once even when the mapped handle differs from the stored handle, keeps `declaration` and `position` unchanged, and borrows the supplied checked type and location; it compares unequal to a `Field` read and to an `Index` over the same operand.
- id: FR-009-AC-2
  path: spec/functional/FR-009-tuple-project-and-index-nodes.md
  role: examined
  excerpt: An `Index` read maps `collection` before `index`, each exactly once, including when the stored handles have descending indices, keeps `kind` unchanged and borrows the supplied checked type and location; `IndexedKind` has exactly the three kinds `Sequence`, `OrderedSet` and `Map`.
- id: FR-680-AC-7
  path: FSL PR #2892 (external)
  role: context_only
  excerpt: Source/model p[1] must have TupleProject declaration equal to the tuple type Composite NodeKey and position 1; no DeclarationKey; emitted `filament.op.tuple.project` declaration is that node key.
```
