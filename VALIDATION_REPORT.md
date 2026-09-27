# Validation report, v0.2.0

## Local verification

Tested on Linux with Rust 1.98.1 and the checked in `Cargo.lock`.

| Gate | Result |
| --- | --- |
| `cargo check --workspace --all-targets --locked` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 61 tests |

The original archive had 48 passing tests and a failing rustfmt check. The
added tests exercise external grant denial, independent review denial, exact
target and action matching, denied policy rejection, missing capability and
effect rejection, unsupported declaration rejection, audit failures before and
after an effect, host failure, and source digest pinning.

CLI checks: `check examples/secure_service.dcr` succeeds;
`check examples/invalid/denied_shell.dcr` returns error status 2;
`simulate examples/secure_service.dcr review_gate` returns error status 2 with
`human_review_required`; and `simulate examples/simulated_network.dcr request`
completes against the no-op host.

This is local verification of the delivered files. GitHub CI will run only
after the files are committed and pushed. There has been no independent
security audit, production adapter validation, or remote CI verification in
this handoff.
