use super::{ByteSpan, SurfaceImport, SurfaceImportItem, SurfaceParser, Visibility};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_imports(&self) -> Vec<SurfaceImport> {
        let declaration_starts = self.top_level_declaration_starts();
        declaration_starts
            .iter()
            .enumerate()
            .filter_map(|(position, start)| {
                let end = declaration_starts
                    .get(position + 1)
                    .copied()
                    .unwrap_or(self.non_eof_token_count);
                self.parse_import(*start, end)
            })
            .collect()
    }

    fn parse_import(&self, start: usize, end: usize) -> Option<SurfaceImport> {
        let first = self.next_significant_token(start, end)?;
        let (visibility, import_index) = if self.raw_at(first) == Some("pub") {
            (
                Visibility::Public,
                self.next_significant_token(first + 1, end)?,
            )
        } else {
            (Visibility::Private, first)
        };
        if self.raw_at(import_index) != Some("import") {
            return None;
        }
        let (from_index, specifier_index) = self.find_import_from(import_index + 1, end)?;
        let items = self.parse_import_items(import_index + 1, from_index);
        Some(SurfaceImport {
            visibility,
            specifier: super::unquote(self.tokens.get(specifier_index)?.raw.as_str()),
            items,
            span: ByteSpan {
                start: self.tokens.get(first)?.start,
                end: self.tokens.get(specifier_index)?.end,
            },
        })
    }

    pub(super) fn find_import_from(&self, start: usize, end: usize) -> Option<(usize, usize)> {
        let mut brace_depth = 0usize;
        let mut cursor = start;
        while let Some(index) = self.next_significant_token(cursor, end) {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationBraceLeft) => brace_depth += 1,
                Some(TokenKind::PunctuationBraceRight) => {
                    brace_depth = brace_depth.saturating_sub(1);
                }
                _ if brace_depth == 0 && self.raw_at(index) == Some("from") => {
                    if let Some(specifier_index) = self.next_significant_token(index + 1, end) {
                        if self.kind_at(specifier_index) == Some(TokenKind::LiteralString) {
                            return Some((index, specifier_index));
                        }
                    }
                }
                _ => {}
            }
            cursor = index + 1;
        }
        None
    }

    fn parse_import_items(&self, start: usize, end: usize) -> Vec<SurfaceImportItem> {
        let mut items = Vec::new();
        let mut cursor = start;
        while let Some(index) = self.next_significant_token(cursor, end) {
            if self.raw_at(index) == Some("*") {
                let (alias, after_alias) = self.parse_optional_import_alias(index + 1, end, true);
                if let Some((alias, alias_span)) = alias {
                    let Some(name_span) = self.byte_span(index) else {
                        cursor = after_alias;
                        continue;
                    };
                    items.push(SurfaceImportItem {
                        namespace: "namespace".to_owned(),
                        name: "*".to_owned(),
                        name_span,
                        alias: Some(alias),
                        alias_span: Some(alias_span),
                    });
                    cursor = after_alias;
                    continue;
                }
                cursor = after_alias;
                continue;
            }
            if self.raw_at(index) == Some("operator") {
                let Some(operator_start) = self.next_significant_token(index + 1, end) else {
                    break;
                };
                if let Some((name, after_operator)) = self.operator_spelling(operator_start) {
                    let Some((first, final_token)) = self.tokens.get(operator_start).zip(
                        after_operator
                            .checked_sub(1)
                            .and_then(|index| self.tokens.get(index)),
                    ) else {
                        cursor = after_operator;
                        continue;
                    };
                    items.push(SurfaceImportItem {
                        namespace: "operator".to_owned(),
                        name,
                        name_span: ByteSpan {
                            start: first.start,
                            end: final_token.end,
                        },
                        alias: None,
                        alias_span: None,
                    });
                    cursor = after_operator;
                    continue;
                }
            }
            if matches!(
                self.kind_at(index),
                Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
            ) {
                if let Some(token) = self.tokens.get(index) {
                    let (alias, after_alias) =
                        self.parse_optional_import_alias(index + 1, end, false);
                    let (alias, alias_span) = alias.unzip();
                    items.push(SurfaceImportItem {
                        namespace: "value".to_owned(),
                        name: token.raw.clone(),
                        name_span: ByteSpan {
                            start: token.start,
                            end: token.end,
                        },
                        alias,
                        alias_span,
                    });
                    cursor = after_alias;
                    continue;
                }
            }
            cursor = index + 1;
        }
        items
    }

    fn parse_optional_import_alias(
        &self,
        start: usize,
        end: usize,
        namespace: bool,
    ) -> (Option<(String, ByteSpan)>, usize) {
        let Some(as_index) = self.next_significant_token(start, end) else {
            return (None, start);
        };
        if self.raw_at(as_index) != Some("as") {
            return (None, start);
        }
        let mut alias_tokens = Vec::new();
        let mut cursor = as_index + 1;
        while let Some(index) = self.next_significant_token(cursor, end) {
            if !namespace
                && matches!(
                    self.kind_at(index),
                    Some(TokenKind::PunctuationComma | TokenKind::PunctuationBraceRight)
                )
            {
                break;
            }
            alias_tokens.push(index);
            cursor = index + 1;
        }
        let next_cursor = if namespace {
            end
        } else {
            alias_tokens.last().map_or(as_index + 1, |last| last + 1)
        };
        if alias_tokens.len() != 1 {
            return (None, next_cursor);
        }
        let alias_index = alias_tokens[0];
        let valid_kind = if namespace {
            self.kind_at(alias_index) == Some(TokenKind::IdentifierLower)
        } else {
            matches!(
                self.kind_at(alias_index),
                Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
            )
        };
        if !valid_kind {
            return (None, next_cursor);
        }
        let Some(alias) = self
            .identifier_name_at(alias_index)
            .filter(|name| !super::validation::is_reserved(name))
        else {
            return (None, next_cursor);
        };
        (
            self.byte_span(alias_index).map(|span| (alias, span)),
            next_cursor,
        )
    }
}
