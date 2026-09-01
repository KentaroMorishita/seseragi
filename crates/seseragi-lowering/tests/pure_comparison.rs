use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
    CoreType, TypeScriptFunction,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_int_comparison_to_sync_boolean_function() {
    let source = "pub fn isZero value: Int -> Bool = value == 0\n";
    let typed = type_module("artifact/pure-comparison/main.ssrg", source);
    let core = lower_typed_module(typed);
    let CoreExpr::Binary {
        operator, type_ref, ..
    } = &core.functions[0].body
    else {
        panic!("expected comparison binary expression");
    };
    assert_eq!(operator, "==");
    assert_eq!(
        type_ref,
        &CoreType::Named {
            name: "Bool".to_owned(),
            arguments: Vec::new(),
        }
    );

    let typescript = lower_core_module_to_typescript_ir(core);
    assert_eq!(
        typescript.runtime_requirements,
        vec!["core.int", "core.int.eq-dictionary", "core.bool"]
    );
    assert_eq!(typescript.imports.len(), 1);
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            is_async: false,
            ..
        }
    ));

    let bundle = emit_typescript_module(typescript, source);
    assert_eq!(
        bundle.typescript,
        "import { intEq as _ssrg_int_eq_dictionary } from \"@seseragi/runtime/equality\"\n\nexport const isZero = (value: number) => _ssrg_int_eq_dictionary[\"eq\"](value)(0)\n"
    );
}

#[test]
fn lowers_boolean_and_string_equality_through_runtime_dictionaries() {
    let source = "\
pub fn sameBool left: Bool -> right: Bool -> Bool = left == right
pub fn sameString left: String -> right: String -> Bool = left != right
";
    let typed = type_module("artifact/primitive-equality/main.ssrg", source);
    let core = lower_typed_module(typed);
    assert!(core.functions.iter().all(|function| matches!(
        function.body,
        CoreExpr::Binary {
            type_ref: CoreType::Named { ref name, .. },
            ..
        } if name == "Bool"
    )));

    let typescript = lower_core_module_to_typescript_ir(core);
    assert_eq!(
        typescript.runtime_requirements,
        vec![
            "core.bool",
            "core.bool.eq-dictionary",
            "core.string",
            "core.string.eq-dictionary"
        ]
    );
    assert_eq!(typescript.imports.len(), 2);
    let bundle = emit_typescript_module(typescript, source);
    assert_eq!(
        bundle.typescript,
        "import { boolEq as _ssrg_bool_eq_dictionary, stringEq as _ssrg_string_eq_dictionary } from \"@seseragi/runtime/equality\"\n\nexport const sameBool = (left: boolean) => (right: boolean) => _ssrg_bool_eq_dictionary[\"eq\"](left)(right)\nexport const sameString = (left: string) => (right: string) => _ssrg_string_eq_dictionary[\"eq\"](left)(right) === false\n"
    );
}
