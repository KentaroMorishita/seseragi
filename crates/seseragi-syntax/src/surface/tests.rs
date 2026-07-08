use super::*;

#[test]
fn constructs_let_surface_module() {
    let module = SurfaceModule {
        schema: 1,
        source: "main.ssrg".to_owned(),
        imports: Vec::new(),
        declarations: vec![SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "answer".to_owned(),
            name_span: ByteSpan { start: 8, end: 14 },
            type_ref: Some(TypeRef::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 16, end: 19 },
            }),
            span: ByteSpan { start: 0, end: 24 },
        }],
    };

    let json = serde_json::to_value(&module).expect("surface module serializes");

    assert_eq!(json["schema"], 1);
    assert_eq!(json["source"], "main.ssrg");
    assert_eq!(json["declarations"][0]["kind"], "let");
    assert_eq!(json["declarations"][0]["visibility"], "public");
    assert_eq!(json["declarations"][0]["typeRef"]["kind"], "named");
}

#[test]
fn constructs_effect_function_surface_decl() {
    let decl = SurfaceDecl::EffectFn {
        visibility: Visibility::Private,
        name: "main".to_owned(),
        name_span: ByteSpan { start: 10, end: 14 },
        return_type: Some(TypeRef::Named {
            name: "Unit".to_owned(),
            arguments: Vec::new(),
            span: ByteSpan { start: 18, end: 22 },
        }),
        span: ByteSpan { start: 0, end: 72 },
    };

    let json = serde_json::to_value(&decl).expect("surface decl serializes");

    assert_eq!(json["kind"], "effectFn");
    assert_eq!(json["visibility"], "private");
    assert_eq!(json["returnType"]["name"], "Unit");
}

#[test]
fn parses_pure_function_surface_decl() {
    let module = parse_surface_ast("main.ssrg", "pub fn add x: Int -> y: Int -> Int = x + y\n");

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Fn {
            visibility: Visibility::Public,
            name: "add".to_owned(),
            name_span: ByteSpan { start: 7, end: 10 },
            type_parameters: Vec::new(),
            parameters: vec![
                SurfaceParameter {
                    name: "x".to_owned(),
                    name_span: ByteSpan { start: 11, end: 12 },
                    type_ref: TypeRef::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 14, end: 17 },
                    },
                },
                SurfaceParameter {
                    name: "y".to_owned(),
                    name_span: ByteSpan { start: 21, end: 22 },
                    type_ref: TypeRef::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 24, end: 27 },
                    },
                },
            ],
            return_type: TypeRef::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 31, end: 34 },
            },
            constraints: Vec::new(),
            span: ByteSpan { start: 0, end: 42 },
        }
    );
}

#[test]
fn parses_pure_function_type_parameters_and_constraints() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub fn member<A> target: A -> values: List<A> -> Bool\nwhere Eq<A> =\n  contains target values\n",
    );

    let SurfaceDecl::Fn {
        type_parameters,
        constraints,
        span,
        ..
    } = &module.declarations[0]
    else {
        panic!("expected function declaration");
    };

    assert_eq!(type_parameters, &vec!["A".to_owned()]);
    assert_eq!(constraints, &vec!["Eq".to_owned()]);
    assert_eq!(*span, ByteSpan { start: 0, end: 92 });
}

#[test]
fn parses_public_let_surface_ast() {
    let module = parse_surface_ast("main.ssrg", "pub let answer: Int = 42\n");

    assert_eq!(module.schema, 1);
    assert_eq!(module.source, "main.ssrg");
    assert_eq!(
        module.declarations,
        vec![SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "answer".to_owned(),
            name_span: ByteSpan { start: 8, end: 14 },
            type_ref: Some(TypeRef::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 16, end: 19 },
            }),
            span: ByteSpan { start: 0, end: 24 },
        }]
    );
}

#[test]
fn parses_multiple_lets_with_visibility_only() {
    let module = parse_surface_ast("main.ssrg", "let first = 1\npub let second = 2\n");

    assert_eq!(module.declarations.len(), 2);
    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Let {
            visibility: Visibility::Private,
            name: "first".to_owned(),
            name_span: ByteSpan { start: 4, end: 9 },
            type_ref: None,
            span: ByteSpan { start: 0, end: 13 },
        }
    );
    assert_eq!(
        module.declarations[1],
        SurfaceDecl::Let {
            visibility: Visibility::Public,
            name: "second".to_owned(),
            name_span: ByteSpan { start: 22, end: 28 },
            type_ref: None,
            span: ByteSpan { start: 14, end: 32 },
        }
    );
}

#[test]
fn parses_effect_do_surface_decl() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {\n    value <- console.readLine ()\n  }\n",
    );

    assert_eq!(
        module.declarations,
        vec![SurfaceDecl::EffectFn {
            visibility: Visibility::Public,
            name: "main".to_owned(),
            name_span: ByteSpan { start: 14, end: 18 },
            return_type: Some(TypeRef::Named {
                name: "Unit".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 22, end: 26 },
            }),
            span: ByteSpan { start: 0, end: 104 },
        }]
    );
}

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
            span: ByteSpan { start: 0, end: 38 },
        }
    );
}

#[test]
fn parses_rich_interface_surface_declarations() {
    let module = parse_surface_ast(
        "main.ssrg",
        include_str!("../../../../examples/spec/artifacts/interface-schema-1/rich/main.ssrg"),
    );

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            specifier: "./dep".to_owned(),
            items: vec![SurfaceImportItem {
                namespace: "value".to_owned(),
                name: "identity".to_owned(),
                alias: None,
            }],
            span: ByteSpan { start: 0, end: 32 },
        }]
    );
    assert_eq!(module.declarations.len(), 3);
    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Newtype {
            visibility: Visibility::Public,
            name: "Score".to_owned(),
            name_span: ByteSpan { start: 46, end: 51 },
            representation: TypeRef::Named {
                name: "Int".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 54, end: 57 },
            },
            span: ByteSpan { start: 34, end: 57 },
        }
    );
    assert_eq!(
        module.declarations[1],
        SurfaceDecl::Operator {
            visibility: Visibility::Public,
            type_parameters: Vec::new(),
            fixity: "infixl".to_owned(),
            precedence: 4,
            spelling: "<+>".to_owned(),
            parameters: vec![
                SurfaceParameter {
                    name: "left".to_owned(),
                    name_span: ByteSpan { start: 87, end: 91 },
                    type_ref: TypeRef::Named {
                        name: "Score".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan { start: 93, end: 98 },
                    },
                },
                SurfaceParameter {
                    name: "right".to_owned(),
                    name_span: ByteSpan {
                        start: 102,
                        end: 107
                    },
                    type_ref: TypeRef::Named {
                        name: "Score".to_owned(),
                        arguments: Vec::new(),
                        span: ByteSpan {
                            start: 109,
                            end: 114
                        },
                    },
                },
            ],
            return_type: TypeRef::Named {
                name: "Score".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan {
                    start: 118,
                    end: 123
                },
            },
            constraints: Vec::new(),
            span: ByteSpan {
                start: 59,
                end: 141
            },
        }
    );
    assert_eq!(
        module.declarations[2],
        SurfaceDecl::Instance {
            type_parameters: Vec::new(),
            trait_name: "Show".to_owned(),
            arguments: vec![TypeRef::Named {
                name: "Score".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan {
                    start: 157,
                    end: 162
                },
            }],
            constraints: Vec::new(),
            span: ByteSpan {
                start: 143,
                end: 210
            },
        }
    );
}

#[test]
fn parses_instance_type_parameters_and_constraints() {
    let module = parse_surface_ast(
        "main.ssrg",
        "instance<A> Show<Box<A>>\nwhere Show<A> {\n  fn show value: Box<A> -> String = \"Box\"\n}\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Instance {
            type_parameters: vec!["A".to_owned()],
            trait_name: "Show".to_owned(),
            arguments: vec![TypeRef::Named {
                name: "Box".to_owned(),
                arguments: vec![TypeRef::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 21, end: 22 },
                }],
                span: ByteSpan { start: 17, end: 23 },
            }],
            constraints: vec!["Show".to_owned()],
            span: ByteSpan { start: 0, end: 84 },
        }
    );
}

#[test]
fn parses_trait_declarations() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub trait Ord<A>\nwhere Eq<A> {\n  fn compare x: A -> y: A -> Ordering\n}\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Trait {
            visibility: Visibility::Public,
            name: "Ord".to_owned(),
            name_span: ByteSpan { start: 10, end: 13 },
            type_parameters: vec!["A".to_owned()],
            constraints: vec!["Eq".to_owned()],
            span: ByteSpan { start: 0, end: 70 },
        }
    );
}

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
fn parses_operator_type_parameters() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub operator<A> infixr 5 <+>\n  left: A -> right: A -> A =\n  left\n",
    );

    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Operator {
            visibility: Visibility::Public,
            type_parameters: vec!["A".to_owned()],
            fixity: "infixr".to_owned(),
            precedence: 5,
            spelling: "<+>".to_owned(),
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
            span: ByteSpan { start: 0, end: 64 },
        }
    );
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

    assert_eq!(constraints, &vec!["Semigroup".to_owned()]);
    assert_eq!(*span, ByteSpan { start: 0, end: 83 });
}

#[test]
fn parses_operator_import_items() {
    let module = parse_surface_ast(
        "main.ssrg",
        "import { identity, operator <+> } from \"./dep\"\n",
    );

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            specifier: "./dep".to_owned(),
            items: vec![
                SurfaceImportItem {
                    namespace: "value".to_owned(),
                    name: "identity".to_owned(),
                    alias: None,
                },
                SurfaceImportItem {
                    namespace: "operator".to_owned(),
                    name: "<+>".to_owned(),
                    alias: None,
                },
            ],
            span: ByteSpan { start: 0, end: 46 },
        }]
    );
}

#[test]
fn parses_aliased_import_items() {
    let module = parse_surface_ast("main.ssrg", "import { parse as parseJson } from \"json\"\n");

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            specifier: "json".to_owned(),
            items: vec![SurfaceImportItem {
                namespace: "value".to_owned(),
                name: "parse".to_owned(),
                alias: Some("parseJson".to_owned()),
            }],
            span: ByteSpan { start: 0, end: 41 },
        }]
    );
}

#[test]
fn parses_namespace_import_items() {
    let module = parse_surface_ast("main.ssrg", "import * as text from \"std/text\"\n");

    assert_eq!(
        module.imports,
        vec![SurfaceImport {
            specifier: "std/text".to_owned(),
            items: vec![SurfaceImportItem {
                namespace: "namespace".to_owned(),
                name: "*".to_owned(),
                alias: Some("text".to_owned()),
            }],
            span: ByteSpan { start: 0, end: 32 },
        }]
    );
}
