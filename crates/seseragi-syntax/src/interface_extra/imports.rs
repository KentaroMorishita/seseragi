use super::{find_raw, raw_at, unquote};
use crate::interface_model::{InterfaceDependency, InterfaceImport, ModuleInterface};
use crate::surface::ByteSpan;
use crate::token::{Token, TokenKind};

pub(super) fn enrich_dependencies(
    interface: &mut ModuleInterface,
    source_name: &str,
    tokens: &[Token],
) {
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

        let specifier_text = unquote(&specifier.raw);
        let module = dependency_module_name(&interface.module, source_name, &specifier_text);
        let imports = tokens[index + 1..from_index]
            .iter()
            .filter(|token| token.kind == TokenKind::IdentifierLower)
            .map(|token| InterfaceImport {
                namespace: "value".to_owned(),
                name: token.raw.clone(),
                symbol: format!("{module}::{}", token.raw),
            })
            .collect::<Vec<_>>();
        interface.dependencies.push(InterfaceDependency {
            specifier: specifier_text,
            module,
            origin: ByteSpan {
                start: tokens[index].start,
                end: specifier.end,
            },
            imports,
        });
    }
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
