---
id: SR-17952900000002
title: "QSL-2734 checked/eval semantic-value cohort gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-eval; QSL-2734; PR #47; whole-repository computed traceability and scoped dependency adoption"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-eval/FR-010
    type: references
---

## Summary

Frozen candidate b5ac6f62737473e009c4b03c4aa2f086419d65ff; source base 89573102cc309941786e3693b2f71e254dddc91c. Ticket QSL-2734. Both manifests and the single Cargo-generated semantic-value lock entry move together from 6e809434d5fd7bde4c5b88ec40cd809efcca595d to merged fa5e78aa71fcea5418492b67f13a33124af84459. No source, tests, workflows, public signatures or other lock entries change. The pin delta is complete and the supplied three focused compatibility tests pass. The computed repository matrix retains a pre-existing traceability backlog.

## Verdict

**FAIL for repository-wide gap analysis; scoped pin delta is MERGEABLE after disposition round 1.** Sixteen existing untagged criteria prevent a global PASS. Root explicitly accepted this unchanged baseline for the three-line manifest/lock adoption only. This does not qualify the whole repository or complete the FSL consumer obligation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed whole-repository matrix retains 16 untagged criteria, including FR-001-AC-1. Existing comment-based trace IDs do not bind under the active module. This unchanged baseline prevents a repository-wide gap-analysis PASS; the three-line dependency pin delta introduces no criterion or trace change. | spec/functional/FR-001-ieee-checked-node-vocabulary.md:44 |

## Coverage

Plan completion: not assessed

Reconciliation: quoin matrix --repo /Users/peter/dev/qsl-2734-checked-eval-code --json, quoin 0.36.7 / quire 0.52.0 (engine 0.62.2). The matrix reports 54 criteria: 36 tagged, 16 untagged, 2 method-without-symbol, 0 ignored-only. All 54 have no run evidence in the evidence store; the retained shell logs above provide focused execution evidence separately and were not fabricated as stored evidence. FR-010-AC-1 and AC-2 bind the two exercised checked projection tests.

Coverage extraction reports 114 Rust candidates, 84 tagged candidates and 44 bound symbols, with 51 unmatched tags and two untracked symbols carrying FR-400-AC-11 in node_projection.rs:24 and node_read.rs:14. There are 124 trace-id-outside-marker diagnostics. These unchanged inputs were verified by the source/spec/test diff being empty. No tag or criterion was added or weakened by the pin update. The machine-readable matrix and coverage outputs remain in the scratch directory.

Reverse inventory examined the checked node/read vocabulary and eval result, registry/session, host-body, collection, integer, live-count/depth and admission interfaces. The master specification explicitly limits its current inventory and states other behavior is outside it; public functions also cite upstream requirements in module documentation. No new behavior or stub is introduced. Source/test scans found no todo!, unimplemented!, TODO, FIXME or ignored-test markers. Existing integer modulo delegates deliberately to its owning kernel, and public re-exports are genuine API surfaces, not hollow implementations. A complete global mapping of every baseline public item to an externally owned requirement was not established; no global reverse-audit PASS is claimed. Optional semantic fan-out was not requested and was not run.

No spec review is applicable: no spec text changes. No source rewrite or additional Cargo qualification was performed.

## Examined Scope

Disposition round 1 verified the unchanged frozen head b5ac6f62737473e009c4b03c4aa2f086419d65ff, the three-file dependency-only diff and the computed counts of 16 untagged, 36 tagged and 2 method-without-symbol criteria. No code fix, Cargo run or new review was performed.

```yaml
{
  "scope": [
    {
      "id": "crates/filament-checked/Cargo.toml",
      "path": "crates/filament-checked/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"fa5e78aa71fcea5418492b67f13a33124af84459\", default-features = false }"
    },
    {
      "id": "crates/filament-eval/Cargo.toml",
      "path": "crates/filament-eval/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"fa5e78aa71fcea5418492b67f13a33124af84459\", default-features = false }"
    },
    {
      "id": "Cargo.lock",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "source = \"git+https://github.com/agent-ix/filament-semantic-value?rev=fa5e78aa71fcea5418492b67f13a33124af84459#fa5e78aa71fcea5418492b67f13a33124af84459\""
    },
    {
      "id": "FR-010-AC-1",
      "path": "spec/functional/FR-010-quantity-scale-nodes.md",
      "role": "examined",
      "excerpt": "`QuantityScale` and `QuantityScaleDiv` reads each retain their distinct operation and map `quantity` before `factor`, each exactly once, including when the stored handles have descending indices; neither compares equal to `Quantity` multiplication or division over the same handles, nor to each other."
    },
    {
      "id": "FR-010-AC-2",
      "path": "spec/functional/FR-010-quantity-scale-nodes.md",
      "role": "examined",
      "excerpt": "Each read borrows the supplied checked result type and location without reconstructing either; neither descriptor has a unit, law, mode, member or magnitude field."
    },
    {
      "id": "FR-001-AC-1",
      "path": "spec/functional/FR-001-ieee-checked-node-vocabulary.md",
      "role": "examined",
      "excerpt": "A checked `filament.op.ieee.float32.fma` node projects as a typed float32 fused multiply-add read with its admitted rounding mode and three operands in source order."
    },
    {
      "id": "spec/spec.md",
      "path": "spec/spec.md",
      "role": "context_only",
      "excerpt": "Other crate behavior remains outside this requirement inventory."
    }
  ]
}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Round 1: root explicitly accepts the unchanged repository assurance debt for scoped PR #47 only. The three-line manifest/lock cohort update changes no source, criterion or test tag; unrelated trace repair and broader qualification are outside the authorized focused pin scope. The global gap verdict remains FAIL, and QSL-2734 remains In Progress until the FSL consumer merges. |

