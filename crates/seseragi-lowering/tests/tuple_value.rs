use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
    CoreType, TypeScriptExpr, TypeScriptFunction,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_tuple_values_without_a_runtime_helper() {
    let source = "pub fn pair left: Int -> right: Bool -> (Int, Bool) = (left, right)\n";
    let typed = type_module("artifact/tuple-value/main.ssrg", source);
    let core = lower_typed_module(typed);
    let CoreExpr::Tuple {
        elements, type_ref, ..
    } = &core.functions[0].body
    else {
        panic!("expected core tuple");
    };
    assert_eq!(elements.len(), 2);
    assert_eq!(
        type_ref,
        &CoreType::Tuple {
            elements: vec![named_core_type("Int"), named_core_type("Bool")],
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
            body: TypeScriptExpr::Tuple { elements },
            ..
        } if elements.len() == 2
    ));

    let bundle = emit_typescript_module(typescript, source);
    assert_eq!(
        bundle.typescript,
        "export const pair = (left: bigint) => (right: boolean) => [left, right] as const\n"
    );
}

#[test]
fn emits_a_top_level_tuple_with_a_readonly_typescript_type() {
    let source = "pub let pair: (Int, Bool) = (1, True)\n";
    let typed = type_module("artifact/tuple-binding/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    assert!(typescript.imports.is_empty());
    let bundle = emit_typescript_module(typescript, source);
    assert_eq!(
        bundle.typescript,
        "export const pair: readonly [bigint, boolean] = [1n, true] as const;\n"
    );
}

fn named_core_type(name: &str) -> CoreType {
    CoreType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
