use super::*;
use crate::surface::parse_surface_ast;
use crate::surface_model::{SurfaceDecl, SurfaceDoItem, SurfacePattern};

fn first_body(source: &str) -> SurfaceExpr {
    let module = parse_surface_ast("main.ssrg", source);
    match &module.declarations[0] {
        SurfaceDecl::Let { body, .. }
        | SurfaceDecl::Fn { body, .. }
        | SurfaceDecl::EffectFn { body, .. } => body.clone().expect("valid declaration body"),
        declaration => panic!("declaration has no expression body: {declaration:?}"),
    }
}

#[test]
fn parses_curried_application_left_associatively_before_binary_operators() {
    let body = first_body("pub fn use value: Int -> Int = add value 1 + 2\n");

    let SurfaceExpr::Binary { left, right, .. } = body else {
        panic!("expected binary expression");
    };
    assert!(matches!(*right, SurfaceExpr::Integer { ref raw, .. } if raw == "2"));
    let SurfaceExpr::Application {
        function, argument, ..
    } = *left
    else {
        panic!("expected outer application");
    };
    assert!(matches!(*argument, SurfaceExpr::Integer { ref raw, .. } if raw == "1"));
    let SurfaceExpr::Application {
        function, argument, ..
    } = *function
    else {
        panic!("expected inner application");
    };
    assert!(matches!(*function, SurfaceExpr::Name { ref name, .. } if name == "add"));
    assert!(matches!(*argument, SurfaceExpr::Name { ref name, .. } if name == "value"));
}

#[test]
fn preserves_grouped_expression_boundaries() {
    let body = first_body("pub fn grouped value: Int -> Int = add (value + 1) 2\n");

    let SurfaceExpr::Application { function, .. } = body else {
        panic!("expected outer application");
    };
    let SurfaceExpr::Application { argument, .. } = *function else {
        panic!("expected application containing grouped argument");
    };
    assert!(matches!(
        *argument,
        SurfaceExpr::Grouped { value, .. }
            if matches!(*value, SurfaceExpr::Binary { ref operator, .. } if operator == "+")
    ));
}

#[test]
fn parses_tuple_values_without_losing_grouped_expressions() {
    let body = first_body("pub fn pair left: Int -> right: Bool -> (Int, Bool) = (left, right)\n");

    assert!(matches!(
        body,
        SurfaceExpr::Tuple { elements, .. }
            if elements.len() == 2
                && matches!(&elements[0], SurfaceExpr::Name { name, .. } if name == "left")
                && matches!(&elements[1], SurfaceExpr::Name { name, .. } if name == "right")
    ));
}

#[test]
fn keeps_a_tuple_as_one_application_argument() {
    let body =
        first_body("pub fn usePair left: Int -> right: Bool -> Int = consume (left, right)\n");

    assert!(matches!(
        body,
        SurfaceExpr::Application { argument, .. }
            if matches!(*argument, SurfaceExpr::Tuple { ref elements, .. } if elements.len() == 2)
    ));
}

#[test]
fn rejects_one_element_and_trailing_comma_tuples() {
    for source in ["pub let singleton = (1,)\n", "pub let trailing = (1, 2,)\n"] {
        let module = parse_surface_ast("main.ssrg", source);
        assert!(matches!(
            &module.declarations[0],
            SurfaceDecl::Let { body: None, .. }
        ));
    }
}

#[test]
fn parses_nested_tuple_patterns_in_do_bindings() {
    let body =
        first_body("effect fn main = do { (first, (_, second)) <- readPair (); succeed second }\n");

    let SurfaceExpr::Do { items, .. } = body else {
        panic!("expected do expression");
    };
    assert!(matches!(
        &items[0],
        SurfaceDoItem::Bind {
            pattern: SurfacePattern::Tuple { elements, .. },
            ..
        } if elements.len() == 2
            && matches!(&elements[0], SurfacePattern::Name { name, .. } if name == "first")
            && matches!(
                &elements[1],
                SurfacePattern::Tuple { elements, .. }
                    if matches!(&elements[0], SurfacePattern::Wildcard { .. })
                        && matches!(&elements[1], SurfacePattern::Name { name, .. } if name == "second")
            )
    ));
}

#[test]
fn distinguishes_constructor_patterns_from_bindings() {
    let body = first_body(
        "effect fn main = do { (Present value, Missing) <- readPair (); succeed value }\n",
    );

    let SurfaceExpr::Do { items, .. } = body else {
        panic!("expected do expression");
    };
    assert!(matches!(
        &items[0],
        SurfaceDoItem::Bind {
            pattern: SurfacePattern::Tuple { elements, .. },
            ..
        } if matches!(
            &elements[0],
            SurfacePattern::Constructor {
                name,
                argument: Some(argument),
                ..
            } if name == "Present"
                && matches!(argument.as_ref(), SurfacePattern::Name { name, .. } if name == "value")
        ) && matches!(
            &elements[1],
            SurfacePattern::Constructor {
                name,
                argument: None,
                ..
            } if name == "Missing"
        )
    ));
}

#[test]
fn parses_namespace_qualified_constructor_patterns() {
    let source = "fn render value: Label -> String = match value {\n  domain.Rock -> \"rock\"\n  domain.Just item -> item\n  (domain.Rock, domain.Just value) -> value\n}\n";
    let body = first_body(source);

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    let SurfacePattern::Constructor {
        name,
        name_span,
        argument,
        ..
    } = &arms[0].pattern
    else {
        panic!("expected qualified constructor pattern");
    };
    assert_eq!(name, "domain.Rock");
    assert_eq!(&source[name_span.start..name_span.end], "domain.Rock");
    assert!(argument.is_none());
    assert!(matches!(
        &arms[1].pattern,
        SurfacePattern::Constructor {
            name,
            argument: Some(argument),
            ..
        } if name == "domain.Just"
            && matches!(argument.as_ref(), SurfacePattern::Name { name, .. } if name == "item")
    ));
    assert!(matches!(
        &arms[2].pattern,
        SurfacePattern::Tuple { elements, .. }
            if matches!(
                &elements[0],
                SurfacePattern::Constructor { name, argument: None, .. }
                    if name == "domain.Rock"
            ) && matches!(
                &elements[1],
                SurfacePattern::Constructor {
                    name,
                    argument: Some(argument),
                    ..
                } if name == "domain.Just"
                    && matches!(
                        argument.as_ref(),
                        SurfacePattern::Name { name, .. } if name == "value"
                    )
            )
    ));
}

#[test]
fn recovers_malformed_qualified_constructor_patterns() {
    let body = first_body(
        "fn recover value: Label -> String = match value {\n  domain. -> \"missing\"\n  domain.rock -> \"lower\"\n  domain.Rock.More -> \"nested\"\n  Domain.Rock -> \"upper alias\"\n  domain.Rock -> \"ok\"\n}\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert_eq!(arms.len(), 5);
    assert!(arms[..4]
        .iter()
        .all(|arm| matches!(arm.pattern, SurfacePattern::Error { .. })));
    assert!(matches!(
        &arms[4].pattern,
        SurfacePattern::Constructor {
            name,
            argument: None,
            ..
        } if name == "domain.Rock"
    ));
}

#[test]
fn permits_a_line_break_after_low_precedence_application() {
    let body = first_body("effect fn greet = println $\n  \"hello\"\n");

    assert!(matches!(
        body,
        SurfaceExpr::Application { function, argument, .. }
            if matches!(*function, SurfaceExpr::Name { ref name, .. } if name == "println")
                && matches!(*argument, SurfaceExpr::String { ref raw, .. } if raw == "\"hello\"")
    ));
}

#[test]
fn separates_do_items_from_the_final_result_with_semicolons() {
    let body = first_body("effect fn main = do { print \"loading\"; println \"done\" }\n");

    let SurfaceExpr::Do { items, result, .. } = body else {
        panic!("expected do expression");
    };
    assert_eq!(items.len(), 1);
    assert!(matches!(items[0], SurfaceDoItem::Expression { .. }));
    assert!(matches!(
        result.as_deref(),
        Some(SurfaceExpr::Application { function, .. })
            if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "println")
    ));
}

#[test]
fn distinguishes_do_bind_let_and_final_result() {
    let body = first_body(
        "effect fn main =\n  do {\n    line <- readLine ()\n    let kept = line\n    succeed kept\n  }\n",
    );

    let SurfaceExpr::Do { items, result, .. } = body else {
        panic!("expected do expression");
    };
    assert_eq!(items.len(), 2);
    assert!(matches!(
        &items[0],
        SurfaceDoItem::Bind {
            pattern: SurfacePattern::Name { name, .. },
            ..
        } if name == "line"
    ));
    assert!(matches!(
        &items[1],
        SurfaceDoItem::Let {
            pattern: SurfacePattern::Name { name, .. },
            ..
        } if name == "kept"
    ));
    assert!(result.is_some());
}

#[test]
fn parses_rps_match_with_tuple_constructor_patterns_and_wildcard() {
    let body = first_body(
        "fn decide first: Hand -> second: Hand -> Outcome =\n  match (first, second) {\n    (Rock, Rock) -> Draw\n    (Rock, Scissors) -> Player1Wins\n    (Paper, Rock) -> Player1Wins\n    _ -> Player2Wins\n  }\n",
    );

    let SurfaceExpr::Match {
        scrutinee, arms, ..
    } = body
    else {
        panic!("expected match expression");
    };
    assert!(matches!(*scrutinee, SurfaceExpr::Tuple { ref elements, .. } if elements.len() == 2));
    assert_eq!(arms.len(), 4);
    assert!(matches!(
        &arms[0].pattern,
        SurfacePattern::Tuple { elements, .. }
            if matches!(&elements[0], SurfacePattern::Constructor { name, argument: None, .. } if name == "Rock")
                && matches!(&elements[1], SurfacePattern::Constructor { name, argument: None, .. } if name == "Rock")
    ));
    assert!(matches!(arms[3].pattern, SurfacePattern::Wildcard { .. }));
    assert!(matches!(arms[3].body, SurfaceExpr::Name { ref name, .. } if name == "Player2Wins"));
}

#[test]
fn preserves_literal_patterns_and_nested_tuple_positions() {
    let body = first_body(
        "fn accepts value: (Int, String, Bool) -> Bool =\n  match value {\n    (42, \"go\", True) -> True\n    (0, \"stop\", False) -> False\n    _ -> False\n  }\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert!(matches!(
        &arms[0].pattern,
        SurfacePattern::Tuple { elements, .. }
            if matches!(&elements[0], SurfacePattern::Integer { raw, .. } if raw == "42")
                && matches!(&elements[1], SurfacePattern::String { raw, .. } if raw == "\"go\"")
                && matches!(&elements[2], SurfacePattern::Boolean { value: true, .. })
    ));
    assert!(matches!(
        &arms[1].pattern,
        SurfacePattern::Tuple { elements, .. }
            if matches!(&elements[0], SurfacePattern::Integer { raw, .. } if raw == "0")
                && matches!(&elements[1], SurfacePattern::String { raw, .. } if raw == "\"stop\"")
                && matches!(&elements[2], SurfacePattern::Boolean { value: false, .. })
    ));
}

#[test]
fn preserves_match_payload_binding_and_guard() {
    let body = first_body(
        "fn render value: Label -> String =\n  match value {\n    Present item when ready -> item\n    Missing -> \"missing\"\n  }\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert!(matches!(
        &arms[0].pattern,
        SurfacePattern::Constructor {
            name,
            argument: Some(argument),
            ..
        } if name == "Present"
            && matches!(argument.as_ref(), SurfacePattern::Name { name, .. } if name == "item")
    ));
    assert!(matches!(
        arms[0].guard,
        Some(SurfaceExpr::Name { ref name, .. }) if name == "ready"
    ));
    assert!(matches!(arms[0].body, SurfaceExpr::Name { ref name, .. } if name == "item"));
}

#[test]
fn keeps_malformed_match_arms_as_explicit_error_nodes() {
    let body = first_body(
        "fn recover value: Label -> String = match value {\n  Missing\n  Present item -> item\n  Missing ->;\n  _ -> \"ok\"\n}\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert_eq!(arms.len(), 4);
    assert!(matches!(arms[0].body, SurfaceExpr::Error { .. }));
    assert!(matches!(arms[1].body, SurfaceExpr::Name { .. }));
    assert!(matches!(arms[2].body, SurfaceExpr::Error { .. }));
    assert!(matches!(arms[3].body, SurfaceExpr::String { .. }));
}

#[test]
fn recovers_a_missing_match_arm_body_before_following_arms() {
    let body = first_body(
        "fn recover value: Label -> String = match value {\n  Missing ->\n  Present item -> item\n  Missing -> \"missing\"\n}\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert_eq!(arms.len(), 3);
    assert!(matches!(arms[0].body, SurfaceExpr::Error { .. }));
    assert!(matches!(arms[1].body, SurfaceExpr::Name { ref name, .. } if name == "item"));
    assert!(matches!(arms[2].body, SurfaceExpr::String { .. }));
}

#[test]
fn keeps_a_leading_pipeline_in_the_previous_match_arm_body() {
    let body = first_body(
        "fn render value: Label -> String = match value {\n  Present item -> item\n    |> normalize\n  Missing -> \"missing\"\n}\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    assert_eq!(arms.len(), 2);
    assert!(matches!(
        arms[0].body,
        SurfaceExpr::Binary { ref operator, .. } if operator == "|>"
    ));
}
