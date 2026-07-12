use super::type_module;
use crate::{semantic_diagnostics, TypedDecl, TypedDoStatement, TypedExpr, TypedType};

#[test]
fn derives_from_either_failure_and_success_from_a_concrete_parameter() {
    let typed = type_module(
        "artifact/effect-from-either/main.ssrg",
        "pub type Hand = | Rock\n\
         pub type InputError = | Invalid\n\
         pub effect fn resume result: Either<InputError, Hand> = fromEither result\n",
    );

    let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[2] else {
        panic!("expected effect function");
    };
    assert_eq!(
        effect.environment,
        TypedType::Record {
            closed: true,
            fields: Vec::new(),
        }
    );
    assert_eq!(effect.failure, named("InputError"));
    assert_eq!(effect.success, named("Hand"));
    assert!(matches!(
        body,
        TypedExpr::EffectCall {
            operation,
            effect: call_effect,
            arguments,
            ..
        } if operation == "std/effect::fromEither"
            && call_effect.environment == TypedType::Record {
                closed: true,
                fields: Vec::new(),
            }
            && call_effect.failure == named("InputError")
            && call_effect.success == named("Hand")
            && matches!(arguments.as_slice(), [TypedExpr::Variable { name, type_ref, .. }]
                if name == "result"
                    && type_ref == &applied(
                        "Either",
                        vec![named("InputError"), named("Hand")]
                    ))
    ));
}

#[test]
fn derives_from_either_contract_from_a_typed_pure_call() {
    let typed = type_module(
        "artifact/effect-from-either-call/main.ssrg",
        "type Hand = | Rock\n\
         type InputError = | Invalid\n\
         fn accepted hand: Hand -> Either<InputError, Hand> = Right hand\n\
         pub effect fn main = fromEither (accepted Rock)\n",
    );

    let TypedDecl::EffectFn { effect, body, .. } = &typed.declarations[3] else {
        panic!("expected effect function");
    };
    assert_eq!(effect.failure, named("InputError"));
    assert_eq!(effect.success, named("Hand"));
    assert!(matches!(
        body,
        TypedExpr::EffectCall {
            operation,
            arguments,
            ..
        } if operation == "std/effect::fromEither"
            && matches!(arguments.as_slice(), [TypedExpr::Call { callee, type_ref, .. }]
                if callee == "artifact/effect-from-either-call::accepted"
                    && type_ref == &applied(
                        "Either",
                        vec![named("InputError"), named("Hand")]
                    ))
    ));
}

#[test]
fn preserves_the_sum_owner_for_a_do_bind_used_by_a_later_pure_call() {
    let typed = type_module(
        "artifact/effect-from-either-do-bind/main.ssrg",
        "type Hand = | Rock\n\
         type InputError = | Invalid\n\
         fn accepted hand: Hand -> Either<InputError, Hand> = Right hand\n\
         fn keep hand: Hand -> Hand = hand\n\
         pub effect fn main =\n\
           do {\n\
             hand <- fromEither (accepted Rock)\n\
             succeed (keep hand)\n\
           }\n",
    );

    let TypedDecl::EffectFn { body, .. } = &typed.declarations[4] else {
        panic!("expected effect function");
    };
    let TypedExpr::DoBlock {
        statements, result, ..
    } = body
    else {
        panic!("expected typed do block");
    };
    assert!(matches!(
        statements.as_slice(),
        [TypedDoStatement::Bind { type_ref, .. }] if type_ref == &named("Hand")
    ));
    assert!(matches!(
        result.as_ref(),
        TypedExpr::EffectCall { arguments, .. }
            if matches!(arguments.as_slice(), [TypedExpr::Call { type_ref, .. }]
                if type_ref == &named("Hand"))
    ));
}

#[test]
fn reports_non_either_from_either_source_as_a_type_error() {
    let diagnostics = semantic_diagnostics(
        "artifact/effect-from-either-wrong-source/main.ssrg",
        "effect fn main = fromEither \"not either\"\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    let diagnostic = &diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "SES-T0101");
    assert_eq!(
        diagnostic.message_key,
        "effect.from-either-source-not-either"
    );
    assert_eq!(
        diagnostic.related[0].message,
        "expected Either<E, A>, received String"
    );
    assert_ne!(diagnostic.message_key, "name.unresolved");
}

#[test]
fn requires_exactly_one_from_either_argument() {
    for (source, actual) in [
        ("effect fn main = fromEither\n", 0),
        ("effect fn main = fromEither \"first\" \"second\"\n", 2),
    ] {
        let diagnostics = semantic_diagnostics(
            format!("artifact/effect-from-either-arity-{actual}/main.ssrg"),
            source,
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        let diagnostic = &diagnostics.diagnostics[0];
        assert_eq!(diagnostic.code, "SES-T0101");
        assert_eq!(diagnostic.message_key, "call.arity-mismatch");
        assert_eq!(
            diagnostic.related[0].message,
            format!("expected 1 argument, received {actual}")
        );
    }
}

#[test]
fn does_not_treat_shadowed_from_either_as_an_intrinsic() {
    let diagnostics = semantic_diagnostics(
        "artifact/effect-shadowed-from-either/main.ssrg",
        "effect fn main =\n  do {\n    let fromEither = \"plain\"\n    fromEither\n  }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "effect.compact-body-not-effect"
    );
}

#[test]
fn leaves_unknown_from_either_argument_to_name_resolution() {
    let diagnostics = semantic_diagnostics(
        "artifact/effect-from-either-unknown/main.ssrg",
        "effect fn main = fromEither missing\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
    assert_eq!(diagnostics.diagnostics[0].message_key, "name.unresolved");
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn applied(name: &str, arguments: Vec<TypedType>) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments,
    }
}
