use crate::{ResolvedInterface, ResolvedInterfaceDecl, SymbolId};
use seseragi_syntax::{InterfaceExport, InterfaceInstance, InterfaceOperator, ModuleInterface};

mod body;

pub use body::resolve_linked_module;
pub use body::resolve_module;
pub(crate) use body::resolve_module_from_interface;

pub fn resolve_module_interface(interface: ModuleInterface) -> ResolvedInterface {
    let ModuleInterface {
        schema,
        source,
        module,
        exports,
        operators,
        instances,
        ..
    } = interface;

    let mut next_symbol = 0u32;
    let mut declarations = Vec::new();
    for export in exports {
        let symbol = SymbolId(next_symbol);
        next_symbol += 1;
        if let Some(declaration) = resolved_export(symbol, export, &operators) {
            declarations.push(declaration);
        }
    }
    for instance in instances {
        let symbol = SymbolId(next_symbol);
        next_symbol += 1;
        declarations.push(resolved_instance(symbol, instance));
    }

    ResolvedInterface {
        schema,
        source,
        module,
        declarations,
    }
}

fn resolved_export(
    symbol: SymbolId,
    export: InterfaceExport,
    operators: &[InterfaceOperator],
) -> Option<ResolvedInterfaceDecl> {
    match export.namespace.as_str() {
        "value" if export.declaration_kind.as_deref() == Some("constructor") => {
            Some(ResolvedInterfaceDecl::Constructor {
                symbol,
                name: export.name,
                owner: export.constructor_of?,
                visibility: export.visibility,
                scheme: export.scheme,
                declaration: export.declaration,
            })
        }
        "value" => Some(ResolvedInterfaceDecl::Value {
            symbol,
            name: export.name,
            visibility: export.visibility,
            declaration: export.declaration,
        }),
        "type" => Some(ResolvedInterfaceDecl::Type {
            symbol,
            name: export.name,
            visibility: export.visibility,
            declaration_kind: export.declaration_kind,
            declaration: export.declaration,
            representation: export.representation,
        }),
        "operator" => {
            let operator = operators
                .iter()
                .find(|operator| operator.symbol == export.symbol);
            Some(ResolvedInterfaceDecl::Operator {
                symbol,
                name: export.name,
                visibility: export.visibility,
                fixity: operator.map(|operator| operator.fixity.clone()),
                precedence: operator.map(|operator| operator.precedence),
                declaration: export.declaration,
            })
        }
        _ => None,
    }
}

fn resolved_instance(symbol: SymbolId, instance: InterfaceInstance) -> ResolvedInterfaceDecl {
    ResolvedInterfaceDecl::Instance {
        symbol,
        trait_name: instance.trait_name,
        head: instance.head,
        declaration: instance.origin,
    }
}

#[cfg(test)]
mod linked_tests;
#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{
        parse_module_interface, ByteSpan, InterfaceScheme, InterfaceType, Visibility,
    };

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
            vec![ResolvedInterfaceDecl::Value {
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
            ResolvedInterfaceDecl::Value {
                symbol: SymbolId(0),
                name: "first".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 0, end: 22 },
            }
        );
        assert_eq!(
            resolved.declarations[1],
            ResolvedInterfaceDecl::Value {
                symbol: SymbolId(1),
                name: "second".to_owned(),
                visibility: Visibility::Public,
                declaration: ByteSpan { start: 23, end: 46 },
            }
        );
    }

    #[test]
    fn resolves_rich_interface_metadata() {
        let interface = parse_module_interface(
            "artifact/rich/main.ssrg",
            include_str!("../../../examples/spec/artifacts/interface-schema-1/rich/main.ssrg"),
        );
        let resolved = resolve_module_interface(interface);

        assert_eq!(
            resolved.declarations,
            vec![
                ResolvedInterfaceDecl::Type {
                    symbol: SymbolId(0),
                    name: "Score".to_owned(),
                    visibility: Visibility::Public,
                    declaration_kind: Some("newtype".to_owned()),
                    declaration: ByteSpan { start: 34, end: 57 },
                    representation: Some(InterfaceType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    }),
                },
                ResolvedInterfaceDecl::Constructor {
                    symbol: SymbolId(1),
                    name: "Score".to_owned(),
                    owner: "artifact/rich::Score".to_owned(),
                    visibility: Visibility::Public,
                    scheme: InterfaceScheme {
                        type_parameters: Vec::new(),
                        constraints: Vec::new(),
                        type_ref: InterfaceType::Function {
                            parameter: Box::new(InterfaceType::Named {
                                name: "Int".to_owned(),
                                arguments: Vec::new(),
                            }),
                            result: Box::new(InterfaceType::Named {
                                name: "Score".to_owned(),
                                arguments: Vec::new(),
                            }),
                        },
                    },
                    declaration: ByteSpan { start: 46, end: 51 },
                },
                ResolvedInterfaceDecl::Operator {
                    symbol: SymbolId(2),
                    name: "<+>".to_owned(),
                    visibility: Visibility::Public,
                    fixity: Some("infixl".to_owned()),
                    precedence: Some(4),
                    declaration: ByteSpan {
                        start: 59,
                        end: 141
                    },
                },
                ResolvedInterfaceDecl::Instance {
                    symbol: SymbolId(3),
                    trait_name: "Show".to_owned(),
                    head: InterfaceType::Apply {
                        constructor: "Show".to_owned(),
                        arguments: vec![InterfaceType::Named {
                            name: "Score".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                    declaration: ByteSpan {
                        start: 143,
                        end: 210
                    },
                },
            ]
        );
    }

    #[test]
    fn resolves_public_adt_constructors_with_owner_and_scheme() {
        let interface = parse_module_interface(
            "artifact/public-type/main.ssrg",
            "pub type Maybe<A> =\n  | Nothing\n  | Just A\n",
        );
        let resolved = resolve_module_interface(interface);

        assert_eq!(resolved.declarations.len(), 3);
        assert!(matches!(
            &resolved.declarations[0],
            ResolvedInterfaceDecl::Type {
                symbol: SymbolId(0),
                name,
                declaration_kind: Some(kind),
                ..
            } if name == "Maybe" && kind == "type"
        ));
        assert_eq!(
            resolved.declarations[1],
            ResolvedInterfaceDecl::Constructor {
                symbol: SymbolId(1),
                name: "Nothing".to_owned(),
                owner: "artifact/public-type::Maybe".to_owned(),
                visibility: Visibility::Public,
                scheme: InterfaceScheme {
                    type_parameters: vec!["A".to_owned()],
                    constraints: Vec::new(),
                    type_ref: InterfaceType::Named {
                        name: "Maybe".to_owned(),
                        arguments: vec![InterfaceType::Named {
                            name: "A".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                },
                declaration: ByteSpan { start: 24, end: 31 },
            }
        );
        assert!(matches!(
            &resolved.declarations[2],
            ResolvedInterfaceDecl::Constructor {
                symbol: SymbolId(2),
                name,
                owner,
                scheme: InterfaceScheme {
                    type_ref: InterfaceType::Function { .. },
                    ..
                },
                declaration: ByteSpan { start: 36, end: 40 },
                ..
            } if name == "Just" && owner == "artifact/public-type::Maybe"
        ));
    }
}
