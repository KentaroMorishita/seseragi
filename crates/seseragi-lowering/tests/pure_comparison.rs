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
        vec!["core.int64", "core.bool"]
    );
    assert!(typescript.imports.is_empty());
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
        "export const isZero = (value: bigint) => value === 0n\n"
    );
}

#[test]
fn lowers_boolean_and_string_equality_without_runtime_imports() {
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
        vec!["core.bool", "core.string"]
    );
    assert!(typescript.imports.is_empty());
    let bundle = emit_typescript_module(typescript, source);
    assert_eq!(
        bundle.typescript,
        "export const sameBool = (left: boolean, right: boolean) => left === right\nexport const sameString = (left: string, right: string) => left !== right\n"
    );
}
