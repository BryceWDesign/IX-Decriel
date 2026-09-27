# Checked language subset, v0.2.0

```
module name {
    capability network vendor;
    effect network vendor;
    effect evidence evidence_trace;
    policy deny shell_access;
    policy requires human_review;
    policy ensures evidence_trace;

    fn dispatch {
        network vendor;
    }
}
```

The checked operations are `read`, `write`, `network`, and `execute`, each
followed by one identifier and `;`. Identifiers are exact symbolic host
bindings. For every operation in a function, the module must declare both
`capability <action> <target>;` and `effect <action> <target>;`.

`policy deny <target>;` prohibits all operations on that exact target.
`policy requires human_review;` makes the external review callback mandatory
for every operation. `policy ensures evidence_trace;` requires
`effect evidence evidence_trace;` and a successful pre-action audit callback.
The source SHA-256 is passed to the external grant, review, and host callbacks.
These features compose by intersection, so no declaration can override a deny.

The parser retains legacy declaration shells for AST inspection. `check`
rejects a bodyless function, unsupported declaration, duplicate authority,
denied operation, or operation missing either declaration. The checker does
not infer network destinations, filesystem paths, secret flows, or shell
commands from identifiers. Only a host integration can bind an identifier to
a real resource.

`decriel simulate` executes this control flow against a no-op host with a
synthetic grant and no human approval. It returns a nonzero status on denial.
Use the Rust `check`, `GrantAuthority`, `ReviewAuthority`, and `OperationHost`
interfaces for an actual host integration. Review the threat model before
binding these interfaces to real effects.
