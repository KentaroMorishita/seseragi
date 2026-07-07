use serde::{Deserialize, Serialize};
use seseragi_syntax::ByteSpan;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    pub source: String,
    pub start: usize,
    pub end: usize,
}

pub(crate) fn source_span(source: &str, span: ByteSpan) -> SourceSpan {
    SourceSpan {
        source: source.to_owned(),
        start: span.start,
        end: span.end,
    }
}
