//! Token model for Decriel source text.
//!
//! Tokens are the first structured representation of Decriel source. They keep
//! syntax explicit before later waves add parsing, capability validation, effect
//! tracking, and runtime enforcement.

use decriel_diagnostics::SourceSpan;

/// Reserved Decriel keywords recognized by the lexer.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Keyword {
    /// Declares a Decriel module.
    Module,
    /// Declares explicit authority granted to code.
    Capability,
    /// Declares observable behavior performed by code.
    Effect,
    /// Declares policy constraints.
    Policy,
    /// Declares a function.
    Fn,
    /// Declares a local binding.
    Let,
    /// Imports a named item.
    Use,
    /// Allows a declared action or capability.
    Allow,
    /// Denies a declared action or capability.
    Deny,
    /// Marks secret-sensitive data or authority.
    Secret,
    /// Declares network authority.
    Network,
    /// Declares read authority.
    Read,
    /// Declares write authority.
    Write,
    /// Declares execution authority.
    Execute,
    /// Declares trace output.
    Trace,
    /// Declares evidence output.
    Evidence,
    /// Declares a requirement.
    Requires,
    /// Declares a guarantee.
    Ensures,
    /// Imports from a named module or package.
    From,
    /// Declares a package import.
    Import,
}

impl Keyword {
    /// Returns the keyword for an identifier text when the text is reserved.
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Option<Self> {
        match identifier {
            "module" => Some(Self::Module),
            "capability" => Some(Self::Capability),
            "effect" => Some(Self::Effect),
            "policy" => Some(Self::Policy),
            "fn" => Some(Self::Fn),
            "let" => Some(Self::Let),
            "use" => Some(Self::Use),
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            "secret" => Some(Self::Secret),
            "network" => Some(Self::Network),
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "execute" => Some(Self::Execute),
            "trace" => Some(Self::Trace),
            "evidence" => Some(Self::Evidence),
            "requires" => Some(Self::Requires),
            "ensures" => Some(Self::Ensures),
            "from" => Some(Self::From),
            "import" => Some(Self::Import),
            _ => None,
        }
    }

    /// Returns the canonical source spelling for this keyword.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Capability => "capability",
            Self::Effect => "effect",
            Self::Policy => "policy",
            Self::Fn => "fn",
            Self::Let => "let",
            Self::Use => "use",
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Secret => "secret",
            Self::Network => "network",
            Self::Read => "read",
            Self::Write => "write",
            Self::Execute => "execute",
            Self::Trace => "trace",
            Self::Evidence => "evidence",
            Self::Requires => "requires",
            Self::Ensures => "ensures",
            Self::From => "from",
            Self::Import => "import",
        }
    }
}

/// Punctuation and operator symbols recognized by the lexer.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Symbol {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `=`
    Equals,
    /// `->`
    Arrow,
}

impl Symbol {
    /// Returns the canonical source spelling for this symbol.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::Colon => ":",
            Self::Semicolon => ";",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Equals => "=",
            Self::Arrow => "->",
        }
    }
}

/// Kind of token recognized from Decriel source text.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    /// Reserved keyword.
    Keyword(Keyword),
    /// User-defined identifier.
    Identifier(String),
    /// Double-quoted string literal after escape processing.
    StringLiteral(String),
    /// Punctuation or operator symbol.
    Symbol(Symbol),
    /// End-of-file marker.
    EndOfFile,
}

/// Token recognized from Decriel source text.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    kind: TokenKind,
    span: SourceSpan,
}

impl Token {
    /// Creates a token from a kind and source span.
    #[must_use]
    pub const fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    /// Returns the token kind.
    #[must_use]
    pub const fn kind(&self) -> &TokenKind {
        &self.kind
    }

    /// Returns the token source span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    /// Returns true when this token is the end-of-file marker.
    #[must_use]
    pub const fn is_end_of_file(&self) -> bool {
        matches!(self.kind, TokenKind::EndOfFile)
    }
}

#[cfg(test)]
mod tests {
    use super::{Keyword, Symbol};

    #[test]
    fn keyword_lookup_recognizes_reserved_words() {
        assert_eq!(Keyword::from_identifier("module"), Some(Keyword::Module));
        assert_eq!(
            Keyword::from_identifier("capability"),
            Some(Keyword::Capability)
        );
        assert_eq!(Keyword::from_identifier("network"), Some(Keyword::Network));
        assert_eq!(Keyword::from_identifier("custom_name"), None);
    }

    #[test]
    fn keyword_source_spelling_is_stable() {
        assert_eq!(Keyword::Module.as_str(), "module");
        assert_eq!(Keyword::LeastAuthorityCandidate.as_str(), "least-authority");
    }

    #[test]
    fn symbol_source_spelling_is_stable() {
        assert_eq!(Symbol::LeftBrace.as_str(), "{");
        assert_eq!(Symbol::RightBrace.as_str(), "}");
        assert_eq!(Symbol::Arrow.as_str(), "->");
    }
}
