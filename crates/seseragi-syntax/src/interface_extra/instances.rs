use super::{named_type, raw_at};
use crate::interface_model::{InterfaceInstance, InterfaceType, ModuleInterface};
use crate::surface::ByteSpan;
use crate::token::{Token, TokenKind};

pub(super) fn enrich_instances(interface: &mut ModuleInterface, source: &str, tokens: &[Token]) {
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
