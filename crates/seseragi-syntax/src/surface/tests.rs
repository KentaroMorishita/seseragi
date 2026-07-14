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
            body: None,
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
        type_parameters: Vec::new(),
        parameters: Vec::new(),
        inferred_contract: false,
        return_type: Some(TypeRef::Named {
            name: "Unit".to_owned(),
            arguments: Vec::new(),
            span: ByteSpan { start: 18, end: 22 },
        }),
        requirements: Vec::new(),
        failure: None,
        constraints: Vec::new(),
        body: None,
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
            body: Some(SurfaceExpr::Binary {
                operator: "+".to_owned(),
                operator_span: ByteSpan { start: 39, end: 40 },
                left: Box::new(SurfaceExpr::Name {
                    name: "x".to_owned(),
                    span: ByteSpan { start: 37, end: 38 },
                }),
                right: Box::new(SurfaceExpr::Name {
                    name: "y".to_owned(),
                    span: ByteSpan { start: 41, end: 42 },
                }),
                span: ByteSpan { start: 37, end: 42 },
            }),
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
    assert_eq!(constraints[0].name, "Eq");
    assert!(matches!(
        constraints[0].arguments.as_slice(),
        [TypeRef::Named { name, .. }] if name == "A"
    ));
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
            body: Some(SurfaceExpr::Integer {
                raw: "42".to_owned(),
                span: ByteSpan { start: 22, end: 24 },
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
            body: Some(SurfaceExpr::Integer {
                raw: "1".to_owned(),
                span: ByteSpan { start: 12, end: 13 },
            }),
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
            body: Some(SurfaceExpr::Integer {
                raw: "2".to_owned(),
                span: ByteSpan { start: 31, end: 32 },
            }),
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
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            inferred_contract: false,
            return_type: Some(TypeRef::Named {
                name: "Unit".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 22, end: 26 },
            }),
            requirements: vec![SurfaceRequirement::Shorthand {
                name: "Console".to_owned(),
                span: ByteSpan { start: 32, end: 39 },
            }],
            failure: Some(TypeRef::Named {
                name: "ConsoleError".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan { start: 46, end: 58 },
            }),
            constraints: Vec::new(),
            body: Some(SurfaceExpr::Do {
                items: vec![SurfaceDoItem::Bind {
                    pattern: SurfacePattern::Name {
                        name: "value".to_owned(),
                        name_span: ByteSpan { start: 72, end: 77 },
                        span: ByteSpan { start: 72, end: 77 },
                    },
                    value: SurfaceExpr::Application {
                        function: Box::new(SurfaceExpr::Name {
                            name: "console.readLine".to_owned(),
                            span: ByteSpan { start: 81, end: 97 },
                        }),
                        argument: Box::new(SurfaceExpr::Unit {
                            span: ByteSpan {
                                start: 98,
                                end: 100,
                            },
                        }),
                        span: ByteSpan {
                            start: 81,
                            end: 100,
                        },
                    },
                    span: ByteSpan {
                        start: 72,
                        end: 100,
                    },
                }],
                result: None,
                span: ByteSpan {
                    start: 63,
                    end: 104,
                },
            }),
            span: ByteSpan { start: 0, end: 104 },
        }]
    );
}

#[test]
fn parses_compact_inferred_effect_function_surface_decl() {
    let module = parse_surface_ast(
        "main.ssrg",
        "pub effect fn greet name: String =\n  println \"hello\"\n",
    );

    assert_eq!(
        module.declarations,
        vec![SurfaceDecl::EffectFn {
            visibility: Visibility::Public,
            name: "greet".to_owned(),
            name_span: ByteSpan { start: 14, end: 19 },
            type_parameters: Vec::new(),
            parameters: vec![SurfaceParameter {
                name: "name".to_owned(),
                name_span: ByteSpan { start: 20, end: 24 },
                type_ref: TypeRef::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 26, end: 32 },
                },
            }],
            inferred_contract: true,
            return_type: None,
            requirements: Vec::new(),
            failure: None,
            constraints: Vec::new(),
            body: Some(SurfaceExpr::Application {
                function: Box::new(SurfaceExpr::Name {
                    name: "println".to_owned(),
                    span: ByteSpan { start: 37, end: 44 },
                }),
                argument: Box::new(SurfaceExpr::String {
                    raw: "\"hello\"".to_owned(),
                    span: ByteSpan { start: 45, end: 52 },
                }),
                span: ByteSpan { start: 37, end: 52 },
            }),
            span: ByteSpan { start: 0, end: 52 },
        }]
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
                name_span: ByteSpan { start: 9, end: 17 },
                alias: None,
                alias_span: None,
            }],
            span: ByteSpan { start: 0, end: 32 },
        }]
    );
    assert_eq!(module.declarations.len(), 3);
    assert_eq!(
        module.declarations[0],
        SurfaceDecl::Newtype {
            visibility: Visibility::Public,
            opaque: false,
            name: "Score".to_owned(),
            name_span: ByteSpan { start: 46, end: 51 },
            type_parameters: Vec::new(),
            deriving: Vec::new(),
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
            constraints: vec![SurfaceConstraint {
                name: "Show".to_owned(),
                arguments: vec![TypeRef::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 36, end: 37 },
                }],
                span: ByteSpan { start: 31, end: 38 },
            }],
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
            constraints: vec![SurfaceConstraint {
                name: "Eq".to_owned(),
                arguments: vec![TypeRef::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                    span: ByteSpan { start: 26, end: 27 },
                }],
                span: ByteSpan { start: 23, end: 28 },
            }],
            span: ByteSpan { start: 0, end: 70 },
        }
    );
}
