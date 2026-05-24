//! Diagnostic spans, severities, and reports for the Decriel toolchain.
//!
//! This crate keeps error reporting structured from the beginning so parser,
//! policy, effect, capability, and runtime diagnostics can share the same
//! source-location model.

/// One-based source location.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceLocation {
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub column: usize,
}

impl SourceLocation {
    /// Creates a source location.
    ///
    /// Line and column values are one-based. Values of zero are accepted at this
    /// layer so callers can report raw input state before normalization.
    #[must_use]
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Inclusive-start, exclusive-end source span.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceSpan {
    /// Start location.
    pub start: SourceLocation,
    /// End location.
    pub end: SourceLocation,
}

impl SourceSpan {
    /// Creates a source span.
    #[must_use]
    pub const fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }
}

/// Severity level for a Decriel diagnostic.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    /// Informational note.
    Note,
    /// Warning that does not stop checking.
    Warning,
    /// Error that stops successful checking.
    Error,
}

impl DiagnosticSeverity {
    /// Returns the stable lowercase text label for this severity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Structured diagnostic message.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    message: String,
    span: Option<SourceSpan>,
}

impl Diagnostic {
    /// Creates a diagnostic without a source span.
    #[must_use]
    pub fn new(severity: DiagnosticSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a diagnostic with a source span.
    #[must_use]
    pub fn with_span(
        severity: DiagnosticSeverity,
        message: impl Into<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Returns this diagnostic's severity.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns this diagnostic's human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns this diagnostic's source span, if present.
    #[must_use]
    pub const fn span(&self) -> Option<SourceSpan> {
        self.span
    }

    /// Returns true when this diagnostic is an error.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.severity, DiagnosticSeverity::Error)
    }

    /// Renders the diagnostic as a stable one-line report.
    #[must_use]
    pub fn render_line(&self) -> String {
        match self.span {
            Some(span) => format!(
                "{}:{}:{}: {}: {}",
                span.start.line,
                span.start.column,
                span.end.column,
                self.severity.as_str(),
                self.message
            ),
            None => format!("{}: {}", self.severity.as_str(), self.message),
        }
    }
}

/// Collection of diagnostics emitted while checking a Decriel file.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct DiagnosticReport {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    /// Creates an empty report.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic to the report.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Returns all diagnostics in insertion order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns true when at least one error diagnostic is present.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }

    /// Returns the number of diagnostics in the report.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns true when the report has no diagnostics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Renders all diagnostics as stable newline-separated report lines.
    #[must_use]
    pub fn render(&self) -> String {
        self.diagnostics
            .iter()
            .map(Diagnostic::render_line)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticReport, DiagnosticSeverity, SourceLocation, SourceSpan};

    #[test]
    fn severity_labels_are_stable() {
        assert_eq!(DiagnosticSeverity::Note.as_str(), "note");
        assert_eq!(DiagnosticSeverity::Warning.as_str(), "warning");
        assert_eq!(DiagnosticSeverity::Error.as_str(), "error");
    }

    #[test]
    fn report_tracks_error_presence() {
        let mut report = DiagnosticReport::new();

        assert!(report.is_empty());
        assert!(!report.has_errors());

        report.push(Diagnostic::new(
            DiagnosticSeverity::Warning,
            "capability declaration will be checked in a later wave",
        ));

        assert_eq!(report.len(), 1);
        assert!(!report.has_errors());

        report.push(Diagnostic::new(
            DiagnosticSeverity::Error,
            "invalid Decriel source",
        ));

        assert_eq!(report.len(), 2);
        assert!(report.has_errors());
    }

    #[test]
    fn diagnostic_preserves_span() {
        let span = SourceSpan::new(SourceLocation::new(2, 4), SourceLocation::new(2, 12));
        let diagnostic =
            Diagnostic::with_span(DiagnosticSeverity::Error, "expected declaration", span);

        assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
        assert_eq!(diagnostic.message(), "expected declaration");
        assert_eq!(diagnostic.span(), Some(span));
        assert!(diagnostic.is_error());
    }

    #[test]
    fn diagnostic_without_span_renders_stable_line() {
        let diagnostic = Diagnostic::new(DiagnosticSeverity::Warning, "review capability scope");

        assert_eq!(diagnostic.render_line(), "warning: review capability scope");
    }

    #[test]
    fn diagnostic_with_span_renders_stable_line() {
        let span = SourceSpan::new(SourceLocation::new(3, 5), SourceLocation::new(3, 16));
        let diagnostic =
            Diagnostic::with_span(DiagnosticSeverity::Error, "expected module name", span);

        assert_eq!(diagnostic.render_line(), "3:5:16: error: expected module name");
    }

    #[test]
    fn report_renders_diagnostics_in_insertion_order() {
        let first = Diagnostic::new(DiagnosticSeverity::Error, "first");
        let second = Diagnostic::new(DiagnosticSeverity::Warning, "second");
        let mut report = DiagnosticReport::new();

        report.push(first);
        report.push(second);

        assert_eq!(report.render(), "error: first\nwarning: second");
    }
}
