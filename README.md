# IX-Decriel

Decriel is a research programming language for explicit effects and
capabilities. This v0.2.0 release implements a small checked language and a
mediated Rust execution interface. It does not ship an operating system
sandbox, a real network/file/process adapter, authenticated human approvals,
or a production audit store.

## What works

- A lexer, parser, AST, source spans, and diagnostics for modules and
  declarations.
- Checked function bodies containing `read`, `write`, `network`, or `execute`
  operations against exact symbolic targets.
- Semantic rejection when an operation lacks a matching capability or effect,
  targets a `policy deny` symbol, or uses an unsupported security form.
- A mediated execution API that requires an **external grant** independently
  of source declarations and a **human review** provider when policy requires
  it. An approval interface is not an authentication service by itself.
- Source SHA-256 passed to grant, review, and host callbacks for revision
  binding. A configured evidence policy blocks effects when the host's
  pre-action record callback fails. Failure after an effect is reported without
  claiming rollback.
- A simulation CLI with a synthetic grant, no real effects, and no human
  approval. Denials exit nonzero.

## Try it

Requires Rust 1.98.1. The repository pins the toolchain and `Cargo.lock`.

```sh
cargo check --workspace --all-targets --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo run --locked -- check examples/secure_service.dcr
cargo run --locked -- simulate examples/simulated_network.dcr request
```

For an intentional compile-time policy denial:

```sh
cargo run --locked -- check examples/invalid/denied_shell.dcr
```

For an intentional runtime review denial in the no-op simulation:

```sh
cargo run --locked -- simulate examples/secure_service.dcr review_gate
```

The latter two commands return exit status 2. `ast <file>` displays parsed
syntax, `inspect <file>` reports source metrics, and `--help` lists commands.

## Language and boundary

```text
module example {
    capability network vendor;
    effect network vendor;
    effect evidence evidence_trace;
    policy requires human_review;
    policy ensures evidence_trace;

    fn request {
        network vendor;
    }
}
```

The `vendor` identifier is a symbolic binding. It is not a destination or a
permission granted by the source author. The trusted host must bind it to a
specific resource. To perform a real effect, an integrator calls `check` and
`CheckedModule::run` with implementations of `GrantAuthority`,
`ReviewAuthority`, and `OperationHost`. The grant and review providers must
authenticate and scope their decisions to the digest, function, operation,
target, and execution context. The host must isolate and audit real effects.

Legacy `fn name;` syntax is still recognized by `ast` but fails `check`. The
same is true of unsupported security declarations such as `capability secret`
and `policy allow`; the compiler does not silently treat them as enforcement.

Read [the checked language subset](docs/LANGUAGE_SUBSET.md) and
[the threat model](docs/THREAT_MODEL.md) before building an adapter. The
[validation report](VALIDATION_REPORT.md) records the local checks and their
limits. [Handoff instructions](HANDOFF.md) cover Windows validation and an
existing GitHub clone.

## Project status

The original archive's CI failure was formatting drift. This handoff pins
rustfmt through the Rust toolchain file and passes local formatting, Clippy,
build checking, and 61 tests. Remote GitHub CI has not run for this handoff
until it is committed and pushed. Security claims are limited to the checked
subset and its trusted-host interface. Independent security review remains
necessary before using it to protect real resources.

Decriel stands for Declarative Effects and Capabilities for Runtime Integrity,
Evidence, and Least-Authority. Copyright 2026 Bryce Lovell. Apache-2.0;
see `LICENSE` and `NOTICE`.
