---
id: SR-17940486709564847047970
title: "QSL-2958 filament-eval semantic-value pin review"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-eval; QSL-2958 dependency alignment; PR #51; Cargo.lock and two crate manifests"
review_set: subset
---

## Summary

Reviewed PR #51 at `47254dd10360a4fa475713f4a61f1200f4a48d6a` against `origin/main`. The PR changes only the `filament-semantic-value` revision in the `filament-checked` and `filament-eval` manifests and the shared lock entry, from `4274b4eb4abeae4ccd457a6de681625acb8fc57d` to merged commit `689f259eab592b62cb362f5e649dba16adf874f7`. Cargo tree resolves one semantic-value source for both consumers. This is a dependency-alignment PR only; it does not implement or claim the downstream FSL ordered-position acceptance criteria.

## Verdict

**PASS — no defects found in the PR diff.** The exact pin and lock entry agree, and both consumer crates resolve the same nominal semantic-value package.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review Evidence

The complete diff is limited to `Cargo.lock`, `crates/filament-checked/Cargo.toml`, and `crates/filament-eval/Cargo.toml`. It changes no Rust source, tests, feature flags, workflows, or public signatures. `cargo tree --locked -e normal -i filament-semantic-value` shows one source at `689f259e` used by both crates. `git diff --check` passed.

Focused verification: `cargo check --locked -p filament-checked --features construct --lib --tests -j 4` and `cargo check --locked -p filament-eval --lib --tests -j 4` passed. Filtered test `cargo test -p filament-eval --test integer modulo_returns_the_kernel_value_and_ordered_charges -j 4` passed (1 passed). `cargo deny check` passed with existing unmatched license allowance and git wildcard warnings.

Repository checks expose unrelated existing formatting/lint failures in unchanged `crates/filament-eval/tests/application_live_occurrences.rs`: `cargo fmt --all -- --check` reports a reordered import, and scoped Clippy reports `type_complexity` at line 208. That file is not in the PR diff. No unfiltered test suite was run. There is no applicable AssuranceProfile or local `AGENTS.md`/`CLAUDE.md` in this repository. The review does not qualify the later FSL consuming PR.

## Examined Scope

```yaml
scope:
  - id: crates/filament-checked/Cargo.toml
    path: crates/filament-checked/Cargo.toml
    role: examined
    excerpt: 'filament-semantic-value = { git = "https://github.com/agent-ix/filament-semantic-value", rev = "689f259eab592b62cb362f5e649dba16adf874f7", default-features = false }'
  - id: crates/filament-eval/Cargo.toml
    path: crates/filament-eval/Cargo.toml
    role: examined
    excerpt: 'filament-semantic-value = { git = "https://github.com/agent-ix/filament-semantic-value", rev = "689f259eab592b62cb362f5e649dba16adf874f7", default-features = false }'
  - id: Cargo.lock
    path: Cargo.lock
    role: examined
    excerpt: 'source = "git+https://github.com/agent-ix/filament-semantic-value?rev=689f259eab592b62cb362f5e649dba16adf874f7#689f259eab592b62cb362f5e649dba16adf874f7"'
```
