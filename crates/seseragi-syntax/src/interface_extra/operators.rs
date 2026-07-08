use super::{named_type, raw_at};
use crate::interface_model::{
    InterfaceExport, InterfaceOperator, InterfaceScheme, InterfaceType, ModuleInterface,
};
use crate::surface::{ByteSpan, Visibility};
use crate::token::{Token, TokenKind};

pub(super) fn enrich_operators(interface: &mut ModuleInterface, source: &str, tokens: &[Token]) {
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
        let origin = ByteSpan {
            start: tokens[index].start,
            end: declaration_end_before_blank_line(source, tokens[index].start),
        };
        let symbol = format!("{}::operator({spelling})", interface.module);
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

fn declaration_end_before_blank_line(source: &str, start: usize) -> usize {
    source[start..]
        .find("\n\n")
        .map(|offset| start + offset)
        .unwrap_or(source.len())
        .min(source.len())
}
