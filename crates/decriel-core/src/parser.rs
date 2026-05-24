//! Parser foundation for Decriel source text.
//!
//! This parser turns tokenized Decriel source into an AST. At this stage it
//! accepts module shells and declaration shells. Later waves attach capability
//! semantics, effect tracking, policy validation, runtime enforcement, evidence,
//! secret tracking, supply-chain security, cryptographic APIs, privacy profiles,
//! proof obligations, and backend lowering.

use decriel_diagnostics::{Diagnostic, DiagnosticReport, DiagnosticSeverity, SourceSpan};

use crate::ast::{
    CapabilityAction, Declaration, DecrielModule, EffectAction, Identifier, PolicyAction,
};
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

        let mut declarations = Vec::new();

        while !self.check_symbol(Symbol::RightBrace) && !self.current().is_end_of_file() {
            declarations.push(self.parse_declaration()?);
        }

        let close_token =
            self.consume_symbol(Symbol::RightBrace, "expected '}' to close module body")?;
        self.consume_end_of_file()?;

        let span = SourceSpan::new(module_token.span().start, close_token.span().end);

        Some(DecrielModule::new(module_name, declarations, span))
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        match self.current().kind() {
            TokenKind::Keyword(Keyword::Capability) => self.parse_capability_declaration(),
            TokenKind::Keyword(Keyword::Effect) => self.parse_effect_declaration(),
            TokenKind::Keyword(Keyword::Policy) => self.parse_policy_declaration(),
            TokenKind::Keyword(Keyword::Fn) => self.parse_function_declaration(),
            _ => {
                self.report_current(
                    "expected declaration: capability, effect, policy, or function shell",
                );
                None
            }
        }
    }

    fn parse_capability_declaration(&mut self) -> Option<Declaration> {
        let start = self.consume_keyword(Keyword::Capability, "expected 'capability'")?;
        let action = self.consume_capability_action()?;
        let target = self.consume_identifier("expected capability target")?;
        let semicolon =
            self.consume_symbol(Symbol::Semicolon, "expected ';' after capability declaration")?;
        let span = SourceSpan::new(start.span().start, semicolon.span().end);

        Some(Declaration::Capability {
            action,
            target,
            span,
        })
    }

    fn parse_effect_declaration(&mut self) -> Option<Declaration> {
        let start = self.consume_keyword(Keyword::Effect, "expected 'effect'")?;
        let action = self.consume_effect_action()?;
        let target = self.consume_identifier("expected effect target")?;
        let semicolon =
            self.consume_symbol(Symbol::Semicolon, "expected ';' after effect declaration")?;
        let span = SourceSpan::new(start.span().start, semicolon.span().end);

        Some(Declaration::Effect {
            action,
            target,
            span,
        })
    }

    fn parse_policy_declaration(&mut self) -> Option<Declaration> {
        let start = self.consume_keyword(Keyword::Policy, "expected 'policy'")?;
        let action = self.consume_policy_action()?;
        let target = self.consume_identifier("expected policy target")?;
        let semicolon =
            self.consume_symbol(Symbol::Semicolon, "expected ';' after policy declaration")?;
        let span = SourceSpan::new(start.span().start, semicolon.span().end);

        Some(Declaration::Policy {
            action,
            target,
            span,
        })
    }

    fn parse_function_declaration(&mut self) -> Option<Declaration> {
        let start = self.consume_keyword(Keyword::Fn, "expected 'fn'")?;
        let name = self.consume_identifier("expected function name after 'fn'")?;
        let semicolon =
            self.consume_symbol(Symbol::Semicolon, "expected ';' after function declaration")?;
        let span = SourceSpan::new(start.span().start, semicolon.span().end);

        Some(Declaration::Function { name, span })
    }

    fn consume_capability_action(&mut self) -> Option<CapabilityAction> {
        let token = self.current().clone();
        let action = match token.kind() {
            TokenKind::Keyword(Keyword::Read) => CapabilityAction::Read,
            TokenKind::Keyword(Keyword::Write) => CapabilityAction::Write,
            TokenKind::Keyword(Keyword::Network) => CapabilityAction::Network,
            TokenKind::Keyword(Keyword::Execute) => CapabilityAction::Execute,
            TokenKind::Keyword(Keyword::Secret) => CapabilityAction::Secret,
            _ => {
                self.report_token(
                    &token,
                    "expected capability action: read, write, network, execute, or secret",
                );
                return None;
            }
        };

        self.advance();
        Some(action)
    }

    fn consume_effect_action(&mut self) -> Option<EffectAction> {
        let token = self.current().clone();
        let action = match token.kind() {
            TokenKind::Keyword(Keyword::Read) => EffectAction::Read,
            TokenKind::Keyword(Keyword::Write) => EffectAction::Write,
            TokenKind::Keyword(Keyword::Network) => EffectAction::Network,
            TokenKind::Keyword(Keyword::Trace) => EffectAction::Trace,
            TokenKind::Keyword(Keyword::Evidence) => EffectAction::Evidence,
            _ => {
                self.report_token(
                    &token,
                    "expected effect action: read, write, network, trace, or evidence",
                );
                return None;
            }
        };

        self.advance();
        Some(action)
    }

    fn consume_policy_action(&mut self) -> Option<PolicyAction> {
        let token = self.current().clone();
        let action = match token.kind() {
            TokenKind::Keyword(Keyword::Allow) => PolicyAction::Allow,
            TokenKind::Keyword(Keyword::Deny) => PolicyAction::Deny,
            TokenKind::Keyword(Keyword::Requires) => PolicyAction::Requires,
            TokenKind::Keyword(Keyword::Ensures) => PolicyAction::Ensures,
            _ => {
                self.report_token(
                    &token,
                    "expected policy action: allow, deny, requires, or ensures",
                );
                return None;
            }
        };

        self.advance();
        Some(action)
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
                Some(Identifier::new(name.to_owned(), token.span()))
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
    use crate::ast::{CapabilityAction, Declaration, EffectAction, PolicyAction};
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
    fn parser_accepts_declaration_shells() {
        let document = SourceDocument::new(
            "declarations.dcr",
            "module secure_service {
                capability network outbound_api;
                effect trace audit_event;
                policy deny shell_access;
                fn review_gate;
            }",
        );

        let result = parse(&document);

        assert!(!result.has_errors());

        let module = result.module();
        assert!(module.is_some());

        if let Some(module) = module {
            let declarations = module.declarations();

            assert_eq!(module.name().name(), "secure_service");
            assert_eq!(module.len(), 4);
            assert_eq!(declarations[0].kind_name(), "capability");
            assert_eq!(declarations[1].kind_name(), "effect");
            assert_eq!(declarations[2].kind_name(), "policy");
            assert_eq!(declarations[3].kind_name(), "function");

            if let Declaration::Capability { action, target, .. } = &declarations[0] {
                assert_eq!(*action, CapabilityAction::Network);
                assert_eq!(target.name(), "outbound_api");
            } else {
                assert_eq!(declarations[0].kind_name(), "capability");
            }

            if let Declaration::Effect { action, target, .. } = &declarations[1] {
                assert_eq!(*action, EffectAction::Trace);
                assert_eq!(target.name(), "audit_event");
            } else {
                assert_eq!(declarations[1].kind_name(), "effect");
            }

            if let Declaration::Policy { action, target, .. } = &declarations[2] {
                assert_eq!(*action, PolicyAction::Deny);
                assert_eq!(target.name(), "shell_access");
            } else {
                assert_eq!(declarations[2].kind_name(), "policy");
            }

            if let Declaration::Function { name, .. } = &declarations[3] {
                assert_eq!(name.name(), "review_gate");
            } else {
                assert_eq!(declarations[3].kind_name(), "function");
            }
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
    fn parser_rejects_invalid_declaration_shell() {
        let document = SourceDocument::new("invalid_declaration.dcr", "module main { let bad; }");

        let result = parse(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert!(result.module().is_none());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "expected declaration: capability, effect, policy, or function shell"
        );
    }

    #[test]
    fn parser_rejects_invalid_capability_action() {
        let document = SourceDocument::new(
            "invalid_capability.dcr",
            "module main { capability trace audit; }",
        );

        let result = parse(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert!(result.module().is_none());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message(),
            "expected capability action: read, write, network, execute, or secret"
        );
    }

    #[test]
    fn parser_renders_syntax_diagnostics() {
        let document = SourceDocument::new("render_error.dcr", "module {}");

        let result = parse(&document);
        let rendered = result.diagnostics().render();

        assert!(result.has_errors());
        assert!(rendered.contains("error: expected module name after 'module'"));
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
