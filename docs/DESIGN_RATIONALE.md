# Design rationale

The attached research projects provided behavioral patterns, not source code
or a runtime dependency. Decriel remains a Rust workspace.

| Pattern considered | Decriel choice |
| --- | --- |
| BlackFox live authority separates authorization, verified evidence, and human review | Separate external `GrantAuthority`, `ReviewAuthority`, and pre-action `OperationHost::record` gates |
| HapticSight places independent safety authority between proposal and actuation | `CheckedModule::run` is the only mediated route from a checked step to the host adapter |
| SynapDrive distinguishes simulation from external action | CLI simulation labels its synthetic grant and uses a no-op host |
| Orbital edge assurance emphasizes replayable decisions and claim boundaries | Stable reason codes and a source digest are returned, while durable replay and signed evidence remain out of scope |
| Autonomy assurance case runtime models explicit evidence obligations | The evidence policy rejects a host record failure before a step, and exposes post-action record failure without a rollback claim |
| IX language project explores structured language declarations | Decriel retains its own narrow grammar and checks only implemented forms |

The useful combination is the intersection of checked source intent, a host
grant, an independent review decision, and audit availability at the exact
operation boundary. SHA-256 source binding prevents an integrator that pins
grants and reviews to the digest from reusing them for changed source. These
building blocks have prior art; the repository makes no novelty or certification
claim. Its value depends on secure host integration and independent review.
