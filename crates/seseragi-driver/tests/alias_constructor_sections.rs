use seseragi_driver::{
    compile_module, compile_project, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;
use seseragi_semantics::TypedInstanceImplementation;

#[path = "support/frontend_evidence.rs"]
mod evidence;

const CASES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "type Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>",
        "Flip<Int>",
        "Pair<_,Int>",
        "Pair<A,Int>",
        "Pair (True,42)",
    ),
    (
        "type Triple<A,B,C> = | Triple (A,B,C)\nalias Duplicate<A,B> = Triple<A,A,B>",
        "Duplicate<Int>",
        "Triple<Int,Int>",
        "Triple<Int,Int,A>",
        "Triple (1,2,True)",
    ),
    (
        "type Box<A> = | Box A\nalias Ignore<A,B> = Box<B>",
        "Ignore<Int>",
        "Box",
        "Box<A>",
        "Box True",
    ),
    (
        "type Higher<F<_>,A> = | Higher F<A>\nalias Wrapped<F<_>,A> = Higher<F,A>",
        "Wrapped<Maybe>",
        "Higher<Maybe>",
        "Higher<Maybe,A>",
        "Higher (Just True)",
    ),
    (
        "type Higher<F<_>,A> = | Higher F<A>\nalias Wrapped<F<_>,A> = Higher<F,A>",
        "Wrapped<Either<_,Int>>",
        "Higher<Either<_,Int>>",
        "Higher<Either<_,Int>,A>",
        "Higher (Left True)",
    ),
    (
        "type Int = | LocalInt\ntype Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>",
        "Flip<Int>",
        "Pair<_,Int>",
        "Pair<A,Int>",
        "Pair (True,LocalInt)",
    ),
];

#[test]
fn fixed_alias_arguments_project_to_the_same_checked_section_as_direct_heads() {
    for &(declarations, alias, direct, parameter, value) in CASES {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let mut control = None;
            for head in [direct, alias] {
                let source = format!(
                    "{declarations}\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\n\
                     instance Tag<{head}> {{ fn tag<A> value: {parameter} -> String = \"ok\" }}\n\
                     pub fn probe -> String = tag<{head},Bool> ({value})\n"
                );
                let compiled = compile_module(
                    CompileInput::new("sections-雫.ssrg", "fixture/alias-sections", &source)
                        .with_profile(profile),
                )
                .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
                let instance = &compiled.typed_hir.instances[0];
                let TypedInstanceImplementation::UserDefined { methods } = &instance.implementation
                else {
                    panic!("{instance:?}")
                };
                assert_eq!(methods[0].parameters.len(), 1);
                assert_eq!(methods[0].scheme.type_parameters.len(), 1);
                let facts = (
                    instance.identity.clone(),
                    instance.argument_identities.clone(),
                    instance.arguments.clone(),
                );
                if let Some(control) = &control {
                    assert_eq!(&facts, control);
                } else {
                    control = Some(facts);
                }
            }
        }
    }
}

#[test]
fn proved_alias_sections_detect_unused_duplicates_against_the_direct_owner() {
    for &(declarations, alias, direct, parameter, _) in CASES {
        for (first, second) in [(alias, direct), (direct, alias)] {
            let source = format!(
                "// 雫🙂\n{declarations}\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\n\
                 instance Tag<{first}> {{ fn tag<A> value: {parameter} -> String = \"first\" }}\n\
                 instance Tag<{second}> {{ fn tag<A> value: {parameter} -> String = \"second\" }}\n"
            );
            let error = compile_module(CompileInput::new(
                "duplicate.ssrg",
                "fixture/alias-sections",
                &source,
            ))
            .unwrap_err();
            let diagnostic = error
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.message_key == "trait.instance-duplicate")
                .unwrap_or_else(|| panic!("{source}\n{error:#?}"));
            assert!(source[diagnostic.primary.start..diagnostic.primary.end]
                .starts_with(&format!("instance Tag<{second}>")));
            assert!(!diagnostic.related.is_empty());
            assert!(diagnostic
                .related
                .iter()
                .all(|related| related.primary.end <= source.len()));
        }
    }
}

#[test]
fn unbound_permutation_repetition_erasure_and_wrong_method_owner_remain_positioned_refusals() {
    for (source, key) in [
        ("type Pair<A,B> = | Pair (A,B)\nalias Repeat<A> = Pair<A,A>\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance Tag<Repeat> { fn tag<A> value:Pair<A,A> -> String = \"bad\" }\n", "trait.instance-head-unresolved"),
        ("type Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>\ntrait Tag<F<_,_>> { fn tag<A,B> value:F<A,B> -> String }\ninstance Tag<Flip> { fn tag<A,B> value:Pair<B,A> -> String = \"bad\" }\n", "trait.instance-head-unresolved"),
        ("alias Erase<A> = Int\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance Tag<Erase> { fn tag<A> value:Int -> String = \"bad\" }\n", "trait.instance-head-unresolved"),
        ("type Box<A> = | Box A\nalias Ignore<A,B> = Box<B>\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance<T> Tag<Ignore<T>> { fn tag<A> value:Box<A> -> String = \"bad\" }\n", "trait.instance-head-unresolved"),
        ("type Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance Tag<Flip<Int>> { fn tag<A> value:Pair<Int,A> -> String = \"bad\" }\n", "trait.instance-method-signature-mismatch"),
        ("type Pair<A,B> = | Pair (A,B)\ntype Other<A,B> = | Other (A,B)\nalias Flip<A,B> = Pair<B,A>\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance Tag<Flip<Int>> { fn tag<A> value:Other<A,Int> -> String = \"bad\" }\n", "trait.instance-method-signature-mismatch"),
    ] {
        let error = compile_module(CompileInput::new("refused-雫.ssrg", "fixture/alias-sections", source)).unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.message_key == key
            && diagnostic.primary.start < diagnostic.primary.end
            && diagnostic.primary.end <= source.len()), "{source}\n{error:#?}");
    }
}

#[test]
fn kind_proofs_reject_a_value_prefix_where_the_alias_declares_a_constructor() {
    let declarations = "type Higher<F<_>,A> = | Higher F<A>\nalias Wrapped<F<_>,A> = Higher<F,A>\n";
    for (tail, key, positioned) in [
        (
            "trait Tag<G<_>> { fn tag<A> value:G<A> -> String }\ninstance Tag<Wrapped<Int>> { fn tag<A> value:Higher<Maybe,A> -> String = \"bad\" }\n",
            "trait.instance-head-unresolved",
            "instance Tag<Wrapped<Int>>",
        ),
        (
            "pub fn unused value: Wrapped<Int,Int> -> Unit = ()\n",
            "alias.kind-mismatch",
            "Int",
        ),
    ] {
        let source = format!("{declarations}{tail}");
        let error = compile_module(CompileInput::new("wrong-kind.ssrg", "fixture/alias-sections", &source)).unwrap_err();
        let diagnostic = error.diagnostics.iter().find(|diagnostic| diagnostic.message_key == key).unwrap_or_else(|| panic!("{source}\n{error:#?}"));
        let range = &source[diagnostic.primary.start..diagnostic.primary.end];
        if key == "alias.kind-mismatch" {
            assert_eq!(diagnostic.code, "SES-T0604");
            assert_eq!(range, positioned);
        } else {
            assert_eq!(diagnostic.code, "SES-T0101");
            assert!(range.starts_with(positioned), "{range}");
        }
    }
}

fn project(
    main: &str,
    profile: BuildProfile,
) -> Result<seseragi_driver::CompiledProject, seseragi_driver::ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    graph
        .add_module("fixture/section::model".to_owned(), vec![])
        .unwrap();
    graph
        .add_module(
            "fixture/section::facade".to_owned(),
            vec![("./model".to_owned(), "fixture/section::model".to_owned())],
        )
        .unwrap();
    graph
        .add_module(
            "fixture/section::main".to_owned(),
            vec![("./facade".to_owned(), "fixture/section::facade".to_owned())],
        )
        .unwrap();
    let inputs = [
        ("model", include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-constructor-sections/src/model.ssrg")),
        ("facade", include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-constructor-sections/src/facade.ssrg")),
        ("main", main),
    ].map(|(name, source)| ProjectModuleInput::new(format!("{name}.ssrg"),format!("fixture/section::{name}"),source.to_owned(),format!("dist/{name}.js")).with_profile(profile));
    compile_project(graph, inputs)
}

#[test]
fn opaque_facades_keep_the_provider_kind_metadata_and_distinct_local_owner() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-constructor-sections/src/main.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = project(source, profile).unwrap_or_else(|error| panic!("{error:#?}"));
        let main = &compiled.modules["fixture/section::main"];
        assert_eq!(
            main.typed_hir.instances[0].argument_identities,
            ["fixture/section::model::Sealed<_,std/prelude::Int>"]
        );
        evidence::record(
            "opaque-alias-section",
            profile,
            "main",
            "saved=sealed\nagain=sealed\n",
            [
                ("model", &compiled.modules["fixture/section::model"]),
                ("facade", &compiled.modules["fixture/section::facade"]),
                ("main", main),
            ],
        );
        let wrong = source.replace("create<Bool, Int> True 42", "Local (True, 42)");
        assert_ne!(wrong, source);
        let error = project(&wrong, profile).unwrap_err();
        let seseragi_driver::ProjectCompileError::Diagnostics { modules } = error else {
            panic!("{error:#?}");
        };
        assert!(
            modules
                .iter()
                .any(|module| module.module == "fixture/section::main"
                    && module
                        .diagnostics
                        .diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.code == "SES-T0101"
                            && diagnostic.primary.start < diagnostic.primary.end
                            && diagnostic.primary.end <= wrong.len())),
            "{modules:#?}"
        );
    }
}

#[test]
fn section_programs_preserve_checked_evidence_and_original_ts_in_both_profiles() {
    for (label, source, stdout) in [
        (
            "alias-constructor-sections",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/alias-constructor-sections.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/alias-constructor-sections.stdout"
            ),
        ),
        (
            "alias-section-effects",
            include_str!("../../../examples/spec/fixtures/rust-backend/alias-section-effects.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/alias-section-effects.stdout"
            ),
        ),
        (
            "alias-section-evaluation",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/alias-section-evaluation.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/alias-section-evaluation.stdout"
            ),
        ),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(
                CompileInput::new("sections-雫.ssrg", "fixture/alias-sections", source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{label}\n{error:#?}"));
            evidence::record(label, profile, "main", stdout, [("main", &compiled)]);
        }
    }
}
