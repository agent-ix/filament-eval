---
id: SR-17940486709564847047965
title: QSL-2418 checked TupleProject declaration code-review
type: SpecReview
analysis: code-review
scope: 'agent-ix/filament-eval; QSL-2418; Cargo.toml; Cargo.lock; crates/filament-checked/src/node.rs; crates/filament-checked/tests/node_projection.rs; crates/filament-checked/tests/tuple_index_nodes.rs'
review_set: subset
relationships:
- target: ix://agent-ix/filament-eval/FR-009
  type: references
---
## Summary

The PR changes the checked `TupleProject.declaration` field to the existing `filament_exact::NodeKey` type and updates the two in-repository construction fixtures. `map_ref` keeps the declaration unchanged while mapping the operand. The focused local contract is covered. Repository search found no standalone serialized wire/vector surface for `TupleProject`; the only local producer-facing path is the checked node plus `map_ref`/`NodeRead`. FSL source/model emission is outside this repository and remains subject to FSL PR #2892.

## Verdict

**PASS** — no defects introduced in the PR delta.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Evidence

`NodeKind::TupleProject` now requires `NodeKey`, so a `DeclarationKey` cannot be supplied through this variant. Its `map_ref` arm clones the declaration and retains the position while mapping the operand once. The changed `tuple_index_nodes` test supplies a `NodeKey` and verifies the mapped operand, unchanged declaration and position, and borrowed metadata identity; `node_projection` retains the variant in its all-operation projection exercise. The computed matrix marks FR-009-AC-1 and FR-009-AC-2 tagged by tests. No new enum variant, conversion, local key vocabulary, serialization, vector, admission, or evaluation path appears in the diff. No applicable AssuranceProfile or repository-specific Rust skill was found. No CI workflow changed.

Gates: `cargo fmt --check --package filament-checked` passed; `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed; `cargo test -p filament-checked --all-features` passed (38 tests total, 0 failed); `cargo deny check` passed using the writable `CARGO_HOME=/home/peter/dev/.cargo-home` after the default advisory database lock failed on a read-only path. Cargo deny emitted existing wildcard dependency and unmatched-license warnings. Workspace-wide `cargo fmt --check` reports formatting differences in unchanged `filament-eval` test files; the touched `filament-checked` package formatting check passes.

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
- id: FR-009
  path: spec/functional/FR-009-tuple-project-and-index-nodes.md
  role: context_only
  excerpt: The checked node vocabulary SHALL represent a tuple position read as `TupleProject { operand, declaration: NodeKey, position: usize }` and an ordered-collection or map element read as `Index { collection, index, kind: IndexedKind }`, where `IndexedKind` is exactly `Sequence`, `OrderedSet` or `Map`.
- id: FR-680-AC-7
  path: FSL PR #2892 (external)
  role: context_only
  excerpt: Source/model p[1] must have TupleProject declaration equal to the tuple type Composite NodeKey and position 1; no DeclarationKey; emitted `filament.op.tuple.project` declaration is that node key.
```
