use serde::{Deserialize, Serialize};

mod line_index;

pub use line_index::{EncodedPosition, LineColumn, LineIndex, LineIndexError, PositionEncoding};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceSnapshot {
    source_name: String,
    text: String,
}

impl SourceSnapshot {
    pub fn new(source_name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            text: text.into(),
        }
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn line_index(&self) -> LineIndex {
        LineIndex::new(&self.text)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_keeps_source_identity_and_text() {
        let snapshot = SourceSnapshot::new("main.ssrg", "let answer = 42");

        assert_eq!(snapshot.source_name(), "main.ssrg");
        assert_eq!(snapshot.text(), "let answer = 42");
        assert_eq!(snapshot.len(), 15);
        assert!(!snapshot.is_empty());
    }

    #[test]
    fn span_reports_byte_length() {
        let span = Span::new(4, 10);

        assert_eq!(span.len(), 6);
        assert!(!span.is_empty());
        assert!(Span::new(4, 4).is_empty());
    }

    #[test]
    fn source_snapshot_creates_line_index() {
        let snapshot = SourceSnapshot::new("main.ssrg", "a\nb");

        assert_eq!(
            snapshot.line_index().locate(2),
            LineColumn { line: 2, column: 1 }
        );
    }
}
