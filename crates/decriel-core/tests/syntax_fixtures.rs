use std::error::Error;
use std::fs;
use std::path::PathBuf;

use decriel_core::{SourceDocument, parse, render_module_ast};

fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    path.push("..");
    path.push("..");
    path.push("examples");

    for part in parts {
        path.push(part);
    }

    path
}

fn read_fixture(parts: &[&str]) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(fixture_path(parts))?)
}

#[test]
fn minimal_fixture_parses_as_empty_module() -> Result<(), Box<dyn Error>> {
    let source = read_fixture(&["minimal.dcr"])?;
    let document = SourceDocument::new("examples/minimal.dcr", source);
    let result = parse(&document);

    assert!(!result.has_errors(), "{}", result.diagnostics().render());
    assert!(result.module().is_some());

    if let Some(module) = result.module() {
        assert_eq!(module.name().name(), "minimal");
        assert_eq!(module.len(), 0);
        assert!(module.is_empty());
    }

    Ok(())
}

#[test]
fn secure_service_fixture_parses_declaration_surface() -> Result<(), Box<dyn Error>> {
    let source = read_fixture(&["secure_service.dcr"])?;
    let document = SourceDocument::new("examples/secure_service.dcr", source);
    let result = parse(&document);

    assert!(!result.has_errors(), "{}", result.diagnostics().render());
    assert!(result.module().is_some());

    if let Some(module) = result.module() {
        let rendered = render_module_ast(module);

        assert_eq!(module.name().name(), "secure_service");
        assert_eq!(module.len(), 13);
        assert!(rendered.contains("declaration capability action=read target=customer_documents"));
        assert!(rendered.contains("declaration capability action=write target=verification_result"));
        assert!(rendered.contains("declaration capability action=network target=approved_vendor_api"));
        assert!(rendered.contains("declaration capability action=secret target=access_token"));
        assert!(rendered.contains("declaration effect action=trace target=audit_event"));
        assert!(rendered.contains("declaration effect action=evidence target=review_bundle"));
        assert!(rendered.contains("declaration policy action=deny target=shell_access"));
        assert!(rendered.contains("declaration policy action=requires target=human_review"));
        assert!(rendered.contains("declaration policy action=ensures target=evidence_trace"));
        assert!(rendered.contains("declaration function name=review_gate"));
    }

    Ok(())
}

#[test]
fn invalid_missing_module_name_fixture_reports_syntax_error() -> Result<(), Box<dyn Error>> {
    let source = read_fixture(&["invalid", "missing_module_name.dcr"])?;
    let document = SourceDocument::new("examples/invalid/missing_module_name.dcr", source);
    let result = parse(&document);
    let rendered = result.diagnostics().render();

    assert!(result.has_errors());
    assert!(result.module().is_none());
    assert!(rendered.contains("expected module name after 'module'"));

    Ok(())
}
