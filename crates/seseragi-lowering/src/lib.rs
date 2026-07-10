mod core;
mod effect_ops;
mod emit;
mod span;
mod typescript;

pub use core::{
    lower_typed_module, CoreBinding, CoreExpr, CoreFunction, CoreModule, CoreParameter,
    CoreRecordField, CoreStatement, CoreType,
};
pub use emit::{
    emit_typescript_module, GeneratedBundle, GeneratedModule, GeneratedOutputs, GeneratedRuntime,
    SourceMap,
};
pub(crate) use span::source_span;
pub use span::SourceSpan;
pub use typescript::{
    lower_core_module_to_typescript_ir, TypeScriptBinding, TypeScriptExpr, TypeScriptFunction,
    TypeScriptImport, TypeScriptModule, TypeScriptParameter, TypeScriptStatement, TypeScriptType,
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
            "export const first = (left: bigint, right: bigint) => left\n"
        );
    }

    #[test]
    fn lowers_integer_add_function_to_typescript_binary_expression() {
        let source = "pub fn add x: Int -> y: Int -> Int = x + y\n";
        let typed = type_module("artifact/add-fn/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert_eq!(bundle.metadata.runtime.requirements, vec!["core.int64"]);
        assert_eq!(
            bundle.typescript,
            "export const add = (x: bigint, y: bigint) => x + y\n"
        );
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
    fn lowers_unit_result_to_typescript_undefined_expression() {
        let source = "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n";
        let typed = type_module("artifact/effect-do/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);

        assert_eq!(typescript.runtime_requirements, vec!["core.unit"]);
        assert!(matches!(
            &typescript.functions[0],
            TypeScriptFunction::ConstFunction {
                body: TypeScriptExpr::Undefined,
                ..
            }
        ));
    }

    #[test]
    fn lowers_single_effect_do_statement_to_typescript_call() {
        let source =
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { println \"hello\" }\n";
        let typed = type_module("artifact/effect-do-println/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => _ssrg_console_println(\"hello\")"
        ));
    }

    #[test]
    fn lowers_multiple_effect_do_statements_to_typescript_sequence() {
        let source =
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { println \"one\" println \"two\" }\n";
        let typed = type_module("artifact/effect-do-multiple/main.ssrg", source);
        let core = lower_typed_module(typed);
        let typescript = lower_core_module_to_typescript_ir(core);
        let bundle = emit_typescript_module(typescript, source);

        assert!(bundle.typescript.contains(
            "export const main = (_unit: undefined) => (() => { _ssrg_console_println(\"one\"); _ssrg_console_println(\"two\"); return undefined; })()"
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
            "const ignored: undefined = _ssrg_console_print(\"hello\"); _ssrg_console_println(\"done\");"
        ));
    }

    #[test]
    fn lowers_async_stdin_bind_to_awaited_async_function() {
        let source =
            "pub effect fn main =\n  do {\n    first <- readLine ()\n    second <- readLine ()\n  }\n";
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
            vec!["core.unit", "effect.stdin.readLine"]
        );
        assert!(bundle.typescript.contains(
            "export const main = async (_unit: undefined) => (async () => { const first: string | undefined = await _ssrg_stdin_readLine(); const second: string | undefined = await _ssrg_stdin_readLine(); return undefined; })()"
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
