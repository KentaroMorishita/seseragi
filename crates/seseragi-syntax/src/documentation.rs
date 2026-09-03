use crate::{lex, parse_surface_ast, ByteSpan, SurfaceDecl, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentTestMode {
    Check,
    Run,
    CompileFail { code: String },
}

impl DocumentTestMode {
    pub fn label(&self) -> String {
        match self {
            Self::Check => "check".to_owned(),
            Self::Run => "run".to_owned(),
            Self::CompileFail { code } => format!("compile_fail {code}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentTestBlock {
    pub declaration: String,
    pub ordinal: usize,
    pub mode: DocumentTestMode,
    pub source: String,
    pub expected_stdout: Option<String>,
    pub comment_range: ByteSpan,
    line_origins: Vec<usize>,
}

impl DocumentTestBlock {
    pub fn original_offset(&self, generated_offset: usize) -> usize {
        let mut line = 0;
        let mut column = 0;
        for (index, byte) in self.source.bytes().enumerate() {
            if index == generated_offset {
                break;
            }
            if byte == b'\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        self.line_origins
            .get(line)
            .copied()
            .unwrap_or(self.comment_range.start)
            .saturating_add(column)
    }
}

#[derive(Clone, Debug)]
struct DocLine {
    text: String,
    content_start: usize,
    line_start: usize,
    line_end: usize,
    module: bool,
}

/// Extracts testable fenced blocks from lexical document comments while
/// retaining a byte mapping back to the original source.
pub fn extract_document_tests(
    source_name: impl Into<String>,
    source: &str,
) -> Vec<DocumentTestBlock> {
    let source_name = source_name.into();
    let tokens = lex(source_name.clone(), source);
    let surface = parse_surface_ast(source_name, source);
    let mut lines = Vec::new();
    for token in tokens
        .tokens
        .iter()
        .filter(|token| token.kind == TokenKind::TriviaComment)
    {
        let (prefix, module) = if token.raw.starts_with("///") {
            ("///", false)
        } else if token.raw.starts_with("//!") {
            ("//!", true)
        } else {
            continue;
        };
        let mut content_start = token.start + prefix.len();
        if source.as_bytes().get(content_start) == Some(&b' ') {
            content_start += 1;
        }
        lines.push(DocLine {
            text: source[content_start..token.end].to_owned(),
            content_start,
            line_start: token.start,
            line_end: token.end,
            module,
        });
    }

    let mut groups: Vec<Vec<DocLine>> = Vec::new();
    for line in lines {
        let continues = groups
            .last()
            .and_then(|group| group.last())
            .is_some_and(|previous| {
                previous.module == line.module
                    && source[previous.line_end..line.line_start]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count()
                        == 1
                    && source[previous.line_end..line.line_start]
                        .chars()
                        .all(crate::unicode::is_whitespace)
            });
        if continues {
            groups.last_mut().unwrap().push(line);
        } else {
            groups.push(vec![line]);
        }
    }

    let mut blocks = Vec::new();
    for group in groups {
        let declaration = if group[0].module {
            "<module>".to_owned()
        } else {
            let end = group.last().unwrap().line_end;
            let Some(declaration) = surface.declarations.iter().find(|declaration| {
                declaration.span().start >= end
                    && source[end..declaration.span().start]
                        .chars()
                        .all(crate::unicode::is_whitespace)
                    && source[end..declaration.span().start].matches('\n').count() <= 1
            }) else {
                continue;
            };
            declaration_name(declaration)
        };
        parse_group(&group, declaration, &mut blocks);
    }
    blocks
}

fn parse_group(group: &[DocLine], declaration: String, blocks: &mut Vec<DocumentTestBlock>) {
    let mut index = 0;
    let mut ordinal = 0;
    while index < group.len() {
        let Some(info) = group[index].text.strip_prefix("```") else {
            index += 1;
            continue;
        };
        let info = info.trim();
        let mode = if info == "seseragi" {
            Some(DocumentTestMode::Check)
        } else if info == "seseragi run" {
            Some(DocumentTestMode::Run)
        } else if let Some(code) = info.strip_prefix("seseragi compile_fail ") {
            (!code.trim().is_empty()).then(|| DocumentTestMode::CompileFail {
                code: code.trim().to_owned(),
            })
        } else {
            None
        };
        let testable = mode.is_some();
        index += 1;
        let content_start = index;
        while index < group.len() && group[index].text.trim() != "```" {
            index += 1;
        }
        let content_end = index;
        if testable {
            ordinal += 1;
            let source = group[content_start..content_end]
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            let line_origins = group[content_start..content_end]
                .iter()
                .map(|line| line.content_start)
                .collect();
            let expected_stdout = matches!(mode, Some(DocumentTestMode::Run))
                .then(|| expected_stdout(&group[content_start..content_end]));
            blocks.push(DocumentTestBlock {
                declaration: declaration.clone(),
                ordinal,
                mode: mode.unwrap(),
                source,
                expected_stdout,
                comment_range: ByteSpan {
                    start: group[content_start.saturating_sub(1)].line_start,
                    end: group
                        .get(content_end)
                        .map_or_else(|| group.last().unwrap().line_end, |line| line.line_end),
                },
                line_origins,
            });
        }
        if index < group.len() {
            index += 1;
        }
    }
}

fn expected_stdout(lines: &[DocLine]) -> String {
    let Some(marker) = lines
        .iter()
        .position(|line| line.text.trim() == "// Expected stdout:")
    else {
        return String::new();
    };
    let values = lines[marker + 1..]
        .iter()
        .take_while(|line| line.text.trim_start().starts_with("//"))
        .map(|line| {
            line.text
                .trim_start()
                .strip_prefix("//")
                .unwrap()
                .strip_prefix(' ')
                .unwrap_or_else(|| line.text.trim_start().strip_prefix("//").unwrap())
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        String::new()
    } else {
        values.join("\n") + "\n"
    }
}

fn declaration_name(declaration: &SurfaceDecl) -> String {
    match declaration {
        SurfaceDecl::Let { pattern, .. } => pattern
            .bindings()
            .first()
            .map(|binding| binding.name.clone())
            .unwrap_or_else(|| "<pattern>".to_owned()),
        SurfaceDecl::EffectFn { name, .. }
        | SurfaceDecl::Fn { name, .. }
        | SurfaceDecl::Newtype { name, .. }
        | SurfaceDecl::Alias { name, .. }
        | SurfaceDecl::Type { name, .. }
        | SurfaceDecl::Struct { name, .. }
        | SurfaceDecl::Trait { name, .. } => name.clone(),
        SurfaceDecl::Operator { spelling, .. } => spelling.clone(),
        SurfaceDecl::Impl { .. } => "<impl>".to_owned(),
        SurfaceDecl::Instance { trait_name, .. } => trait_name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_attached_blocks_in_source_order_and_maps_offsets() {
        let source = "//! module docs\n//! ```seseragi\n//! let module_value: Int = 1\n//! ```\n/// docs\n/// ```seseragi\n/// let value: Int = 1\n/// ```\n/// ```seseragi compile_fail SES-T0101\n/// let bad: Int = \"x\"\n/// ```\npub fn add left: Int -> Int = left\n";
        let blocks = extract_document_tests("math.ssrg", source);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].declaration, "<module>");
        assert_eq!(blocks[0].ordinal, 1);
        assert_eq!(blocks[1].declaration, "add");
        assert_eq!(blocks[1].ordinal, 1);
        assert_eq!(&source[blocks[2].original_offset(4)..][..3], "bad");
    }

    #[test]
    fn ignores_detached_and_no_test_fences() {
        let source = "/// ```seseragi\n/// let skipped = 1\n/// ```\n\npub let value = 1\n/// ```seseragi no_test\n/// nope\n/// ```\npub let other = 2\n";
        assert!(extract_document_tests("main.ssrg", source).is_empty());
    }
}
