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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentComment {
    /// None denotes a module comment. Other ranges are exact declaration
    /// regions, shared with interface/member metadata rather than name joins.
    pub declaration: Option<ByteSpan>,
    pub declaration_name: String,
    pub markdown: String,
    pub comment_range: ByteSpan,
    line_origins: Vec<usize>,
    line_ranges: Vec<ByteSpan>,
}

impl DocumentComment {
    pub fn original_offset(&self, offset: usize) -> usize {
        let before = &self.markdown.as_bytes()[..offset.min(self.markdown.len())];
        let line = before.iter().filter(|byte| **byte == b'\n').count();
        let column = before.len()
            - before
                .iter()
                .rposition(|byte| *byte == b'\n')
                .map_or(0, |index| index + 1);
        self.line_origins
            .get(line)
            .copied()
            .unwrap_or(self.comment_range.start)
            + column
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentComments {
    pub comments: Vec<DocumentComment>,
    pub unattached: Vec<ByteSpan>,
}

pub fn extract_document_comments(source_name: impl Into<String>, source: &str) -> DocumentComments {
    let source_name = source_name.into();
    let surface = parse_surface_ast(source_name.clone(), source);
    let first_significant = lex(source_name.clone(), source)
        .tokens
        .into_iter()
        .find(|token| {
            !matches!(
                token.kind,
                TokenKind::TriviaSpace
                    | TokenKind::TriviaNewline
                    | TokenKind::TriviaComment
                    | TokenKind::Eof
            )
        })
        .map_or(source.len(), |token| token.start);
    let mut regions = surface
        .declarations
        .iter()
        .map(|declaration| (declaration.span(), declaration_name(declaration)))
        .collect::<Vec<_>>();
    for declaration in &surface.declarations {
        match declaration {
            SurfaceDecl::Trait { name, methods, .. } => regions.extend(
                methods
                    .iter()
                    .map(|method| (method.span, format!("{name}.{}", method.name))),
            ),
            SurfaceDecl::Impl {
                target,
                members,
                span,
                ..
            } => {
                let owner = match target {
                    crate::TypeRef::Named { name, .. } => name.clone(),
                    _ => format!("<impl@{}>", span.start),
                };
                regions.extend(members.iter().map(|member| match member {
                    crate::SurfaceImplMember::Method { method, .. } => {
                        (method.span, format!("{owner}.{}", method.name))
                    }
                    crate::SurfaceImplMember::Operator { span, spelling, .. } => {
                        (*span, format!("{owner}.({spelling})"))
                    }
                }));
            }
            SurfaceDecl::Instance {
                trait_name,
                methods,
                span,
                ..
            } => regions.extend(methods.iter().map(|method| {
                (
                    method.span,
                    format!("<instance({trait_name})@{}>.{}", span.start, method.name),
                )
            })),
            _ => {}
        }
    }
    fn foreign(
        members: &[crate::SurfaceForeignMember],
        owner: &str,
        regions: &mut Vec<(ByteSpan, String)>,
    ) {
        for member in members {
            let (span, name) = match member {
                crate::SurfaceForeignMember::Function { span, name, .. }
                | crate::SurfaceForeignMember::Value { span, name, .. }
                | crate::SurfaceForeignMember::OpaqueType { span, name, .. }
                | crate::SurfaceForeignMember::Namespace { span, name, .. } => (*span, name),
            };
            let name = if owner.is_empty() {
                name.clone()
            } else {
                format!("{owner}.{name}")
            };
            regions.push((span, name.clone()));
            if let crate::SurfaceForeignMember::Namespace { members, .. } = member {
                foreign(members, &name, regions);
            }
        }
    }
    for module in &surface.foreign_modules {
        foreign(&module.members, "", &mut regions);
    }
    regions.sort_by_key(|(span, _)| (span.start, span.end));
    let mut result = DocumentComments::default();
    for group in document_groups(source_name.clone(), source) {
        let first = &group[0];
        let last = group.last().unwrap();
        let range = ByteSpan {
            start: first.line_start,
            end: last.line_end,
        };
        let (declaration, declaration_name) = if first.module {
            if first.line_start > first_significant {
                result.unattached.push(range);
                continue;
            }
            (None, "<module>".to_owned())
        } else {
            let attached = regions.iter().find(|(span, _)| {
                span.start >= last.line_end
                    && source[last.line_end..span.start]
                        .chars()
                        .all(crate::unicode::is_whitespace)
                    && source[last.line_end..span.start].matches('\n').count() <= 1
            });
            let Some((span, name)) = attached else {
                result.unattached.push(range);
                continue;
            };
            (Some(*span), name.clone())
        };
        result.comments.push(DocumentComment {
            declaration,
            declaration_name,
            markdown: group
                .iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            comment_range: range,
            line_origins: group.iter().map(|line| line.content_start).collect(),
            line_ranges: group
                .iter()
                .map(|line| ByteSpan {
                    start: line.line_start,
                    end: line.line_end,
                })
                .collect(),
        });
    }
    result
}

/// Extracts testable fenced blocks from lexical document comments while
/// retaining a byte mapping back to the original source.
pub fn extract_document_tests(
    source_name: impl Into<String>,
    source: &str,
) -> Vec<DocumentTestBlock> {
    let mut blocks = Vec::new();
    for comment in extract_document_comments(source_name, source).comments {
        let group = comment
            .markdown
            .split('\n')
            .enumerate()
            .map(|(index, text)| DocLine {
                text: text.to_owned(),
                content_start: comment.line_origins[index],
                line_start: comment.line_ranges[index].start,
                line_end: comment.line_ranges[index].end,
                module: comment.declaration.is_none(),
            })
            .collect::<Vec<_>>();
        parse_group(&group, comment.declaration_name, &mut blocks);
    }
    blocks
}

fn document_groups(source_name: String, source: &str) -> Vec<Vec<DocLine>> {
    let tokens = lex(source_name.clone(), source);
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

    groups
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

    #[test]
    fn keeps_document_comment_regions_and_utf8_link_offsets_for_members() {
        let source = concat!(
            "//! Module 🙂\n",
            "/// Function [current].\n",
            "pub deprecated \"Use current\" fn old -> Int = 1\n",
            "pub trait Render<A> {\n",
            "  /// Trait member.\n",
            "  deprecated \"Use render\" fn old value: A -> String\n",
            "}\n",
            "pub struct Box { pub value: Int }\n",
            "impl Box {\n",
            "  /// Inherent member.\n",
            "  pub deprecated \"Use read\" fn old self: Box -> Int = self.value\n",
            "}\n",
            "pub foreign \"typescript\" from \"./host.js\" {\n",
            "  /// Foreign member.\n",
            "  deprecated \"Use fresh\" pure fn oldHost -> String\n",
            "}\n",
            "/// Detached.\n\nfn current -> Int = 2\n",
            "//! Misplaced module.\n",
        );
        let comments = extract_document_comments("λ/api.ssrg", source);
        assert_eq!(comments.comments.len(), 5, "{comments:#?}");
        assert_eq!(comments.unattached.len(), 2);
        assert!(comments.comments[0].declaration.is_none());
        for comment in comments.comments.iter().skip(1) {
            let region = comment.declaration.unwrap();
            assert!(source[region.start..region.end].contains("deprecated"));
            assert!(!source[region.start..region.end].starts_with("///"));
        }
        let function = &comments.comments[1];
        let label = function.markdown.find("[current]").unwrap();
        assert_eq!(
            &source[function.original_offset(label)
                ..function.original_offset(label + "[current]".len())],
            "[current]"
        );
    }

    #[test]
    fn extracts_member_fences_with_qualified_ids_and_original_locations() {
        let source = concat!(
            "pub trait Render<A> {\n",
            "  /// ```seseragi\n  /// let traitExample: Int = 1\n  /// ```\n",
            "  fn old value: A -> String\n}\n",
            "pub struct Box { pub value: Int }\nimpl Box {\n",
            "  /// ```seseragi\n  /// let methodExample: Int = 2\n  /// ```\n",
            "  pub fn read self: Box -> Int = self.value\n}\n",
        );
        let blocks = extract_document_tests("api.ssrg", source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].declaration, "Render.old");
        assert_eq!(blocks[1].declaration, "Box.read");
        for (block, name) in blocks.iter().zip(["traitExample", "methodExample"]) {
            let offset = block.source.find(name).unwrap();
            assert_eq!(
                &source[block.original_offset(offset)..block.original_offset(offset + name.len())],
                name
            );
        }
    }
}
