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
