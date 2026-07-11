use crate::{TypedConstraint, TypedDecl, TypedExpr, TypedModule, TypedScheme, TypedType};
use seseragi_syntax::{parse_module_interface, ModuleInterface};

mod adt;
#[cfg(test)]
mod adt_tests;
mod analysis;
mod effect;
mod effect_analysis;
mod effect_body;
mod function_body;
mod functions;
mod interface;
#[cfg(test)]
mod match_tests;
mod pure_issues;
mod resolution;
mod semantic_types;
mod surface;
mod surface_expr;
#[cfg(test)]
mod tuple_tests;
mod type_ref;

pub(crate) use analysis::{analyze_pure_function, PureFunctionAnalysis};
pub(crate) use effect_analysis::{analyze_effect_function, EffectFunctionIssue};
pub(crate) use function_body::FunctionBodyIssue;
use interface::typed_interface_from_modules;
pub(crate) use pure_issues::{ConditionalIssue, MatchIssue, PureCallIssue};
pub(crate) use resolution::TypedResolution;
use surface::typed_decl_from_surface;
pub(crate) use surface_expr::{analyze_resolved_expression, PureExpressionContext};
use type_ref::typed_type_from_interface_type;
pub(crate) use type_ref::{
    inferred_type_from_expr, typed_type_contains_hole, typed_type_from_type_ref,
};

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
    let resolved = crate::resolve_module(source_name, source);
    let resolution = TypedResolution::new(&resolved);
    let declarations = resolved
        .declarations
        .clone()
        .into_iter()
        .filter_map(|declaration| typed_decl_from_surface(declaration, &resolution))
        .collect();

    TypedModule {
        schema: 1,
        stage: "typed-hir".to_owned(),
        source: resolved.source,
        module: resolved.module,
        declarations,
    }
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
    use crate::{unit_type, TypedDoStatement, TypedEffect, TypedParameter, TypedRecordField};
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
                    effect: TypedEffect {
                        environment: TypedType::Record {
                            closed: true,
                            fields: vec![TypedRecordField {
                                name: "console".to_owned(),
                                optional: false,
                                type_ref: TypedType::Named {
                                    name: "Console".to_owned(),
                                    arguments: Vec::new()
                                }
                            }]
                        },
                        failure: TypedType::Named {
                            name: "ConsoleError".to_owned(),
                            arguments: Vec::new()
                        },
                        success: unit_type()
                    },
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
                    effect: TypedEffect {
                        environment: TypedType::Record {
                            closed: true,
                            fields: vec![TypedRecordField {
                                name: "console".to_owned(),
                                optional: false,
                                type_ref: TypedType::Named {
                                    name: "Console".to_owned(),
                                    arguments: Vec::new()
                                }
                            }]
                        },
                        failure: TypedType::Named {
                            name: "ConsoleError".to_owned(),
                            arguments: Vec::new()
                        },
                        success: unit_type()
                    },
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
    fn types_compact_read_line_with_stdin_and_maybe_string_contract() {
        let typed = type_module(
            "artifact/effect-compact-read-line/main.ssrg",
            "pub effect fn nextLine = readLine ()\n",
        );

        let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        assert_eq!(
            effect.environment,
            TypedType::Record {
                closed: true,
                fields: vec![TypedRecordField {
                    name: "stdin".to_owned(),
                    optional: false,
                    type_ref: TypedType::Named {
                        name: "Stdin".to_owned(),
                        arguments: Vec::new(),
                    },
                }],
            }
        );
        assert_eq!(
            effect.failure,
            TypedType::Named {
                name: "StdinError".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert_eq!(
            effect.success,
            TypedType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![TypedType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );
        assert!(matches!(
            body,
            TypedExpr::EffectCall { operation, arguments, .. }
                if operation == "std/prelude::readLine" && arguments.is_empty()
        ));
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
        let TypedExpr::DoBlock {
            statements, result, ..
        } = body
        else {
            panic!("expected do block body");
        };
        assert!(statements.is_empty());
        assert!(matches!(
            result.as_ref(),
            TypedExpr::EffectCall { operation, .. } if operation == "std/prelude::println"
        ));
    }

    #[test]
    fn compact_do_block_keeps_multiple_println_statements() {
        let typed = type_module(
            "artifact/effect-compact-do-multiple/main.ssrg",
            "effect fn greet =\n  do {\n    println \"one\"\n    println \"two\"\n  }\n",
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
        let TypedExpr::DoBlock {
            statements, result, ..
        } = body
        else {
            panic!("expected do block body");
        };
        assert_eq!(statements.len(), 1);
        assert!(matches!(
            result.as_ref(),
            TypedExpr::EffectCall { operation, .. } if operation == "std/prelude::println"
        ));
    }

    #[test]
    fn types_effect_arguments_from_resolved_parameters() {
        let typed = type_module(
            "artifact/effect-parameter/main.ssrg",
            "effect fn greet name: String = println name\n",
        );

        let TypedDecl::EffectFn { body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        assert!(matches!(
            body,
            TypedExpr::EffectCall {
                operation,
                arguments,
                ..
            } if operation == "std/prelude::println"
                && matches!(arguments.as_slice(), [TypedExpr::Variable { name, type_ref, .. }]
                    if name == "name" && type_ref == &TypedType::Named {
                        name: "String".to_owned(),
                        arguments: Vec::new(),
                    })
        ));
    }

    #[test]
    fn resolves_do_bindings_in_later_effect_arguments() {
        let typed = type_module(
            "artifact/effect-resolved-bind/main.ssrg",
            "effect fn copy =\n  do {\n    line <- readLine ()\n    succeed line\n  }\n",
        );

        let TypedDecl::EffectFn { body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        let TypedExpr::DoBlock {
            statements, result, ..
        } = body
        else {
            panic!("expected do block");
        };
        assert!(matches!(
            statements.as_slice(),
            [TypedDoStatement::Bind {
                name,
                type_ref: TypedType::Named { name: type_name, arguments },
                ..
            }] if name == "line" && type_name == "Maybe"
                && matches!(arguments.as_slice(), [TypedType::Named { name, arguments }]
                    if name == "String" && arguments.is_empty())
        ));
        assert!(matches!(
            result.as_ref(),
            TypedExpr::EffectCall {
                operation,
                arguments,
                ..
            } if operation == "std/effect::succeed"
                && matches!(arguments.as_slice(), [TypedExpr::Variable { name, type_ref, .. }]
                    if name == "line" && matches!(type_ref,
                        TypedType::Named { name, arguments }
                            if name == "Maybe" && arguments.len() == 1))
        ));
    }

    #[test]
    fn types_grouped_effect_expression_from_surface_ast() {
        let typed = type_module(
            "artifact/effect-grouped/main.ssrg",
            "effect fn greet = (println \"hello\")\n",
        );

        let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        assert!(matches!(
            body,
            TypedExpr::EffectCall { operation, .. }
                if operation == "std/prelude::println"
        ));
        assert!(matches!(
            &effect.environment,
            TypedType::Record { fields, .. }
                if matches!(fields.as_slice(), [TypedRecordField { name, .. }]
                    if name == "console")
        ));
    }

    #[test]
    fn types_pure_let_and_resolves_it_in_final_effect() {
        let typed = type_module(
            "artifact/effect-do-pure-let/main.ssrg",
            "effect fn greet =\n  do {\n    let message = \"hello\"\n    println message\n  }\n",
        );

        let TypedDecl::EffectFn { body, .. } = &typed.declarations[0] else {
            panic!("expected effect function declaration");
        };
        let TypedExpr::DoBlock {
            statements, result, ..
        } = body
        else {
            panic!("expected do block");
        };
        assert!(matches!(
            statements.as_slice(),
            [TypedDoStatement::PureLet {
                name,
                type_ref: TypedType::Named { name: type_name, arguments },
                value: TypedExpr::String { value, .. },
                ..
            }] if name == "message" && type_name == "String" && arguments.is_empty()
                && value == "hello"
        ));
        assert!(matches!(
            result.as_ref(),
            TypedExpr::EffectCall { arguments, .. }
                if matches!(arguments.as_slice(), [TypedExpr::Variable { name, type_ref, .. }]
                    if name == "message" && matches!(type_ref,
                        TypedType::Named { name, arguments }
                            if name == "String" && arguments.is_empty()))
        ));
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
        let TypedExpr::DoBlock {
            statements, result, ..
        } = body
        else {
            panic!("expected do block body");
        };
        assert_eq!(statements.len(), 1);
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
            result.as_ref(),
            TypedExpr::EffectCall { operation, .. } if operation == "std/prelude::println"
        ));
    }

    #[test]
    fn types_succeed_as_final_do_result() {
        let typed = type_module(
            "artifact/effect-do/main.ssrg",
            "pub effect fn main -> Unit =\n  do { succeed () }\n",
        );

        let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[0] else {
            panic!("expected effect function");
        };
        assert_eq!(
            effect.environment,
            TypedType::Record {
                closed: true,
                fields: Vec::new()
            }
        );
        assert_eq!(
            effect.failure,
            TypedType::Named {
                name: "Never".to_owned(),
                arguments: Vec::new()
            }
        );
        let TypedExpr::DoBlock {
            statements, result, ..
        } = body
        else {
            panic!("expected do block");
        };
        assert!(statements.is_empty());
        assert!(matches!(
            result.as_ref(),
            TypedExpr::EffectCall { operation, .. } if operation == "std/effect::succeed"
        ));
    }

    #[test]
    fn infers_succeed_success_from_its_argument() {
        let typed = type_module(
            "artifact/effect-succeed-value/main.ssrg",
            "pub effect fn ready = succeed \"ready\"\n",
        );

        let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[0] else {
            panic!("expected effect function");
        };
        let string_type = TypedType::Named {
            name: "String".to_owned(),
            arguments: Vec::new(),
        };
        assert_eq!(effect.success, string_type);
        assert!(matches!(
            body,
            TypedExpr::EffectCall {
                operation,
                effect: call_effect,
                arguments,
                ..
            } if operation == "std/effect::succeed"
                && call_effect.success == string_type
                && matches!(arguments.as_slice(), [TypedExpr::String { value, .. }]
                    if value == "ready")
        ));
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

    #[test]
    fn types_saturated_multi_parameter_top_level_pure_call() {
        let typed = type_module(
            "artifact/pure-calls/main.ssrg",
            "pub fn add x: Int -> y: Int -> Int = x + y\npub fn total unit: Unit -> Int = add 1 2\n",
        );

        let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
            panic!("expected function declaration");
        };
        let TypedExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } = body
        else {
            panic!("expected saturated direct call");
        };

        assert_eq!(callee, "artifact/pure-calls::add");
        assert_eq!(type_ref, &int_type());
        assert!(matches!(
            arguments.as_slice(),
            [
                TypedExpr::Integer { value: first, .. },
                TypedExpr::Integer { value: second, .. }
            ] if first == "1" && second == "2"
        ));
    }

    #[test]
    fn types_parameter_forwarded_to_top_level_pure_call() {
        let typed = type_module(
            "artifact/pure-call-parameter/main.ssrg",
            "pub fn identity value: Int -> Int = value\npub fn use value: Int -> Int = identity value\n",
        );

        let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
            panic!("expected function declaration");
        };
        let TypedExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } = body
        else {
            panic!("expected saturated direct call");
        };

        assert_eq!(callee, "artifact/pure-call-parameter::identity");
        assert_eq!(type_ref, &int_type());
        assert!(matches!(
            arguments.as_slice(),
            [TypedExpr::Variable { name, type_ref, .. }]
                if name == "value" && type_ref == &int_type()
        ));
    }

    #[test]
    fn types_partial_top_level_application_as_remaining_function() {
        let typed = type_module(
            "artifact/partial-call/main.ssrg",
            "pub fn add left: Int -> right: Int -> Int = left + right\npub fn addTo value: Int -> (Int -> Int) = add value\n",
        );

        let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
            panic!("expected function declaration");
        };
        let TypedExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } = body
        else {
            panic!("expected partial direct call");
        };

        assert_eq!(callee, "artifact/partial-call::add");
        assert_eq!(arguments.len(), 1);
        assert_eq!(
            type_ref,
            &TypedType::Function {
                parameter: Box::new(int_type()),
                result: Box::new(int_type()),
            }
        );
    }

    #[test]
    fn types_nested_surface_expression_as_a_call_argument() {
        let typed = type_module(
            "artifact/pure-expression-argument/main.ssrg",
            "pub fn identity value: Int -> Int = value\npub fn use unit: Unit -> Int = identity (if True then 1 else 2)\n",
        );

        let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
            panic!("expected function declaration");
        };
        let TypedExpr::Call {
            arguments,
            type_ref,
            ..
        } = body
        else {
            panic!("expected direct call");
        };
        assert_eq!(type_ref, &int_type());
        assert!(matches!(
            arguments.as_slice(),
            [TypedExpr::If {
                type_ref: TypedType::Named { name, arguments },
                ..
            }] if name == "Int" && arguments.is_empty()
        ));
    }

    fn int_type() -> TypedType {
        TypedType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        }
    }
}
