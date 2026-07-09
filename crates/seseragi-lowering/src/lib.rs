mod core;
mod emit;
mod span;
mod typescript;

pub use core::{
    lower_typed_module, CoreBinding, CoreExpr, CoreFunction, CoreModule, CoreParameter,
};
pub use emit::{
    emit_typescript_module, GeneratedBundle, GeneratedModule, GeneratedOutputs, GeneratedRuntime,
    SourceMap,
};
pub(crate) use span::source_span;
pub use span::SourceSpan;
pub use typescript::{
    lower_core_module_to_typescript_ir, TypeScriptBinding, TypeScriptExpr, TypeScriptFunction,
    TypeScriptImport, TypeScriptModule, TypeScriptParameter, TypeScriptType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_semantics::type_module;

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
