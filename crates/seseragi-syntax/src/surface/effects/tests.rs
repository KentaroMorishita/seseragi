use super::*;
use crate::surface::parse_surface_ast;

#[test]
fn preserves_explicit_effect_contract_clauses() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub effect fn run value: Int -> Unit\nwith console: Console, Logger\nfails AppError\nwhere Show<Int> =\n  succeed ()\n",
    );

    let SurfaceDecl::EffectFn {
        inferred_contract,
        return_type,
        requirements,
        failure,
        constraints,
        body,
        ..
    } = &module.declarations[0]
    else {
        panic!("expected effect function declaration");
    };
    assert!(!inferred_contract);
    assert!(matches!(
        return_type,
        Some(TypeRef::Named { name, .. }) if name == "Unit"
    ));
    assert!(matches!(
        &requirements[0],
        SurfaceRequirement::Field { name, type_ref: TypeRef::Named { name: type_name, .. }, .. }
            if name == "console" && type_name == "Console"
    ));
    assert!(matches!(
        &requirements[1],
        SurfaceRequirement::Shorthand { name, .. } if name == "Logger"
    ));
    assert!(matches!(
        failure,
        Some(TypeRef::Named { name, .. }) if name == "AppError"
    ));
    assert_eq!(constraints, &["Show".to_owned()]);
    assert!(body.is_some());
}
