//! Command-line entry point for the Decriel toolchain.

use std::env;
use std::process::ExitCode;

use decriel_core::LanguageIdentity;

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut args = args.into_iter();

    match args.next().as_deref() {
        None | Some("--version") | Some("version") => Ok(LanguageIdentity::current().display_line()),
        Some("--help") | Some("help") => Ok(help_text()),
        Some(command) => Err(format!(
            "unknown command '{command}'. Run 'decriel --help' for available commands."
        )),
    }
}

fn help_text() -> String {
    [
        LanguageIdentity::current().display_line(),
        "",
        "Usage:",
        "  decriel --version",
        "  decriel --help",
        "",
        "Commands:",
        "  version       Print the Decriel toolchain version.",
        "  help          Print this help text.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn version_command_reports_decriel_identity() {
        let output = run(["version".to_owned()]).expect("version command should succeed");

        assert!(output.contains("Decriel"));
        assert!(output.contains("IX-Decriel"));
    }

    #[test]
    fn help_command_reports_usage() {
        let output = run(["help".to_owned()]).expect("help command should succeed");

        assert!(output.contains("Usage:"));
        assert!(output.contains("decriel --version"));
    }

    #[test]
    fn unknown_command_fails() {
        let error = run(["check".to_owned()]).expect_err("unknown command should fail");

        assert!(error.contains("unknown command"));
        assert!(error.contains("decriel --help"));
    }
}
