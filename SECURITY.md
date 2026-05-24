# Security Policy

## Project status

IX-Decriel is the official public research repository for Decriel, a
security-first programming language originated and created by Bryce Lovell.

Decriel is in active research and implementation. It should not be treated as
production-ready, formally certified, externally audited, or suitable for
protecting real secrets until the repository contains working implementation,
tests, review evidence, and appropriate security review for the relevant use
case.

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

The project goal is to build Decriel around explicit capabilities, declared
effects, least authority, secret-safe data handling, policy enforcement,
runtime evidence, and human-reviewable behavior.

Security claims must be backed by implementation, tests, examples, and evidence.
