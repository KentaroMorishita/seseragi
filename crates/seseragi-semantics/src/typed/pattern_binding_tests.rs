use crate::{
    analysis_document, resolve_module, semantic_diagnostics, type_module,
    type_module_public_interface, TypedBlockStatement, TypedDecl, TypedDoStatement, TypedExpr,
    TypedMonadDoStatement, TypedPattern, TypedType,
};

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn function(parameter: TypedType, result: TypedType) -> TypedType {
    TypedType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}

#[test]
fn propagates_top_level_tuple_function_bindings_to_analysis() {
    let source = concat!(
        "let operations: (Int -> Int, Int -> Int) = ",
        "(\\value: Int -> value + 1, \\value: Int -> value * 2)\n",
        "let (increment, double) = operations\n",
        "let result: Int = increment 10 + double 10\n",
    );
    let diagnostics = semantic_diagnostics("pattern/top-tuple/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");

    let resolved = resolve_module("pattern/top-tuple/main.ssrg", source);
    let typed = type_module("pattern/top-tuple/main.ssrg", source);
    let TypedDecl::Let {
        pattern: TypedPattern::Tuple { elements, .. },
        ..
    } = &typed.declarations[1]
    else {
        panic!("expected a typed tuple binding");
    };
    let expected = function(named("Int"), named("Int"));
    assert!(elements.iter().all(|element| matches!(
        element,
        TypedPattern::Binding { type_ref, .. } if type_ref == &expected
    )));

    let analysis = analysis_document(diagnostics, resolved, &typed);
    for name in ["increment", "double"] {
        let symbol = analysis
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .unwrap_or_else(|| panic!("missing {name} analysis symbol"));
        assert_eq!(symbol.type_name.as_deref(), Some("Int -> Int"));
    }
}

#[test]
fn exports_each_public_top_level_pattern_binding_with_its_nested_type() {
    let source = concat!(
        "pub let (number, label): (Int, String) = (42, \"answer\")\n",
        "let checked: Int = number\n",
    );
    let interface = type_module_public_interface("pattern/public/main.ssrg", source);
    let number = interface
        .exports
        .iter()
        .find(|export| export.name == "number")
        .expect("number export");
    let label = interface
        .exports
        .iter()
        .find(|export| export.name == "label")
        .expect("label export");
    assert_eq!(
        number.scheme.type_ref,
        seseragi_syntax::InterfaceType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        }
    );
    assert_eq!(
        label.scheme.type_ref,
        seseragi_syntax::InterfaceType::Named {
            name: "String".to_owned(),
            arguments: Vec::new(),
        }
    );
}

#[test]
fn types_block_record_and_struct_pattern_bindings() {
    let source = concat!(
        "struct User { id: Int, name: String }\n",
        "fn identifier user: User -> Int = {\n",
        "  let User { id, name } = user\n",
        "  id\n",
        "}\n",
        "fn coordinate point: { x: Int, y: Int } -> Int = {\n",
        "  let { x, y } = point\n",
        "  x + y\n",
        "}\n",
    );
    let diagnostics = semantic_diagnostics("pattern/block-record/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");

    let typed = type_module("pattern/block-record/main.ssrg", source);
    for declaration in typed
        .declarations
        .iter()
        .filter(|declaration| matches!(declaration, TypedDecl::Fn { .. }))
    {
        let TypedDecl::Fn {
            body: TypedExpr::Block { statements, .. },
            ..
        } = declaration
        else {
            panic!("expected a block function");
        };
        assert!(matches!(
            statements.as_slice(),
            [TypedBlockStatement::Let {
                pattern: TypedPattern::Record { fields, .. },
                ..
            }] if fields.iter().all(|field| matches!(
                &field.pattern,
                TypedPattern::Binding { type_ref, .. }
                    if type_ref == &named("Int") || type_ref == &named("String")
            ))
        ));
    }
}

#[test]
fn types_tuple_patterns_in_effect_and_monad_do() {
    let source = concat!(
        "effect fn effectPair = do {\n",
        "  let (left, right) = (1, 2)\n",
        "  succeed (left + right)\n",
        "}\n",
        "fn maybePair value: Maybe<(Int, Int)> -> Maybe<Int> = do {\n",
        "  (left, right) <- value\n",
        "  pure (left + right)\n",
        "}\n",
    );
    let diagnostics = semantic_diagnostics("pattern/do-tuple/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");

    let typed = type_module("pattern/do-tuple/main.ssrg", source);
    assert!(matches!(
        &typed.declarations[0],
        TypedDecl::EffectFn {
            body: TypedExpr::DoBlock { statements, .. },
            ..
        } if matches!(
            statements.as_slice(),
            [TypedDoStatement::PureLet {
                pattern: TypedPattern::Tuple { elements, .. },
                ..
            }] if elements.iter().all(|element| matches!(
                element,
                TypedPattern::Binding { type_ref, .. } if type_ref == &named("Int")
            ))
        )
    ));
    assert!(matches!(
        &typed.declarations[1],
        TypedDecl::Fn {
            body: TypedExpr::MonadDo { statements, .. },
            ..
        } if matches!(
            statements.as_slice(),
            [TypedMonadDoStatement::Bind {
                pattern: TypedPattern::Tuple { elements, .. },
                ..
            }] if elements.iter().all(|element| matches!(
                element,
                TypedPattern::Binding { type_ref, .. } if type_ref == &named("Int")
            ))
        )
    ));
}

#[test]
fn types_annotated_do_let_with_the_shared_binding_rules() {
    let source = concat!(
        "effect fn effectValue = do {\n",
        "  let missing: Maybe<String> = Nothing\n",
        "  succeed missing\n",
        "}\n",
        "fn maybeValue value: Maybe<Int> -> Maybe<Int> = do {\n",
        "  current <- value\n",
        "  let offset: Int = 1\n",
        "  pure $ current + offset\n",
        "}\n",
    );
    let diagnostics = semantic_diagnostics("pattern/do-annotation/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");

    let typed = type_module("pattern/do-annotation/main.ssrg", source);
    assert!(matches!(
        &typed.declarations[0],
        TypedDecl::EffectFn {
            body: TypedExpr::DoBlock { statements, .. },
            ..
        } if matches!(
            statements.as_slice(),
            [TypedDoStatement::PureLet {
                pattern: TypedPattern::Binding { type_ref, .. },
                ..
            }] if type_ref == &TypedType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![named("String")],
            }
        )
    ));
    assert!(matches!(
        &typed.declarations[1],
        TypedDecl::Fn {
            body: TypedExpr::MonadDo { statements, .. },
            ..
        } if matches!(
            statements.as_slice(),
            [
                TypedMonadDoStatement::Bind { .. },
                TypedMonadDoStatement::PureLet {
                    pattern: TypedPattern::Binding { type_ref, .. },
                    ..
                }
            ] if type_ref == &named("Int")
        )
    ));
}

#[test]
fn reports_the_same_annotation_mismatch_in_effect_and_monad_do() {
    let source = concat!(
        "effect fn effectValue = do {\n",
        "  let brokenEffect: String = 1\n",
        "  succeed brokenEffect\n",
        "}\n",
        "fn maybeValue value: Maybe<Int> -> Maybe<Int> = do {\n",
        "  current <- value\n",
        "  let brokenMonad: String = 1\n",
        "  pure current\n",
        "}\n",
    );
    let diagnostics = semantic_diagnostics("pattern/do-annotation-mismatch/main.ssrg", source);

    assert_eq!(diagnostics.diagnostics.len(), 2, "{diagnostics:#?}");
    assert!(diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == "SES-T0101"
            && diagnostic.message_key == "let.type-mismatch"));
}

#[test]
fn types_nested_constructor_bindings_independent_of_the_outer_type() {
    let source = concat!(
        "type User = | User (String, Int)\n",
        "type Box<A> = | Boxed A\n",
        "fn maybeName value: Maybe<User> -> String =\n",
        "  match value {\n",
        "    Just (User (name, age)) -> name\n",
        "    Nothing -> \"missing\"\n",
        "  }\n",
        "fn eitherName value: Either<String, User> -> String =\n",
        "  match value {\n",
        "    Right (User (name, age)) -> name\n",
        "    Left error -> error\n",
        "  }\n",
        "fn boxName value: Box<User> -> String =\n",
        "  match value {\n",
        "    Boxed (User (name, age)) -> name\n",
        "  }\n",
    );
    let diagnostics = semantic_diagnostics("pattern/nested-generic/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");

    let analysis = analysis_document(
        diagnostics,
        resolve_module("pattern/nested-generic/main.ssrg", source),
        &type_module("pattern/nested-generic/main.ssrg", source),
    );
    let names = analysis
        .symbols
        .iter()
        .filter(|symbol| symbol.name == "name")
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 3);
    assert!(names
        .iter()
        .all(|symbol| symbol.type_name.as_deref() == Some("String")));
}

#[test]
fn accepts_only_irrefutable_array_and_list_let_shapes() {
    let valid = concat!(
        "let values: Array<Int> = [1, 2]\n",
        "let [...arrayValues] = values\n",
        "let items: List<Int> = `[1, 2]\n",
        "let `[...listValues] = items\n",
        "let copiedArray: Array<Int> = arrayValues\n",
        "let copiedList: List<Int> = listValues\n",
    );
    let valid_diagnostics = semantic_diagnostics("pattern/collection-valid/main.ssrg", valid);
    assert!(
        valid_diagnostics.diagnostics.is_empty(),
        "{valid_diagnostics:#?}"
    );

    let invalid = "let values: Array<Int> = [1, 2]\nlet [head, ...tail] = values\n";
    let invalid_diagnostics = semantic_diagnostics("pattern/collection-invalid/main.ssrg", invalid);
    assert!(invalid_diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message_key == "binding.refutable-pattern"));
}

#[test]
fn accepts_irrefutable_single_constructor_patterns() {
    let source = concat!(
        "newtype UserId = Int\n",
        "type Boxed = | Box Int\n",
        "let UserId rawId = UserId 42\n",
        "let Box boxed = Box 7\n",
        "let result: Int = rawId + boxed\n",
    );
    let diagnostics = semantic_diagnostics("pattern/constructor/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn rejects_duplicate_names_within_one_binding_pattern() {
    let diagnostics = semantic_diagnostics(
        "pattern/duplicate/main.ssrg",
        "let (value, value) = (1, 2)\n",
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "name.duplicate-definition"),
        "{diagnostics:#?}"
    );
}
