use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedInstanceEvidence, TypedPattern, TypedType};

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
fn instantiates_standard_sum_constructors_from_the_declared_result() {
    let typed = type_module(
        "artifact/prelude-context/main.ssrg",
        "type Hand = | Rock\n\
         type HandInputError = | InvalidHand\n\
         fn accepted hand: Hand -> Either<HandInputError, Hand> = Right hand\n\
         fn rejected error: HandInputError -> Either<HandInputError, Hand> = Left error\n\
         fn absent unit: Unit -> Maybe<Hand> = Nothing\n",
    );

    let expected_either = applied("Either", vec![named("HandInputError"), named("Hand")]);
    for index in [2, 3] {
        let TypedDecl::Fn { body, .. } = &typed.declarations[index] else {
            panic!("expected Either constructor function");
        };
        assert_eq!(body_type(body), expected_either);
    }
    let TypedDecl::Fn { body, .. } = &typed.declarations[4] else {
        panic!("expected Maybe constructor function");
    };
    assert_eq!(body_type(body), applied("Maybe", vec![named("Hand")]));
}

#[test]
fn propagates_expected_sum_types_through_nested_expressions() {
    let typed = type_module(
        "artifact/prelude-nested-context/main.ssrg",
        "type Hand = | Rock\n\
         type HandInputError = | InvalidHand\n\
         fn nested unit: Unit -> Maybe<Maybe<Hand>> = Just Nothing\n\
         fn choose valid: Bool -> Either<HandInputError, Hand> =\n\
           if valid then Right Rock else Left InvalidHand\n\
         fn pair unit: Unit -> (Maybe<Hand>, Either<HandInputError, Hand>) =\n\
           (Nothing, Right Rock)\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[2] else {
        panic!("expected nested function");
    };
    assert_eq!(
        body_type(body),
        applied("Maybe", vec![applied("Maybe", vec![named("Hand")])])
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[3] else {
        panic!("expected conditional function");
    };
    assert_eq!(
        body_type(body),
        applied("Either", vec![named("HandInputError"), named("Hand")],)
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[4] else {
        panic!("expected tuple function");
    };
    assert_eq!(
        body_type(body),
        TypedType::Tuple {
            elements: vec![
                applied("Maybe", vec![named("Hand")]),
                applied("Either", vec![named("HandInputError"), named("Hand")],),
            ],
        }
    );
}

#[test]
fn propagates_expected_sum_type_to_match_arm_bodies() {
    let typed = type_module(
        "artifact/prelude-match-context/main.ssrg",
        "type Hand = | Rock | Paper\n\
         type HandInputError = | InvalidHand\n\
         fn parse hand: Hand -> Either<HandInputError, Hand> =\n\
           match hand { Rock -> Right Rock; _ -> Left InvalidHand }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[2] else {
        panic!("expected parse function");
    };
    let TypedExpr::Match { arms, type_ref, .. } = body else {
        panic!("expected match expression");
    };
    let expected = applied("Either", vec![named("HandInputError"), named("Hand")]);
    assert_eq!(type_ref, &expected);
    assert!(arms.iter().all(|arm| body_type(&arm.body) == expected));
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

#[test]
fn substitutes_iterator_element_types_through_nested_standard_results() {
    let typed = type_module(
        "artifact/prelude-iterator/main.ssrg",
        "fn inspect iterator: Iterator<Int> -> Int =\n\
           match next iterator {\n\
             Nothing -> 0\n\
             Just (value, _) -> value\n\
           }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected inspect function");
    };
    let TypedExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert!(matches!(
        &arms[1].pattern,
        TypedPattern::Constructor {
            symbol,
            argument: Some(argument),
            ..
        } if symbol == "std/prelude::Just"
            && matches!(argument.as_ref(), TypedPattern::Tuple { elements, .. }
                if matches!(&elements[0], TypedPattern::Binding { type_ref, .. }
                    if type_ref == &named("Int")))
    ));
}

#[test]
fn selects_array_reducible_evidence_for_standard_reduce() {
    let typed = type_module(
        "artifact/array-reduce/main.ssrg",
        "pub fn sum values: Array<Int> -> Int = reduce 0 (+) values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected sum function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            evidence,
            type_ref,
            ..
        } if callee == "std/prelude::reduce"
            && type_ref == &named("Int")
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity },
                ..
            }] if identity == "std/array::Reducible")
    ));
    let TypedExpr::Call { arguments, .. } = body else {
        unreachable!();
    };
    assert!(matches!(
        arguments.as_slice(),
        [_, TypedExpr::Variable { name, evidence, .. }, _]
            if name == "+"
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    constraint: crate::TypedConstraint { name, .. },
                    evidence: TypedInstanceEvidence::Standard { identity },
                }] if name == "Add" && identity == "std/int::Add")
    ));
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

fn body_type(expression: &TypedExpr) -> TypedType {
    crate::typed::type_ref::inferred_type_from_expr(expression)
}
