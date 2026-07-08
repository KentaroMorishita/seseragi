use crate::interface_model::{
    InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceInstance, InterfaceOperator,
    InterfaceScheme, InterfaceType, ModuleInterface,
};
use crate::lexer::lex;
use crate::surface::{ByteSpan, Visibility};
use crate::token::{Token, TokenKind};

pub(crate) fn enrich_module_interface(
    interface: &mut ModuleInterface,
    source_name: &str,
    source: &str,
) {
    let tokens = lex(interface.source.clone(), source)
        .tokens
        .into_iter()
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::TriviaComment
                    | TokenKind::TriviaNewline
                    | TokenKind::TriviaSpace
                    | TokenKind::Eof
            )
        })
        .collect::<Vec<_>>();

    enrich_dependencies(interface, source_name, &tokens);
    enrich_newtypes(interface, &tokens);
    enrich_operators(interface, source, &tokens);
    enrich_instances(interface, source, &tokens);
}

fn enrich_dependencies(interface: &mut ModuleInterface, source_name: &str, tokens: &[Token]) {
    for index in 0..tokens.len() {
        if raw_at(tokens, index) != Some("import") {
            continue;
        }
        let Some(from_index) = find_raw(tokens, index + 1, "from") else {
            continue;
        };
        let Some(specifier) = tokens.get(from_index + 1) else {
            continue;
        };
        if specifier.kind != TokenKind::LiteralString {
            continue;
        }

        let imports = tokens[index + 1..from_index]
            .iter()
            .filter(|token| token.kind == TokenKind::IdentifierLower)
            .map(|token| InterfaceImport {
                namespace: "value".to_owned(),
                name: token.raw.clone(),
                symbol: format!(
                    "{}::{}",
                    dependency_module_name(
                        &interface.module,
                        source_name,
                        &unquote(&specifier.raw)
                    ),
                    token.raw
                ),
            })
            .collect::<Vec<_>>();
        interface.dependencies.push(InterfaceDependency {
            specifier: unquote(&specifier.raw),
            module: dependency_module_name(
                &interface.module,
                source_name,
                &unquote(&specifier.raw),
            ),
            origin: ByteSpan {
                start: tokens[index].start,
                end: specifier.end,
            },
            imports,
        });
    }
}

fn enrich_newtypes(interface: &mut ModuleInterface, tokens: &[Token]) {
    for index in 0..tokens.len() {
        if raw_at(tokens, index) != Some("pub") || raw_at(tokens, index + 1) != Some("newtype") {
            continue;
        }
        let Some(name) = tokens.get(index + 2) else {
            continue;
        };
        let Some(representation) = tokens.get(index + 4) else {
            continue;
        };
        if name.kind != TokenKind::IdentifierUpper
            || representation.kind != TokenKind::IdentifierUpper
        {
            continue;
        }

        interface.exports.push(InterfaceExport {
            symbol: format!("{}::{}", interface.module, name.raw),
            namespace: "type".to_owned(),
            name: name.raw.clone(),
            visibility: Visibility::Public,
            declaration_kind: Some("newtype".to_owned()),
            declaration: ByteSpan {
                start: tokens[index].start,
                end: representation.end,
            },
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.raw.clone(),
                    arity: 0,
                },
            },
            representation: Some(named_type(&representation.raw)),
        });
    }
}

fn enrich_operators(interface: &mut ModuleInterface, source: &str, tokens: &[Token]) {
    for index in 0..tokens.len() {
        if raw_at(tokens, index) != Some("pub") || raw_at(tokens, index + 1) != Some("operator") {
            continue;
        }
        let (Some(fixity), Some(precedence)) = (tokens.get(index + 2), tokens.get(index + 3))
        else {
            continue;
        };
        let Some(precedence) = precedence.raw.parse::<u32>().ok() else {
            continue;
        };
        let Some((spelling, after_spelling)) = operator_spelling(tokens, index + 4) else {
            continue;
        };
        let end = declaration_end_before_blank_line(source, tokens[index].start);
        let symbol = format!("{}::operator({spelling})", interface.module);
        let origin = ByteSpan {
            start: tokens[index].start,
            end,
        };
        let type_ref = operator_type(tokens, after_spelling);

        interface.exports.push(InterfaceExport {
            symbol: symbol.clone(),
            namespace: "operator".to_owned(),
            name: spelling.clone(),
            visibility: Visibility::Public,
            declaration_kind: Some("custom-operator".to_owned()),
            declaration: origin,
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref,
            },
            representation: None,
        });
        interface.operators.push(InterfaceOperator {
            symbol,
            spelling,
            fixity: fixity.raw.clone(),
            precedence,
            origin,
        });
    }
}

fn enrich_instances(interface: &mut ModuleInterface, source: &str, tokens: &[Token]) {
    for index in 0..tokens.len() {
        if raw_at(tokens, index) != Some("instance") {
            continue;
        }
        let (Some(trait_name), Some(argument)) = (tokens.get(index + 1), tokens.get(index + 3))
        else {
            continue;
        };
        if trait_name.kind != TokenKind::IdentifierUpper
            || argument.kind != TokenKind::IdentifierUpper
        {
            continue;
        }
        interface.instances.push(InterfaceInstance {
            trait_name: trait_name.raw.clone(),
            head: InterfaceType::Apply {
                constructor: trait_name.raw.clone(),
                arguments: vec![named_type(&argument.raw)],
            },
            constraints: Vec::new(),
            origin: ByteSpan {
                start: tokens[index].start,
                end: declaration_end_to_matching_brace(source, tokens[index].start),
            },
        });
    }
}

fn operator_type(tokens: &[Token], start: usize) -> InterfaceType {
    let type_names = tokens[start..]
        .iter()
        .take_while(|token| token.kind != TokenKind::OperatorEquals)
        .filter(|token| token.kind == TokenKind::IdentifierUpper)
        .map(|token| token.raw.clone())
        .collect::<Vec<_>>();

    match type_names.as_slice() {
        [parameter, result] => function_type(named_type(parameter), named_type(result)),
        [first, second, result, ..] => function_type(
            named_type(first),
            function_type(named_type(second), named_type(result)),
        ),
        [single] => named_type(single),
        _ => named_type("Unit"),
    }
}

fn operator_spelling(tokens: &[Token], start: usize) -> Option<(String, usize)> {
    let first = tokens.get(start)?;
    if !is_operator_spelling_token(first) {
        return None;
    }
    let mut spelling = first.raw.clone();
    let mut cursor = start + 1;
    let mut previous_end = first.end;
    while let Some(token) = tokens.get(cursor) {
        if token.start != previous_end || !is_operator_spelling_token(token) {
            break;
        }
        spelling.push_str(&token.raw);
        previous_end = token.end;
        cursor += 1;
    }
    Some((spelling, cursor))
}

fn is_operator_spelling_token(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::OperatorArithmetic
            | TokenKind::OperatorComparison
            | TokenKind::OperatorCustom
            | TokenKind::OperatorPipeline
            | TokenKind::OperatorBind
            | TokenKind::OperatorApply
            | TokenKind::OperatorRangeExclusive
            | TokenKind::OperatorRangeInclusive
    )
}

fn function_type(parameter: InterfaceType, result: InterfaceType) -> InterfaceType {
    InterfaceType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}

fn named_type(name: &str) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn raw_at(tokens: &[Token], index: usize) -> Option<&str> {
    tokens.get(index).map(|token| token.raw.as_str())
}

fn find_raw(tokens: &[Token], start: usize, raw: &str) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| (token.raw == raw).then_some(index))
}

fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw)
        .to_owned()
}

fn dependency_module_name(module_name: &str, source_name: &str, specifier: &str) -> String {
    if let Some(relative) = specifier.strip_prefix("./") {
        return format!("{module_name}/{relative}");
    }
    source_name
        .rsplit_once('/')
        .map(|(parent, _)| format!("{parent}/{specifier}"))
        .unwrap_or_else(|| specifier.to_owned())
}

fn declaration_end_before_blank_line(source: &str, start: usize) -> usize {
    source[start..]
        .find("\n\n")
        .map(|offset| start + offset)
        .unwrap_or(source.len())
        .min(source.len())
}

fn declaration_end_to_matching_brace(source: &str, start: usize) -> usize {
    let mut depth = 0usize;
    for (offset, char) in source[start..].char_indices() {
        match char {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return start + offset + char.len_utf8();
                }
            }
            _ => {}
        }
    }
    source.len()
}
