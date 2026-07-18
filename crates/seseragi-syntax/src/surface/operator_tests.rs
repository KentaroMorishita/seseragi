use super::*;

#[test]
fn parses_operator_type_parameters() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub operator<A> infixr 5 <+>\n  left: A -> right: A -> A =\n  left\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Operator {
            visibility: Visibility::Public,
            type_parameters: vec![crate::TypeParameter::value("A")],
            fixity: "infixr".to_owned(),
            precedence: 5,
            spelling: "<+>".to_owned(),
            spelling_span: ByteSpan { start: 25, end: 28 },
            parameters: vec![
                SurfaceParameter {
                    name: "left".to_owned(),
                    name_span: ByteSpan { start: 31, end: 35 },
                    type_ref: TypeRef::Named {
                        name: "A".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 37, end: 38 },
                    },
                },
                SurfaceParameter {
                    name: "right".to_owned(),
                    name_span: ByteSpan { start: 42, end: 47 },
                    type_ref: TypeRef::Named {
                        name: "A".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 49, end: 50 },
                    },
                },
            ],
            return_type: TypeRef::Named {
                name: "A".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 54, end: 55 },
            },
            constraints: Vec::new(),
            body: Some(SurfaceExpr::Name {
                name: "left".to_owned(),
                span: ByteSpan { start: 60, end: 64 },
            }),
            span: ByteSpan { start: 0, end: 64 },
        }
    );
}

#[test]
fn retains_operator_spelling_span_and_body_expression() {
    let source = "operator infixl 4 <+> left: Int -> right: Int -> Int = left + right\n";
    let module = parse_surface_ast("main.ssrg", source);

    assert!(matches!(
        &module.declarations[0],
        SurfaceDecl::Operator {
            spelling,
            spelling_span: ByteSpan { start: 18, end: 21 },
            body: Some(SurfaceExpr::Binary { operator, .. }),
            ..
        } if spelling == "<+>" && operator == "+"
    ));
}

#[test]
fn rejoins_dot_inside_a_custom_operator_spelling() {
    let source = "operator infixl 4 <.> left: Int -> right: Int -> Int = left\n";
    let module = parse_surface_ast("main.ssrg", source);

    assert!(matches!(
        &module.declarations[0],
        SurfaceDecl::Operator {
            spelling,
            spelling_span: ByteSpan { start: 18, end: 21 },
            ..
        } if spelling == "<.>"
    ));
}

#[test]
fn parses_operator_constraints() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub operator<A> infixr 5 <+>\n  left: A -> right: A -> A\nwhere Semigroup<A> =\n  left\n",
    );

    let SurfaceDecl::Operator {
        constraints, span, ..
    } = &module.declarations[0]
    else {
        panic!("expected operator declaration");
    };

    assert_eq!(constraints[0].name, "Semigroup");
    assert!(matches!(
        constraints[0].arguments.as_slice(),
        [TypeRef::Named { name, .. }] if name == "A"
    ));
    assert_eq!(*span, ByteSpan { start: 0, end: 83 });
}
