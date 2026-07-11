mod core;
mod effect_ops;
mod emit;
mod int_ops;
mod span;
mod typescript;

pub use core::{
    lower_typed_module, CoreAdt, CoreAdtVariant, CoreBinding, CoreDecisionBinding,
    CoreDecisionBranch, CoreDecisionProjection, CoreDecisionTest, CoreExpr, CoreFunction,
    CoreModule, CoreParameter, CoreRecordField, CoreStatement, CoreType,
};
pub use emit::{
    emit_typescript_module, GeneratedBundle, GeneratedModule, GeneratedOutputs, GeneratedRuntime,
    SourceMap,
};
pub(crate) use span::source_span;
pub use span::SourceSpan;
pub use typescript::{
    lower_core_module_to_typescript_ir, TypeScriptAdt, TypeScriptAdtVariant, TypeScriptBinding,
    TypeScriptDecisionBinding, TypeScriptDecisionBranch, TypeScriptDecisionProjection,
    TypeScriptDecisionTest, TypeScriptExpr, TypeScriptFunction, TypeScriptImport, TypeScriptModule,
    TypeScriptParameter, TypeScriptStatement, TypeScriptType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_semantics::type_module;
    use seseragi_syntax::Visibility;

    #[test]
    fn lowers_public_let_to_core_binding() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);

        assert_eq!(core.stage, "core-ir");
        assert_eq!(core.module, "artifact/basic");
        assert_eq!(core.bindings.len(), 1);
        assert!(matches!(core.bindings[0].value, CoreExpr::Int64 { .. }));
        assert!(core.functions.is_empty());
    }

    #[test]
    fn lowers_console_println_effect_operation() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );
        let core = lower_typed_module(typed);

        assert!(core.bindings.is_empty());
        assert_eq!(core.functions.len(), 1);
        assert_eq!(core.functions[0].parameters[0].id, "unit");
        assert!(matches!(
            core.functions[0].body,
            CoreExpr::EffectOperation { .. }
        ));
    }

    #[test]
    fn lowers_core_binding_to_typescript_const() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.stage, "typescript-ir");
        assert_eq!(typescript.runtime_requirements, vec!["core.int64"]);
        assert_eq!(typescript.bindings.len(), 1);
        assert!(typescript.functions.is_empty());
    }

    #[test]
    fn lowers_adt_constructors_to_tagged_typescript_values() {
        let source = "\
pub type Hand =
  | Rock
  | Paper
  | Scissors

pub type Label =
  | Missing
  | Present String

pub let opening: Hand = Rock

pub fn wrap value: String -> Label =
  Present value
";
        let typed = type_module("artifact/adt-constructors/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.adts.len(), 2);
        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.adt", "core.string"]
        );
        assert_eq!(typescript.adts[0].name, "Hand");
        assert_eq!(
            typescript.adts[1].variants[1].payload,
            Some(TypeScriptType::String)
        );
        assert!(typescript.imports.is_empty());

        let bundle = emit_typescript_module(typescript, source);
        assert_eq!(
            bundle.metadata.exports,
            vec!["Rock", "Paper", "Scissors", "Missing", "Present", "opening", "wrap"]
        );
        assert_eq!(
            bundle.typescript,
            "\
export type Hand =
  | { readonly tag: \"Rock\" }
  | { readonly tag: \"Paper\" }
  | { readonly tag: \"Scissors\" };
export const Rock: Hand = { tag: \"Rock\" } as const;
export const Paper: Hand = { tag: \"Paper\" } as const;
export const Scissors: Hand = { tag: \"Scissors\" } as const;
export type Label =
  | { readonly tag: \"Missing\" }
  | { readonly tag: \"Present\"; readonly value: string };
export const Missing: Label = { tag: \"Missing\" } as const;
export const Present = (value: string): Label => ({ tag: \"Present\", value } as const);
export const opening: Hand = Rock;
export const wrap = (value: string) => Present(value)
"
        );
        assert_eq!(
            bundle.source_map.names,
            vec![
                "Hand", "Rock", "Paper", "Scissors", "Label", "Missing", "Present", "opening",
                "wrap", "Present"
            ]
        );
        assert_eq!(
            bundle.source_map.mappings,
            "AAAAA;;;;AACIC;AACAC;AACAC;AAEJC;;;AACIC;AACAC;AAEJC;AAEAC"
        );
    }

    #[test]
    fn keeps_opaque_and_private_adt_constructors_out_of_runtime_exports() {
        let source = "\
pub opaque type Token =
  | Token String

type Internal =
  | Hidden
";
        let typed = type_module("artifact/opaque-adts/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains("export type Token ="));
        assert!(bundle
            .typescript
            .contains("const Token = (value: string): Token =>"));
        assert!(!bundle.typescript.contains("export const Token"));
        assert!(bundle.typescript.contains("type Internal ="));
        assert!(bundle
            .typescript
            .contains("const Hidden: Internal = { tag: \"Hidden\" } as const;"));
        assert!(bundle.metadata.exports.is_empty());
    }

    #[test]
    fn lowers_core_effect_to_typescript_imported_call() {
        let typed = type_module(
            "artifact/effect-main/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n",
        );
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.unit", "effect.console.println", "core.string"]
        );
        assert_eq!(typescript.imports[0].local, "_ssrg_console_println");
        assert_eq!(typescript.functions.len(), 1);
    }

    #[test]
    fn lowers_string_binding_to_typescript_string_const() {
        let source = "pub let greeting: String = \"hello\"\n";
        let typed = type_module("artifact/string-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.string"]);
        assert_eq!(
            bundle.typescript,
            "export const greeting: string = \"hello\";\n"
        );
    }

    #[test]
    fn lowers_boolean_binding_to_typescript_boolean_const() {
        let source = "pub let enabled: Bool = True\n";
        let typed = type_module("artifact/bool-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.bool"]);
        assert_eq!(bundle.typescript, "export const enabled: boolean = true;\n");
    }

    #[test]
    fn lowers_identity_function_to_typescript_arrow_function() {
        let source = "pub fn identity value: Int -> Int = value\n";
        let typed = type_module("artifact/identity-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int64"]);
        assert_eq!(bundle.metadata.exports, vec!["identity"]);
        assert_eq!(
            bundle.typescript,
            "export const identity = (value: bigint) => value\n"
        );
    }

    #[test]
    fn lowers_multi_parameter_function_to_typescript_arrow_function() {
        let source = "pub fn first left: Int -> right: Int -> Int = left\n";
        let typed = type_module("artifact/first-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int64"]);
        assert_eq!(
            bundle.typescript,
            "export const first = (left: bigint) => (right: bigint) => left\n"
        );
    }

    #[test]
    fn lowers_integer_add_function_to_checked_runtime_call() {
        let source = "pub fn add x: Int -> y: Int -> Int = x + y\n";
        let typed = type_module("artifact/add-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.metadata.runtime.requirements,
            vec!["core.int64", "core.int64.add"]
        );
        assert_eq!(
            bundle.typescript,
            "import { add as _ssrg_int64_add } from \"@seseragi/runtime/int64\"\n\nexport const add = (x: bigint) => (y: bigint) => _ssrg_int64_add(x, y)\n"
        );
    }

    #[test]
    fn freshens_runtime_import_that_collides_with_user_function() {
        let source = "pub fn _ssrg_int64_add value: Int -> Int = value\npub fn add x: Int -> y: Int -> Int = x + y\n";
        let typed = type_module("artifact/runtime-name-collision/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("import { add as _ssrg_int64_add_1 } from \"@seseragi/runtime/int64\""));
        assert!(bundle
            .typescript
            .contains("export const _ssrg_int64_add = (value: bigint) => value"));
        assert!(bundle
            .typescript
            .contains("export const add = (x: bigint) => (y: bigint) => _ssrg_int64_add_1(x, y)"));
    }

    #[test]
    fn lowers_module_qualified_pure_call_without_runtime_helper_import() {
        let source = "pub fn invoke value: Int -> Int = default value\n";
        let origin = SourceSpan {
            source: "main.ssrg".to_owned(),
            start: 0,
            end: source.len(),
        };
        let int_type = CoreType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        };
        let core = CoreModule {
            schema: 1,
            stage: "core-ir".to_owned(),
            module: "artifact/calls".to_owned(),
            adts: Vec::new(),
            bindings: Vec::new(),
            functions: vec![CoreFunction {
                symbol: "artifact/calls::invoke".to_owned(),
                visibility: Visibility::Public,
                origin: origin.clone(),
                parameters: vec![CoreParameter {
                    id: "value".to_owned(),
                    kind: "named".to_owned(),
                    type_ref: int_type.clone(),
                }],
                body: CoreExpr::Call {
                    callee: "artifact/calls::default".to_owned(),
                    arguments: vec![CoreExpr::Variable {
                        name: "value".to_owned(),
                        type_ref: int_type.clone(),
                        origin: origin.clone(),
                    }],
                    type_ref: int_type,
                    origin,
                },
            }],
        };

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript.clone(), source);

        assert_eq!(typescript.runtime_requirements, vec!["core.int64"]);
        assert!(typescript
            .runtime_requirements
            .iter()
            .all(|requirement| !requirement.starts_with("effect.")));
        assert!(typescript.imports.is_empty());
        assert_eq!(
            bundle.typescript,
            "export const invoke = (value: bigint) => _default(value)\n"
        );
        assert_eq!(bundle.source_map.names, vec!["invoke", "_default"]);
    }

    #[test]
    fn lowers_typed_pure_function_call_without_runtime_helper_import() {
        let source = "\
pub fn identity value: Int -> Int = value
pub fn useIdentity value: Int -> Int = identity value
";
        let typed = type_module("artifact/calls/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            ..
        } = &core.functions[1].body
        else {
            panic!("expected pure call in second function body");
        };

        assert_eq!(callee, "artifact/calls::identity");
        assert_eq!(arguments.len(), 1);
        assert_eq!(
            type_ref,
            &CoreType::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
            }
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript.clone(), source);

        assert_eq!(typescript.runtime_requirements, vec!["core.int64"]);
        assert!(typescript
            .runtime_requirements
            .iter()
            .all(|requirement| !requirement.starts_with("effect.")));
        assert!(typescript.imports.is_empty());
        assert_eq!(
            bundle.typescript,
            "export const identity = (value: bigint) => value\nexport const useIdentity = (value: bigint) => identity(value)\n"
        );
        assert_eq!(
            bundle.source_map.names,
            vec!["identity", "useIdentity", "identity"]
        );
    }

    #[test]
    fn lowers_partial_application_to_curried_typescript_call() {
        let source = "pub fn add left: Int -> right: Int -> Int = left + right\npub fn addTo value: Int -> (Int -> Int) = add value\n";
        let typed = type_module("artifact/partial-call/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .metadata
            .runtime
            .requirements
            .iter()
            .all(|requirement| !requirement.starts_with("effect.")));
        assert!(bundle.typescript.contains(
            "export const add = (left: bigint) => (right: bigint) => _ssrg_int64_add(left, right)"
        ));
        assert!(bundle
            .typescript
            .contains("export const addTo = (value: bigint) => add(value)"));
    }

    #[test]
    fn deduplicates_runtime_helper_imports_across_functions() {
        let source = "\
pub effect fn first -> Unit
with Console
fails ConsoleError =
  println \"one\"

pub effect fn second -> Unit
with Console
fails ConsoleError =
  println \"two\"
";
        let typed = type_module("artifact/two-effects/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.imports.len(), 1);
        assert_eq!(typescript.imports[0].feature, "effect.console.println");
    }

    #[test]
    fn sanitizes_typescript_parameter_and_variable_names() {
        let source = "pub fn pick default: Int -> Int = default\n";
        let typed = type_module("artifact/reserved-param/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.typescript,
            "export const pick = (_default: bigint) => _default\n"
        );
    }

    #[test]
    fn lowers_succeed_final_do_result_to_cold_effect() {
        let source = "pub effect fn main -> Unit =\n  do { succeed () }\n";
        let typed = type_module("artifact/effect-do/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(
            typescript.runtime_requirements,
            vec!["core.unit", "effect.core.succeed"]
        );
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                body: TypeScriptExpr::RuntimeCall { callee, .. },
                ..
            } if callee == "_ssrg_effect_succeed"
        ));
    }

    #[test]
    fn lowers_succeed_value_with_its_concrete_success_type() {
        let source = "pub effect fn ready = succeed \"ready\"\n";
        let typed = type_module("artifact/effect-succeed-value/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation {
            success, arguments, ..
        } = &core.functions[0].body
        else {
            panic!("expected effect operation");
        };
        assert_eq!(
            success,
            &CoreType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert!(
            matches!(arguments.as_slice(), [CoreExpr::String { value, .. }] if value == "ready")
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle
            .typescript
            .contains("_ssrg_effect_succeed(\"ready\")"));
    }

    #[test]
    fn lowers_adt_failure_to_a_cold_runtime_effect() {
        let source = "pub type AppError = | Invalid\npub effect fn reject = fail Invalid\n";
        let typed = type_module("artifact/effect-fail-adt/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation {
            failure, success, ..
        } = &core.functions[0].body
        else {
            panic!("expected effect operation");
        };
        assert_eq!(
            failure,
            &CoreType::Named {
                name: "AppError".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert_eq!(
            success,
            &CoreType::Named {
                name: "Never".to_owned(),
                arguments: Vec::new(),
            }
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == "effect.core.fail"));
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains("_ssrg_effect_fail(Invalid)"));
        assert!(!bundle.typescript.contains("throw"));
        assert!(!bundle.typescript.contains("await"));
    }

    #[test]
    fn lowers_single_sync_effect_as_do_result() {
        let source =
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { println \"hello\" }\n";
        let typed = type_module("artifact/effect-do-println/main.ssrg", source);
        let core = lower_typed_module(typed);
        assert!(matches!(
            &core.functions[0].body,
            CoreExpr::EffectOperation { operation, success, .. }
                if operation == "console.println" && success == &CoreType::Named { name: "Unit".to_owned(), arguments: Vec::new() }
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                body: TypeScriptExpr::RuntimeCall { .. },
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_console_println(\"hello\")"
        ));
    }

    #[test]
    fn lowers_single_async_operation_as_cold_value_producing_effect() {
        let source = "pub effect fn main =\n  do { readLine () }\n";
        let typed = type_module("artifact/effect-do-read-line/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::EffectOperation { success, .. } = &core.functions[0].body else {
            panic!("expected readLine do result");
        };
        assert_eq!(
            success,
            &CoreType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                is_async: false,
                body: TypeScriptExpr::RuntimeCall { .. },
                ..
            }
        ));
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("export const main = (_unit: undefined) => _ssrg_stdin_readLine()"));
    }

    #[test]
    fn lowers_multiple_effect_do_statements_to_typescript_sequence() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    println \"one\"\n    println \"two\"\n  }\n";
        let typed = type_module("artifact/effect-do-multiple/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_console_println(\"one\"), () => _ssrg_console_println(\"two\"))"
        ));
    }

    #[test]
    fn lowers_pure_do_let_without_flat_map() {
        let source =
            "pub effect fn main =\n  do {\n    let message = \"hello\"\n    println message\n  }\n";
        let typed = type_module("artifact/effect-do-pure-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Sequence { statements, .. } = &core.functions[0].body else {
            panic!("expected do sequence");
        };
        assert!(matches!(
            statements.as_slice(),
            [CoreStatement::PureLet {
                name,
                value: CoreExpr::String { value, .. },
                ..
            }] if name == "message" && value == "hello"
        ));

        let typescript = lower_core_module_to_typescript_ir(core);
        assert!(typescript
            .runtime_requirements
            .iter()
            .all(|requirement| requirement != "effect.core.flatMap"));
        assert!(typescript
            .imports
            .iter()
            .all(|import| import.feature != "effect.core.flatMap"));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains(
            "(() => { const message: string = \"hello\"; return _ssrg_console_println(message); })()"
        ));
    }

    #[test]
    fn keeps_pure_do_let_inside_the_preceding_effect_continuation() {
        let source = "pub effect fn main =\n  do {\n    line <- readLine ()\n    let copy = line\n    succeed copy\n  }\n";
        let typed = type_module("artifact/effect-do-bind-pure-let/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert!(typescript
            .runtime_requirements
            .iter()
            .any(|requirement| requirement == "effect.core.flatMap"));
        let bundle = emit_typescript_module(typescript, source);
        assert!(bundle.typescript.contains(
            "_ssrg_effect_flatMap(_ssrg_stdin_readLine(), (line: string | undefined) => (() => { const copy: string | undefined = line; return _ssrg_effect_succeed(copy); })())"
        ));
    }

    #[test]
    fn lowers_do_bind_statement_to_typescript_const() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    ignored <- print \"hello\"\n    println \"done\"\n  }\n";
        let typed = type_module("artifact/effect-do-bind/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "_ssrg_effect_flatMap(_ssrg_console_print(\"hello\"), (ignored: undefined) => _ssrg_console_println(\"done\"))"
        ));
    }

    #[test]
    fn lowers_async_stdin_bind_to_cold_flat_map_chain() {
        let source =
            "pub effect fn main =\n  do {\n    first <- readLine ()\n    second <- readLine ()\n    succeed ()\n  }\n";
        let typed = type_module("artifact/effect-stdin-read-line/main.ssrg", source);
        let core = lower_typed_module(typed);
        let CoreExpr::Sequence { statements, .. } = &core.functions[0].body else {
            panic!("expected do block sequence");
        };
        let CoreStatement::Bind {
            value:
                CoreExpr::EffectOperation {
                    requirements,
                    failure,
                    success,
                    ..
                },
            ..
        } = &statements[0]
        else {
            panic!("expected readLine bind");
        };
        assert_eq!(
            requirements,
            &CoreType::Record {
                closed: true,
                fields: vec![CoreRecordField {
                    name: "stdin".to_owned(),
                    optional: false,
                    type_ref: CoreType::Named {
                        name: "Stdin".to_owned(),
                        arguments: Vec::new(),
                    },
                }],
            }
        );
        assert_eq!(
            failure,
            &CoreType::Named {
                name: "StdinError".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert_eq!(
            success,
            &CoreType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.metadata.runtime.requirements,
            vec![
                "core.unit",
                "effect.core.flatMap",
                "effect.stdin.readLine",
                "effect.core.succeed"
            ]
        );
        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_effect_flatMap(_ssrg_stdin_readLine(), (first: string | undefined) => _ssrg_effect_flatMap(_ssrg_stdin_readLine(), (second: string | undefined) => _ssrg_effect_succeed(undefined)))"
        ));
    }

    #[test]
    fn emits_basic_typescript_module() {
        let typed = type_module("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, "pub let answer: Int = 42\n");

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int64"]);
        assert_eq!(bundle.metadata.exports, vec!["answer"]);
        assert_eq!(bundle.typescript, "export const answer: bigint = 42n;\n");
        assert_eq!(bundle.source_map.names, vec!["answer"]);
        assert_eq!(bundle.source_map.mappings, "AAAAA");
    }

    #[test]
    fn maps_generated_declaration_to_its_original_source_line() {
        let source = "// generated code keeps this offset\n\npub let answer: Int = 42\n";
        let typed = type_module("artifact/source-map/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.source_map.mappings, "AAEAA");
    }

    #[test]
    fn emits_effect_typescript_module() {
        let source =
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println \"hello\"\n";
        let typed = type_module("artifact/effect-main/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle
            .typescript
            .contains("import { println as _ssrg_console_println }"));
        assert_eq!(bundle.metadata.exports, vec!["main"]);
        assert_eq!(bundle.source_map.names, vec!["main", "println"]);
        assert_eq!(bundle.source_map.mappings, ";;AAAAA");
    }

    #[test]
    fn emits_unit_result_as_plain_undefined() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n";
        let typed = type_module("artifact/effect-do/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(
            bundle.typescript,
            "export const main = (_unit: undefined) => undefined\n"
        );
    }
}
