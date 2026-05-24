//! Command-line entry point for the Decriel toolchain.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use decriel_core::{LanguageIdentity, SourceDocument, parse};

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => write_success(&output),
        Err(message) => write_failure(&message),
    }
}

fn write_success(output: &str) -> ExitCode {
    if output.is_empty() {
        return ExitCode::SUCCESS;
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match writeln!(handle, "{output}") {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}

fn write_failure(message: &str) -> ExitCode {
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    match writeln!(handle, "{message}") {
        Ok(()) => ExitCode::from(2),
        Err(_) => ExitCode::from(1),
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut args = args.into_iter();

    match args.next().as_deref() {
        None | Some("--version") | Some("version") => Ok(LanguageIdentity::current().display_line()),
        Some("--help") | Some("help") => Ok(help_text()),
        Some("inspect") => inspect_command(&mut args),
        Some("check") => check_command(&mut args),
        Some(command) => Err(format!(
            "unknown command '{command}'. Run 'decriel --help' for available commands."
        )),
    }
}

fn inspect_command(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let Some(path_text) = args.next() else {
        return Err("missing file path. Usage: decriel inspect <file>".to_owned());
    };

    if let Some(extra) = args.next() {
        return Err(format!(
            "unexpected extra argument '{extra}'. Usage: decriel inspect <file>"
        ));
    }

    inspect_source(Path::new(&path_text))
}

fn inspect_source(path: &Path) -> Result<String, String> {
    let source = read_source_file(path)?;
    let document = SourceDocument::new(path, source);

    Ok(format!(
        "status: source-loaded\nsource_path: {}\nbyte_length: {}\nline_count: {}\ntrailing_newline: {}",
        document.path().display(),
        document.byte_len(),
        document.line_count(),
        document.has_trailing_newline()
    ))
}

fn check_command(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let Some(path_text) = args.next() else {
        return Err("missing file path. Usage: decriel check <file>".to_owned());
    };

    if let Some(extra) = args.next() {
        return Err(format!(
            "unexpected extra argument '{extra}'. Usage: decriel check <file>"
        ));
    }

    check_source(Path::new(&path_text))
}

fn check_source(path: &Path) -> Result<String, String> {
    let source = read_source_file(path)?;
    let document = SourceDocument::new(path, source);

    check_loaded_source(&document)
}

fn check_loaded_source(document: &SourceDocument) -> Result<String, String> {
    let result = parse(document);

    if result.has_errors() {
        let diagnostics = result.diagnostics().render();

        return Err(format!(
            "status: syntax-error\nsource_path: {}\n{}",
            document.path().display(),
            diagnostics
        ));
    }

    let Some(module) = result.module() else {
        return Err(format!(
            "status: syntax-error\nsource_path: {}\nerror: parser produced no module",
            document.path().display()
        ));
    };

    Ok(format!(
        "status: syntax-ok\nsource_path: {}\nmodule: {}\ndeclarations: {}",
        document.path().display(),
        module.name().name(),
        module.len()
    ))
}

fn read_source_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read '{}': {error}", path.display()))
}

fn help_text() -> String {
    [
        LanguageIdentity::current().display_line(),
        "",
        "Usage:",
        "  decriel --version",
        "  decriel --help",
        "  decriel inspect <file>",
        "  decriel check <file>",
        "",
        "Commands:",
        "  version          Print the Decriel toolchain version.",
        "  help             Print this help text.",
        "  inspect <file>   Load a source file and report foundational source metrics.",
        "  check <file>     Parse a Decriel source file and report syntax status.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use decriel_core::SourceDocument;

    use super::{check_loaded_source, run};

    #[test]
    fn version_command_reports_decriel_identity() {
        let result = run(["version".to_owned()]);

        assert!(result.as_ref().is_ok());

        if let Ok(output) = result {
            assert!(output.contains("Decriel"));
            assert!(output.contains("IX-Decriel"));
        }
    }

    #[test]
    fn help_command_reports_usage() {
        let result = run(["help".to_owned()]);

        assert!(result.as_ref().is_ok());

        if let Ok(output) = result {
            assert!(output.contains("Usage:"));
            assert!(output.contains("decriel --version"));
            assert!(output.contains("decriel inspect <file>"));
            assert!(output.contains("decriel check <file>"));
        }
    }

    #[test]
    fn inspect_command_reports_source_metrics_for_existing_file() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_path = manifest_dir.join("src/main.rs");
        let result = run([
            "inspect".to_owned(),
            source_path.to_string_lossy().into_owned(),
        ]);

        assert!(result.as_ref().is_ok());

        if let Ok(output) = result {
            assert!(output.contains("status: source-loaded"));
            assert!(output.contains("source_path:"));
            assert!(output.contains("byte_length:"));
            assert!(output.contains("line_count:"));
            assert!(output.contains("trailing_newline:"));
        }
    }

    #[test]
    fn inspect_command_requires_file_path() {
        let result = run(["inspect".to_owned()]);

        assert!(result.as_ref().is_err());

        if let Err(error) = result {
            assert!(error.contains("missing file path"));
            assert!(error.contains("decriel inspect <file>"));
        }
    }

    #[test]
    fn check_command_requires_file_path() {
        let result = run(["check".to_owned()]);

        assert!(result.as_ref().is_err());

        if let Err(error) = result {
            assert!(error.contains("missing file path"));
            assert!(error.contains("decriel check <file>"));
        }
    }

    #[test]
    fn check_loaded_source_reports_valid_decriel_syntax() {
        let document = SourceDocument::new(
            Path::new("valid.dcr"),
            "module secure_service {
                capability network outbound_api;
                effect trace audit_event;
                policy deny shell_access;
                fn review_gate;
            }",
        );

        let result = check_loaded_source(&document);

        assert!(result.as_ref().is_ok());

        if let Ok(output) = result {
            assert!(output.contains("status: syntax-ok"));
            assert!(output.contains("source_path: valid.dcr"));
            assert!(output.contains("module: secure_service"));
            assert!(output.contains("declarations: 4"));
        }
    }

    #[test]
    fn check_loaded_source_reports_invalid_decriel_syntax() {
        let document = SourceDocument::new(Path::new("invalid.dcr"), "module {}");

        let result = check_loaded_source(&document);

        assert!(result.as_ref().is_err());

        if let Err(error) = result {
            assert!(error.contains("status: syntax-error"));
            assert!(error.contains("source_path: invalid.dcr"));
            assert!(error.contains("expected module name after 'module'"));
        }
    }

    #[test]
    fn unknown_command_fails() {
        let result = run(["execute".to_owned()]);

        assert!(result.as_ref().is_err());

        if let Err(error) = result {
            assert!(error.contains("unknown command"));
            assert!(error.contains("decriel --help"));
        }
    }
}
