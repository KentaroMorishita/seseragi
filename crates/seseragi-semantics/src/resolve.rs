use crate::{ResolvedDecl, ResolvedModule, SymbolId};
use seseragi_syntax::ModuleInterface;

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

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{parse_module_interface, ByteSpan, Visibility};

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
}
