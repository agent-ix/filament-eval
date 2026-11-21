---
id: SR-17940486709564847047968
title: QSL-2734 semantic-value pin delta gap review
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-eval; QSL-2734; PR #46 pin-only delta and existing quantity compatibility evidence"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-eval/FR-010
    type: references
---

## Summary

Compared the requested dependency adoption, manifest/lock resolution, existing typed consumers and focused compatibility evidence for PR #46 at 12305269ccdb0a34202909aaadf03e2c68ba6094. No behavior, source, criterion or test tag is added by this PR, and no new reverse gap or stub is introduced.

## Verdict

**PASS for the explicitly scoped pin delta — MERGEABLE.** This is not a repository-wide gap-analysis PASS: the caller explicitly bounded review to the pin-only change and excluded an unrelated full-repository audit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Reconciliation: quire matrix --scope /Users/peter/dev/qsl-2734-eval-pins --format json (quire 0.52.0, engine 0.62.2); no run evidence read by that computation. Repository context: 36 tagged criteria, 16 untagged criteria, 2 method-without-symbol criteria; all source/spec/trace inputs to that context are unchanged by this PR. FR-010-AC-1 and FR-010-AC-2 are tagged by the exercised quantity projection tests. Repository-wide trace hygiene is not claimed complete.

The changed dependency source fulfills the owner's explicit pin request; one lock source and both crate manifests preserve shared public type identity. Inspected existing typed projections/result imports and behavioral assertions rather than inventing a new requirement for a dependency revision. No implementation or test stub is added. The six supplied focused compatibility tests passed, and checked/eval locked builds passed in dependency order. New dimension-equivalence behavior remains owned by semantic-value FR-134/TC-931, already merged upstream; FSL integration remains consumer-owned.

Optional semantic fan-out was not requested and was not run. Full-repository reverse-gap and stub inventory was outside the caller's pin-only scope; no broader audit conclusion is asserted.

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
