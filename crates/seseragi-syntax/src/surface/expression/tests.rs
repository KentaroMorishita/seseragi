use super::*;
use crate::surface::parse_surface_ast;
use crate::surface_model::{
    SurfaceDecl, SurfaceDoItem, SurfacePattern, SurfaceRecordItem, SurfaceRecordPatternField,
};

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
fn parses_template_text_and_interpolated_expressions() {
    let source = "pub fn greet name: String -> String = `Hello, ${name |> identity}!`\n";
    let body = first_body(source);

    let SurfaceExpr::Template { parts, .. } = body else {
        panic!("expected template expression");
    };
    assert!(matches!(
        &parts[0],
        crate::SurfaceTemplatePart::Text { value, .. } if value == "Hello, "
    ));
    assert!(matches!(
        &parts[1],
        crate::SurfaceTemplatePart::Interpolation { value, .. }
            if matches!(value.as_ref(), SurfaceExpr::Application { .. })
    ));
    assert!(matches!(
        &parts[2],
        crate::SurfaceTemplatePart::Text { value, .. } if value == "!"
    ));
}

#[test]
fn parses_multiline_templates_and_literal_interpolation_markers() {
    let body = first_body("pub let message: String = `first\\${value}\r\nsecond ${\"ok\"}`\n");

    let SurfaceExpr::Template { parts, .. } = body else {
        panic!("expected template expression");
    };
    assert!(matches!(
        &parts[0],
        crate::SurfaceTemplatePart::Text { value, .. }
            if value == "first${value}\nsecond "
    ));
    assert!(matches!(
        &parts[1],
        crate::SurfaceTemplatePart::Interpolation { value, .. }
            if matches!(value.as_ref(), SurfaceExpr::String { raw, .. } if raw == "\"ok\"")
    ));
}

#[test]
fn parses_record_literals_with_explicit_and_shorthand_fields() {
    let body = first_body(
        "pub fn profile name: String -> score: Int -> { name: String, score: Int } = { name, score: score }\n",
    );

    let SurfaceExpr::Record { items, .. } = body else {
        panic!("expected record expression");
    };
    assert_eq!(items.len(), 2);
    assert!(matches!(
        &items[0],
        SurfaceRecordItem::Field { name, value: SurfaceExpr::Name { name: value, .. }, .. }
            if name == "name" && value == "name"
    ));
    assert!(matches!(
        &items[1],
        SurfaceRecordItem::Field { name, value: SurfaceExpr::Name { name: value, .. }, .. }
            if name == "score" && value == "score"
    ));
}

#[test]
fn parses_record_spread_in_source_order() {
    let body = first_body(
        "pub fn relabel base: { label: String } -> { label: String } = { ...base, label: \"next\" }\n",
    );

    let SurfaceExpr::Record { items, .. } = body else {
        panic!("expected record expression");
    };
    assert!(matches!(
        &items[0],
        SurfaceRecordItem::Spread { value, .. }
            if matches!(value, SurfaceExpr::Name { name, .. } if name == "base")
    ));
    assert!(matches!(
        &items[1],
        SurfaceRecordItem::Field { name, .. } if name == "label"
    ));
}

#[test]
fn parses_nominal_struct_values_and_patterns() {
    let module = parse_surface_ast(
        "main.ssrg",
        "fn rename user: User -> User = User { ...user, name: \"Mio\" }\n\
         fn display user: User -> String = match user { User { name } -> name }\n",
    );
    let SurfaceDecl::Fn { body, .. } = &module.declarations[0] else {
        panic!("expected rename function");
    };
    assert!(matches!(
        body,
        Some(SurfaceExpr::Struct { name, items, .. })
            if name == "User" && items.len() == 2
    ));
    let SurfaceDecl::Fn { body, .. } = &module.declarations[1] else {
        panic!("expected display function");
    };
    let Some(SurfaceExpr::Match { arms, .. }) = body else {
        panic!("expected match");
    };
    assert!(matches!(
        &arms[0].pattern,
        SurfacePattern::Struct { name, fields, .. }
            if name == "User" && fields.len() == 1
    ));
}

#[test]
fn parses_explicit_struct_type_arguments() {
    let body = first_body("pub let boxed = Box<String> { value: \"hello\" }\n");

    let SurfaceExpr::Struct {
        name,
        type_arguments: Some(type_arguments),
        items,
        ..
    } = body
    else {
        panic!("expected explicitly typed struct expression");
    };
    assert_eq!(name, "Box");
    assert_eq!(items.len(), 1);
    assert!(matches!(
        type_arguments.as_slice(),
        [crate::TypeRef::Named { name, arguments, .. }]
            if name == "String" && arguments.is_empty()
    ));
}

#[test]
fn keeps_uppercase_comparisons_when_type_arguments_are_not_followed_by_a_struct_body() {
    let body = first_body("pub let compared = Left < Right > Value\n");

    assert!(matches!(
        body,
        SurfaceExpr::Binary {
            operator,
            left,
            right,
            ..
        } if operator == ">"
            && matches!(left.as_ref(), SurfaceExpr::Binary { operator, .. } if operator == "<")
            && matches!(right.as_ref(), SurfaceExpr::Name { name, .. } if name == "Value")
    ));
}

#[test]
fn parses_required_record_field_access_without_baking_in_namespace_resolution() {
    let body = first_body("pub fn name user: { name: String } -> String = user.name\n");

    assert!(matches!(
        body,
        SurfaceExpr::Member { receiver, field, .. }
            if field == "name"
                && matches!(receiver.as_ref(), SurfaceExpr::Name { name, .. } if name == "user")
    ));
}

#[test]
fn rejects_invalid_template_escapes_instead_of_erasing_text() {
    let body = first_body(
        r#"pub let message: String = `bad\qescape`
"#,
    );

    assert!(matches!(body, SurfaceExpr::Error { .. }));
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
fn parses_an_arithmetic_operator_as_a_grouped_function_value() {
    let body = first_body("pub let add: (Int -> Int -> Int) = (+)\n");

    assert!(matches!(
        body,
        SurfaceExpr::Grouped { value, .. }
            if matches!(*value, SurfaceExpr::Name { ref name, .. } if name == "+")
    ));
}

#[test]
fn parses_annotated_and_curried_lambdas_as_nested_expressions() {
    let body = first_body("fn answer -> Int = combine (\\first: Int second -> first + second)\n");
    let SurfaceExpr::Application { argument, .. } = body else {
        panic!("expected lambda argument")
    };
    let SurfaceExpr::Grouped { value, .. } = argument.as_ref() else {
        panic!("expected grouped lambda")
    };
    let SurfaceExpr::Lambda {
        parameter: first,
        body,
        ..
    } = value.as_ref()
    else {
        panic!("expected outer lambda")
    };
    assert_eq!(first.name, "first");
    assert!(matches!(
        first.type_ref,
        Some(crate::TypeRef::Named { ref name, .. }) if name == "Int"
    ));
    let SurfaceExpr::Lambda {
        parameter: second,
        body,
        ..
    } = body.as_ref()
    else {
        panic!("expected inner lambda")
    };
    assert_eq!(second.name, "second");
    assert!(second.type_ref.is_none());
    assert!(matches!(body.as_ref(), SurfaceExpr::Binary { operator, .. } if operator == "+"));
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
fn continues_multiline_pipelines_in_do_bind_let_and_result() {
    let body = first_body(
        "effect fn main =\n  do {\n    input <-\n      readLine ()\n      |> mapError StdinFailure\n    let parsed =\n      input\n      |> parseInput\n    parsed\n    |> fromEither\n  }\n",
    );

    let SurfaceExpr::Do { items, result, .. } = body else {
        panic!("expected do expression");
    };
    assert_eq!(items.len(), 2);

    let SurfaceDoItem::Bind { value, .. } = &items[0] else {
        panic!("expected bind item");
    };
    assert!(matches!(
        value,
        SurfaceExpr::Application { function, .. }
            if matches!(
                function.as_ref(),
                SurfaceExpr::Application { function, .. }
                    if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "mapError")
            )
    ));

    let SurfaceDoItem::Let { value, .. } = &items[1] else {
        panic!("expected pure let item");
    };
    assert!(matches!(
        value,
        SurfaceExpr::Application { function, .. }
            if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "parseInput")
    ));
    assert!(matches!(
        result.as_deref(),
        Some(SurfaceExpr::Application { function, .. })
            if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "fromEither")
    ));
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
fn parses_record_patterns_with_shorthand_explicit_and_optional_fields() {
    let body = first_body(
        "fn describe value: { status: String, name: String, score?: Int } -> String =\n  match value {\n    { status: \"ok\", name, score? } -> name\n    _ -> \"missing\"\n  }\n",
    );

    let SurfaceExpr::Match { arms, .. } = body else {
        panic!("expected match expression");
    };
    let SurfacePattern::Record { fields, .. } = &arms[0].pattern else {
        panic!("expected record pattern");
    };
    assert_eq!(fields.len(), 3);
    assert!(matches!(
        &fields[0],
        SurfaceRecordPatternField {
            name,
            optional: false,
            pattern: SurfacePattern::String { raw, .. },
            ..
        } if name == "status" && raw == "\"ok\""
    ));
    assert!(matches!(
        &fields[1],
        SurfaceRecordPatternField {
            name,
            optional: false,
            pattern: SurfacePattern::Name { name: binding, .. },
            ..
        } if name == "name" && binding == "name"
    ));
    assert!(matches!(
        &fields[2],
        SurfaceRecordPatternField {
            name,
            optional: true,
            pattern: SurfacePattern::Name { name: binding, .. },
            ..
        } if name == "score" && binding == "score"
    ));
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
        SurfaceExpr::Application { ref function, ref argument, .. }
            if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "normalize")
                && matches!(argument.as_ref(), SurfaceExpr::Name { name, .. } if name == "item")
    ));
}

#[test]
fn desugars_pipeline_chains_into_left_associative_application() {
    let body = first_body("fn calculate value: Int -> Int = value |> add 5 |> double\n");

    assert!(matches!(
        body,
        SurfaceExpr::Application { function, argument, .. }
            if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "double")
                && matches!(
                    argument.as_ref(),
                    SurfaceExpr::Application { function, argument, .. }
                        if matches!(
                            function.as_ref(),
                            SurfaceExpr::Application { function, argument, .. }
                                if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "add")
                                    && matches!(argument.as_ref(), SurfaceExpr::Integer { raw, .. } if raw == "5")
                        ) && matches!(argument.as_ref(), SurfaceExpr::Name { name, .. } if name == "value")
                )
    ));
}

#[test]
fn desugars_functor_applicative_and_monad_operators_to_trait_methods() {
    let mapped = first_body("fn mapped value: Maybe<Int> -> Maybe<Int> = increment <$> value\n");
    assert_trait_operator_application(&mapped, "map", "increment", "value");

    let applied = first_body("fn applied value: Maybe<Int> -> Maybe<Int> = wrapped <*> value\n");
    assert_trait_operator_application(&applied, "apply", "wrapped", "value");

    let bound = first_body("fn bound value: Maybe<Int> -> Maybe<Int> = value >>= incrementMaybe\n");
    assert_trait_operator_application(&bound, "flatMap", "incrementMaybe", "value");
}

#[test]
fn retains_custom_infix_sequences_as_one_flat_chain() {
    let source = "fn combine left: Int -> middle: Int -> right: Int -> tail: Int -> Int = left + middle <+> right * 2 <**> tail\n";
    let body = first_body(source);
    let SurfaceExpr::InfixChain { first, steps, span } = body else {
        panic!("expected unresolved infix chain");
    };

    assert!(matches!(
        first.as_ref(),
        SurfaceExpr::Name { name, .. } if name == "left"
    ));
    assert_eq!(
        steps
            .iter()
            .map(|step| step.operator.as_str())
            .collect::<Vec<_>>(),
        ["+", "<+>", "*", "<**>"]
    );
    assert!(matches!(
        &steps[0].operand,
        SurfaceExpr::Name { name, .. } if name == "middle"
    ));
    assert!(matches!(
        &steps[2].operand,
        SurfaceExpr::Integer { raw, .. } if raw == "2"
    ));
    let custom_start = source.find("<+>").unwrap();
    assert_eq!(
        steps[1].operator_span,
        ByteSpan {
            start: custom_start,
            end: custom_start + 3,
        }
    );
    assert_eq!(span.start, source.find("left +").unwrap());
    assert_eq!(span.end, source.rfind("tail").unwrap() + "tail".len());
}

#[test]
fn rejoins_dot_inside_a_custom_infix_chain() {
    let body = first_body("fn compose left: Int -> right: Int -> Int = left <.> right\n");
    let SurfaceExpr::InfixChain { steps, .. } = body else {
        panic!("expected unresolved infix chain");
    };

    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].operator, "<.>");
}

#[test]
fn keeps_multiline_monad_bind_in_one_expression() {
    let body =
        first_body("fn bound value: Maybe<Int> -> Maybe<Int> =\n  value\n  >>= incrementMaybe\n");

    assert_trait_operator_application(&body, "flatMap", "incrementMaybe", "value");
}

fn assert_trait_operator_application(
    expression: &SurfaceExpr,
    method: &str,
    first: &str,
    second: &str,
) {
    assert!(matches!(
        expression,
        SurfaceExpr::Application { function, argument, .. }
            if matches!(argument.as_ref(), SurfaceExpr::Name { name, .. } if name == second)
                && matches!(
                    function.as_ref(),
                    SurfaceExpr::Application { function, argument, .. }
                        if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == method)
                            && matches!(argument.as_ref(), SurfaceExpr::Name { name, .. } if name == first)
                )
    ));
}

#[test]
fn low_precedence_application_wraps_a_pipeline_chain() {
    let body = first_body("fn renderAnswer value: Int -> String = render $ value |> double\n");

    assert!(matches!(
        body,
        SurfaceExpr::Application { function, argument, .. }
            if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "render")
                && matches!(
                    argument.as_ref(),
                    SurfaceExpr::Application { function, argument, .. }
                        if matches!(function.as_ref(), SurfaceExpr::Name { name, .. } if name == "double")
                            && matches!(argument.as_ref(), SurfaceExpr::Name { name, .. } if name == "value")
                )
    ));
}

#[test]
fn parses_empty_nested_and_trailing_comma_array_literals() {
    let body = first_body("fn arrays -> Array<Array<Int>> = [[], [1, 2,],]\n");

    let SurfaceExpr::Array { elements, .. } = body else {
        panic!("expected array expression");
    };
    assert_eq!(elements.len(), 2);
    assert!(matches!(&elements[0], SurfaceExpr::Array { elements, .. } if elements.is_empty()));
    assert!(matches!(&elements[1], SurfaceExpr::Array { elements, .. } if elements.len() == 2));

    let mixed = first_body("fn mixed -> Array<Int> = [1, \"bad\"]\n");
    assert!(matches!(mixed, SurfaceExpr::Array { elements, .. } if elements.len() == 2));
}

#[test]
fn parses_persistent_list_literals_without_confusing_templates() {
    let body = first_body("fn values -> List<Int> = `[1, 2, 3]\n");

    assert!(matches!(
        body,
        SurfaceExpr::List { elements, .. } if elements.len() == 3
    ));
}

#[test]
fn parses_array_comprehension_generators_and_guards() {
    let body =
        first_body("fn squares limit: Int -> Array<Int> = [n * n | n <- 1..=limit, n % 2 == 0]\n");

    let SurfaceExpr::ArrayComprehension {
        element, clauses, ..
    } = body
    else {
        panic!("expected array comprehension");
    };
    assert!(matches!(
        element.as_ref(),
        SurfaceExpr::Binary { operator, .. } if operator == "*"
    ));
    assert!(matches!(
        &clauses[0],
        crate::SurfaceComprehensionClause::Generator { pattern, source, .. }
            if matches!(pattern, SurfacePattern::Name { name, .. } if name == "n")
                && matches!(source, SurfaceExpr::Binary { operator, .. } if operator == "..=")
    ));
    assert!(matches!(
        &clauses[1],
        crate::SurfaceComprehensionClause::Guard { condition, .. }
            if matches!(condition, SurfaceExpr::Binary { operator, .. } if operator == "==")
    ));
}

#[test]
fn parses_list_comprehensions_with_the_same_clause_grammar() {
    let body = first_body(
        "fn squares values: List<Int> -> List<Int> = `[value * value | value <- values, value % 2 == 1]\n",
    );

    let SurfaceExpr::ListComprehension {
        element, clauses, ..
    } = body
    else {
        panic!("expected list comprehension");
    };
    assert!(matches!(
        element.as_ref(),
        SurfaceExpr::Binary { operator, .. } if operator == "*"
    ));
    assert_eq!(clauses.len(), 2);
}

#[test]
fn parses_multiline_comprehension_with_multiple_generators() {
    let body = first_body(
        "fn pairs limit: Int -> Array<(Int, Int)> = [\n  (left, right)\n  | left <- 1..=limit,\n    right <- [left, limit],\n    left < right\n]\n",
    );

    let SurfaceExpr::ArrayComprehension { clauses, .. } = body else {
        panic!("expected array comprehension");
    };
    assert_eq!(clauses.len(), 3);
    assert!(matches!(
        &clauses[1],
        crate::SurfaceComprehensionClause::Generator { source, .. }
            if matches!(source, SurfaceExpr::Array { elements, .. } if elements.len() == 2)
    ));
}

#[test]
fn parses_exclusive_and_inclusive_range_operators() {
    for (source, expected) in [
        ("fn values -> Range<Int> = 1..10\n", ".."),
        ("fn values -> Range<Int> = 1..=10\n", "..="),
    ] {
        let body = first_body(source);
        assert!(matches!(
            body,
            SurfaceExpr::Binary { operator, left, right, .. }
                if operator == expected
                    && matches!(left.as_ref(), SurfaceExpr::Integer { raw, .. } if raw == "1")
                    && matches!(right.as_ref(), SurfaceExpr::Integer { raw, .. } if raw == "10")
        ));
    }
}

#[test]
fn range_binds_between_comparison_and_arithmetic() {
    let body = first_body("fn contains -> Bool = 1 + 1..=10 == 2..=10\n");

    assert!(matches!(
        body,
        SurfaceExpr::Binary { operator, left, right, .. }
            if operator == "=="
                && matches!(
                    left.as_ref(),
                    SurfaceExpr::Binary { operator, left, .. }
                        if operator == "..="
                            && matches!(
                                left.as_ref(),
                                SurfaceExpr::Binary { operator, .. } if operator == "+"
                            )
                )
                && matches!(
                    right.as_ref(),
                    SurfaceExpr::Binary { operator, .. } if operator == "..="
                )
    ));
}
