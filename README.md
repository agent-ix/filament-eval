# filament-eval

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)
[![Filament](https://img.shields.io/badge/Filament-Specification%20Language-1F6FEB)](https://github.com/agent-ix/filament-spec-language)

filament-eval is a `no_std` Rust workspace with the checked terms an evaluator reads (`filament-checked`) and the evaluation core that runs them (`filament-eval`).

`filament-eval` begins the application of a checked closure, steps the resulting session one finite transition at a time, and runs registered host bodies next to interpreted code on one heap task stack. Evaluation is bounded by the `work_units` fuel of a single `filament_exact::Meter`, and a session ends with an evaluation or a call failure. `filament-checked` holds the checked node vocabulary, declaration keys and the admitted integer-division law that the evaluator and the checker share.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="A Session steps a CheckedClosure one transition at a time. The Registry supplies the resumable host bodies and the closure advances interpreted code. Frames sit on one heap task stack, each step charges a Meter, and a step returns Runnable or a Terminal result." src="docs/images/hero-light.svg">
</picture>

## Getting started

Add the crates as git dependencies. Both build without `std`.

```toml
[dependencies]
filament-eval = { git = "https://github.com/agent-ix/filament-eval", branch = "main" }
filament-checked = { git = "https://github.com/agent-ix/filament-eval", branch = "main", default-features = false }
```

To work from a clone, install the Rust toolchain pinned in `rust-toolchain.toml` (it includes the `thumbv7em-none-eabi` target) and run `make ci`.

## Usage

An evaluator implements `CheckedClosure` for its checked terms. A caller registers host bodies in a `Registry`, begins a call, and steps the session until it ends:

```rust
use filament_eval::{CallFailure, CheckedClosure, Evaluation, Progress, Registry};
use filament_exact::{Meter, NodeKey, Value};

fn call<C: CheckedClosure>(
    registry: &mut Registry<C>,
    callable: NodeKey,
    arguments: Vec<Value>,
    environment: &C::Environment,
    meter: &mut Meter,
) -> Result<Evaluation, CallFailure> {
    let mut session = registry.begin_call(callable, arguments, environment, meter)?;
    loop {
        match session.step() {
            Progress::Runnable(next) => session = next,
            Progress::Terminal(result) => return result,
        }
    }
}
```

The first successful `begin_call` seals the registry. `Session::run` performs the same loop.

A host body implements `Body`. Each call to `resume` runs one transition and returns `Transition::Continue`, `Transition::Call` with a `ChildRequest` for a callee, or `Transition::Done` with the body's `Evaluation`. The scheduler validates the request, charges `function.call`, runs the callee and delivers its result to the suspended body once.

## Reference

| Item | Crate | Purpose |
| --- | --- | --- |
| `CheckedClosure` | `filament-eval` | The contract an evaluator implements for one checked closure: signatures, admission and interpreted transitions. |
| `Registry` | `filament-eval` | Binds a `BodyFactory` to a callable and signature the closure admitted, and begins calls and expression evaluations. |
| `Admission` | `filament-eval` | The typed result of registering a body: accepted, sealed, not admitted, signature mismatch or duplicate binding. |
| `Session`, `Progress` | `filament-eval` | The live application and the result of one step: `Runnable` or `Terminal`. |
| `Body`, `BodyFactory`, `ChildRequest`, `Resume`, `Transition` | `filament-eval` | The resumable host body contract. |
| `Evaluation`, `FamilyOutcome`, `CallFailure` | `filament-eval` | The evaluation result vocabulary. |
| `NodeKind`, `NodeRead` | `filament-checked` | The checked expression node vocabulary and how an evaluator reads a node. |
| `DeclarationKey` | `filament-checked` | The declaration identity a domain package supplies. |
| `AdmittedIntegerDivision` | `filament-checked` | The admitted `div` and `rem` law of a checked package. |

The `construct` feature of `filament-checked` exposes the constructors of checked terms. The checker enables it, and it is the checking authority.

## Ecosystem

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-dark.svg">
  <img alt="The Filament repositories. The main chain runs from the standard, filament-specification, to the compiler, filament-spec-language, then filament-contract-ir, filament-contract-codegen and filament-contract-runtime. filament-spec-language also feeds filament-protocol, and filament-contract-ir feeds filament-analyze. filament-observation feeds filament-protocol and filament-mltl. Above the chain, filament-driver runs it, filament-integration tests it and filament-verification plans verification. Below it, shared libraries used throughout: filament-eval, filament-exact, filament-semantic-value, filament-walk, quire-canonical and filament-verification-contracts." src="docs/images/ecosystem-light.svg">
</picture>

## Building and testing

```
make fmt-check       # formatting
make lint            # clippy, warnings are errors
make test            # cargo test, default and all features
make build-no-std    # build for thumbv7em-none-eabi
make deny            # license and source policy
make docs            # rustdoc, warnings are errors
make ci              # every gate above
```

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) and sign the [CLA](CLA.md) before opening a pull request. Questions are welcome on [Discord](https://discord.gg/k8DVhuYBR2).

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
