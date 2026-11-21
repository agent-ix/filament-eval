---
id: SR-17940486709564847047969
title: QSL-2158 checked and eval WorkScope owner pins code review
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-eval; QSL-2158; Cargo.lock; crates/filament-checked/Cargo.toml; crates/filament-eval/Cargo.toml"
review_set: subset
---

## Summary

Reviewed PR #48 at 45885a83eb335d32075b20e6e3963ebecaff0485, against a0e8e1254827de71738cbf87646e09ef68531692. Code-review includes the Rust configuration lane. The three-file delta contains seven revision substitutions: both packages' exact and semantic-value production dependencies, eval's exact test-support dev dependency, and the two lockfile sources.

## Verdict

**PASS for the pin-only change; no findings.** This is not an unconditional merge-gate acceptance. The supplied default-feature all-target check is red because an unchanged test imports a feature-gated `construct` module. The existing all-feature configuration passes. Root has referred disposition of that default gate to arch; this review does not waive it or authorize changing the unrelated test/feature policy. Root must apply the approved final gate to the exact final head on current main before merging.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Evidence

Static byte comparison against the base confirms that replacing only the two old owner revisions yields every changed file exactly. All three TOML documents parse. Both direct consumers and the test-support dev alias share accepted exact a58474cc105a86f3307a11def3fc001e2459679f and semantic-value 31d807ccb8ffd5a88beb0b1a570b80acb099f5e9. The lock contains exactly one exact package (0.3.0) and one semantic-value package (0.1.5), with matching query and fragment revisions. Their dependency lists and every other lock package/version/source are unchanged. Both accepted revisions are reachable from fetched upstream main. Exact's accepted change has no Cargo manifest/lock delta; semantic-value's accepted Cargo delta is only its matching exact source revision. Feature flags, default-features settings, source, public APIs, tests, unsafe policy and workflows are unchanged; no compatibility layer or copied owner API was added.

Supplied Titan evidence at this frozen head: `node_projection` passed 3, `integer` passed 1, all with zero failures/ignored. `cargo check --locked --all-targets --all-features -p filament-checked -p filament-eval` passed. The earlier same-packages default-feature all-target check failed E0432 at `crates/filament-checked/tests/node_read.rs:5:24` importing `construct`. Inspection confirms that the base has the same unconditional test import, while `src/lib.rs:26` exports the module only with the existing `construct` feature and the package default is `std`. No failure was hidden and no gate, feature or source was changed to obtain the all-feature result. The supplied four passing runtime tests are not a whole-suite claim. No local Cargo was run. Full lint, no_std target and broad test gates were not rerun under the explicit focused-only policy; no new evidence for those lanes is claimed.

No repository-local AGENTS.md/CLAUDE.md or applicable AssuranceProfile was found. Read README, CONTRIBUTING, workspace lints, Makefile, rustfmt, Clippy and deny configuration. Gap-analysis and spec-review are N/A because this authorized mechanical dependency change adds no production Rust or specification obligations; no whole-repository matrix assessment was performed. FSL's cumulative WorkScope forwarding is subsequent work, outside this pin PR.

## Publication

One private marked QSL-2158 comment is prepared locally. Linear and ix-board reads/writes, labels and status lookups are held until explicit root release after the local budget window. No review findings or excerpts were posted to GitHub. This artifact is written only in scratch; the reviewer did not edit or commit the frozen branch.

## Examined Scope

```yaml
{
  "scope": [
    {
      "id": "crates/filament-checked/Cargo.toml-owner-1",
      "path": "crates/filament-checked/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\", default-features = false }"
    },
    {
      "id": "crates/filament-checked/Cargo.toml-owner-2",
      "path": "crates/filament-checked/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-exact = { git = \"https://github.com/agent-ix/filament-exact\", rev = \"a58474cc105a86f3307a11def3fc001e2459679f\", default-features = false }"
    },
    {
      "id": "crates/filament-eval/Cargo.toml-owner-1",
      "path": "crates/filament-eval/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\", default-features = false }"
    },
    {
      "id": "crates/filament-eval/Cargo.toml-owner-2",
      "path": "crates/filament-eval/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-exact = { git = \"https://github.com/agent-ix/filament-exact\", rev = \"a58474cc105a86f3307a11def3fc001e2459679f\", default-features = false }"
    },
    {
      "id": "crates/filament-eval/Cargo.toml-owner-3",
      "path": "crates/filament-eval/Cargo.toml",
      "role": "examined",
      "excerpt": "filament-exact = { git = \"https://github.com/agent-ix/filament-exact\", rev = \"a58474cc105a86f3307a11def3fc001e2459679f\", default-features = false, features = [\"test-support\"] }"
    },
    {
      "id": "lock-owner-source-1",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "source = \"git+https://github.com/agent-ix/filament-exact?rev=a58474cc105a86f3307a11def3fc001e2459679f#a58474cc105a86f3307a11def3fc001e2459679f\""
    },
    {
      "id": "lock-owner-source-2",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "source = \"git+https://github.com/agent-ix/filament-semantic-value?rev=31d807ccb8ffd5a88beb0b1a570b80acb099f5e9#31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\""
    },
    {
      "id": "existing-construct-feature",
      "path": "crates/filament-checked/Cargo.toml",
      "role": "context_only",
      "excerpt": "default = [\"std\"]\n# `std` for the crate's own dependencies; without it the crate is `no_std` + `alloc`.\nstd = [\"serde/std\", \"thiserror/std\"]\n# The `construct` module: the only constructors of a checked term. Only the\n# checker enables it (QSpec FR-400, \"Checking authority\").\nconstruct = []"
    },
    {
      "id": "existing-construct-export",
      "path": "crates/filament-checked/src/lib.rs",
      "role": "context_only",
      "excerpt": "#[cfg(feature = \"construct\")]\npub mod construct {"
    },
    {
      "id": "existing-construct-test-import",
      "path": "crates/filament-checked/tests/node_read.rs",
      "role": "context_only",
      "excerpt": "use filament_checked::{construct, NodeKind, NodeRead};"
    }
  ]
}
```
