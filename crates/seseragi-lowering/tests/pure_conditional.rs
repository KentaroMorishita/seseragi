use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
    CoreType, TypeScriptFunction,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_typed_conditional_to_sync_typescript_expression() {
    let source =
        "pub fn classify value: Int -> String = if value == 0 then \"zero\" else \"other\"\n";
    let typed = type_module("artifact/pure-if/main.ssrg", source);
    let core = lower_typed_module(typed);
    let CoreExpr::If {
        condition,
        type_ref,
        ..
    } = &core.functions[0].body
    else {
        panic!("expected core conditional");
    };
    assert!(matches!(condition.as_ref(), CoreExpr::Binary { .. }));
    assert_eq!(
        type_ref,
        &CoreType::Named {
            name: "String".to_owned(),
            arguments: Vec::new(),
        }
    );

    let typescript = lower_core_module_to_typescript_ir(core);
    assert_eq!(
        typescript.runtime_requirements,
        vec![
            "core.int",
            "core.string",
            "core.int.eq-dictionary",
            "core.bool"
        ]
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
        "import { intEq as _ssrg_int_eq_dictionary } from \"@seseragi/runtime/equality\"\n\nexport const classify = (value: number) => _ssrg_int_eq_dictionary[\"eq\"](value)(0) ? \"zero\" : \"other\"\n"
    );
}
