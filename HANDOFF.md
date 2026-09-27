# IX-Decriel v0.2.0 handoff

## Contents

The ZIP contains the full Rust workspace, committed lockfile, toolchain pin,
CI workflow, functional language subset, positive and negative example
programs, security and language documentation, and validation report. There
are no compiled binaries or external donor repositories inside it.

## Windows PowerShell verification

Install Rust with rustup, extract the ZIP, and run these commands in the
directory containing `Cargo.toml`:

```powershell
cargo check --workspace --all-targets --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo run --locked -- check examples/secure_service.dcr
cargo run --locked -- simulate examples/simulated_network.dcr request
```

The reviewed service intentionally denies simulation without authenticated
human review. Its CLI command returns exit status 2:

```powershell
cargo run --locked -- simulate examples/secure_service.dcr review_gate
```

## Publish to the existing GitHub repository

Use a clean clone to preserve Git history. Copy the extracted handoff contents
into that clone, including `.github`, but exclude the extracted folder itself
and do not copy a generated `target` folder. Then, in the clone:

```powershell
git status --short
git diff --check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
git add .
git commit -m "Implement checked Decriel authority and mediated execution"
git push origin main
```

Inspect the Rust workflow status for the pushed commit. Local green status is
not a claim that GitHub has completed CI.

## Before a real integration

Read `SECURITY.md` and `docs/THREAT_MODEL.md`. Use authenticated and
revision-bound grant/review providers, a trusted adapter with exact symbolic
resource bindings, and a durable audit callback. Do not treat the CLI
simulation as a network client, a human approval flow, or a sandbox.
