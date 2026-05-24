//! Source document model for Decriel files.
//!
//! The source document layer gives the toolchain a shared, tested way to hold
//! source text, count lines, and translate byte offsets into human-readable
//! locations before the parser arrives in Wave 1.

use std::path::{Path, PathBuf};

/// UTF-8 source text loaded for Decriel analysis.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceDocument {
    path: PathBuf,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceDocument {
    /// Creates a source document from a path and UTF-8 text.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        let text = text.into();
        let line_starts = compute_line_starts(&text);

        Self {
            path: path.into(),
            text,
            line_starts,
        }
    }

    /// Returns the source path associated with this document.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the complete source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the source byte length.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.text.len()
    }

    /// Returns true when the source text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the number of source lines visible to diagnostics.
    ///
    /// An empty file has zero lines. A file ending in a trailing newline does not
    /// create an additional empty diagnostic line.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns true when the source text ends with a line feed.
    #[must_use]
    pub fn has_trailing_newline(&self) -> bool {
        self.text.ends_with('\n')
    }

    /// Converts a byte offset into a one-based `(line, column)` location.
    ///
    /// Returns `None` when the byte offset is outside the source text, when the
    /// offset does not land on a UTF-8 character boundary, or when the document
    /// is empty.
    #[must_use]
    pub fn line_col_for_byte(&self, byte_offset: usize) -> Option<(usize, usize)> {
        if self.text.is_empty()
            || byte_offset > self.text.len()
            || !self.text.is_char_boundary(byte_offset)
        {
            return None;
        }

        let line_index = match self.line_starts.binary_search(&byte_offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let line_start = self.line_starts[line_index];
        let column = self.text[line_start..byte_offset].chars().count() + 1;

        Some((line_index + 1, column))
    }
}

fn compute_line_starts(text: &str) -> Vec<usize> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut starts = vec![0];

    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' && index + 1 < text.len() {
            starts.push(index + 1);
        }
    }

    starts
}

#[cfg(test)]
mod tests {
    use super::SourceDocument;

    #[test]
    fn source_document_tracks_basic_metrics() {
        let document = SourceDocument::new("examples/basic.dcr", "module main\ncapability file\n");

        assert_eq!(document.path().to_string_lossy(), "examples/basic.dcr");
        assert_eq!(document.byte_len(), 28);
        assert_eq!(document.line_count(), 2);
        assert!(document.has_trailing_newline());
        assert!(!document.is_empty());
    }

    #[test]
    fn empty_source_has_no_diagnostic_lines() {
        let document = SourceDocument::new("empty.dcr", "");

        assert_eq!(document.byte_len(), 0);
        assert_eq!(document.line_count(), 0);
        assert!(!document.has_trailing_newline());
        assert!(document.is_empty());
        assert_eq!(document.line_col_for_byte(0), None);
    }

    #[test]
    fn byte_offsets_map_to_one_based_locations() {
        let document = SourceDocument::new("offsets.dcr", "alpha\nbeta\ngamma");

        assert_eq!(document.line_col_for_byte(0), Some((1, 1)));
        assert_eq!(document.line_col_for_byte(5), Some((1, 6)));
        assert_eq!(document.line_col_for_byte(6), Some((2, 1)));
        assert_eq!(document.line_col_for_byte(10), Some((2, 5)));
        assert_eq!(document.line_col_for_byte(11), Some((3, 1)));
        assert_eq!(document.line_col_for_byte(16), Some((3, 6)));
        assert_eq!(document.line_col_for_byte(17), None);
    }

    #[test]
    fn utf8_offsets_must_land_on_character_boundaries() {
        let document = SourceDocument::new("utf8.dcr", "αβ\nsecure");

        assert_eq!(document.line_col_for_byte(0), Some((1, 1)));
        assert_eq!(document.line_col_for_byte(2), Some((1, 2)));
        assert_eq!(document.line_col_for_byte(4), Some((1, 3)));
        assert_eq!(document.line_col_for_byte(1), None);
        assert_eq!(document.line_col_for_byte(3), None);
    }
}
