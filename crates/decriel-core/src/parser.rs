//! Parser foundation for Decriel source text.
//!
//! This parser turns tokenized Decriel source into an AST. At this stage it
//! accepts a module shell and rejects malformed source with structured
//! diagnostics. Declaration parsing is added by later Wave 1 commits.

use decriel_diagnostics::{Diagnostic, DiagnosticReport, DiagnosticSeverity, SourceSpan};

use crate::ast::{DecrielModule, Identifier};
use crate::lexer::lex;
use crate::source::SourceDocument;
use crate::token::{Keyword, Symbol, Token, TokenKind};

/// Result of parsing a Decriel source document.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParseResult {
    module: Option<DecrielModule>,
    diagnostics: DiagnosticReport,
}

impl ParseResult {
    /// Creates a parse result.
    #[must_use]
    pub const fn new(module: Option<DecrielModule>, diagnostics: DiagnosticReport) -> Self {
        Self {
            module,
            diagnostics,
        }
    }

    /// Returns the parsed module when parsing succeeded.
    #[must_use]
    pub const fn module(&self) -> Option<&DecrielModule> {
        self.module.as_ref()
    }

    /// Returns diagnostics emitted while parsing.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticReport {
        &self.diagnostics
    }

    /// Returns true when parsing emitted at least one error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Consumes this result and returns the module plus diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (Option<DecrielModule>, DiagnosticReport) {
        (self.module, self.diagnostics)
    }
}

/// Parses a Decriel source document.
#[must_use]
pub fn parse(document: &SourceDocument) -> ParseResult {
    let lex_result = lex(document);
    let (tokens, diagnostics) = lex_result.into_parts();

    if diagnostics.has_errors() {
        return ParseResult::new(None, diagnostics);
    }

    Parser::new(tokens, diagnostics).parse_document()
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    diagnostics: DiagnosticReport,
}

impl Parser {
    fn new(tokens: Vec<Token>, diagnostics: DiagnosticReport) -> Self {
        Self {
            tokens,
            position: 0,
            diagnostics,
        }
    }

    fn parse_document(mut self) -> ParseResult {
        let module = self.parse_module();

        if self.diagnostics.has_errors() {
            ParseResult::new(None, self.diagnostics)
        } else {
            ParseResult::new(module, self.diagnostics)
        }
    }

    fn parse_module(&mut self) -> Option<DecrielModule> {
        let module_token = self.consume_keyword(
            Keyword::Module,
            "expected 'module' declaration at start of Decriel source",
        )?;
        let module_name = self.consume_identifier("expected module name after 'module'")?;
        self.consume_symbol(Symbol::LeftBrace, "expected '{' after module name")?;

        if !self.check_symbol(Symbol::RightBrace) {
            self.report_current("expected '}' to close module body");
            return None;
        }

        let close_token = self.consume_symbol(Symbol::RightBrace, "expected '}' to close module body")?;
        self.consume_end_of_file()?;

        let span = SourceSpan::new(module_token.span().start, close_token.span().end);

        Some(DecrielModule::new(module_name, Vec::new(), span))
    }

    fn consume_keyword(&mut self, expected: Keyword, message: &str) -> Option<Token> {
        let token = self.current().clone();

        match token.kind() {
            TokenKind::Keyword(found) if *found == expected => {
                self.advance();
                Some(token)
            }
            _ => {
                self.report_token(&token, message);
                None
            }
        }
    }

    fn consume_symbol(&mut self, expected: Symbol, message: &str) -> Option<Token> {
        let token = self.current().clone();

        match token.kind() {
            TokenKind::Symbol(found) if *found == expected => {
                self.advance();
                Some(token)
            }
            _ => {
                self.report_token(&token, message);
                None
            }
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Option<Identifier> {
        let token = self.current().clone();

        match token.kind() {
            TokenKind::Identifier(name) => {
                self.advance();
                Some(Identifier::new(name.clone(), token.span()))
            }
            _ => {
                self.report_token(&token, message);
                None
            }
        }
    }

    fn consume_end_of_file(&mut self) -> Option<Token> {
        let token = self.current().clone();

        if token.is_end_of_file() {
            self.advance();
            Some(token)
        } else {
            self.report_token(&token, "expected end of file after module declaration");
            None
        }
    }

    fn check_symbol(&self, expected: Symbol) -> bool {
        matches!(self.current().kind(), TokenKind::Symbol(found) if *found == expected)
    }

    fn current(&self) -> &Token {
        let last_index = self.tokens.len().saturating_sub(1);
        let index = self.position.min(last_index);

        &self.tokens[index]
    }

    fn advance(&mut self) {
        if !self.current().is_end_of_file() {
            self.position += 1;
        }
    }

    fn report_current(&mut self, message: &str) {
        let token = self.current().clone();
        self.report_token(&token, message);
    }

    fn report_token(&mut self, token: &Token, message: &str) {
        self.diagnostics.push(Diagnostic::with_span(
            DiagnosticSeverity::Error,
            message,
            token.span(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::source::SourceDocument;

    #[test]
    fn parser_accepts_empty_module_shell() {
        let document = SourceDocument::new("empty_module.dcr", "module secure_service {}");

        let result = parse(&document);

        assert!(!result.has_errors());

        let module = result.module();
        assert!(module.is_some());

        if let Some(module) = module {
            assert_eq!(module.name().name(), "secure_service");
            assert!(module.is_empty());
        }
    }

    #[test]
    fn parser_rejects_source_without_module_declaration() {
        let document = SourceDocument::new("not_module.dcr", "policy allow {}");

        let result = parse(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert!(result.module().is_none());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "expected 'module' declaration at start of Decriel source"
        );
    }

    #[test]
    fn parser_rejects_missing_module_name() {
        let document = SourceDocument::new("missing_name.dcr", "module {}");

        let result = parse(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert!(result.module().is_none());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message(), "expected module name after 'module'");
    }

    #[test]
    fn parser_rejects_unclosed_module_body() {
        let document = SourceDocument::new("unclosed.dcr", "module secure_service {");

        let result = parse(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert!(result.module().is_none());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message(), "expected '}' to close module body");
    }

    #[test]
    fn parser_preserves_lexer_errors() {
        let document = SourceDocument::new("lex_error.dcr", "module @bad {}");

        let result = parse(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert!(result.module().is_none());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message(), "invalid character '@'");
    }
}
