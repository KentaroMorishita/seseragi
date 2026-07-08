mod imports;
mod instances;
mod newtypes;
mod operators;

use crate::interface_model::{InterfaceType, ModuleInterface};
use crate::lexer::lex;
use crate::token::{Token, TokenKind};

pub(crate) fn enrich_module_interface(
    interface: &mut ModuleInterface,
    source_name: &str,
    source: &str,
) {
    let tokens = significant_tokens(interface.source.clone(), source);

    imports::enrich_dependencies(interface, source_name, &tokens);
    newtypes::enrich_newtypes(interface, &tokens);
    operators::enrich_operators(interface, source, &tokens);
    instances::enrich_instances(interface, source, &tokens);
}

fn significant_tokens(source_name: impl Into<String>, source: &str) -> Vec<Token> {
    lex(source_name, source)
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
        .collect()
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

fn named_type(name: &str) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
