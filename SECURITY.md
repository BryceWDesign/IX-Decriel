# Security Policy

## Project status

IX-Decriel is the official public research repository for Decriel, a
security-first programming language originated and created by Bryce Lovell.

Decriel v0.2.0 has a checked, mediated operation subset. It is **not** a
production sandbox, formally verified compiler, independently audited security
product, or suitable for protecting real secrets. The CLI simulation has no
external effects and its grant is synthetic. No actual network, file, or
process operation ships in this repository.

## Reporting security issues

Please report security issues through a private channel rather than a public
issue when the report could expose a vulnerability, exploit path, secret-handling
failure, unsafe runtime behavior, supply-chain weakness, or policy bypass.

If private repository security advisories are available, use GitHub's private
vulnerability reporting flow for this repository. If that is not available,
contact the repository owner directly through the public profile associated with
this repository.

## What to report

Useful reports include:

- memory-safety issues in the Rust implementation
- unsafe behavior that bypasses declared authority
- incorrect diagnostics that allow invalid Decriel source to pass
- dependency or build-chain risks
- secret exposure in logs, traces, diagnostics, or test output
- runtime behavior that violates fail-closed expectations
- incorrect attribution, license, or notice handling

## Out of scope

This repository does not accept reports that require unauthorized access,
service disruption, credential theft, social engineering, data destruction, or
testing against systems that the reporter does not own or have permission to
test.

## Security posture

The implemented subset checks exact `(action, symbolic target)` declarations,
rejects forbidden targets and unsupported security declarations, and mediates
operations behind external grant, review, and host adapter interfaces. The
`evidence_trace` policy requires the host audit callback before an effect. A
post-effect audit failure is reported, but cannot undo a completed effect.

Source SHA-256 binds callbacks to the checked bytes. A digest alone does not
authenticate a grant or approval, secure the host, make audit storage durable,
or protect a runtime from an untrusted OS process. Integration must verify
reviewer identity, scope, freshness, revocation, and host resource binding.
See `docs/THREAT_MODEL.md` for the trust boundary and limitations.

Security claims must be backed by implementation, tests, examples, and evidence.
