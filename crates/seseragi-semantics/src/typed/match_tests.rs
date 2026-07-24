use super::type_module;
use crate::{TypedDecl, TypedExpr, TypedPattern, TypedType};

#[test]
fn types_exhaustive_adt_tuple_match_as_a_total_expression() {
    let typed = type_module(
        "artifact/rps/main.ssrg",
        "type Hand = | Rock | Paper | Scissors\n\
         type Outcome = | Player1Wins | Player2Wins | Draw\n\
         fn decide first: Hand -> second: Hand -> Outcome =\n\
           match (first, second) {\n\
             (Rock, Rock) -> Draw\n\
             (Paper, Paper) -> Draw\n\
             (Scissors, Scissors) -> Draw\n\
             (Rock, Scissors) -> Player1Wins\n\
             (Paper, Rock) -> Player1Wins\n\
             (Scissors, Paper) -> Player1Wins\n\
             _ -> Player2Wins\n\
           }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[2] else {
        panic!("expected decide function");
    };
    let TypedExpr::Match {
        scrutinee,
        arms,
        exhaustive,
        type_ref,
        ..
    } = body
    else {
        panic!("expected typed match");
    };
    assert!(matches!(scrutinee.as_ref(), TypedExpr::Tuple { .. }));
    assert_eq!(arms.len(), 7);
    assert!(*exhaustive);
    assert_eq!(type_ref, &named("Outcome"));
    assert!(matches!(
        &arms[0].pattern,
        TypedPattern::Tuple { elements, .. }
            if matches!(&elements[0], TypedPattern::Constructor { symbol, .. }
                if symbol == "artifact/rps::Rock")
                && matches!(&elements[1], TypedPattern::Constructor { symbol, .. }
                    if symbol == "artifact/rps::Rock")
    ));
}

#[test]
fn instantiates_generic_constructor_payload_from_scrutinee_arguments() {
    let typed = type_module(
        "artifact/result/main.ssrg",
        "type Result<E, A> = | Failure E | Success A\n\
         fn valueOrZero result: Result<String, Int> -> Int =\n\
           match result {\n\
             Success value -> value\n\
             Failure _ -> 0\n\
           }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected valueOrZero function");
    };
    let TypedExpr::Match {
        arms, exhaustive, ..
    } = body
    else {
        panic!("expected typed match");
    };
    assert!(*exhaustive);
    let TypedPattern::Constructor {
        argument: Some(argument),
        ..
    } = &arms[0].pattern
    else {
        panic!("expected payload constructor pattern");
    };
    assert!(matches!(
        argument.as_ref(),
        TypedPattern::Binding { type_ref, .. } if type_ref == &named("Int")
    ));
    assert!(matches!(
        &arms[0].body,
        TypedExpr::Variable { type_ref, .. } if type_ref == &named("Int")
    ));
}

#[test]
fn carries_generic_payload_type_through_constructor_application_scrutinee() {
    let typed = type_module(
        "artifact/result-call/main.ssrg",
        "type Result<E, A> = | Failure E | Success A\n\
         fn fromValue value: Int -> Int =\n\
           match Success value {\n\
             Success item -> item\n\
             Failure _ -> 0\n\
           }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected fromValue function");
    };
    let TypedExpr::Match {
        arms, exhaustive, ..
    } = body
    else {
        panic!("expected typed match");
    };
    assert!(*exhaustive);
    let TypedPattern::Constructor {
        argument: Some(argument),
        ..
    } = &arms[0].pattern
    else {
        panic!("expected payload constructor pattern");
    };
    assert!(matches!(
        argument.as_ref(),
        TypedPattern::Binding { type_ref, .. } if type_ref == &named("Int")
    ));
    assert!(matches!(
        &arms[0].body,
        TypedExpr::Variable { type_ref, .. } if type_ref == &named("Int")
    ));
}

#[test]
fn invalid_scrutinee_suppresses_pattern_and_coverage_cascades() {
    let typed = type_module(
        "artifact/match-invalid/main.ssrg",
        "type Choice = | One | Two\nfn choose value: Choice -> Int = match missing { One -> 1 }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected choose function");
    };
    let TypedExpr::Match { exhaustive, .. } = body else {
        panic!("expected typed match");
    };
    assert!(!exhaustive);
}

#[test]
fn does_not_mark_an_invalid_unreachable_arm_set_as_total() {
    let typed = type_module(
        "artifact/match-unreachable/main.ssrg",
        "type Choice = | One | Two\nfn choose value: Choice -> Int = match value { _ -> 0; One -> 1 }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[1] else {
        panic!("expected choose function");
    };
    let TypedExpr::Match { exhaustive, .. } = body else {
        panic!("expected typed match");
    };
    assert!(!exhaustive);
}

#[test]
fn types_string_literal_patterns_and_keeps_the_catchall_total() {
    let typed = type_module(
        "artifact/string-pattern/main.ssrg",
        "fn isRock input: String -> Bool = match input { \"rock\" -> True; _ -> False }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected isRock function");
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
        TypedPattern::String { value, type_ref, .. }
            if value == "rock" && type_ref == &named("String")
    ));
}

#[test]
fn types_boolean_literal_patterns_as_a_finite_total_match() {
    let typed = type_module(
        "artifact/bool-pattern/main.ssrg",
        "fn invert value: Bool -> Bool = match value { True -> False; False -> True }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected invert function");
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
        TypedPattern::Boolean { value: true, .. }
    ));
}

#[test]
fn types_structural_record_patterns_and_their_bindings() {
    let typed = type_module(
        "artifact/record-pattern/main.ssrg",
        "fn render value: { label: String, name: String } -> String =\n\
           match value {\n\
             { label: \"Player\", name } -> name\n\
             { name } -> name\n\
           }\n",
    );

    let TypedDecl::Fn { body, .. } = &typed.declarations[0] else {
        panic!("expected render function");
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
        TypedPattern::Record { fields, .. }
            if matches!(&fields[0].pattern, TypedPattern::String { value, .. }
                if value == "Player")
                && matches!(&fields[1].pattern, TypedPattern::Binding { name, type_ref, .. }
                    if name == "name" && type_ref == &named("String"))
    ));
    assert!(matches!(
        &arms[1].pattern,
        TypedPattern::Record { fields, .. }
            if fields.len() == 1
                && matches!(&fields[0].pattern, TypedPattern::Binding { name, .. }
                    if name == "name")
    ));
}

#[test]
fn types_array_and_list_rest_patterns_as_exhaustive_matches() {
    let typed = type_module(
        "artifact/collection-pattern/main.ssrg",
        "fn arrayHead values: Array<Int> -> Int = match values { [] -> 0; [head, ...tail] -> head }\n\
         fn listHead values: List<Int> -> Int = match values { `[] -> 0; `[head, ...tail] -> head }\n",
    );

    for declaration in &typed.declarations {
        let TypedDecl::Fn { body, .. } = declaration else {
            panic!("expected function");
        };
        let TypedExpr::Match {
            arms, exhaustive, ..
        } = body
        else {
            panic!("expected match");
        };
        assert!(*exhaustive);
        match &arms[1].pattern {
            TypedPattern::Array {
                elements,
                rest: Some(rest),
                ..
            }
            | TypedPattern::List {
                elements,
                rest: Some(rest),
                ..
            } => {
                assert!(matches!(
                    &elements[0],
                    TypedPattern::Binding { name, type_ref, .. }
                        if name == "head" && type_ref == &named("Int")
                ));
                assert!(matches!(
                    rest.as_ref(),
                    TypedPattern::Binding { name, .. } if name == "tail"
                ));
            }
            pattern => panic!("expected collection rest pattern, received {pattern:?}"),
        }
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
