use super::*;

use crate::surface_model::{SurfaceField, SurfaceVariant};

#[test]
fn parses_alias_declarations() {
    let module = parse_surface_ast("main.ssrg", "pub alias Boxed<A> = Box<A>\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Alias {
            visibility: Visibility::Public,
            name: "Boxed".to_owned(),
            name_span: ByteSpan { start: 10, end: 15 },
            type_parameters: vec!["A".to_owned()],
            target: TypeRef::Named {
                name: "Box".to_owned(),
                arguments: vec![TypeRef::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 25, end: 26 },
                }],
                span: ByteSpan { start: 21, end: 27 },
            },
            span: ByteSpan { start: 0, end: 27 },
        }
    );
}

#[test]
fn parses_type_declarations() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub type Maybe<A> =\n  | Nothing\n  | Just A\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Type {
            visibility: Visibility::Public,
            opaque: false,
            name: "Maybe".to_owned(),
            name_span: ByteSpan { start: 9, end: 14 },
            type_parameters: vec!["A".to_owned()],
            deriving: Vec::new(),
            variants: vec![
                SurfaceVariant {
                    name: "Nothing".to_owned(),
                    name_span: ByteSpan { start: 24, end: 31 },
                    payload: None,
                },
                SurfaceVariant {
                    name: "Just".to_owned(),
                    name_span: ByteSpan { start: 36, end: 40 },
                    payload: Some(TypeRef::Named {
                        name: "A".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 41, end: 42 },
                    }),
                },
            ],
            span: ByteSpan { start: 0, end: 42 },
        }
    );
}

#[test]
fn parses_deriving_clauses_on_nominal_types() {
    let module = parse_surface_ast(
        "main.ssrg",
        "type Color deriving Eq, Ord, Show =\n  | Red\n  | Blue\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Type {
            visibility: Visibility::Private,
            opaque: false,
            name: "Color".to_owned(),
            name_span: ByteSpan { start: 5, end: 10 },
            type_parameters: Vec::new(),
            deriving: vec!["Eq".to_owned(), "Ord".to_owned(), "Show".to_owned()],
            variants: vec![
                SurfaceVariant {
                    name: "Red".to_owned(),
                    name_span: ByteSpan { start: 40, end: 43 },
                    payload: None,
                },
                SurfaceVariant {
                    name: "Blue".to_owned(),
                    name_span: ByteSpan { start: 48, end: 52 },
                    payload: None,
                },
            ],
            span: ByteSpan { start: 0, end: 52 },
        }
    );
}

#[test]
fn parses_opaque_nominal_declarations() {
    let module = parse_surface_ast("main.ssrg", "pub opaque newtype UserId = Int\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Newtype {
            visibility: Visibility::Public,
            opaque: true,
            name: "UserId".to_owned(),
            name_span: ByteSpan { start: 19, end: 25 },
            type_parameters: Vec::new(),
            deriving: Vec::new(),
            representation: TypeRef::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 28, end: 31 },
            },
            span: ByteSpan { start: 0, end: 31 },
        }
    );
}

#[test]
fn parses_private_opaque_nominal_declarations() {
    let module = parse_surface_ast("main.ssrg", "opaque newtype Secret = Int\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Newtype {
            visibility: Visibility::Private,
            opaque: true,
            name: "Secret".to_owned(),
            name_span: ByteSpan { start: 15, end: 21 },
            type_parameters: Vec::new(),
            deriving: Vec::new(),
            representation: TypeRef::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 24, end: 27 },
            },
            span: ByteSpan { start: 0, end: 27 },
        }
    );
}

#[test]
fn parses_struct_declarations() {
    let module = parse_surface_ast("main.ssrg", "pub struct Box<A> {\n  value: A,\n}\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Struct {
            visibility: Visibility::Public,
            opaque: false,
            name: "Box".to_owned(),
            name_span: ByteSpan { start: 11, end: 14 },
            type_parameters: vec!["A".to_owned()],
            deriving: Vec::new(),
            fields: vec![SurfaceField {
                name: "value".to_owned(),
                name_span: ByteSpan { start: 22, end: 27 },
                type_ref: TypeRef::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 29, end: 30 },
                },
            }],
            span: ByteSpan { start: 0, end: 33 },
        }
    );
}
