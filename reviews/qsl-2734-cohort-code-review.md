---
id: SR-17952900000001
title: "QSL-2734 checked/eval semantic-value cohort code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-eval; QSL-2734; PR #47; Cargo.lock and both public crate manifests"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-eval/FR-010
    type: references
---

## Summary

Frozen candidate b5ac6f62737473e009c4b03c4aa2f086419d65ff; source base 89573102cc309941786e3693b2f71e254dddc91c. Ticket QSL-2734. Both manifests and the single Cargo-generated semantic-value lock entry move together from 6e809434d5fd7bde4c5b88ec40cd809efcca595d to merged fa5e78aa71fcea5418492b67f13a33124af84459. No source, tests, workflows, public signatures or other lock entries change.

## Verdict

**PASS — scoped code and Rust review is mergeable.** Both public consumers resolve one nominal semantic-value source. The upstream revision is reachable from its default branch and adds Unit::coherence without changing an existing public signature. This review does not qualify the unmerged FSL consuming cohort or close QSL-2734.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed the complete three-line delta, manifest/lock agreement, upstream reachability, public semantic-value imports and typed checked projections/result seam. No patch section, vendored implementation, nominal duplicate, API shim, warning suppression, gate change or source/test weakening is introduced. No applicable AssuranceProfile or local AGENTS.md/CLAUDE.md exists in the reviewed tree. Rust API and testing guidance was loaded; the change contains no Rust source edits.

The supplied named checks were read in full: checked quantity_scale_nodes scale_reads_ has 2 passed, 0 failed, 0 ignored; eval result a_kernel_outcome_passes_through_the_seam_unchanged has 1 passed, 0 failed, 0 ignored, 3 filtered. The first proves quantity/factor order, distinct scale/divide descriptors and borrowed metadata; the second is a Boolean kernel-outcome seam control, not a unit-arithmetic proof. Both compile their consumer against the new pin. Unit-graph arithmetic remains owned and tested upstream; no local unit-graph implementation exists. Logs: /private/tmp/qsl-2734-eval-checked-scale.log and /private/tmp/qsl-2734-eval-result.log. No Cargo command or broad gate was repeated by the reviewer under the explicit focused-check policy. git diff --check passed.

## Examined Scope

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

