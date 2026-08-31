use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedInstanceEvidence, TypedPattern, TypedType};

#[test]
fn preserves_unary_result_types_through_generic_and_trait_evidence_calls() {
    let source = concat!(
        "trait Render<A> { fn render value: A -> String }\n",
        "instance Render<Int> { fn render value: Int -> String = show value }\n",
        "fn identity<A> value: A -> A = value\n",
        "effect fn main = do {\n",
        "  println \"first\"\n",
        "  -1 |> identity |> debug |> println\n",
        "  -1 |> show |> println\n",
        "  -2 |> render |> println\n",
        "  -0.0 |> show |> println\n",
        "  !True |> debug |> println\n",
        "  [-3, -4] |> debug |> println\n",
        "}\n",
    );

    let diagnostics = crate::semantic_diagnostics("artifact/unary-evidence/main.ssrg", source);
    assert!(
        diagnostics.diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:#?}"
    );
}

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
                evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
                }] if name == "Add" && identity == "std/int::Add")
    ));
}

#[test]
fn retains_unannotated_standard_arithmetic_results_across_top_level_bindings() {
    let typed = type_module(
        "artifact/arithmetic-result-inference/main.ssrg",
        "let difference = 1 - 2\n\
         let annotated: Int = 1 - 2\n\
         let joined = \"Sese\" + \"ragi\"\n\
         pub fn render unit: Unit -> (String, String, String) =\n\
           (debug difference, debug annotated, debug joined)\n",
    );

    for index in [0, 1] {
        let TypedDecl::Let { scheme, value, .. } = &typed.declarations[index] else {
            panic!("expected arithmetic binding");
        };
        assert_eq!(scheme.type_ref, named("Int"));
        assert!(matches!(
            value,
            TypedExpr::Binary {
                type_ref,
                evidence,
                ..
            } if type_ref == &named("Int")
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
                    ..
                }] if identity == "std/int::Sub")
        ));
    }

    let TypedDecl::Let { scheme, value, .. } = &typed.declarations[2] else {
        panic!("expected String Add binding");
    };
    assert_eq!(scheme.type_ref, named("String"));
    assert!(matches!(
        value,
        TypedExpr::Binary {
            type_ref,
            evidence,
            ..
        } if type_ref == &named("String")
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity, .. },
                ..
            }] if identity == "std/string::Add")
    ));

    let TypedDecl::Fn { body, .. } = &typed.declarations[3] else {
        panic!("expected render function");
    };
    let TypedExpr::Tuple { elements, .. } = body else {
        panic!("expected rendered tuple");
    };
    assert!(elements.iter().all(|element| matches!(
        element,
        TypedExpr::Call {
            callee,
            evidence,
            type_ref,
            ..
        } if callee == "std/prelude::Debug::debug"
            && type_ref == &named("String")
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { .. },
                ..
            }])
    )));
}

#[test]
fn retains_concrete_generic_call_results_across_top_level_bindings() {
    let typed = type_module(
        "artifact/top-level-call-inference/main.ssrg",
        "fn wrapMaybe<A> value: A -> Maybe<A> = Just value\n\
         fn wrapEither<A> value: A -> Either<String, A> = Right value\n\
         fn wrapArray<A> value: A -> Array<A> = [value]\n\
         fn wrapList<A> value: A -> List<A> = `[value]\n\
         let maybeValue = wrapMaybe 42\n\
         let eitherValue = wrapEither 42\n\
         let arrayValue = wrapArray 42\n\
         let listValue = wrapList 42\n\
         pub fn render unit: Unit -> (String, String, String, String) =\n\
           (debug maybeValue, debug eitherValue, debug arrayValue, debug listValue)\n",
    );

    let expected = [
        applied("Maybe", vec![named("Int")]),
        applied("Either", vec![named("String"), named("Int")]),
        applied("Array", vec![named("Int")]),
        applied("List", vec![named("Int")]),
    ];
    for (declaration, expected) in typed.declarations[4..8].iter().zip(expected) {
        let TypedDecl::Let { scheme, .. } = declaration else {
            panic!("expected inferred top-level binding");
        };
        assert_eq!(scheme.type_ref, expected);
    }

    let TypedDecl::Fn { body, .. } = &typed.declarations[8] else {
        panic!("expected render function");
    };
    let TypedExpr::Tuple { elements, .. } = body else {
        panic!("expected rendered tuple");
    };
    assert!(elements.iter().all(|element| matches!(
        element,
        TypedExpr::Call {
            callee,
            evidence,
            type_ref,
            ..
        } if callee == "std/prelude::Debug::debug"
            && type_ref == &named("String")
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { .. },
                ..
            }])
    )));
}

#[test]
fn composes_array_show_from_a_scoped_element_dictionary() {
    let typed = type_module(
        "artifact/scoped-array-show/main.ssrg",
        "pub fn renderMany<A> values: Array<A> -> String\n\
         where Show<A> =\n\
           show values\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected renderMany function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            callee,
            evidence,
            ..
        } if callee == "std/prelude::Show::show"
            && matches!(
                evidence.as_slice(),
                [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard {
                        identity,
                        evidence_arguments,
                        ..
                    },
                    ..
                }] if identity == "std/array::Show"
                    && matches!(
                        evidence_arguments.as_slice(),
                        [crate::TypedCallEvidence {
                            evidence: TypedInstanceEvidence::Parameter { index: 0 },
                            ..
                        }]
                    )
            )
    ));
}

#[test]
fn composes_maybe_show_from_a_local_element_dictionary() {
    let typed = type_module(
        "artifact/local-maybe-show/main.ssrg",
        "type Badge = | Active | Paused\n\
         instance Show<Badge> {\n\
           fn show value: Badge -> String =\n\
             match value { Active -> \"active\"; Paused -> \"paused\" }\n\
         }\n\
         pub fn renderBadge value: Maybe<Badge> -> String =\n\
           show value\n",
    );

    let body = typed
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::Fn { symbol, body, .. } if symbol.ends_with("::renderBadge") => Some(body),
            _ => None,
        })
        .expect("expected renderBadge function");
    assert!(matches!(
        body,
        TypedExpr::Call {
            evidence,
            ..
        } if matches!(
            evidence.as_slice(),
            [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard {
                    identity,
                    evidence_arguments,
                    ..
                },
                ..
            }] if identity == "std/maybe::Show"
                && matches!(
                    evidence_arguments.as_slice(),
                    [crate::TypedCallEvidence {
                        evidence: TypedInstanceEvidence::Local { identity, .. },
                        ..
                    }] if identity == "std/prelude::Show<artifact/local-maybe-show::Badge>"
                )
        )
    ));
}

#[test]
fn composes_a_local_instance_requirement_through_standard_collection_evidence() {
    let typed = type_module(
        "artifact/local-instance-collection-requirement/main.ssrg",
        "type Badge = | Active\n\
         instance Show<Badge> {\n\
           fn show value: Badge -> String = \"active\"\n\
         }\n\
         trait Render<A> {\n\
           fn render value: A -> String\n\
         }\n\
         instance<T> Render<Maybe<T>>\n\
         where Show<Array<T>> {\n\
           fn render value: Maybe<T> -> String = \"rendered\"\n\
         }\n\
         pub fn label value: Maybe<Badge> -> String =\n\
           render value\n",
    );

    let body = typed
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::Fn { symbol, body, .. } if symbol.ends_with("::label") => Some(body),
            _ => None,
        })
        .expect("expected label function");
    assert!(matches!(
        body,
        TypedExpr::Call {
            evidence,
            ..
        } if matches!(
            evidence.as_slice(),
            [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Local {
                    evidence_arguments,
                    ..
                },
                ..
            }] if matches!(
                evidence_arguments.as_slice(),
                [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard {
                        identity,
                        evidence_arguments,
                        ..
                    },
                    ..
                }] if identity == "std/array::Show"
                    && matches!(
                        evidence_arguments.as_slice(),
                        [crate::TypedCallEvidence {
                            evidence: TypedInstanceEvidence::Local { identity, .. },
                            ..
                        }] if identity
                            == "std/prelude::Show<artifact/local-instance-collection-requirement::Badge>"
                    )
            )
        )
    ));
}

#[test]
fn does_not_apply_the_standard_maybe_dictionary_to_a_shadowing_local_adt() {
    let typed = type_module(
        "artifact/shadowed-maybe-show/main.ssrg",
        "type Maybe<A> = | Local A\n\
         pub fn render value: Maybe<String> -> String =\n\
           show value\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected render function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            evidence,
            type_ref: TypedType::Hole,
            ..
        } if evidence.is_empty()
    ));
}

#[test]
fn allows_a_local_show_instance_for_a_shadowing_maybe_adt() {
    let source = "type Maybe<A> = | Local A\n\
                  instance<A> Show<Maybe<A>> {\n\
                    fn show value: Maybe<A> -> String = \"local\"\n\
                  }\n\
                  pub fn render value: Maybe<String> -> String =\n\
                    show value\n";
    let diagnostics =
        crate::semantic_diagnostics("artifact/shadowed-maybe-instance/main.ssrg", source);
    assert!(
        diagnostics.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        diagnostics.diagnostics
    );

    let typed = type_module("artifact/shadowed-maybe-instance/main.ssrg", source);
    let body = typed
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::Fn { symbol, body, .. } if symbol.ends_with("::render") => Some(body),
            _ => None,
        })
        .expect("expected render function");
    assert!(matches!(
        body,
        TypedExpr::Call {
            evidence,
            ..
        } if matches!(
            evidence.as_slice(),
            [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Local {
                    identity,
                    type_arguments,
                    ..
                },
                ..
            }] if identity
                == "std/prelude::Show<artifact/shadowed-maybe-instance::Maybe<$0>>"
                && type_arguments == &vec![named("String")]
        )
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
                evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity: reducible, .. },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: zero, .. },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: add, .. },
                    ..
                },
            ] if reducible == "std/array::Reducible"
                && zero == "std/int::Zero"
                && add == "std/int::Add")
    ));
}

#[test]
fn selects_reducible_one_and_mul_evidence_for_standard_product() {
    let typed = type_module(
        "artifact/collection-product/main.ssrg",
        "pub fn total values: List<Int> -> Int = product values\n",
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
        } if callee == "std/prelude::product"
            && type_ref == &named("Int")
            && matches!(evidence.as_slice(), [
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: reducible, .. },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: one, .. },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: mul, .. },
                    ..
                },
            ] if reducible == "std/list::Reducible"
                && one == "std/int::One"
                && mul == "std/int::Mul")
    ));
}

#[test]
fn selects_iterable_evidence_for_short_circuit_aggregates() {
    let typed = type_module(
        "artifact/collection-predicates/main.ssrg",
        "pub fn hasPositive values: Range<Int> -> Bool = any (\\value: Int -> value > 0) values\n\
         pub fn allPositive values: Array<Int> -> Bool = all (\\value: Int -> value > 0) values\n",
    );

    for (declaration, expected_callee, expected_identity) in [
        (
            &typed.declarations[0],
            "std/prelude::any",
            "std/range::Iterable",
        ),
        (
            &typed.declarations[1],
            "std/prelude::all",
            "std/array::Iterable",
        ),
    ] {
        let TypedDecl::Fn { body, .. } = declaration else {
            panic!("expected predicate aggregate function");
        };
        assert!(matches!(
            body,
            TypedExpr::Call { callee, evidence, type_ref, .. }
                if callee == expected_callee
                    && type_ref == &named("Bool")
                    && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                        evidence: TypedInstanceEvidence::Standard { identity, .. },
                        ..
                    }] if identity == expected_identity)
        ));
    }
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
                    evidence: TypedInstanceEvidence::Standard { identity: reducible, .. },
                    ..
                },
                crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity: monoid, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
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
            evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
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
                        evidence: TypedInstanceEvidence::Standard { identity, .. },
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

#[test]
fn selects_operator_abi_instances_for_direct_standard_trait_methods() {
    let source = "pub let addTwenty: Int -> Int = add 20\n\
                  pub fn values unit: Unit -> (Bool, Int, Float, String) =\n\
                    (eq 21 21, addTwenty 22, mul 6.0 7.0, add \"sese\" \"ragi\")\n";
    let diagnostics =
        crate::semantic_diagnostics("artifact/direct-trait-methods/main.ssrg", source);
    assert!(
        diagnostics.diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:#?}"
    );

    let typed = type_module("artifact/direct-trait-methods/main.ssrg", source);
    let TypedDecl::Let { value, .. } = &typed.declarations[0] else {
        panic!("expected partial add binding");
    };
    assert!(matches!(
        value,
        TypedExpr::Call { callee, evidence, .. }
            if callee == "std/prelude::Add::add"
                && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Standard { identity, .. },
                    ..
                }] if identity == "std/int::Add")
    ));
}

#[test]
fn keeps_operator_abi_instances_out_of_generic_function_evidence() {
    let diagnostics = crate::semantic_diagnostics(
        "artifact/generic-eq-boundary/main.ssrg",
        "pub fn same<A> left: A -> right: A -> Bool\n\
         where Eq<A> = eq left right\n\
         pub fn answer unit: Unit -> Bool = same 21 21\n",
    );
    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
    }));
}

#[test]
fn accepts_a_user_traversable_instance_with_canonical_constraints() {
    let source = "pub type Box<A> = | Box A\n\
                  instance Functor<Box> {\n\
                    fn map<A, B> f: (A -> B) -> value: Box<A> -> Box<B> =\n\
                      match value { Box item -> Box (f item) }\n\
                  }\n\
                  instance Traversable<Box> {\n\
                    fn traverse<G<_>, A, B>\n\
                      f: (A -> G<B>) -> value: Box<A> -> G<Box<B>>\n\
                    where Applicative<G> =\n\
                      match value { Box item -> map Box (f item) }\n\
                  }\n";
    let diagnostics = crate::semantic_diagnostics("artifact/user-traversable/main.ssrg", source);
    assert!(
        diagnostics.diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:#?}"
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
