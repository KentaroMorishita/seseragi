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

#[test]
fn selects_array_reducible_evidence_for_standard_join() {
    let typed = type_module(
        "artifact/collection-join/main.ssrg",
        "pub fn labels values: Array<String> -> String = join \", \" values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected labels function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            evidence,
            type_ref,
            ..
        } if callee == "std/prelude::join"
            && type_ref == &named("String")
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity },
                ..
            }] if identity == "std/array::Reducible")
    ));
}

#[test]
fn selects_reducible_zero_and_add_evidence_for_standard_sum() {
    let typed = type_module(
        "artifact/collection-sum/main.ssrg",
        "pub fn total values: Array<Int> -> Int = sum values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected total function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            evidence,
            type_ref,
            ..
        } if callee == "std/prelude::sum"
            && type_ref == &named("Int")
            && matches!(evidence.as_slice(), [
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: reducible },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: zero },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: add },
                    ..
                },
            ] if reducible == "std/array::Reducible"
                && zero == "std/int::Zero"
                && add == "std/int::Add")
    ));
}

#[test]
fn selects_reducible_and_monoid_evidence_for_standard_combine() {
    let typed = type_module(
        "artifact/collection-combine/main.ssrg",
        "pub fn combined values: Array<String> -> String = combine values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected combined function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            evidence,
            type_ref,
            ..
        } if callee == "std/prelude::combine"
            && type_ref == &named("String")
            && matches!(evidence.as_slice(), [
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: reducible },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: monoid },
                    ..
                },
            ] if reducible == "std/array::Reducible"
                && monoid == "std/string::Monoid")
    ));
}

#[test]
fn selects_standard_power_evidence_for_an_operator_function_value() {
    let typed = type_module(
        "artifact/power-operator-reference/main.ssrg",
        "fn apply operation: (Int -> Int -> Int) -> left: Int -> right: Int -> Int = operation left right\n\
         pub fn power base: Int -> exponent: Int -> Int = apply (**) base exponent\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected power function");
    };
    let TypedExpr::Call { arguments, .. } = body else {
        panic!("expected apply call");
    };
    assert!(matches!(
        arguments.as_slice(),
        [TypedExpr::Variable { name, evidence, .. }, _, _]
            if name == "**"
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    constraint: crate::TypedConstraint { name, .. },
                    evidence: TypedInstanceEvidence::Standard { identity },
                }] if name == "Pow" && identity == "std/int::Pow")
    ));
}

#[test]
fn selects_prelude_either_dictionaries_for_explicit_monad_calls() {
    let typed = type_module(
        "artifact/prelude-either-monad/main.ssrg",
        "fn increment value: Int -> Int = value + 1\n\
         fn bind value: Either<String, Int> -> Either<String, Int> =\n\
           value >>= (\\item -> pure $ increment item)\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected bind function");
    };
    let TypedExpr::Call {
        callee,
        arguments,
        evidence,
        type_ref,
        ..
    } = body
    else {
        panic!("expected flatMap call");
    };
    assert_eq!(callee, "std/prelude::Monad::flatMap");
    assert_eq!(
        type_ref,
        &applied("Either", vec![named("String"), named("Int")])
    );
    assert!(matches!(
        evidence.as_slice(),
        [crate::TypedCallEvidence {
            evidence: TypedInstanceEvidence::Standard { identity },
            ..
        }] if identity == "std/either::Monad"
    ));
    let TypedExpr::Lambda { body, .. } = &arguments[0] else {
        panic!("expected explicit lambda");
    };
    assert!(matches!(
        body.as_ref(),
        TypedExpr::Call { callee, evidence, .. }
            if callee == "std/prelude::Applicative::pure"
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity },
                    ..
                }] if identity == "std/either::Applicative")
    ));
}

#[test]
fn selects_the_prelude_maybe_functor_without_source_declarations() {
    let typed = type_module(
        "artifact/prelude-maybe-functor/main.ssrg",
        "fn increment value: Int -> Int = value + 1\n\
         fn transform value: Maybe<Int> -> Maybe<Int> = map increment value\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected transform function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call { callee, evidence, type_ref, .. }
            if callee == "std/prelude::Functor::map"
                && type_ref == &applied("Maybe", vec![named("Int")])
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity },
                    ..
                }] if identity == "std/maybe::Functor")
    ));
}

#[test]
fn selects_the_prelude_array_monad_without_source_declarations() {
    let typed = type_module(
        "artifact/prelude-array-monad/main.ssrg",
        "fn expand value: Int -> Array<Int> = [value, value + 10]\n\
         fn expanded values: Array<Int> -> Array<Int> = flatMap expand values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected expanded function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call { callee, evidence, type_ref, .. }
            if callee == "std/prelude::Monad::flatMap"
                && type_ref == &applied("Array", vec![named("Int")])
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity },
                    ..
                }] if identity == "std/array::Monad")
    ));
}

#[test]
fn selects_the_prelude_list_monad_without_source_declarations() {
    let typed = type_module(
        "artifact/prelude-list-monad/main.ssrg",
        "fn expand value: Int -> List<Int> = `[value, value + 10]\n\
         fn expanded values: List<Int> -> List<Int> = flatMap expand values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected expanded function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call { callee, evidence, type_ref, .. }
            if callee == "std/prelude::Monad::flatMap"
                && type_ref == &applied("List", vec![named("Int")])
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity },
                    ..
                }] if identity == "std/list::Monad")
    ));
}

#[test]
fn selects_the_prelude_effect_functor_without_source_declarations() {
    let typed = type_module(
        "artifact/prelude-effect-functor/main.ssrg",
        "fn increment value: Int -> Int = value + 1\n\
         effect fn incremented -> Int = map increment (succeed 41)\n",
    );

    let TypedDecl::EffectFn { body, .. } = &typed.declarations[1] else {
        panic!("expected incremented effect function");
    };
    assert!(
        matches!(
            body,
            TypedExpr::Call { callee, evidence, type_ref, .. }
                if callee == "std/prelude::Functor::map"
                    && type_ref == &applied(
                        "Effect",
                        vec![
                            TypedType::Record {
                                fields: Vec::new(),
                                closed: true,
                            },
                            named("Never"),
                            named("Int"),
                        ],
                    )
                    && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                        evidence: TypedInstanceEvidence::Standard { identity },
                        ..
                    }] if identity == "std/effect::Functor")
        ),
        "{body:#?}"
    );

    let typed = type_module(
        "artifact/prelude-effect-value/main.ssrg",
        "let source: Effect<{}, Never, Int> = pure 41\n\
         pub effect fn main = do { value <- source; succeed () }\n",
    );
    let TypedDecl::EffectFn { body, .. } = &typed.declarations[1] else {
        panic!("expected effect main function");
    };
    assert!(
        matches!(
            body,
            TypedExpr::DoBlock { statements, .. }
                if matches!(statements.as_slice(), [crate::TypedDoStatement::Bind {
                    value: TypedExpr::Variable { type_ref, .. },
                    ..
                }] if matches!(type_ref, TypedType::Named { name, arguments } if name == "Effect" && arguments.len() == 3))
        ),
        "{body:#?}"
    );
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
