use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
};
use seseragi_semantics::type_module;

#[test]
fn desugars_logical_operators_to_core_control_flow() {
    let source = concat!(
        "pub fn decide left: Bool -> middle: Bool -> right: Bool -> Bool =\n",
        "  left || middle && right\n",
    );
    let typed = type_module("artifact/logical-short-circuit/main.ssrg", source);
    let core = lower_typed_module(typed);

    let CoreExpr::If {
        then_branch,
        else_branch,
        ..
    } = &core.functions[0].body
    else {
        panic!("logical or must lower to core control flow");
    };
    assert!(matches!(
        then_branch.as_ref(),
        CoreExpr::Boolean { value: true, .. }
    ));
    assert!(matches!(else_branch.as_ref(), CoreExpr::If { .. }));

    let typescript = lower_core_module_to_typescript_ir(core);
    let generated = emit_typescript_module(typescript, source);
    assert_eq!(
        generated.typescript,
        "export const decide = (left: boolean) => (middle: boolean) => (right: boolean) => left ? true : middle ? right : false\n"
    );
}

#[test]
fn keeps_each_required_right_hand_side_in_one_conditional_branch() {
    let source = concat!(
        "pub fn andValue left: Bool -> right: Bool -> Bool = left && right\n",
        "pub fn orValue left: Bool -> right: Bool -> Bool = left || right\n",
    );
    let typed = type_module("artifact/logical-branches/main.ssrg", source);
    let core = lower_typed_module(typed);

    assert!(matches!(
        &core.functions[0].body,
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } if matches!(then_branch.as_ref(), CoreExpr::Variable { name, .. } if name == "right")
            && matches!(else_branch.as_ref(), CoreExpr::Boolean { value: false, .. })
    ));
    assert!(matches!(
        &core.functions[1].body,
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } if matches!(then_branch.as_ref(), CoreExpr::Boolean { value: true, .. })
            && matches!(else_branch.as_ref(), CoreExpr::Variable { name, .. } if name == "right")
    ));
}
