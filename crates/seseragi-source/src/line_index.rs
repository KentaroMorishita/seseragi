use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
    Utf32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodedPosition {
    /// Zero-based line number, as required by LSP.
    pub line: usize,
    /// Zero-based code-unit offset in the negotiated encoding.
    pub character: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineIndexError {
    OffsetOutOfBounds { byte_offset: usize, text_len: usize },
    MidScalar { byte_offset: usize },
}

impl fmt::Display for LineIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OffsetOutOfBounds {
                byte_offset,
                text_len,
            } => write!(
                formatter,
                "byte offset {byte_offset} is outside source length {text_len}"
            ),
            Self::MidScalar { byte_offset } => write!(
                formatter,
                "byte offset {byte_offset} is not a Unicode scalar boundary"
            ),
        }
    }
}

impl std::error::Error for LineIndexError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    continuation_offsets: Vec<usize>,
    wide_scalar_offsets: Vec<usize>,
    text_len: usize,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        let mut continuation_offsets = Vec::new();
        let wide_scalar_offsets = text
            .char_indices()
            .filter_map(|(offset, scalar)| (scalar.len_utf8() == 4).then_some(offset))
            .collect();

        for (byte_offset, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(byte_offset + 1);
            } else if byte & 0b1100_0000 == 0b1000_0000 {
                continuation_offsets.push(byte_offset);
            }
        }

        Self {
            line_starts,
            continuation_offsets,
            wide_scalar_offsets,
            text_len: text.len(),
        }
    }

    pub fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Converts a valid UTF-8 byte boundary into a 1-based line and Unicode
    /// scalar column.
    ///
    /// # Panics
    ///
    /// Panics when `byte_offset` is outside the source or lies in the middle
    /// of a Unicode scalar. Use [`Self::try_locate`] for offsets received from
    /// an external tool or another untrusted boundary.
    pub fn locate(&self, byte_offset: usize) -> LineColumn {
        self.try_locate(byte_offset)
            .unwrap_or_else(|error| panic!("cannot locate source position: {error}"))
    }

    /// Converts a UTF-8 byte offset without rounding mid-scalar positions.
    pub fn try_locate(&self, byte_offset: usize) -> Result<LineColumn, LineIndexError> {
        let line_index = self.checked_line_index(byte_offset)?;
        let line_start = self.line_starts[line_index];
        let continuation_count_at_offset = self
            .continuation_offsets
            .partition_point(|offset| *offset < byte_offset);
        let continuation_count_at_line = self
            .continuation_offsets
            .partition_point(|offset| *offset < line_start);
        let extra_bytes = continuation_count_at_offset - continuation_count_at_line;

        Ok(LineColumn {
            line: line_index + 1,
            column: byte_offset - line_start - extra_bytes + 1,
        })
    }

    /// Converts an internal UTF-8 byte offset to a zero-based protocol
    /// position without rounding invalid scalar boundaries.
    pub fn try_locate_encoded(
        &self,
        byte_offset: usize,
        encoding: PositionEncoding,
    ) -> Result<EncodedPosition, LineIndexError> {
        let line_index = self.checked_line_index(byte_offset)?;
        let line_start = self.line_starts[line_index];
        let character = match encoding {
            PositionEncoding::Utf8 => byte_offset - line_start,
            PositionEncoding::Utf32 => {
                let continuation_count_at_offset = self
                    .continuation_offsets
                    .partition_point(|offset| *offset < byte_offset);
                let continuation_count_at_line = self
                    .continuation_offsets
                    .partition_point(|offset| *offset < line_start);
                byte_offset
                    - line_start
                    - (continuation_count_at_offset - continuation_count_at_line)
            }
            PositionEncoding::Utf16 => {
                let scalar_count = self.try_locate(byte_offset)?.column.saturating_sub(1);
                let wide_count_at_offset = self
                    .wide_scalar_offsets
                    .partition_point(|offset| *offset < byte_offset);
                let wide_count_at_line = self
                    .wide_scalar_offsets
                    .partition_point(|offset| *offset < line_start);
                scalar_count + (wide_count_at_offset - wide_count_at_line)
            }
        };

        Ok(EncodedPosition {
            line: line_index,
            character,
        })
    }

    fn checked_line_index(&self, byte_offset: usize) -> Result<usize, LineIndexError> {
        if byte_offset > self.text_len {
            return Err(LineIndexError::OffsetOutOfBounds {
                byte_offset,
                text_len: self.text_len,
            });
        }

        if self
            .continuation_offsets
            .binary_search(&byte_offset)
            .is_ok()
        {
            return Err(LineIndexError::MidScalar { byte_offset });
        }

        Ok(match self.line_starts.binary_search(&byte_offset) {
            Ok(index) => index,
            Err(0) => 0,
            Err(index) => index - 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_lf_offsets_to_one_based_positions() {
        let index = LineIndex::new("one\ntwo\nthree");

        assert_eq!(index.line_starts(), &[0, 4, 8]);
        assert_eq!(index.line_count(), 3);
        assert_eq!(index.locate(0), LineColumn { line: 1, column: 1 });
        assert_eq!(index.locate(4), LineColumn { line: 2, column: 1 });
        assert_eq!(index.locate(11), LineColumn { line: 3, column: 4 });
    }

    #[test]
    fn maps_crlf_offsets_without_normalizing_scalars() {
        let index = LineIndex::new("one\r\ntwo\r\n");

        assert_eq!(index.line_starts(), &[0, 5, 10]);
        assert_eq!(index.line_count(), 3);
        assert_eq!(index.locate(5), LineColumn { line: 2, column: 1 });
        assert_eq!(index.locate(9), LineColumn { line: 2, column: 5 });
        assert_eq!(index.locate(10), LineColumn { line: 3, column: 1 });
    }

    #[test]
    fn counts_unicode_scalars_instead_of_utf8_bytes() {
        let index = LineIndex::new("aé🙂\nβ");

        assert_eq!(index.line_starts(), &[0, 8]);
        assert_eq!(index.locate(1), LineColumn { line: 1, column: 2 });
        assert_eq!(index.locate(3), LineColumn { line: 1, column: 3 });
        assert_eq!(index.locate(7), LineColumn { line: 1, column: 4 });
        assert_eq!(index.locate(8), LineColumn { line: 2, column: 1 });
        assert_eq!(index.locate(10), LineColumn { line: 2, column: 2 });
    }

    #[test]
    fn checked_location_rejects_mid_scalar_offsets_without_rounding() {
        let index = LineIndex::new("aé🙂");

        assert_eq!(
            index.try_locate(2),
            Err(LineIndexError::MidScalar { byte_offset: 2 })
        );
        assert_eq!(
            index.try_locate(4),
            Err(LineIndexError::MidScalar { byte_offset: 4 })
        );
    }

    #[test]
    fn checked_location_accepts_the_end_of_source() {
        let index = LineIndex::new("é");

        assert_eq!(index.try_locate(2), Ok(LineColumn { line: 1, column: 2 }));
    }

    #[test]
    fn checked_location_accepts_the_empty_source_boundary() {
        let index = LineIndex::new("");

        assert_eq!(index.line_starts(), &[0]);
        assert_eq!(index.try_locate(0), Ok(LineColumn { line: 1, column: 1 }));
    }

    #[test]
    fn checked_location_rejects_offsets_after_the_end_of_source() {
        let index = LineIndex::new("é");

        assert_eq!(
            index.try_locate(3),
            Err(LineIndexError::OffsetOutOfBounds {
                byte_offset: 3,
                text_len: 2,
            })
        );
    }

    #[test]
    fn maps_protocol_positions_in_negotiated_code_units() {
        let index = LineIndex::new("aé🙂\nβ");

        assert_eq!(
            index.try_locate_encoded(7, PositionEncoding::Utf8),
            Ok(EncodedPosition {
                line: 0,
                character: 7
            })
        );
        assert_eq!(
            index.try_locate_encoded(7, PositionEncoding::Utf16),
            Ok(EncodedPosition {
                line: 0,
                character: 4
            })
        );
        assert_eq!(
            index.try_locate_encoded(7, PositionEncoding::Utf32),
            Ok(EncodedPosition {
                line: 0,
                character: 3
            })
        );
        assert_eq!(
            index.try_locate_encoded(8, PositionEncoding::Utf16),
            Ok(EncodedPosition {
                line: 1,
                character: 0
            })
        );
    }

    #[test]
    fn protocol_positions_reject_mid_scalar_offsets() {
        let index = LineIndex::new("é");

        assert_eq!(
            index.try_locate_encoded(1, PositionEncoding::Utf16),
            Err(LineIndexError::MidScalar { byte_offset: 1 })
        );
    }

    #[test]
    #[should_panic(expected = "not a Unicode scalar boundary")]
    fn compatibility_location_panics_instead_of_rounding_mid_scalar_offsets() {
        LineIndex::new("é").locate(1);
    }

    #[test]
    #[should_panic(expected = "outside source length")]
    fn compatibility_location_panics_for_offsets_after_the_source() {
        LineIndex::new("a").locate(2);
    }
}
