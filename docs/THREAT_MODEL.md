# Threat model and integration boundary

## What the implementation enforces

`check` parses the entire source and returns a `CheckedModule` only when each
function body uses exact capabilities and effects, does not use a denied target,
and contains no unsupported security promises. A `fn name;` shell remains
parseable for AST inspection but cannot pass semantic checking. There is no
dynamic dispatch, import execution, FFI, shell, path, URL, or arbitrary code
expression in the checked subset.

`CheckedModule::run` considers these independent conditions before calling
`OperationHost::perform`: the operation appears in the checked function,
capability and effect match the exact action and symbol, no deny policy covers
the symbol, the external grant accepts the exact request, and the external
human review accepts it when required. The source SHA-256 is supplied to each
external callback. If `policy ensures evidence_trace` is present, the host's
`record` callback must succeed for the pre-action attempt before `perform`.
On any denial or adapter failure, later steps do not run.

## Trusted computing base

The Rust checker and mediator, the process invoking them, and the host adapter
are trusted. The host must map each symbolic target to a specific resource and
implement isolation for file, network, or process effects. A hostile host can
ignore the decision, misbind a symbol, lie about a write, or fabricate audit
success. The language does not sandbox it. Do not expose independent effect
paths that bypass `run`.

The grant authority must authenticate the principal and constrain action,
target, function, source digest, purpose, validity window, and revocation. The
review authority must verify an actual human decision independently and bind
it to the same context. A caller implemented `true` return value is not proof
of an authenticated grant or review. `NoGrant` and `NoReview` deny by default.

The evidence host must persist both attempt and outcome with a durable binding
to the digest and execution identity if a production audit guarantee is
required. This crate returns in-memory records only. It provides no signature,
append-only store, crash consistency, replay defense, or tamper detection for
those records. SHA-256 only identifies the checked source bytes.

## Failure boundaries

- A failed pre-action record blocks the effect. If the post-action record fails,
  the external effect may already have happened; the report says
  `post_evidence_failed` and stops subsequent steps. No rollback is claimed.
- Host `perform` may partially execute before returning `HostError`. The report
  stops subsequent steps but cannot reverse the partial effect.
- `policy deny shell_access` denies the exact symbol `shell_access`. If a host
  binds another symbol to a shell, this compiler cannot discover that.
- Approval callbacks are called at each step. This crate does not itself
  enforce timeouts, single-use grants, or revocation; those belong in the
  trusted authority implementations.
- The CLI `simulate` adapter accepts a synthetic grant but always uses
  `NoReview` and a no-op host. It never exercises real IO or human approval.
- `capability secret`, `effect trace`, `policy allow`, arbitrary `requires` or
  `ensures` forms, and bodyless functions fail semantic checking. Future
  syntax recognition does not imply implemented secret handling.

## Security review before production use

An adopter needs a reviewed host integration, authenticated authorities,
durable audit storage, resource isolation, revocation behavior, threat tests
against actual adapters, and an independent code and dependency review. None
of those are asserted by this repository.
