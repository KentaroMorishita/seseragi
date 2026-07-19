use super::*;

use crate::surface_model::TypeRecordField;

#[test]
fn parses_nested_type_arguments_in_surface_ast() {
    let module = parse_surface_ast("main.ssrg", "pub let values: Array<Maybe<Int>> = []\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "values".to_owned(),
            name_span: ByteSpan { start: 8, end: 14 },
            type_ref: Some(TypeRef::Named {
                name: "Array".to_owned(),
                arguments: vec![TypeRef::Named {
                    name: "Maybe".to_owned(),
                    arguments: vec![TypeRef::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 28, end: 31 },
                    }],
                    span: ByteSpan { start: 22, end: 32 },
                }],
                span: ByteSpan { start: 16, end: 33 },
            }),
            body: Some(SurfaceExpr::Array {
                elements: Vec::new(),
                span: ByteSpan { start: 36, end: 38 },
            }),
            span: ByteSpan { start: 0, end: 38 },
        }
    );
}

#[test]
fn ignores_record_value_colons_after_an_unannotated_let_equals() {
    let module = parse_surface_ast(
        "main.ssrg",
        "let cardStyle = html.style { variables: { cardShadow: \"soft\" }, padding: \"24px\" }\n",
    );

    let SurfaceDecl::Let { type_ref, body, .. } = &module.declarations[0] else {
        panic!("expected let declaration");
    };
    assert!(type_ref.is_none());
    assert!(body.is_some());
}

#[test]
fn parses_qualified_type_names_in_surface_ast() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub let counts: maps.Map<String, Int> = value\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "counts".to_owned(),
            name_span: ByteSpan { start: 8, end: 14 },
            type_ref: Some(TypeRef::Named {
                name: "maps.Map".to_owned(),
                arguments: vec![
                    TypeRef::Named {
                        name: "String".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 25, end: 31 },
                    },
                    TypeRef::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 33, end: 36 },
                    },
                ],
                span: ByteSpan { start: 16, end: 37 },
            }),
            body: Some(SurfaceExpr::Name {
                name: "value".to_owned(),
                span: ByteSpan { start: 40, end: 45 },
            }),
            span: ByteSpan { start: 0, end: 45 },
        }
    );
}

#[test]
fn parses_record_type_references_in_surface_ast() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub let env: { console: Console, clock?: Clock } = config\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "env".to_owned(),
            name_span: ByteSpan { start: 8, end: 11 },
            type_ref: Some(TypeRef::Record {
                closed: true,
                fields: vec![
                    TypeRecordField {
                        name: "console".to_owned(),
                        name_span: ByteSpan { start: 15, end: 22 },
                        optional: false,
                        type_ref: TypeRef::Named {
                            name: "Console".to_owned(),
                            arguments: Vec::new(),
                            span: ByteSpan { start: 24, end: 31 },
                        },
                    },
                    TypeRecordField {
                        name: "clock".to_owned(),
                        name_span: ByteSpan { start: 33, end: 38 },
                        optional: true,
                        type_ref: TypeRef::Named {
                            name: "Clock".to_owned(),
                            arguments: Vec::new(),
                            span: ByteSpan { start: 41, end: 46 },
                        },
                    },
                ],
                span: ByteSpan { start: 13, end: 48 },
            }),
            body: Some(SurfaceExpr::Name {
                name: "config".to_owned(),
                span: ByteSpan { start: 51, end: 57 },
            }),
            span: ByteSpan { start: 0, end: 57 },
        }
    );
}

#[test]
fn parses_tuple_type_references_in_surface_ast() {
    let module = parse_surface_ast("main.ssrg", "pub let pair: (String, Int) = value\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "pair".to_owned(),
            name_span: ByteSpan { start: 8, end: 12 },
            type_ref: Some(TypeRef::Tuple {
                elements: vec![
                    TypeRef::Named {
                        name: "String".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 15, end: 21 },
                    },
                    TypeRef::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 23, end: 26 },
                    },
                ],
                span: ByteSpan { start: 14, end: 27 },
            }),
            body: Some(SurfaceExpr::Name {
                name: "value".to_owned(),
                span: ByteSpan { start: 30, end: 35 },
            }),
            span: ByteSpan { start: 0, end: 35 },
        }
    );
}

#[test]
fn parses_function_type_references_in_surface_ast() {
    let module = parse_surface_ast("main.ssrg", "pub let mapper: (String -> Int) = value\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "mapper".to_owned(),
            name_span: ByteSpan { start: 8, end: 14 },
            type_ref: Some(TypeRef::Function {
                parameter: Box::new(TypeRef::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 17, end: 23 },
                }),
                result: Box::new(TypeRef::Named {
                    name: "Int".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 27, end: 30 },
                }),
                span: ByteSpan { start: 17, end: 30 },
            }),
            body: Some(SurfaceExpr::Name {
                name: "value".to_owned(),
                span: ByteSpan { start: 34, end: 39 },
            }),
            span: ByteSpan { start: 0, end: 39 },
        }
    );
}

#[test]
fn parses_type_holes_in_partial_type_constructors() {
    let module = parse_surface_ast("main.ssrg", "instance<E> Functor<Either<E, _>> {\n}\n");

    let SurfaceDecl::Instance { arguments, .. } = &module.declarations[0] else {
        panic!("expected instance declaration");
    };

    assert_eq!(
        arguments,
        &vec![TypeRef::Named {
            name: "Either".to_owned(),
            arguments: vec![
                TypeRef::Named {
                    name: "E".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 27, end: 28 },
                },
                TypeRef::Hole {
                    span: ByteSpan { start: 30, end: 31 },
                },
            ],
            span: ByteSpan { start: 20, end: 32 },
        }]
    );
}
