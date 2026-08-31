use super::{
    ByteSpan, ForeignCallKind, ForeignCallMode, SurfaceDecl, SurfaceForeignMember,
    SurfaceForeignModule, SurfaceParser, TypeRef, Visibility,
};
use crate::token::TokenKind;

impl SurfaceParser<'_> {
    pub(super) fn parse_foreign_modules(&self) -> Vec<SurfaceForeignModule> {
        let starts = self.top_level_declaration_starts();
        starts
            .iter()
            .enumerate()
            .filter_map(|(position, start)| {
                let end = starts
                    .get(position + 1)
                    .copied()
                    .unwrap_or(self.non_eof_token_count);
                self.parse_foreign_module(*start, end)
            })
            .collect()
    }

    fn parse_foreign_module(&self, start: usize, end: usize) -> Option<SurfaceForeignModule> {
        let first = self.next_significant_token(start, end)?;
        let (visibility, foreign) = if self.kind_at(first) == Some(TokenKind::KeywordPub) {
            (
                Visibility::Public,
                self.next_significant_token(first + 1, end)?,
            )
        } else {
            (Visibility::Private, first)
        };
        if self.raw_at(foreign) != Some("foreign") {
            return None;
        }
        let language = self.next_significant_token(foreign + 1, end)?;
        if self.kind_at(language) != Some(TokenKind::LiteralString) {
            return None;
        }
        let from = self.next_significant_token(language + 1, end)?;
        if self.raw_at(from) != Some("from") {
            return None;
        }
        let specifier = self.next_significant_token(from + 1, end)?;
        if self.kind_at(specifier) != Some(TokenKind::LiteralString) {
            return None;
        }
        let open = self.next_significant_token(specifier + 1, end)?;
        if self.kind_at(open) != Some(TokenKind::PunctuationBraceLeft) {
            return None;
        }
        let close = self.find_matching_brace(open, end)?;
        Some(SurfaceForeignModule {
            visibility,
            language: super::unquote(self.raw_at(language)?),
            specifier: super::unquote(self.raw_at(specifier)?),
            members: self.parse_foreign_members(open + 1, close),
            span: ByteSpan {
                start: self.tokens.get(start)?.start,
                end: self.tokens.get(close)?.end,
            },
        })
    }

    fn parse_foreign_members(&self, start: usize, end: usize) -> Vec<SurfaceForeignMember> {
        let starts = self.foreign_member_starts(start, end);
        starts
            .iter()
            .enumerate()
            .filter_map(|(position, member_start)| {
                let member_end = starts.get(position + 1).copied().unwrap_or(end);
                self.parse_foreign_member(*member_start, member_end)
            })
            .collect()
    }

    fn foreign_member_starts(&self, start: usize, end: usize) -> Vec<usize> {
        let mut starts = Vec::new();
        let mut brace_depth = 0usize;
        for index in start..end {
            match self.kind_at(index) {
                Some(TokenKind::PunctuationBraceLeft) => brace_depth += 1,
                Some(TokenKind::PunctuationBraceRight) => {
                    brace_depth = brace_depth.saturating_sub(1)
                }
                Some(TokenKind::IdentifierLower | TokenKind::IdentifierUpper)
                    if brace_depth == 0
                        && matches!(
                            self.raw_at(index),
                            Some("pure" | "task" | "opaque" | "namespace")
                        )
                        && self.is_declaration_boundary(index) =>
                {
                    starts.push(index);
                }
                _ => {}
            }
        }
        starts
    }

    fn parse_foreign_member(&self, start: usize, end: usize) -> Option<SurfaceForeignMember> {
        if self.raw_at(start) == Some("namespace") {
            let name_index = self.next_significant_token(start + 1, end)?;
            let name = self.identifier_name_at(name_index)?;
            let next = self.next_significant_token(name_index + 1, end)?;
            let (host_name, open) = if self.kind_at(next) == Some(TokenKind::OperatorEquals) {
                let host = self.next_significant_token(next + 1, end)?;
                if self.kind_at(host) != Some(TokenKind::LiteralString) {
                    return None;
                }
                (
                    super::unquote(self.raw_at(host)?),
                    self.next_significant_token(host + 1, end)?,
                )
            } else {
                (name.clone(), next)
            };
            if self.kind_at(open) != Some(TokenKind::PunctuationBraceLeft) {
                return None;
            }
            let close = self.find_matching_brace(open, end)?;
            return Some(SurfaceForeignMember::Namespace {
                name,
                name_span: self.byte_span(name_index)?,
                host_name,
                members: self.parse_foreign_members(open + 1, close),
                span: self.declaration_span(start, close + 1)?,
            });
        }
        if self.raw_at(start) == Some("opaque") {
            let type_keyword = self.next_significant_token(start + 1, end)?;
            if self.raw_at(type_keyword) != Some("type") {
                return None;
            }
            let name_index = self.next_significant_token(type_keyword + 1, end)?;
            let name = self.identifier_name_at(name_index)?;
            return Some(SurfaceForeignMember::OpaqueType {
                name,
                name_span: self.byte_span(name_index)?,
                span: self.declaration_span(start, end)?,
            });
        }

        let mode = match self.raw_at(start) {
            Some("pure") => ForeignCallMode::Pure,
            Some("task") => ForeignCallMode::Task,
            _ => return None,
        };
        let after_mode = self.next_significant_token(start + 1, end)?;
        let (call_kind, fn_keyword) = match self.raw_at(after_mode) {
            Some("constructor") => (
                ForeignCallKind::Constructor,
                self.next_significant_token(after_mode + 1, end)?,
            ),
            Some("method") => (
                ForeignCallKind::Method,
                self.next_significant_token(after_mode + 1, end)?,
            ),
            Some("property") => (
                ForeignCallKind::Property,
                self.next_significant_token(after_mode + 1, end)?,
            ),
            _ => (ForeignCallKind::Function, after_mode),
        };
        if mode == ForeignCallMode::Pure && self.raw_at(fn_keyword) == Some("value") {
            let name_index = self.next_significant_token(fn_keyword + 1, end)?;
            let name = self.identifier_name_at(name_index)?;
            let colon = self.next_significant_token(name_index + 1, end)?;
            if self.kind_at(colon) != Some(TokenKind::PunctuationColon) {
                return None;
            }
            let equals = self
                .find_significant_token(colon + 1, end, |kind| kind == TokenKind::OperatorEquals);
            let type_end = equals.unwrap_or(end);
            let type_ref = self.parse_type_name(colon + 1, type_end)?;
            let host_name = equals
                .and_then(|equals| self.next_significant_token(equals + 1, end))
                .filter(|index| self.kind_at(*index) == Some(TokenKind::LiteralString))
                .and_then(|index| self.raw_at(index))
                .map(super::unquote)
                .unwrap_or_else(|| name.clone());
            return Some(SurfaceForeignMember::Value {
                name,
                name_span: self.byte_span(name_index)?,
                host_name,
                type_ref,
                span: self.declaration_span(start, end)?,
            });
        }
        if self.kind_at(fn_keyword) != Some(TokenKind::KeywordFn) {
            return None;
        }
        let name_index = self.next_significant_token(fn_keyword + 1, end)?;
        let name = self.identifier_name_at(name_index)?;
        let equals = self.find_significant_token(name_index + 1, end, |kind| {
            kind == TokenKind::OperatorEquals
        });
        let signature_end = equals.unwrap_or(end);
        let (parameters, return_type) =
            self.parse_curried_signature(name_index + 1, signature_end)?;
        let host_name = equals
            .and_then(|equals| self.next_significant_token(equals + 1, end))
            .filter(|index| self.kind_at(*index) == Some(TokenKind::LiteralString))
            .and_then(|index| self.raw_at(index))
            .map(super::unquote)
            .unwrap_or_else(|| name.clone());
        Some(SurfaceForeignMember::Function {
            mode,
            call_kind,
            name,
            name_span: self.byte_span(name_index)?,
            host_name,
            parameters,
            return_type,
            span: self.declaration_span(start, end)?,
        })
    }
}

impl SurfaceForeignModule {
    pub(super) fn declarations(&self) -> Vec<SurfaceDecl> {
        self.members
            .iter()
            .flat_map(|member| foreign_member_declarations(member, None, self.visibility))
            .collect()
    }
}

fn foreign_member_declarations(
    member: &SurfaceForeignMember,
    namespace: Option<&str>,
    visibility: Visibility,
) -> Vec<SurfaceDecl> {
    let qualify = |name: &str| match namespace {
        Some(namespace) => format!("{namespace}.{name}"),
        None => name.to_owned(),
    };
    match member {
        SurfaceForeignMember::Function {
            mode,
            name,
            name_span,
            parameters,
            return_type,
            span,
            ..
        } => vec![SurfaceDecl::Fn {
            visibility,
            name: qualify(name),
            name_span: *name_span,
            type_parameters: Vec::new(),
            parameters: parameters.clone(),
            return_type: match mode {
                ForeignCallMode::Pure => return_type.clone(),
                ForeignCallMode::Task => task_type(return_type.clone(), *span),
            },
            constraints: Vec::new(),
            body: None,
            span: *span,
        }],
        SurfaceForeignMember::OpaqueType {
            name,
            name_span,
            span,
        } => vec![SurfaceDecl::Struct {
            visibility,
            opaque: true,
            name: qualify(name),
            name_span: *name_span,
            type_parameters: Vec::new(),
            deriving: Vec::new(),
            fields: Vec::new(),
            span: *span,
        }],
        SurfaceForeignMember::Value {
            name,
            name_span,
            type_ref,
            span,
            ..
        } => vec![SurfaceDecl::Let {
            visibility,
            pattern: crate::SurfacePattern::Name {
                name: qualify(name),
                name_span: *name_span,
                span: *name_span,
            },
            type_ref: Some(type_ref.clone()),
            body: None,
            span: *span,
        }],
        SurfaceForeignMember::Namespace { name, members, .. } => {
            let qualified = qualify(name);
            members
                .iter()
                .flat_map(|member| {
                    foreign_member_declarations(member, Some(&qualified), visibility)
                })
                .collect()
        }
    }
}

fn task_type(success: TypeRef, span: ByteSpan) -> TypeRef {
    TypeRef::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            TypeRef::Record {
                closed: true,
                fields: Vec::new(),
                span,
            },
            TypeRef::Named {
                name: "Js.Error".to_owned(),
                arguments: Vec::new(),
                span,
            },
            success,
        ],
        span,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_module_interface, parse_surface_ast, ForeignCallKind, ForeignCallMode, SurfaceDecl,
        SurfaceForeignMember,
    };

    #[test]
    fn parses_typescript_foreign_functions_into_callable_declarations() {
        let module = parse_surface_ast(
            "fixture/main.ssrg",
            concat!(
                "pub foreign \"typescript\" from \"../host/api.mjs\" {\n",
                "  opaque type Handle\n",
                "  pure fn label value: Int -> String\n",
                "  task fn loadCount -> Int = \"load_count\"\n",
                "}\n",
            ),
        );
        assert_eq!(module.foreign_modules.len(), 1);
        assert_eq!(module.foreign_modules[0].specifier, "../host/api.mjs");
        assert!(matches!(
            module.foreign_modules[0].members[2],
            SurfaceForeignMember::Function {
                mode: ForeignCallMode::Task,
                ref host_name,
                ..
            } if host_name == "load_count"
        ));
        assert_eq!(module.declarations.len(), 3);
        let SurfaceDecl::Fn { return_type, .. } = &module.declarations[2] else {
            panic!("expected task function declaration");
        };
        assert!(
            matches!(return_type, crate::TypeRef::Named { name, arguments, .. }
            if name == "Effect" && arguments.len() == 3)
        );
    }

    #[test]
    fn preserves_call_kinds_and_nested_namespace_exports() {
        let source = concat!(
            "pub foreign \"typescript\" from \"client\" {\n",
            "  opaque type Client\n",
            "  task constructor fn open url: String -> Client = \"Client\"\n",
            "  task method fn query self: Client -> sql: String -> String\n",
            "  pure property fn name self: Client -> String\n",
            "  namespace metrics = \"Metrics\" {\n",
            "    task fn count values: Array<Float> -> Float\n",
            "    namespace format = \"Format\" {\n",
            "      task fn label value: Float -> String\n",
            "    }\n",
            "  }\n",
            "}\n",
        );
        let module = parse_surface_ast("fixture/main.ssrg", source);
        assert!(matches!(
            module.foreign_modules[0].members[1],
            SurfaceForeignMember::Function {
                call_kind: ForeignCallKind::Constructor,
                ..
            }
        ));
        assert!(matches!(
            module.foreign_modules[0].members[2],
            SurfaceForeignMember::Function {
                call_kind: ForeignCallKind::Method,
                ..
            }
        ));
        assert!(matches!(
            module.foreign_modules[0].members[3],
            SurfaceForeignMember::Function {
                call_kind: ForeignCallKind::Property,
                ..
            }
        ));
        assert!(module.declarations.iter().any(
            |declaration| matches!(declaration, SurfaceDecl::Fn { name, .. } if name == "metrics.format.label")
        ));

        let interface = parse_module_interface("fixture/main.ssrg", source);
        assert!(interface
            .exports
            .iter()
            .any(|export| export.name == "metrics.count"));
        assert!(interface
            .exports
            .iter()
            .any(|export| export.name == "metrics.format.label"));
    }
}
