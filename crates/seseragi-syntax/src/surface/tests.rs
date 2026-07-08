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
            span: ByteSpan {
                start: 59,
                end: 141
            },
        }
    );
    assert_eq!(
        module.declarations[2],
        SurfaceDecl::Instance {
            trait_name: "Show".to_owned(),
            arguments: vec![TypeRef::Named {
                name: "Score".to_owned(),
                arguments: Vec::new(),
                span: ByteSpan {
                    start: 157,
                    end: 162
                },
            }],
            span: ByteSpan {
                start: 143,
                end: 210
            },
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
            span: ByteSpan { start: 0, end: 64 },
        }
    );
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
                },
                SurfaceImportItem {
                    namespace: "operator".to_owned(),
                    name: "<+>".to_owned(),
                },
            ],
            span: ByteSpan { start: 0, end: 46 },
        }]
    );
}
