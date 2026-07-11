use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedPattern, TypedType};

#[test]
fn types_standard_maybe_constructor_from_its_argument() {
    let typed = type_module(
        "artifact/prelude-maybe/main.ssrg",
        "fn wrap value: String -> Maybe<String> = Just value\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected wrapper function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            arguments,
            type_ref: TypedType::Named {
                name,
                arguments: result_arguments,
            },
            ..
        } if callee == "std/prelude::Just"
            && arguments.len() == 1
            && name == "Maybe"
            && result_arguments == &vec![named("String")]
    ));
}

#[test]
fn types_standard_either_patterns_and_proves_the_family_exhaustive() {
    let typed = type_module(
        "artifact/prelude-either/main.ssrg",
        "fn valueOrZero result: Either<String, Int> -> Int = match result { Left _ -> 0; Right value -> value }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected valueOrZero function");
    };
    let TypedExpr::Match {
        arms, exhaustive, ..
    } = body
    else {
        panic!("expected typed match");
    };
    assert!(*exhaustive);
    assert!(matches!(
        &arms[0].pattern,
        TypedPattern::Constructor {
            symbol,
            argument: Some(argument),
            ..
        } if symbol == "std/prelude::Left"
            && matches!(argument.as_ref(), TypedPattern::Wildcard { type_ref, .. }
                if type_ref == &named("String"))
    ));
    assert!(matches!(
        &arms[1].pattern,
        TypedPattern::Constructor {
            symbol,
            argument: Some(argument),
            ..
        } if symbol == "std/prelude::Right"
            && matches!(argument.as_ref(), TypedPattern::Binding { type_ref, .. }
                if type_ref == &named("Int"))
    ));
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
