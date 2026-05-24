//! Lexer for Decriel source text.
//!
//! The lexer turns UTF-8 Decriel source into tokens with source spans and
//! diagnostics. It is intentionally strict: unknown characters and malformed
//! strings are reported as errors instead of being ignored.

use decriel_diagnostics::{
    Diagnostic, DiagnosticReport, DiagnosticSeverity, SourceLocation, SourceSpan,
};

use crate::source::SourceDocument;
use crate::token::{Keyword, Symbol, Token, TokenKind};

/// Result of lexing a Decriel source document.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LexResult {
    tokens: Vec<Token>,
    diagnostics: DiagnosticReport,
}

impl LexResult {
    /// Creates a lex result.
    #[must_use]
    pub fn new(tokens: Vec<Token>, diagnostics: DiagnosticReport) -> Self {
        Self {
            tokens,
            diagnostics,
        }
    }

    /// Returns tokens in source order.
    #[must_use]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Returns diagnostics emitted while lexing.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticReport {
        &self.diagnostics
    }

    /// Returns true when lexing emitted at least one error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Consumes this result and returns tokens plus diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (Vec<Token>, DiagnosticReport) {
        (self.tokens, self.diagnostics)
    }
}

/// Lexes a Decriel source document.
#[must_use]
pub fn lex(document: &SourceDocument) -> LexResult {
    Lexer::new(document).lex_all()
}

struct Lexer<'source> {
    document: &'source SourceDocument,
    position: usize,
    tokens: Vec<Token>,
    diagnostics: DiagnosticReport,
}

impl<'source> Lexer<'source> {
    fn new(document: &'source SourceDocument) -> Self {
        Self {
            document,
            position: 0,
            tokens: Vec::new(),
            diagnostics: DiagnosticReport::new(),
        }
    }

    fn lex_all(mut self) -> LexResult {
        while self.position < self.text().len() {
            self.skip_whitespace_and_comments();

            if self.position >= self.text().len() {
                break;
            }

            self.lex_next_token();
        }

        let end_span = self.span_for(self.position, self.position);
        self.tokens.push(Token::new(TokenKind::EndOfFile, end_span));

        LexResult::new(self.tokens, self.diagnostics)
    }

    fn text(&self) -> &str {
        self.document.text()
    }

    fn lex_next_token(&mut self) {
        let Some(character) = self.peek_char() else {
            return;
        };

        let start = self.position;

        if is_identifier_start(character) {
            self.lex_identifier_or_keyword(start);
            return;
        }

        if character == '"' {
            self.lex_string_literal(start);
            return;
        }

        match character {
            '{' => self.push_single_char_symbol(start, Symbol::LeftBrace),
            '}' => self.push_single_char_symbol(start, Symbol::RightBrace),
            '(' => self.push_single_char_symbol(start, Symbol::LeftParen),
            ')' => self.push_single_char_symbol(start, Symbol::RightParen),
            '[' => self.push_single_char_symbol(start, Symbol::LeftBracket),
            ']' => self.push_single_char_symbol(start, Symbol::RightBracket),
            ':' => self.push_single_char_symbol(start, Symbol::Colon),
            ';' => self.push_single_char_symbol(start, Symbol::Semicolon),
            ',' => self.push_single_char_symbol(start, Symbol::Comma),
            '.' => self.push_single_char_symbol(start, Symbol::Dot),
            '=' => self.push_single_char_symbol(start, Symbol::Equals),
            '-' => self.lex_dash_started_symbol(start),
            _ => self.report_invalid_character(start, character),
        }
    }

    fn lex_identifier_or_keyword(&mut self, start: usize) {
        self.advance_char();

        while let Some(character) = self.peek_char() {
            if !is_identifier_continue(character) {
                break;
            }

            self.advance_char();
        }

        let text = &self.text()[start..self.position];
        let kind = match Keyword::from_identifier(text) {
            Some(keyword) => TokenKind::Keyword(keyword),
            None => TokenKind::Identifier(text.to_owned()),
        };

        self.push_token(kind, start, self.position);
    }

    fn lex_string_literal(&mut self, start: usize) {
        self.advance_char();

        let mut value = String::new();

        loop {
            let Some(character) = self.peek_char() else {
                let span = self.span_for(start, self.position);
                self.diagnostics.push(Diagnostic::with_span(
                    DiagnosticSeverity::Error,
                    "unterminated string literal",
                    span,
                ));
                return;
            };

            if character == '"' {
                self.advance_char();
                self.push_token(TokenKind::StringLiteral(value), start, self.position);
                return;
            }

            if character == '\n' || character == '\r' {
                let span = self.span_for(start, self.position);
                self.diagnostics.push(Diagnostic::with_span(
                    DiagnosticSeverity::Error,
                    "unterminated string literal before line break",
                    span,
                ));
                return;
            }

            if character == '\\' {
                self.lex_escape_sequence(start, &mut value);
                continue;
            }

            self.advance_char();
            value.push(character);
        }
    }

    fn lex_escape_sequence(&mut self, literal_start: usize, value: &mut String) {
        self.advance_char();

        let Some(escaped) = self.peek_char() else {
            let span = self.span_for(literal_start, self.position);
            self.diagnostics.push(Diagnostic::with_span(
                DiagnosticSeverity::Error,
                "unterminated string escape",
                span,
            ));
            return;
        };

        self.advance_char();

        match escaped {
            '"' => value.push('"'),
            '\\' => value.push('\\'),
            'n' => value.push('\n'),
            'r' => value.push('\r'),
            't' => value.push('\t'),
            _ => {
                let span = self.span_for(literal_start, self.position);
                self.diagnostics.push(Diagnostic::with_span(
                    DiagnosticSeverity::Error,
                    format!("unsupported string escape '\\{escaped}'"),
                    span,
                ));
            }
        }
    }

    fn lex_dash_started_symbol(&mut self, start: usize) {
        self.advance_char();

        if self.peek_char() == Some('>') {
            self.advance_char();
            self.push_token(TokenKind::Symbol(Symbol::Arrow), start, self.position);
            return;
        }

        let span = self.span_for(start, self.position);
        self.diagnostics.push(Diagnostic::with_span(
            DiagnosticSeverity::Error,
            "unexpected '-' character; use '->' for arrows",
            span,
        ));
    }

    fn push_single_char_symbol(&mut self, start: usize, symbol: Symbol) {
        self.advance_char();
        self.push_token(TokenKind::Symbol(symbol), start, self.position);
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        let span = self.span_for(start, end);
        self.tokens.push(Token::new(kind, span));
    }

    fn report_invalid_character(&mut self, start: usize, character: char) {
        self.advance_char();

        let span = self.span_for(start, self.position);
        self.diagnostics.push(Diagnostic::with_span(
            DiagnosticSeverity::Error,
            format!("invalid character '{character}'"),
            span,
        ));
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let before = self.position;

            self.skip_whitespace();

            if self.remaining_text().starts_with("//") {
                self.skip_line_comment();
            }

            if self.position == before {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek_char() {
            if !character.is_whitespace() {
                break;
            }

            self.advance_char();
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(character) = self.peek_char() {
            if character == '\n' {
                break;
            }

            self.advance_char();
        }
    }

    fn remaining_text(&self) -> &str {
        &self.text()[self.position..]
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining_text().chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let character = self.peek_char()?;
        self.position += character.len_utf8();
        Some(character)
    }

    fn span_for(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(self.location_for(start), self.location_for(end))
    }

    fn location_for(&self, byte_offset: usize) -> SourceLocation {
        match self.document.line_col_for_byte(byte_offset) {
            Some((line, column)) => SourceLocation::new(line, column),
            None => SourceLocation::new(1, 1),
        }
    }
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_ascii_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use decriel_diagnostics::DiagnosticSeverity;

    use super::lex;
    use crate::source::SourceDocument;
    use crate::token::{Keyword, Symbol, TokenKind};

    #[test]
    fn lexer_recognizes_keywords_identifiers_and_symbols() {
        let document = SourceDocument::new(
            "sample.dcr",
            "module secure_service { capability network -> allow; }",
        );

        let result = lex(&document);
        let tokens = result.tokens();

        assert!(!result.has_errors());
        assert_eq!(tokens[0].kind(), &TokenKind::Keyword(Keyword::Module));
        assert_eq!(
            tokens[1].kind(),
            &TokenKind::Identifier("secure_service".to_owned())
        );
        assert_eq!(tokens[2].kind(), &TokenKind::Symbol(Symbol::LeftBrace));
        assert_eq!(tokens[3].kind(), &TokenKind::Keyword(Keyword::Capability));
        assert_eq!(tokens[4].kind(), &TokenKind::Keyword(Keyword::Network));
        assert_eq!(tokens[5].kind(), &TokenKind::Symbol(Symbol::Arrow));
        assert_eq!(tokens[6].kind(), &TokenKind::Keyword(Keyword::Allow));
        assert_eq!(tokens[7].kind(), &TokenKind::Symbol(Symbol::Semicolon));
        assert_eq!(tokens[8].kind(), &TokenKind::Symbol(Symbol::RightBrace));
        assert!(tokens[9].is_end_of_file());
    }

    #[test]
    fn lexer_recognizes_string_literals_and_escapes() {
        let document = SourceDocument::new("strings.dcr", "\"alpha\\n\\\"beta\\\"\"");

        let result = lex(&document);
        let tokens = result.tokens();

        assert!(!result.has_errors());
        assert_eq!(
            tokens[0].kind(),
            &TokenKind::StringLiteral("alpha\n\"beta\"".to_owned())
        );
        assert!(tokens[1].is_end_of_file());
    }

    #[test]
    fn lexer_skips_line_comments() {
        let document = SourceDocument::new("comments.dcr", "module main // comment\npolicy allow");

        let result = lex(&document);
        let tokens = result.tokens();

        assert!(!result.has_errors());
        assert_eq!(tokens[0].kind(), &TokenKind::Keyword(Keyword::Module));
        assert_eq!(tokens[1].kind(), &TokenKind::Identifier("main".to_owned()));
        assert_eq!(tokens[2].kind(), &TokenKind::Keyword(Keyword::Policy));
        assert_eq!(tokens[3].kind(), &TokenKind::Keyword(Keyword::Allow));
        assert!(tokens[4].is_end_of_file());
    }

    #[test]
    fn lexer_reports_invalid_characters() {
        let document = SourceDocument::new("invalid.dcr", "module @bad");

        let result = lex(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity(), DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].message(), "invalid character '@'");
    }

    #[test]
    fn lexer_reports_unterminated_string_literals() {
        let document = SourceDocument::new("unterminated.dcr", "\"secret");

        let result = lex(&document);
        let diagnostics = result.diagnostics().diagnostics();

        assert!(result.has_errors());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity(), DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].message(), "unterminated string literal");
    }
}
