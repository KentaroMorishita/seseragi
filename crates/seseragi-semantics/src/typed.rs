use crate::{TypedConstraint, TypedDecl, TypedExpr, TypedModule, TypedScheme, TypedType};
use seseragi_syntax::{lex, parse_module_interface, parse_surface_ast, ModuleInterface};
use std::collections::BTreeMap;

mod effect;
mod expr;
mod interface;
mod surface;
mod type_ref;

use interface::typed_interface_from_modules;
use surface::typed_decl_from_surface;
use type_ref::typed_type_from_interface_type;

pub fn type_module_interface(interface: ModuleInterface) -> TypedModule {
    let declarations = interface
        .exports
        .into_iter()
        .filter(|export| export.namespace == "value")
        .filter_map(|export| {
            let type_ref = typed_type_from_interface_type(export.scheme.type_ref)?;
            Some(TypedDecl::Let {
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
                    type_ref,
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
    let top_level_values = top_level_value_types(&surface.declarations, &tokens);
    let declarations = surface
        .declarations
        .into_iter()
        .filter_map(|declaration| {
            typed_decl_from_surface(&interface.module, declaration, &tokens, &top_level_values)
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

fn top_level_value_types(
    declarations: &[seseragi_syntax::SurfaceDecl],
    tokens: &[seseragi_syntax::Token],
) -> BTreeMap<String, TypedType> {
    declarations
        .iter()
        .filter_map(|declaration| {
            let seseragi_syntax::SurfaceDecl::Let {
                name,
                type_ref,
                span,
                ..
            } = declaration
            else {
                return None;
            };
            let type_ref = type_ref
                .as_ref()
                .map(type_ref::typed_type_from_type_ref)
                .or_else(|| {
                    expr::find_value_token(tokens, *span)
                        .map(expr::typed_expr_from_value_token)
                        .map(|value| type_ref::inferred_type_from_expr(&value))
                })?;
            Some((name.clone(), type_ref))
        })
        .collect()
}

pub fn type_module_public_interface(
    source_name: impl Into<String>,
    source: &str,
) -> crate::TypedModuleInterface {
    let source_name = source_name.into();
    let shallow = parse_module_interface(source_name.clone(), source);
    let diagnostics = crate::diagnostics::semantic_diagnostics(source_name.clone(), source);
    if !diagnostics.diagnostics.is_empty() {
        return crate::TypedModuleInterface {
            schema: shallow.schema,
            stage: "typed-interface".to_owned(),
            module: shallow.module,
            source: shallow.source,
            dependencies: shallow.dependencies,
            exports: Vec::new(),
            operators: Vec::new(),
            instances: Vec::new(),
        };
    }
    let typed = type_module(source_name, source);
    typed_interface_from_modules(shallow, typed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{unit_type, TypedEffect, TypedParameter, TypedRecordField};
    use seseragi_syntax::ByteSpan;
    use seseragi_syntax::InterfaceType;
    use seseragi_syntax::Visibility;

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
                inferred_contract: false,
                parameters: vec![TypedParameter::ImplicitUnit {
                    type_ref: unit_type(),
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: vec![TypedRecordField {
                            name: "console".to_owned(),
                            optional: false,
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
    fn types_compact_effect_fn_infers_println_contract() {
        let typed = type_module(
            "artifact/effect-compact-greet/main.ssrg",
            "effect fn greet name: String =\n  println \"hello\"\n",
        );

        assert_eq!(
            typed.declarations,
            vec![TypedDecl::EffectFn {
                symbol: "artifact/effect-compact-greet::greet".to_owned(),
                visibility: Visibility::Private,
                origin: ByteSpan { start: 0, end: 48 },
                inferred_contract: true,
                parameters: vec![TypedParameter::Named {
                    name: "name".to_owned(),
                    type_ref: TypedType::Named {
                        name: "String".to_owned(),
                        arguments: Vec::new(),
                    },
                    origin: ByteSpan { start: 16, end: 20 },
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: vec![TypedRecordField {
                            name: "console".to_owned(),
                            optional: false,
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
                        origin: ByteSpan { start: 41, end: 48 },
                    }],
                    origin: ByteSpan { start: 33, end: 48 },
                },
            }]
        );
    }

    #[test]
    fn typed_interface_exports_compact_effect_fn_contract() {
        let interface = type_module_public_interface(
            "artifact/effect-compact-public/main.ssrg",
            "pub effect fn greet name: String =\n  println \"hello\"\n",
        );

        let export = &interface.exports[0];
        assert_eq!(interface.stage, "typed-interface");
        assert_eq!(export.symbol, "artifact/effect-compact-public::greet");
        assert_eq!(export.declaration_kind, Some("effect-function".to_owned()));
        assert_eq!(
            export.scheme.type_ref,
            InterfaceType::Function {
                parameter: Box::new(InterfaceType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }),
                result: Box::new(InterfaceType::Named {
                    name: "Effect".to_owned(),
                    arguments: vec![
                        InterfaceType::Record {
                            closed: true,
                            fields: vec![seseragi_syntax::InterfaceRecordField {
                                name: "console".to_owned(),
                                optional: false,
                                type_ref: InterfaceType::Named {
                                    name: "Console".to_owned(),
                                    arguments: Vec::new(),
                                },
                            }],
                        },
                        InterfaceType::Named {
                            name: "ConsoleError".to_owned(),
                            arguments: Vec::new(),
                        },
                        InterfaceType::Named {
                            name: "Unit".to_owned(),
                            arguments: Vec::new(),
                        },
                    ],
                }),
            }
        );
    }

    #[test]
    fn typed_interface_omits_exports_when_semantic_diagnostics_exist() {
        let interface = type_module_public_interface(
            "artifact/effect-compact-not-effect/main.ssrg",
            "pub effect fn greet name: String = name\n",
        );

        assert!(interface.exports.is_empty());
        assert!(interface.operators.is_empty());
        assert!(interface.instances.is_empty());
    }

    #[test]
    fn compact_do_block_infers_println_statement_contract() {
        let typed = type_module(
            "artifact/effect-compact-do/main.ssrg",
            "effect fn greet =\n  do { println \"hello\" }\n",
        );

        let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        assert_eq!(
            effect.environment,
            TypedType::Record {
                closed: true,
                fields: vec![TypedRecordField {
                    name: "console".to_owned(),
                    optional: false,
                    type_ref: TypedType::Named {
                        name: "Console".to_owned(),
                        arguments: Vec::new(),
                    },
                }],
            }
        );
        assert_eq!(
            effect.failure,
            TypedType::Named {
                name: "ConsoleError".to_owned(),
                arguments: Vec::new(),
            }
        );
        let TypedExpr::DoBlock { statements, .. } = body else {
            panic!("expected do block body");
        };
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn compact_do_block_keeps_multiple_println_statements() {
        let typed = type_module(
            "artifact/effect-compact-do-multiple/main.ssrg",
            "effect fn greet =\n  do { println \"one\" println \"two\" }\n",
        );

        let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        assert_eq!(
            effect.environment,
            TypedType::Record {
                closed: true,
                fields: vec![TypedRecordField {
                    name: "console".to_owned(),
                    optional: false,
                    type_ref: TypedType::Named {
                        name: "Console".to_owned(),
                        arguments: Vec::new(),
                    },
                }],
            }
        );
        assert_eq!(
            effect.failure,
            TypedType::Named {
                name: "ConsoleError".to_owned(),
                arguments: Vec::new(),
            }
        );
        let TypedExpr::DoBlock { statements, .. } = body else {
            panic!("expected do block body");
        };
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn types_do_bind_statement_separately_from_effect_statement() {
        let typed = type_module(
            "artifact/effect-do-bind/main.ssrg",
            "effect fn record =\n  do {\n    ignored <- print \"hello\"\n    println \"done\"\n  }\n",
        );

        let TypedDecl::EffectFn { body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        let TypedExpr::DoBlock { statements, .. } = body else {
            panic!("expected do block body");
        };
        assert!(matches!(
            &statements[0],
            crate::TypedDoStatement::Bind {
                name,
                type_ref: TypedType::Named { name: type_name, arguments },
                value: TypedExpr::EffectCall { operation, .. },
                ..
            } if name == "ignored" && type_name == "Unit" && arguments.is_empty() && operation == "std/prelude::print"
        ));
        assert!(matches!(
            &statements[1],
            crate::TypedDoStatement::Effect {
                value: TypedExpr::EffectCall { operation, .. }
            } if operation == "std/prelude::println"
        ));
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
                inferred_contract: false,
                parameters: vec![TypedParameter::ImplicitUnit {
                    type_ref: unit_type(),
                }],
                effect: TypedEffect {
                    environment: TypedType::Record {
                        closed: true,
                        fields: vec![TypedRecordField {
                            name: "console".to_owned(),
                            optional: false,
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

    #[test]
    fn type_module_interface_ignores_non_value_exports() {
        let interface = seseragi_syntax::parse_module_interface(
            "artifact/rich/main.ssrg",
            include_str!("../../../examples/spec/artifacts/interface-schema-1/rich/main.ssrg"),
        );
        let typed = type_module_interface(interface);

        assert!(typed.declarations.is_empty());
    }

    #[test]
    fn types_unknown_function_identifier_as_hole_variable() {
        let typed = type_module(
            "artifact/unknown-fn-body/main.ssrg",
            "pub fn useMissing value: Int -> Int = missing\n",
        );

        let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
            panic!("expected function declaration");
        };
        assert_eq!(
            body,
            &TypedExpr::Variable {
                name: "missing".to_owned(),
                type_ref: TypedType::Hole,
                origin: ByteSpan { start: 38, end: 45 },
            }
        );
    }

    #[test]
    fn types_pure_function_reference_to_top_level_binding() {
        let typed = type_module(
            "artifact/top-level-binding/main.ssrg",
            "pub let answer: Int = 42\npub fn answerValue unit: Unit -> Int = answer\n",
        );

        let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
            panic!("expected function declaration");
        };
        assert!(matches!(
            body,
            TypedExpr::Variable {
                name,
                type_ref: TypedType::Named { name: type_name, arguments },
                ..
            } if name == "answer" && type_name == "Int" && arguments.is_empty()
        ));
    }
}
