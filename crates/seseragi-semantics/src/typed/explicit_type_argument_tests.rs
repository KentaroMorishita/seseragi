use super::type_module;
use crate::{semantic_diagnostics, TypedDecl, TypedExpr, TypedType};

#[test]
fn applies_explicit_call_site_type_arguments() {
    let typed = type_module(
        "artifact/explicit-call-type-arguments/main.ssrg",
        "fn identity<A> value: A -> A = value\n\
         pub let answer: Int = identity<Int> 42\n",
    );

    let TypedDecl::Let { value, .. } = &typed.declarations[1] else {
        panic!("expected typed let declaration");
    };
    assert!(matches!(
        value,
        TypedExpr::Call {
            callee,
            type_ref: TypedType::Named { name, arguments },
            ..
        } if callee.ends_with("::identity") && name == "Int" && arguments.is_empty()
    ));
}

#[test]
fn reports_explicit_call_site_type_argument_arity() {
    let diagnostics = semantic_diagnostics(
        "artifact/explicit-call-type-argument-arity/main.ssrg",
        "fn identity<A> value: A -> A = value\n\
         pub let answer = identity<Int, String> 42\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "call.type-argument-arity-mismatch"
    );
}
