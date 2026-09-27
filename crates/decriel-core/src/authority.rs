//! Fail-closed semantics for the bounded Decriel operation language.
//!
//! A symbol is a host binding, not a URL, filesystem path, or process name.
//! Integrators must bind it to a resource in their own trusted host.

use std::collections::HashSet;

use decriel_diagnostics::{Diagnostic, DiagnosticReport, DiagnosticSeverity, SourceSpan};
use sha2::{Digest, Sha256};

use crate::ast::{
    CapabilityAction, Declaration, DecrielModule, EffectAction, Operation, PolicyAction,
};
use crate::parser::parse;
use crate::source::SourceDocument;

/// A module with all supported declarations and function operations checked.
/// Its fields are private so callers cannot construct an unchecked plan.
#[derive(Debug, Clone)]
pub struct CheckedModule {
    module: DecrielModule,
    source_sha256: String,
    capabilities: HashSet<(String, String)>,
    effects: HashSet<(String, String)>,
    denied: HashSet<String>,
    review_required: bool,
    evidence_required: bool,
}

impl CheckedModule {
    /// Name recorded in decisions and sent to the trusted host.
    #[must_use]
    pub fn name(&self) -> &str {
        self.module.name().name()
    }

    /// SHA-256 of the exact UTF-8 source bytes checked by this instance.
    #[must_use]
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    /// Check one requested operation. The review provider is supplied by the
    /// integrating host, never by untrusted Decriel source.
    #[must_use]
    pub fn decide(
        &self,
        function: &str,
        action: CapabilityAction,
        target: &str,
        grant: &impl GrantAuthority,
        reviewer: &impl ReviewAuthority,
    ) -> Decision {
        let pair = (action.as_str().to_owned(), target.to_owned());
        let steps = self.module.declarations().iter().find_map(|declaration| {
            if let Declaration::Executable { name, steps, .. } = declaration {
                (name.name() == function).then_some(steps.as_slice())
            } else {
                None
            }
        });
        let reason = if !matches!(
            action,
            CapabilityAction::Read
                | CapabilityAction::Write
                | CapabilityAction::Network
                | CapabilityAction::Execute
        ) {
            "unsupported_operation"
        } else if steps.is_none() {
            "unknown_function"
        } else if !steps.is_some_and(|steps| {
            steps
                .iter()
                .any(|step| step.action == action && step.target.name() == target)
        }) {
            "undeclared_operation"
        } else if !self.capabilities.contains(&pair) {
            "missing_capability"
        } else if !self.effects.contains(&pair) {
            "missing_effect"
        } else if self.denied.contains(target) {
            "policy_denied"
        } else if !grant.granted(self.source_sha256(), self.name(), function, action, target) {
            "host_grant_required"
        } else if self.review_required
            && !reviewer.approved(self.source_sha256(), self.name(), function, action, target)
        {
            "human_review_required"
        } else {
            "allowed"
        };
        Decision {
            allowed: reason == "allowed",
            reason,
        }
    }

    /// Execute a checked function through a host-controlled adapter. No OS
    /// effects are implemented in this crate. A denied step stops the sequence.
    #[must_use]
    pub fn run(
        &self,
        function: &str,
        grant: &impl GrantAuthority,
        reviewer: &impl ReviewAuthority,
        host: &mut impl OperationHost,
    ) -> RunReport {
        let Some(steps) = self.module.declarations().iter().find_map(|declaration| {
            if let Declaration::Executable { name, steps, .. } = declaration {
                (name.name() == function).then_some(steps.as_slice())
            } else {
                None
            }
        }) else {
            return RunReport {
                source_sha256: self.source_sha256.clone(),
                complete: false,
                records: vec![Record::new(0, "unknown_function", "", "")],
            };
        };

        let mut records = Vec::new();
        for (index, step) in steps.iter().enumerate() {
            let decision = self.decide(function, step.action, step.target.name(), grant, reviewer);
            let attempt = Record::new(
                index + 1,
                "attempt",
                step.action.as_str(),
                step.target.name(),
            );
            let mut reason = decision.reason;
            if self.evidence_required
                && host
                    .record(self.source_sha256(), self.name(), function, &attempt)
                    .is_err()
            {
                reason = "evidence_failed";
            } else if decision.allowed {
                reason = match host.perform(self.source_sha256(), self.name(), function, step) {
                    Ok(()) => "performed",
                    Err(HostError) => "host_failed",
                };
            }
            let outcome = Record::new(index + 1, reason, step.action.as_str(), step.target.name());
            if self.evidence_required
                && host
                    .record(self.source_sha256(), self.name(), function, &outcome)
                    .is_err()
            {
                reason = "post_evidence_failed";
            }
            records.push(Record::new(
                index + 1,
                reason,
                step.action.as_str(),
                step.target.name(),
            ));
            if reason != "performed" {
                return RunReport {
                    source_sha256: self.source_sha256.clone(),
                    complete: false,
                    records,
                };
            }
        }
        RunReport {
            source_sha256: self.source_sha256.clone(),
            complete: true,
            records,
        }
    }

    /// Whether the source requested a per-step in-memory evidence record.
    #[must_use]
    pub const fn evidence_required(&self) -> bool {
        self.evidence_required
    }
}

/// A separate host-owned grant boundary. Source declarations cannot create a
/// runtime grant. The integrator must bind this to authenticated authority.
pub trait GrantAuthority {
    /// Return true only when this exact request falls within an external grant.
    fn granted(
        &self,
        source_sha256: &str,
        module: &str,
        function: &str,
        action: CapabilityAction,
        target: &str,
    ) -> bool;
}

/// Default external grant provider denies every operation.
pub struct NoGrant;

impl GrantAuthority for NoGrant {
    fn granted(&self, _: &str, _: &str, _: &str, _: CapabilityAction, _: &str) -> bool {
        false
    }
}

/// Host-owned review authority. The caller must authenticate, scope, expire,
/// and bind approvals to the requested operation before returning true.
pub trait ReviewAuthority {
    /// Return true only for a verified human approval of this precise request.
    fn approved(
        &self,
        source_sha256: &str,
        module: &str,
        function: &str,
        action: CapabilityAction,
        target: &str,
    ) -> bool;
}

/// A default review provider that grants no authority.
pub struct NoReview;

impl ReviewAuthority for NoReview {
    fn approved(&self, _: &str, _: &str, _: &str, _: CapabilityAction, _: &str) -> bool {
        false
    }
}

/// An adapter failure without sensitive implementation details.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HostError;

/// All external work is delegated to an integrating host. This trait does not
/// sandbox the host; the host must enforce its own binding and isolation.
pub trait OperationHost {
    /// Persist an audit record before admitting an effect and after its result.
    /// A failure before the effect blocks it. An after-effect failure cannot
    /// undo an external effect and is reported as post_evidence_failed.
    fn record(
        &mut self,
        source_sha256: &str,
        module: &str,
        function: &str,
        record: &Record,
    ) -> Result<(), HostError>;

    /// Perform an already authorized operation. An error stops subsequent steps.
    fn perform(
        &mut self,
        source_sha256: &str,
        module: &str,
        function: &str,
        operation: &Operation,
    ) -> Result<(), HostError>;
}

/// An authorization result with a stable machine-readable reason code.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Decision {
    /// Whether the operation may reach the host.
    pub allowed: bool,
    /// Precise reason code.
    pub reason: &'static str,
}

/// A record of one attempted operation. Records are in memory only.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Record {
    /// One-based step index; zero represents an unknown function.
    pub step: usize,
    /// Result reason.
    pub reason: &'static str,
    /// Operation kind.
    pub action: String,
    /// Symbolic target.
    pub target: String,
}

impl Record {
    fn new(step: usize, reason: &'static str, action: &str, target: &str) -> Self {
        Self {
            step,
            reason,
            action: action.to_owned(),
            target: target.to_owned(),
        }
    }
}

/// The mediated sequence result. A failed step stops later steps.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RunReport {
    /// SHA-256 of the exact checked source bytes.
    pub source_sha256: String,
    /// Whether every step returned success from the host.
    pub complete: bool,
    /// One record per attempted step.
    pub records: Vec<Record>,
}

/// Parse and semantically check a source document. Unsupported security
/// declarations fail closed instead of being accepted as inert promises.
pub fn check(document: &SourceDocument) -> Result<CheckedModule, DiagnosticReport> {
    let result = parse(document);
    if result.has_errors() {
        return Err(result.into_parts().1);
    }
    let Some(module) = result.module() else {
        let mut report = DiagnosticReport::new();
        report.push(Diagnostic::new(
            DiagnosticSeverity::Error,
            "parser produced no module",
        ));
        return Err(report);
    };
    check_module(
        module.clone(),
        format!("{:x}", Sha256::digest(document.text().as_bytes())),
    )
}

fn error(report: &mut DiagnosticReport, span: SourceSpan, message: impl Into<String>) {
    report.push(Diagnostic::with_span(
        DiagnosticSeverity::Error,
        message,
        span,
    ));
}

fn check_module(
    module: DecrielModule,
    source_sha256: String,
) -> Result<CheckedModule, DiagnosticReport> {
    let mut report = DiagnosticReport::new();
    let mut capabilities = HashSet::new();
    let mut effects = HashSet::new();
    let mut denied = HashSet::new();
    let mut functions = HashSet::new();
    let mut review_required = false;
    let mut evidence_required = false;

    for declaration in module.declarations() {
        match declaration {
            Declaration::Capability {
                action,
                target,
                span,
            } => {
                if *action == CapabilityAction::Secret {
                    error(&mut report, *span, "secret capability is not implemented");
                } else if !capabilities
                    .insert((action.as_str().to_owned(), target.name().to_owned()))
                {
                    error(&mut report, *span, "duplicate capability");
                }
            }
            Declaration::Effect {
                action,
                target,
                span,
            } => {
                if *action == EffectAction::Trace
                    || (*action == EffectAction::Evidence && target.name() != "evidence_trace")
                {
                    error(
                        &mut report,
                        *span,
                        "effect is not supported by the mediated runtime",
                    );
                } else if !effects.insert((action.as_str().to_owned(), target.name().to_owned())) {
                    error(&mut report, *span, "duplicate effect");
                }
            }
            Declaration::Policy {
                action,
                target,
                span,
            } => match action {
                PolicyAction::Deny => {
                    if !denied.insert(target.name().to_owned()) {
                        error(&mut report, *span, "duplicate deny policy");
                    }
                }
                PolicyAction::Requires if target.name() == "human_review" => {
                    if review_required {
                        error(&mut report, *span, "duplicate human review policy");
                    }
                    review_required = true;
                }
                PolicyAction::Ensures if target.name() == "evidence_trace" => {
                    if evidence_required {
                        error(&mut report, *span, "duplicate evidence policy");
                    }
                    evidence_required = true;
                }
                _ => error(&mut report, *span, "policy form is not implemented"),
            },
            Declaration::Function { name, span } => {
                if !functions.insert(name.name().to_owned()) {
                    error(&mut report, *span, "duplicate function");
                }
                error(&mut report, *span, "function has no executable body");
            }
            Declaration::Executable { name, span, .. } => {
                if !functions.insert(name.name().to_owned()) {
                    error(&mut report, *span, "duplicate function");
                }
            }
        }
    }

    if evidence_required && !effects.contains(&("evidence".to_owned(), "evidence_trace".to_owned()))
    {
        error(
            &mut report,
            module.span(),
            "evidence_trace policy requires 'effect evidence evidence_trace'",
        );
    }

    for declaration in module.declarations() {
        if let Declaration::Executable { steps, .. } = declaration {
            for step in steps {
                let pair = (
                    step.action.as_str().to_owned(),
                    step.target.name().to_owned(),
                );
                if !capabilities.contains(&pair) {
                    error(
                        &mut report,
                        step.span,
                        format!("missing capability {} {}", pair.0, pair.1),
                    );
                }
                if !effects.contains(&pair) {
                    error(
                        &mut report,
                        step.span,
                        format!("missing effect {} {}", pair.0, pair.1),
                    );
                }
                if denied.contains(step.target.name()) {
                    error(
                        &mut report,
                        step.span,
                        format!("policy denies target {}", pair.1),
                    );
                }
            }
        }
    }

    if report.has_errors() {
        Err(report)
    } else {
        Ok(CheckedModule {
            module,
            source_sha256,
            capabilities,
            effects,
            denied,
            review_required,
            evidence_required,
        })
    }
}
