use seseragi_driver::{compile_core_module, compile_module, BuildProfile, CompileInput};
use seseragi_semantics::{resolve_module, ScopeKind, SymbolKind, SymbolNamespace};
use seseragi_syntax::{SurfaceDecl, SurfaceImplMember, TypeRef};

#[path = "support/frontend_evidence.rs"]
mod evidence;

const ORIGINAL_IMPL: &str = "struct Box<A> { value: A }\nimpl<A> Box<A> { operator + self -> other: Int -> Int = other }\npub effect fn main = println (show ((Box { value: True }) + 42))\n";
const EXPLICIT_CONTROL: &str = "struct Box<A> { value: A }\ninstance<A> Add<Box<A>, Int, Int> { fn add left: Box<A> -> right: Int -> Int = right }\npub effect fn main = println (show ((Box { value: True }) + 42))\n";

#[test]
fn generic_impl_and_explicit_instance_keep_the_same_checked_contract() {
    for (label, source) in [
        ("impl-original", ORIGINAL_IMPL),
        ("instance-control", EXPLICIT_CONTROL),
    ] {
        let input = CompileInput::new("generic-impl-雫.ssrg", "fixture/generic-impl", source);
        let core = compile_core_module(input).unwrap_or_else(|error| panic!("{error:#?}"));
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(input.with_profile(profile))
                .unwrap_or_else(|error| panic!("{error:#?}"));
            assert_eq!(compiled.core_ir, core.core_ir);
            evidence::record(label, profile, "main", "42\n", [("main", &compiled)]);
        }
    }
}

#[test]
fn generic_impl_constraints_and_saved_partial_calls_preserve_core_in_both_profiles() {
    for (label, source, expected) in [
        (
            "impl-constraints-partials",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generic-impl-operators.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generic-impl-operators.stdout"
            ),
        ),
        (
            "impl-ref-cold-finalizer",
            include_str!("../../../examples/spec/fixtures/rust-backend/generic-impl-effects.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generic-impl-effects.stdout"
            ),
        ),
        (
            "impl-source-once",
            include_str!("../../../examples/spec/fixtures/rust-backend/generic-impl-trace.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/generic-impl-trace.stdout"),
        ),
    ] {
        let input = CompileInput::new("generic-impl-雫.ssrg", "fixture/generic-impl", source);
        let core = compile_core_module(input).unwrap_or_else(|error| panic!("{error:#?}"));
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(input.with_profile(profile))
                .unwrap_or_else(|error| panic!("{label}: {error:#?}"));
            assert_eq!(compiled.core_ir, core.core_ir);
            evidence::record(label, profile, "main", expected, [("main", &compiled)]);
        }
    }
}

#[test]
fn generated_instances_reuse_the_source_impl_binder_and_separate_sibling_scopes() {
    let source = "struct Box<A> { value: A }\nstruct Other<A> { value: A }\nimpl<A> Box<A> { operator + self -> other: Int -> Int = other; operator - self -> other: Int -> Int = other }\nimpl<A> Other<A> { operator + self -> other: Int -> Int = other }\ninstance<A> Mul<Box<A>, Int, Int> { fn mul left: Box<A> -> right: Int -> Int = right }\n";
    let resolved = resolve_module("generic-scopes.ssrg", source);
    assert!(resolved.issues.is_empty(), "{resolved:#?}");
    let mut source_binders = vec![];
    let mut generated_count = 0;
    for declaration in &resolved.declarations {
        let SurfaceDecl::Impl {
            target,
            members,
            span,
            ..
        } = declaration
        else {
            continue;
        };
        let scope = resolved
            .scopes
            .iter()
            .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == *span)
            .unwrap();
        let binder = resolved
            .symbols
            .iter()
            .find(|symbol| {
                symbol.scope == scope.id
                    && symbol.kind == SymbolKind::TypeParameter
                    && symbol.spelling == "A"
            })
            .unwrap();
        source_binders.push(binder.id);
        let TypeRef::Named { arguments, .. } = target else {
            panic!("nominal target")
        };
        let TypeRef::Named {
            span: reference_span,
            ..
        } = &arguments[0]
        else {
            panic!("source binder")
        };
        assert!(resolved
            .references
            .iter()
            .filter(|reference| reference.namespace == SymbolNamespace::Type
                && reference.origin == *reference_span)
            .all(|reference| reference.target == Some(binder.id)));
        for member in members {
            let SurfaceImplMember::Operator { span, .. } = member else {
                continue;
            };
            let generated = resolved
                .scopes
                .iter()
                .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == *span)
                .unwrap();
            assert_eq!(generated.parent, Some(scope.id));
            assert!(!resolved
                .symbols
                .iter()
                .any(|symbol| symbol.scope == generated.id
                    && symbol.kind == SymbolKind::TypeParameter));
            generated_count += 1;
        }
    }
    assert_eq!(generated_count, 3);
    assert_eq!(source_binders.len(), 2);
    assert_ne!(source_binders[0], source_binders[1]);
    let explicit = resolved
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            SurfaceDecl::Instance {
                trait_name, span, ..
            } if trait_name == "Mul" => Some(*span),
            _ => None,
        })
        .unwrap();
    let scope = resolved
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == explicit)
        .unwrap();
    let binder = resolved
        .symbols
        .iter()
        .find(|symbol| symbol.scope == scope.id && symbol.kind == SymbolKind::TypeParameter)
        .unwrap();
    assert!(!source_binders.contains(&binder.id));
    assert_eq!(
        resolved
            .scopes
            .iter()
            .find(|parent| Some(parent.id) == scope.parent)
            .unwrap()
            .kind,
        ScopeKind::Module
    );
    compile_core_module(CompileInput::new(
        "generic-scopes.ssrg",
        "fixture/generic-scopes",
        source,
    ))
    .unwrap_or_else(|error| panic!("{error:#?}"));
}

#[test]
fn same_named_builtin_binders_remain_local_and_distinct_nominal_owners_do_not_overlap() {
    let source = "struct Box<A> { value: A }\nstruct Other<A> { value: A }\nimpl<Int> Box<Int> { operator + self -> other: Int -> Int = self.value }\nimpl<Int> Other<Int> { operator + self -> other: Int -> Int = other }\npub fn builtin value: Int -> Int = value + 1\npub fn first -> Bool = (Box { value: True }) + False\npub fn second -> Bool = (Other { value: True }) + False\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile_module(
            CompileInput::new("same-named.ssrg", "fixture/same-named", source)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
    }
}

#[test]
fn sugar_preserves_constructor_parameter_kinds_and_their_original_symbols() {
    let source = "struct Higher<F<_>, A> { value: F<A> }\nimpl<F<_>, A> Higher<F, A> { operator + self -> other: Int -> Int = other }\n";
    let resolved = resolve_module("kind-scopes.ssrg", source);
    assert!(resolved.issues.is_empty(), "{resolved:#?}");
    let original = resolved
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            SurfaceDecl::Impl {
                type_parameters,
                target,
                span,
                ..
            } => Some((type_parameters, target, *span)),
            _ => None,
        })
        .unwrap();
    let (parameters, arguments, span) = resolved
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            SurfaceDecl::Instance {
                type_parameters,
                arguments,
                span,
                ..
            } => Some((type_parameters, arguments, *span)),
            _ => None,
        })
        .unwrap();
    assert_eq!(parameters, original.0);
    assert_eq!(
        parameters
            .iter()
            .map(|parameter| parameter.arity)
            .collect::<Vec<_>>(),
        [1, 0]
    );
    assert_eq!(arguments[0], *original.1);
    let scope = resolved
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == original.2)
        .unwrap();
    let generated = resolved
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == span)
        .unwrap();
    assert_eq!(generated.parent, Some(scope.id));
    assert_eq!(
        resolved
            .symbols
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::TypeParameter && symbol.scope == scope.id)
            .count(),
        2
    );
    assert!(!resolved
        .symbols
        .iter()
        .any(|symbol| symbol.kind == SymbolKind::TypeParameter && symbol.scope == generated.id));
}

#[test]
fn generic_impl_overlap_and_functional_dependency_conflicts_are_positioned_before_codegen() {
    for (result, body) in [("Int", "right"), ("String", "\"wrong\"")] {
        let explicit = format!("instance Add<Box<Bool>, Int, {result}> {{ fn add left: Box<Bool> -> right: Int -> {result} = {body} }}");
        let source = format!("// 雫🙂\nstruct Box<A> {{ value: A }}\nimpl<A> Box<A> {{ operator + self -> other: Int -> Int = other }}\n{explicit}\npub effect fn main = println \"MUST_NOT_RUN\"\n");
        let input = CompileInput::new("overlap-雫.ssrg", "fixture/impl-overlap", &source);
        let errors = compile_core_module(input).unwrap_err();
        assert_eq!(errors, compile_module(input).unwrap_err());
        let duplicate = errors
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.code == "SES-T0202"
                    && diagnostic.message_key == "trait.instance-duplicate"
            })
            .unwrap_or_else(|| panic!("{errors:#?}"));
        assert_eq!(
            &source[duplicate.primary.start..duplicate.primary.end],
            explicit
        );
        let original =
            &source[duplicate.related[0].primary.start..duplicate.related[0].primary.end];
        assert_eq!(original, "operator + self -> other: Int -> Int = other");
    }
}

#[test]
fn constraints_wrong_owners_distinct_binders_and_unrepresented_parameters_still_fail() {
    for source in [
        "struct Box<A> { value: A }\nimpl<A> Box<A> where Show<A> { operator + self -> other: Int -> String = show self.value }\ntype Hidden = | Hidden\npub fn wrong -> String = (Box { value: Hidden }) + 42\n",
        "struct Box<A> { value: A }\nstruct Other<A> { value: A }\nimpl<A> Box<A> { operator + self -> other: A -> A = other }\npub fn wrong<A, B> value: Box<A> -> other: B -> A = value + other\n",
        "struct Box<A> { value: A }\nimpl<A> Box<A> { operator + self -> other: A -> A = other }\npub fn wrong -> Int = (Box { value: True }) + False\n",
        "struct Box<A> { value: A }\nimpl<A, B> Box<A> { operator + self -> other: Int -> Int = other }\n",
        "struct Box<A> { value: A }\nimpl<A> Box<A> { fn duplicate<A> self: Box<A> -> A -> A = \\value -> value; operator + self -> other: Int -> Int = other }\n",
    ] {
        let input = CompileInput::new("generic-invalid-雫.ssrg", "fixture/generic-invalid", source);
        let errors = compile_core_module(input).unwrap_err();
        assert_eq!(errors, compile_module(input).unwrap_err());
        assert!(errors.diagnostics.iter().any(|diagnostic| matches!(diagnostic.code.as_str(), "SES-T0101" | "SES-T0201" | "SES-N0002")), "{source}\n{errors:#?}");
        for diagnostic in &errors.diagnostics {
            assert!(source.is_char_boundary(diagnostic.primary.start));
            assert!(source.is_char_boundary(diagnostic.primary.end));
            assert!(diagnostic.primary.end > diagnostic.primary.start);
        }
    }
}

#[test]
fn imported_and_reexported_impls_keep_one_canonical_owner_under_a_local_same_name() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let model = include_str!("../../../examples/spec/fixtures/rust-backend-projects/generic-impl-operators/src/model.ssrg");
    let facade = include_str!("../../../examples/spec/fixtures/rust-backend-projects/generic-impl-operators/src/facade.ssrg");
    let main = include_str!("../../../examples/spec/fixtures/rust-backend-projects/generic-impl-operators/src/main.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for (label, source, valid) in [
            ("impl-import-reexport", main.to_owned(), true),
            (
                "impl-import-wrong-owner",
                main.replace(
                    "DirectBox<Bool> -> Int -> String",
                    "Box<Bool> -> Int -> String",
                ),
                false,
            ),
        ] {
            let mut graph = ModuleGraph::new();
            graph.add_module("proof/model".to_owned(), []).unwrap();
            graph
                .add_module(
                    "proof/facade".to_owned(),
                    [("./model".to_owned(), "proof/model".to_owned())],
                )
                .unwrap();
            graph
                .add_module(
                    "proof/main".to_owned(),
                    [
                        ("./model".to_owned(), "proof/model".to_owned()),
                        ("./facade".to_owned(), "proof/facade".to_owned()),
                    ],
                )
                .unwrap();
            let result = compile_project(
                graph,
                [
                    ProjectModuleInput::new(
                        "src/model.ssrg",
                        "proof/model",
                        model,
                        "dist/model.js",
                    )
                    .with_profile(profile),
                    ProjectModuleInput::new(
                        "src/facade.ssrg",
                        "proof/facade",
                        facade,
                        "dist/facade.js",
                    )
                    .with_profile(profile),
                    ProjectModuleInput::new("src/main.ssrg", "proof/main", &source, "dist/main.js")
                        .with_profile(profile),
                ],
            );
            if valid {
                let project = result.unwrap_or_else(|error| panic!("{label}: {error:#?}"));
                evidence::record(label, profile, "proof/main", include_str!("../../../examples/spec/fixtures/rust-backend-projects/generic-impl-operators/expected.stdout"), project.modules.iter().map(|(id, module)| (id.as_str(), module)));
            } else {
                let Err(seseragi_driver::ProjectCompileError::Diagnostics { modules }) = result
                else {
                    panic!("{label}: local owner must not inherit imported Add")
                };
                assert!(
                    modules.iter().any(|module| module.module == "proof/main"
                        && module
                            .diagnostics
                            .diagnostics
                            .iter()
                            .any(|diagnostic| matches!(
                                diagnostic.code.as_str(),
                                "SES-T0101" | "SES-T0201"
                            ))),
                    "{modules:#?}"
                );
            }
        }
    }
}
