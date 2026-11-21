---
id: SR-17940486709564847047967
title: QSL-2734 checked and eval semantic-value pin code review
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-eval; QSL-2734; Cargo.lock; crates/filament-checked/Cargo.toml; crates/filament-eval/Cargo.toml"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-eval/FR-010
    type: references
---

## Summary

Reviewed PR #46 at 12305269ccdb0a34202909aaadf03e2c68ba6094. Both consumers advance to the requested semantic-value commit without source, feature, API, or unrelated dependency changes.

## Verdict

**PASS — MERGEABLE.** No defect was found in the pin-only delta. The lockfile and both manifests name one semantic-value source at 6e809434d5fd7bde4c5b88ec40cd809efcca595d, verified reachable from upstream main. The observation and FSL consumer pins remain subsequent delivery work.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Evidence

The entire delta is the two dependency revisions and one lockfile source. Locked offline cargo tree resolves exactly one filament-semantic-value package shared by filament-checked and filament-eval. No copied API, new conversion arithmetic, local compatibility layer, feature change, gate change, unsafe code, wire change, or new behavior is introduced. Existing public result types continue importing CatalogCoded, locations and losses from that shared dependency.

Inspected the supplied focused logs: checked build then eval build succeeded; quantity scaling projections passed 2 tests; catalog/result handling passed 1; scheduler loss propagation passed 2; metered arithmetic passed 1. The quantity tests assert operand order, distinct variants, and borrowed type/location identity. Log paths: /private/tmp/qsl-2734-checked-build.log, /private/tmp/qsl-2734-eval-build.log, /private/tmp/qsl-2734-checked-quantity.log, /private/tmp/qsl-2734-eval-catalog.log, /private/tmp/qsl-2734-eval-losses.log, /private/tmp/qsl-2734-eval-meter.log. No extra build or full gate was run by the reviewer, following the owner's focused-check policy.

No applicable AssuranceProfile or repository-local AGENTS.md/CLAUDE.md was found. README, CONTRIBUTING, Cargo lints, deny and formatter configuration were read. Quire matrix reports FR-010-AC-1/2 tagged. Its pre-existing repository trace warnings are not changed by this manifest-only delta. This is not a whole-repository assurance claim.

## Examined Scope

```yaml
{
  "scope": [
    {
      "id": "crates/filament-checked/Cargo.toml",
      "path": "crates/filament-checked/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"6e809434d5fd7bde4c5b88ec40cd809efcca595d\", default-features = false }"
    },
    {
      "id": "crates/filament-eval/Cargo.toml",
      "path": "crates/filament-eval/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"6e809434d5fd7bde4c5b88ec40cd809efcca595d\", default-features = false }"
    },
    {
      "id": "Cargo.lock",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "source = \"git+https://github.com/agent-ix/filament-semantic-value?rev=6e809434d5fd7bde4c5b88ec40cd809efcca595d#6e809434d5fd7bde4c5b88ec40cd809efcca595d\""
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
    }
  ]
}
```
