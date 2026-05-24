//! Core language identity and shared compiler-domain types for Decriel.
//!
//! `decriel-core` holds language-level constants and small domain types that are
//! shared by the command-line interface, diagnostics layer, parser, checker, and
//! later runtime components.

pub mod ast;
pub mod lexer;
pub mod source;
pub mod token;

pub use ast::{
    CapabilityAction, Declaration, DecrielModule, EffectAction, Identifier, PolicyAction,
};
pub use lexer::{LexResult, lex};
pub use source::SourceDocument;
pub use token::{Keyword, Symbol, Token, TokenKind};

/// Public name of the language.
pub const LANGUAGE_NAME: &str = "Decriel";

/// Official public research repository name.
pub const REPOSITORY_NAME: &str = "IX-Decriel";

/// Decriel acronym expansion used by project documentation and diagnostics.
pub const ACRONYM_EXPANSION: &str =
    "Declarative Effects and Capabilities for Runtime Integrity, Evidence, and Least-Authority";

/// Current research-stage toolchain version.
///
/// This intentionally follows the Cargo crate version for the first public
/// research implementation.
pub const TOOLCHAIN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stable identity metadata for the Decriel language and repository.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LanguageIdentity {
    /// Language name.
    pub language_name: &'static str,
    /// Official repository name.
    pub repository_name: &'static str,
    /// Acronym expansion.
    pub acronym_expansion: &'static str,
    /// Toolchain version.
    pub toolchain_version: &'static str,
}

impl LanguageIdentity {
    /// Returns the canonical Decriel identity record.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            language_name: LANGUAGE_NAME,
            repository_name: REPOSITORY_NAME,
            acronym_expansion: ACRONYM_EXPANSION,
            toolchain_version: TOOLCHAIN_VERSION,
        }
    }

    /// Returns a compact single-line identity string suitable for CLI output.
    #[must_use]
    pub fn display_line(self) -> String {
        format!(
            "{} {} ({})",
            self.language_name, self.toolchain_version, self.repository_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ACRONYM_EXPANSION, LANGUAGE_NAME, LanguageIdentity, REPOSITORY_NAME};

    #[test]
    fn identity_uses_locked_language_and_repository_names() {
        let identity = LanguageIdentity::current();

        assert_eq!(identity.language_name, LANGUAGE_NAME);
        assert_eq!(identity.repository_name, REPOSITORY_NAME);
        assert_eq!(identity.language_name, "Decriel");
        assert_eq!(identity.repository_name, "IX-Decriel");
    }

    #[test]
    fn identity_preserves_acronym_expansion() {
        let identity = LanguageIdentity::current();

        assert_eq!(identity.acronym_expansion, ACRONYM_EXPANSION);
        assert!(identity.acronym_expansion.contains("Declarative Effects"));
        assert!(identity.acronym_expansion.contains("Least-Authority"));
    }

    #[test]
    fn display_line_contains_language_version_and_repository() {
        let display = LanguageIdentity::current().display_line();

        assert!(display.contains("Decriel"));
        assert!(display.contains("0.1.0"));
        assert!(display.contains("IX-Decriel"));
    }
}
