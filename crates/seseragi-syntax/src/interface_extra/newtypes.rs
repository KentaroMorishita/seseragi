use super::{named_type, raw_at};
use crate::interface_model::{InterfaceExport, InterfaceScheme, InterfaceType, ModuleInterface};
use crate::surface::{ByteSpan, Visibility};
use crate::token::{Token, TokenKind};

pub(super) fn enrich_newtypes(interface: &mut ModuleInterface, tokens: &[Token]) {
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
