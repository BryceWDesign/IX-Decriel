//! Security boundary tests: denied requests must never reach the host.

use decriel_core::{
    check, CapabilityAction, GrantAuthority, HostError, NoGrant, NoReview, Operation,
    OperationHost, Record, ReviewAuthority, SourceDocument,
};

const BASE: &str = "module gate {
    capability network vendor;
    effect network vendor;
    effect evidence evidence_trace;
    policy deny shell_access;
    policy requires human_review;
    policy ensures evidence_trace;
    fn dispatch { network vendor; }
}";

fn checked(source: &str) -> Result<decriel_core::CheckedModule, String> {
    check(&SourceDocument::new("test.dcr", source)).map_err(|report| report.render())
}

struct Reviewed;

struct Granted;

struct PinnedGrant<'a>(&'a str);

impl GrantAuthority for PinnedGrant<'_> {
    fn granted(&self, digest: &str, _: &str, _: &str, _: CapabilityAction, _: &str) -> bool {
        digest == self.0
    }
}

struct PinnedReview<'a>(&'a str);

impl ReviewAuthority for PinnedReview<'_> {
    fn approved(&self, digest: &str, _: &str, _: &str, _: CapabilityAction, _: &str) -> bool {
        digest == self.0
    }
}

impl GrantAuthority for Granted {
    fn granted(
        &self,
        _: &str,
        module: &str,
        function: &str,
        action: CapabilityAction,
        target: &str,
    ) -> bool {
        module == "gate"
            && function == "dispatch"
            && action == CapabilityAction::Network
            && target == "vendor"
    }
}

impl ReviewAuthority for Reviewed {
    fn approved(
        &self,
        _: &str,
        module: &str,
        function: &str,
        action: CapabilityAction,
        target: &str,
    ) -> bool {
        module == "gate"
            && function == "dispatch"
            && action == CapabilityAction::Network
            && target == "vendor"
    }
}

#[derive(Default)]
struct Host {
    performed: Vec<String>,
    recorded: Vec<String>,
    fail_record_at: usize,
    fail_perform: bool,
}

impl OperationHost for Host {
    fn record(&mut self, _: &str, _: &str, _: &str, record: &Record) -> Result<(), HostError> {
        self.recorded.push(record.reason.to_owned());
        if self.recorded.len() == self.fail_record_at {
            Err(HostError)
        } else {
            Ok(())
        }
    }

    fn perform(
        &mut self,
        _: &str,
        _: &str,
        _: &str,
        operation: &Operation,
    ) -> Result<(), HostError> {
        self.performed.push(operation.target.name().to_owned());
        if self.fail_perform {
            Err(HostError)
        } else {
            Ok(())
        }
    }
}

#[test]
fn review_is_independent_of_declarations_and_blocks_host() -> Result<(), String> {
    let plan = checked(BASE)?;
    let mut host = Host::default();
    let report = plan.run("dispatch", &Granted, &NoReview, &mut host);
    assert!(!report.complete);
    assert_eq!(report.records[0].reason, "human_review_required");
    assert!(host.performed.is_empty());
    assert_eq!(host.recorded, ["attempt", "human_review_required"]);
    Ok(())
}

#[test]
fn scoped_review_allows_exact_checked_operation() -> Result<(), String> {
    let plan = checked(BASE)?;
    let mut host = Host::default();
    let report = plan.run("dispatch", &Granted, &Reviewed, &mut host);
    assert!(report.complete);
    assert_eq!(host.performed, ["vendor"]);
    assert_eq!(host.recorded, ["attempt", "performed"]);
    assert_eq!(report.records[0].reason, "performed");
    assert!(plan.evidence_required());
    Ok(())
}

#[test]
fn source_declarations_do_not_create_an_external_grant() -> Result<(), String> {
    let plan = checked(BASE)?;
    let mut host = Host::default();
    let report = plan.run("dispatch", &NoGrant, &Reviewed, &mut host);
    assert_eq!(report.records[0].reason, "host_grant_required");
    assert!(host.performed.is_empty());
    Ok(())
}

#[test]
fn grants_and_reviews_can_bind_exact_source_revision() -> Result<(), String> {
    let original = checked(BASE)?;
    let changed = checked(&format!("{BASE}\n// revised source"))?;
    assert_ne!(original.source_sha256(), changed.source_sha256());
    assert_eq!(original.source_sha256().len(), 64);
    let mut host = Host::default();
    let report = changed.run(
        "dispatch",
        &PinnedGrant(original.source_sha256()),
        &Reviewed,
        &mut host,
    );
    assert_eq!(report.records[0].reason, "host_grant_required");
    let report = changed.run(
        "dispatch",
        &Granted,
        &PinnedReview(original.source_sha256()),
        &mut host,
    );
    assert_eq!(report.records[0].reason, "human_review_required");
    assert!(host.performed.is_empty());
    Ok(())
}

#[test]
fn exact_target_and_function_cannot_be_substituted() -> Result<(), String> {
    let plan = checked(BASE)?;
    let altered = plan.decide(
        "dispatch",
        CapabilityAction::Network,
        "vendor_other",
        &Granted,
        &Reviewed,
    );
    assert_eq!(altered.reason, "undeclared_operation");
    let altered = plan.decide(
        "other",
        CapabilityAction::Network,
        "vendor",
        &Granted,
        &Reviewed,
    );
    assert_eq!(altered.reason, "unknown_function");
    let altered = plan.decide(
        "dispatch",
        CapabilityAction::Execute,
        "vendor",
        &Granted,
        &Reviewed,
    );
    assert_eq!(altered.reason, "undeclared_operation");
    Ok(())
}

#[test]
fn evidence_prewrite_failure_blocks_effect() -> Result<(), String> {
    let plan = checked(BASE)?;
    let mut host = Host {
        fail_record_at: 1,
        ..Host::default()
    };
    let report = plan.run("dispatch", &Granted, &Reviewed, &mut host);
    assert_eq!(report.records[0].reason, "evidence_failed");
    assert!(host.performed.is_empty());
    Ok(())
}

#[test]
fn postwrite_failure_is_reported_without_claiming_rollback() -> Result<(), String> {
    let plan = checked(BASE)?;
    let mut host = Host {
        fail_record_at: 2,
        ..Host::default()
    };
    let report = plan.run("dispatch", &Granted, &Reviewed, &mut host);
    assert_eq!(report.records[0].reason, "post_evidence_failed");
    assert!(!report.complete);
    assert_eq!(host.performed, ["vendor"]);
    Ok(())
}

#[test]
fn host_failure_stops_subsequent_steps() -> Result<(), String> {
    let source = BASE.replace("network vendor; }", "network vendor; network vendor; }");
    let plan = checked(&source)?;
    let mut host = Host {
        fail_perform: true,
        ..Host::default()
    };
    let report = plan.run("dispatch", &Granted, &Reviewed, &mut host);
    assert_eq!(report.records[0].reason, "host_failed");
    assert_eq!(report.records.len(), 1);
    assert_eq!(host.performed.len(), 1);
    Ok(())
}

#[test]
fn denied_policy_fails_at_compile_time_even_with_capability() {
    let source = "module gate {
        capability execute shell_access;
        effect execute shell_access;
        policy deny shell_access;
        fn dispatch { execute shell_access; }
    }";
    let failure = checked(source).err().unwrap_or_default();
    assert!(failure.contains("policy denies target shell_access"));
}

#[test]
fn missing_capability_or_effect_is_rejected() {
    for source in [
        "module gate { effect network vendor; fn dispatch { network vendor; } }",
        "module gate { capability network vendor; fn dispatch { network vendor; } }",
    ] {
        let failure = checked(source).err().unwrap_or_default();
        assert!(failure.contains("missing capability") || failure.contains("missing effect"));
    }
}

#[test]
fn inert_security_promises_are_errors() {
    for source in [
        "module gate { capability secret password; }",
        "module gate { effect trace audit; }",
        "module gate { policy allow vendor; }",
        "module gate { policy requires token; }",
        "module gate { policy ensures evidence_trace; }",
        "module gate { fn dispatch; }",
    ] {
        assert!(checked(source).is_err(), "accepted inert promise: {source}");
    }
}

#[test]
fn malformed_body_and_duplicate_authority_fail_closed() {
    for source in [
        "module gate { fn dispatch { network vendor } }",
        "module gate { fn dispatch { secret password; } }",
        "module gate { capability network vendor; capability network vendor; }",
        "module gate { fn dispatch {} fn dispatch {} }",
    ] {
        assert!(
            checked(source).is_err(),
            "accepted malformed source: {source}"
        );
    }
}
