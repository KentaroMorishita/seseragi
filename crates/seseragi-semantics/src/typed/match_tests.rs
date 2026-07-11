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

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
