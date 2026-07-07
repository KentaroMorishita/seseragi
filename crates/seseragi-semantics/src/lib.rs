use serde::{Deserialize, Serialize};
use seseragi_syntax::{ByteSpan, InterfaceType, ModuleInterface, Visibility};

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
    Value {
        symbol: SymbolId,
        name: String,
        visibility: Visibility,
        declaration: ByteSpan,
        #[serde(rename = "type")]
        r#type: InterfaceType,
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
        .enumerate()
        .map(|(index, export)| TypedDecl::Value {
            symbol: SymbolId(index as u32),
            name: export.name,
            visibility: export.visibility,
            declaration: export.declaration,
            r#type: export.scheme.type_ref,
        })
        .collect();

    TypedModule {
        schema: interface.schema,
        source: interface.source,
        module: interface.module,
        declarations,
    }
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
    fn types_basic_public_let_interface() {
        let interface =
            parse_module_interface("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let typed = type_module_interface(interface);

        assert_eq!(typed.schema, 1);
        assert_eq!(typed.module, "artifact/basic");
        assert_eq!(typed.source, "main.ssrg");
        assert_eq!(
            typed.declarations,
            vec![TypedDecl::Value {
                symbol: SymbolId(0),
                name: "answer".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 0, end: 24 },
                r#type: InterfaceType::Named {
                    name: "Int".to_owned(),
                    arguments: Vec::new(),
                },
            }]
        );
    }

    #[test]
    fn types_effect_do_interface() {
        let interface = parse_module_interface(
            "artifact/effect-do/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n",
        );
        let typed = type_module_interface(interface);

        assert_eq!(
            typed.declarations,
            vec![TypedDecl::Value {
                symbol: SymbolId(0),
                name: "main".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 0, end: 68 },
                r#type: InterfaceType::Named {
                    name: "Unit".to_owned(),
                    arguments: Vec::new(),
                },
            }]
        );
    }
}
