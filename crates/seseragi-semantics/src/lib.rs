use serde::{Deserialize, Serialize};
use seseragi_syntax::{
    lex, parse_module_interface, parse_surface_ast, ByteSpan, InterfaceType, ModuleInterface,
    SurfaceDecl, Token, TokenKind, TypeRef, Visibility,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedModule {
    pub schema: u32,
    pub source: String,
    pub module: String,
    pub declarations: Vec<ResolvedDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum ResolvedDecl {
    Value {
        symbol: SymbolId,
        name: String,
        visibility: Visibility,
        declaration: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModule {
    pub schema: u32,
    pub stage: String,
    pub source: String,
    pub module: String,
    pub declarations: Vec<TypedDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedDecl {
    Let {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        scheme: TypedScheme,
        value: TypedExpr,
    },
    EffectFn {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        parameters: Vec<TypedParameter>,
        effect: TypedEffect,
        body: TypedExpr,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedScheme {
    pub type_parameters: Vec<String>,
    pub constraints: Vec<TypedConstraint>,
    #[serde(rename = "type")]
    pub type_ref: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedType {
    Named {
        name: String,
        arguments: Vec<TypedType>,
    },
    Record {
        closed: bool,
        fields: Vec<TypedRecordField>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedRecordField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedParameter {
    ImplicitUnit {
        #[serde(rename = "type")]
        type_ref: TypedType,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedEffect {
    pub environment: TypedType,
    pub failure: TypedType,
    pub success: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedExpr {
    Unit {
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    EffectCall {
        operation: String,
        arguments: Vec<TypedExpr>,
        origin: ByteSpan,
    },
    DoBlock {
        statements: Vec<TypedExpr>,
        result: Box<TypedExpr>,
        origin: ByteSpan,
    },
}

pub fn resolve_module_interface(interface: ModuleInterface) -> ResolvedModule {
    let declarations = interface
        .exports
        .into_iter()
        .filter(|export| export.namespace == "value")
        .enumerate()
        .map(|(index, export)| ResolvedDecl::Value {
            symbol: SymbolId(index as u32),
            name: export.name,
            visibility: export.visibility,
            declaration: export.declaration,
        })
        .collect();

    ResolvedModule {
        schema: interface.schema,
        source: interface.source,
        module: interface.module,
        declarations,
    }
}

pub fn type_module_interface(interface: ModuleInterface) -> TypedModule {
    let declarations = interface
        .exports
        .into_iter()
        .filter(|export| export.namespace == "value")
        .map(|export| TypedDecl::Let {
            symbol: export.symbol,
            visibility: export.visibility,
            origin: export.declaration,
            scheme: TypedScheme {
                type_parameters: export.scheme.type_parameters,
                constraints: export
                    .scheme
                    .constraints
                    .into_iter()
                    .map(|constraint| TypedConstraint {
                        name: constraint.name,
                    })
                    .collect(),
                type_ref: typed_type_from_interface_type(export.scheme.type_ref),
            },
            value: TypedExpr::Integer {
                value: "0".to_owned(),
                type_ref: TypedType::Named {
                    name: "Int".to_owned(),
                    arguments: Vec::new(),
                },
                origin: export.declaration,
            },
        })
        .collect();

    TypedModule {
        schema: interface.schema,
        stage: "typed-hir".to_owned(),
        source: interface.source,
        module: interface.module,
        declarations,
    }
}

pub fn type_module(source_name: impl Into<String>, source: &str) -> TypedModule {
    let source_name = source_name.into();
    let interface = parse_module_interface(source_name.clone(), source);
    let surface = parse_surface_ast(interface.source.clone(), source);
    let tokens = lex(interface.source.clone(), source).tokens;
    let declarations = surface
        .declarations
        .into_iter()
        .filter_map(|declaration| typed_decl_from_surface(&interface.module, declaration, &tokens))
        .collect();

    TypedModule {
        schema: interface.schema,
        stage: "typed-hir".to_owned(),
        source: interface.source,
        module: interface.module,
        declarations,
    }
}

fn typed_decl_from_surface(
    module: &str,
    declaration: SurfaceDecl,
    tokens: &[Token],
) -> Option<TypedDecl> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            type_ref,
            span,
            ..
        } => {
            let value = find_value_token(tokens, span)
                .map(typed_expr_from_value_token)
                .unwrap_or_else(|| TypedExpr::Integer {
                    value: "0".to_owned(),
                    type_ref: TypedType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                    origin: span,
                });
            Some(TypedDecl::Let {
                symbol: format!("{module}::{name}"),
                visibility,
                origin: span,
                scheme: TypedScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: type_ref
                        .as_ref()
                        .map(typed_type_from_type_ref)
                        .unwrap_or_else(|| inferred_type_from_expr(&value)),
                },
                value,
            })
        }
        SurfaceDecl::EffectFn {
            visibility,
            name,
            return_type,
            span,
            ..
        } => {
            let with_type = find_type_name_after(tokens, span, TokenKind::KeywordWith);
            let failure = find_type_name_after(tokens, span, TokenKind::KeywordFails)
                .unwrap_or_else(|| "Never".to_owned());
            let success = return_type
                .as_ref()
                .map(typed_type_from_type_ref)
                .unwrap_or_else(unit_type);
            Some(TypedDecl::EffectFn {
                symbol: format!("{module}::{name}"),
                visibility,
                origin: span,
                parameters: vec![TypedParameter::ImplicitUnit {
                    type_ref: unit_type(),
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: with_type
                            .map(|name| TypedRecordField {
                                name: lower_first(&name),
                                type_ref: TypedType::Named {
                                    name,
                                    arguments: Vec::new(),
                                },
                            })
                            .into_iter()
                            .collect(),
                    },
                    failure: TypedType::Named {
                        name: failure,
                        arguments: Vec::new(),
                    },
                    success,
                },
                body: find_effect_body(tokens, span).unwrap_or_else(|| TypedExpr::EffectCall {
                    operation: "std/prelude::unit".to_owned(),
                    arguments: Vec::new(),
                    origin: span,
                }),
            })
        }
    }
}

fn typed_type_from_interface_type(type_ref: InterfaceType) -> TypedType {
    match type_ref {
        InterfaceType::Named { name, arguments } => TypedType::Named {
            name,
            arguments: arguments
                .into_iter()
                .map(typed_type_from_interface_type)
                .collect(),
        },
    }
}

fn typed_type_from_type_ref(type_ref: &TypeRef) -> TypedType {
    match type_ref {
        TypeRef::Named { name, .. } => TypedType::Named {
            name: name.clone(),
            arguments: Vec::new(),
        },
    }
}

fn inferred_type_from_expr(expr: &TypedExpr) -> TypedType {
    match expr {
        TypedExpr::Unit { .. } => unit_type(),
        TypedExpr::Integer { .. } => TypedType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        },
        TypedExpr::String { .. } => TypedType::Named {
            name: "String".to_owned(),
            arguments: Vec::new(),
        },
        TypedExpr::EffectCall { .. } => unit_type(),
        TypedExpr::DoBlock { result, .. } => inferred_type_from_expr(result),
    }
}

fn typed_expr_from_value_token(token: &Token) -> TypedExpr {
    let origin = ByteSpan {
        start: token.start,
        end: token.end,
    };
    match token.kind {
        TokenKind::LiteralInteger => TypedExpr::Integer {
            value: token.raw.clone(),
            type_ref: TypedType::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
            },
            origin,
        },
        TokenKind::LiteralString => TypedExpr::String {
            value: unquote_string(&token.raw),
            type_ref: TypedType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            },
            origin,
        },
        _ => TypedExpr::EffectCall {
            operation: "std/prelude::unknown".to_owned(),
            arguments: Vec::new(),
            origin,
        },
    }
}

fn find_value_token(tokens: &[Token], span: ByteSpan) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))
}

fn find_type_name_after(tokens: &[Token], span: ByteSpan, keyword: TokenKind) -> Option<String> {
    let keyword_index = tokens.iter().position(|token| {
        token.start >= span.start && token.end <= span.end && token.kind == keyword
    })?;
    tokens[keyword_index + 1..]
        .iter()
        .find(|token| {
            token.end <= span.end
                && matches!(
                    token.kind,
                    TokenKind::IdentifierLower | TokenKind::IdentifierUpper
                )
        })
        .map(|token| token.raw.clone())
}

fn find_effect_body(tokens: &[Token], span: ByteSpan) -> Option<TypedExpr> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    let operation = tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))?;

    if operation.kind == TokenKind::KeywordDo {
        return typed_do_block(tokens, span, operation);
    }

    let argument = tokens
        .iter()
        .skip_while(|token| token.start <= operation.start)
        .find(|token| token.end <= span.end && token.kind == TokenKind::LiteralString)
        .map(typed_expr_from_value_token);
    let origin_end = argument
        .as_ref()
        .map(expr_origin_end)
        .unwrap_or(operation.end);

    Some(TypedExpr::EffectCall {
        operation: match operation.raw.as_str() {
            "println" => "std/prelude::println".to_owned(),
            other => other.to_owned(),
        },
        arguments: argument.into_iter().collect(),
        origin: ByteSpan {
            start: operation.start,
            end: origin_end,
        },
    })
}

fn typed_do_block(tokens: &[Token], span: ByteSpan, do_token: &Token) -> Option<TypedExpr> {
    let right_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "}")?;
    Some(TypedExpr::DoBlock {
        statements: Vec::new(),
        result: Box::new(TypedExpr::Unit {
            type_ref: unit_type(),
            origin: ByteSpan {
                start: right_brace.start,
                end: right_brace.start,
            },
        }),
        origin: ByteSpan {
            start: do_token.start,
            end: right_brace.end,
        },
    })
}

fn expr_origin_end(expr: &TypedExpr) -> usize {
    match expr {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::DoBlock { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
        | TypedExpr::EffectCall { origin, .. } => origin.end,
    }
}

fn is_significant(token: &Token) -> bool {
    !matches!(
        token.kind,
        TokenKind::TriviaComment
            | TokenKind::TriviaNewline
            | TokenKind::TriviaSpace
            | TokenKind::Eof
    )
}

fn unit_type() -> TypedType {
    TypedType::Named {
        name: "Unit".to_owned(),
        arguments: Vec::new(),
    }
}

fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn unquote_string(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::parse_module_interface;

    #[test]
    fn resolves_basic_public_let_interface() {
        let interface =
            parse_module_interface("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let resolved = resolve_module_interface(interface);

        assert_eq!(resolved.schema, 1);
        assert_eq!(resolved.module, "artifact/basic");
        assert_eq!(resolved.source, "main.ssrg");
        assert_eq!(
            resolved.declarations,
            vec![ResolvedDecl::Value {
                symbol: SymbolId(0),
                name: "answer".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 0, end: 24 },
            }]
        );
    }

    #[test]
    fn resolves_multiple_lets_interface() {
        let interface = parse_module_interface(
            "artifact/multiple/main.ssrg",
            "pub let first: Int = 1\npub let second: Int = 2\n",
        );
        let resolved = resolve_module_interface(interface);

        assert_eq!(resolved.declarations.len(), 2);
        assert_eq!(
            resolved.declarations[0],
            ResolvedDecl::Value {
                symbol: SymbolId(0),
                name: "first".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 0, end: 22 },
            }
        );
        assert_eq!(
            resolved.declarations[1],
            ResolvedDecl::Value {
                symbol: SymbolId(1),
                name: "second".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 23, end: 46 },
            }
        );
    }

    #[test]
    fn types_basic_public_let() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");

        assert_eq!(typed.schema, 1);
        assert_eq!(typed.stage, "typed-hir");
        assert_eq!(typed.module, "artifact/basic");
        assert_eq!(typed.source, "main.ssrg");
        assert_eq!(
            typed.declarations,
            vec![TypedDecl::Let {
                symbol: "artifact/basic::answer".to_owned(),
                visibility: Visibility::Public,
                origin: ByteSpan { start: 0, end: 24 },
                scheme: TypedScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: TypedType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                },
                value: TypedExpr::Integer {
                    value: "42".to_owned(),
                    type_ref: TypedType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                    origin: ByteSpan { start: 22, end: 24 },
                },
            }]
        );
    }

    #[test]
    fn types_private_and_public_lets() {
        let typed = type_module(
            "artifact/multiple-lets/main.ssrg",
            "let first = 1\npub let second: Int = 2\n",
        );

        assert_eq!(
            typed.declarations,
            vec![
                TypedDecl::Let {
                    symbol: "artifact/multiple-lets::first".to_owned(),
                    visibility: Visibility::Private,
                    origin: ByteSpan { start: 0, end: 13 },
                    scheme: TypedScheme {
                        type_parameters: Vec::new(),
                        constraints: Vec::new(),
                        type_ref: TypedType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        },
                    },
                    value: TypedExpr::Integer {
                        value: "1".to_owned(),
                        type_ref: TypedType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        },
                        origin: ByteSpan { start: 12, end: 13 },
                    },
                },
                TypedDecl::Let {
                    symbol: "artifact/multiple-lets::second".to_owned(),
                    visibility: Visibility::Public,
                    origin: ByteSpan { start: 14, end: 37 },
                    scheme: TypedScheme {
                        type_parameters: Vec::new(),
                        constraints: Vec::new(),
                        type_ref: TypedType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        },
                    },
                    value: TypedExpr::Integer {
                        value: "2".to_owned(),
                        type_ref: TypedType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        },
                        origin: ByteSpan { start: 36, end: 37 },
                    },
                },
            ]
        );
    }

    #[test]
    fn types_effect_main() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );

        assert_eq!(
            typed.declarations,
            vec![TypedDecl::EffectFn {
                symbol: "artifact/effect-main::main".to_owned(),
                visibility: Visibility::Public,
                origin: ByteSpan { start: 0, end: 78 },
                parameters: vec![TypedParameter::ImplicitUnit {
                    type_ref: unit_type(),
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: vec![TypedRecordField {
                            name: "console".to_owned(),
                            type_ref: TypedType::Named {
                                name: "Console".to_owned(),
                                arguments: Vec::new(),
                            },
                        }],
                    },
                    failure: TypedType::Named {
                        name: "ConsoleError".to_owned(),
                        arguments: Vec::new(),
                    },
                    success: unit_type(),
                },
                body: TypedExpr::EffectCall {
                    operation: "std/prelude::println".to_owned(),
                    arguments: vec![TypedExpr::String {
                        value: "hello".to_owned(),
                        type_ref: TypedType::Named {
                            name: "String".to_owned(),
                            arguments: Vec::new(),
                        },
                        origin: ByteSpan { start: 71, end: 78 },
                    }],
                    origin: ByteSpan { start: 63, end: 78 },
                },
            }]
        );
    }

    #[test]
    fn types_empty_do_block_as_unit_result() {
        let typed = type_module(
            "artifact/effect-do/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n",
        );

        assert_eq!(
            typed.declarations,
            vec![TypedDecl::EffectFn {
                symbol: "artifact/effect-do::main".to_owned(),
                visibility: Visibility::Public,
                origin: ByteSpan { start: 0, end: 68 },
                parameters: vec![TypedParameter::ImplicitUnit {
                    type_ref: unit_type(),
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: vec![TypedRecordField {
                            name: "console".to_owned(),
                            type_ref: TypedType::Named {
                                name: "Console".to_owned(),
                                arguments: Vec::new(),
                            },
                        }],
                    },
                    failure: TypedType::Named {
                        name: "ConsoleError".to_owned(),
                        arguments: Vec::new(),
                    },
                    success: unit_type(),
                },
                body: TypedExpr::DoBlock {
                    statements: Vec::new(),
                    result: Box::new(TypedExpr::Unit {
                        type_ref: unit_type(),
                        origin: ByteSpan { start: 67, end: 67 },
                    }),
                    origin: ByteSpan { start: 63, end: 68 },
                },
            }]
        );
    }
}
