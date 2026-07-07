use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self { line_starts }
    }

    pub fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn locate(&self, byte_offset: usize) -> LineColumn {
        let line_index = match self.line_starts.binary_search(&byte_offset) {
            Ok(index) => index,
            Err(0) => 0,
            Err(index) => index - 1,
        };

        LineColumn {
            line: line_index + 1,
            column: byte_offset - self.line_starts[line_index] + 1,
        }
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
    fn line_index_maps_lf_offsets_to_one_based_positions() {
        let index = LineIndex::new("one\ntwo\nthree");

        assert_eq!(index.line_starts(), &[0, 4, 8]);
        assert_eq!(index.line_count(), 3);
        assert_eq!(index.locate(0), LineColumn { line: 1, column: 1 });
        assert_eq!(index.locate(4), LineColumn { line: 2, column: 1 });
        assert_eq!(index.locate(11), LineColumn { line: 3, column: 4 });
    }

    #[test]
    fn line_index_maps_crlf_offsets_without_normalizing_bytes() {
        let index = LineIndex::new("one\r\ntwo\r\n");

        assert_eq!(index.line_starts(), &[0, 5, 10]);
        assert_eq!(index.line_count(), 3);
        assert_eq!(index.locate(5), LineColumn { line: 2, column: 1 });
        assert_eq!(index.locate(9), LineColumn { line: 2, column: 5 });
        assert_eq!(index.locate(10), LineColumn { line: 3, column: 1 });
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
