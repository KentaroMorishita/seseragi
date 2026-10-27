use seseragi_driver::{
    compile_module, compile_project, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;
use seseragi_semantics::{TypedInstanceImplementation, TypedType};

const PROGRAM: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/alias-type-functions.ssrg");

fn profiles() -> [BuildProfile; 2] {
    [BuildProfile::Development, BuildProfile::Release]
}

#[test]
fn unapplied_permuted_repeated_and_erased_aliases_keep_explicit_scoped_type_functions() {
    for profile in profiles() {
        let compiled = compile_module(
            CompileInput::new("functions-雫.ssrg", "fixture/type-functions", PROGRAM)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        assert_eq!(compiled.typed_hir.instances.len(), 3);
        for (instance, arity) in compiled.typed_hir.instances.iter().zip([2, 1, 1]) {
            let [TypedType::TypeFunction {
                parameter_kinds,
                body,
            }] = instance.arguments.as_slice()
            else {
                panic!("{instance:#?}")
            };
            assert_eq!(parameter_kinds, &vec![0; arity]);
            assert_ne!(body.as_ref(), &TypedType::Hole);
            assert!(instance.argument_identities[0]
                .starts_with(&format!("lambda[{}]", vec!["0"; arity].join(","))));
            let TypedInstanceImplementation::UserDefined { methods } = &instance.implementation
            else {
                panic!("{instance:#?}")
            };
            assert_eq!(methods[0].parameters.len(), 1);
            assert_eq!(methods[0].scheme.type_parameters.len(), arity);
        }
        let TypedType::TypeFunction { body, .. } = &compiled.typed_hir.instances[0].arguments[0]
        else {
            unreachable!()
        };
        assert!(
            matches!(body.as_ref(), TypedType::ExternalNamed { canonical, arguments, .. }
            if canonical == "fixture/type-functions::Pair" && arguments == &[
                TypedType::TypeFunctionParameter { depth: 0, index: 1 },
                TypedType::TypeFunctionParameter { depth: 0, index: 0 },
            ])
        );
        let TypedType::TypeFunction { body, .. } = &compiled.typed_hir.instances[2].arguments[0]
        else {
            unreachable!()
        };
        assert!(
            matches!(body.as_ref(), TypedType::ExternalNamed { canonical, arguments, .. }
            if canonical == "std/prelude::Int" && arguments.is_empty())
        );
        let interface = serde_json::to_value(&compiled.typed_interface).unwrap();
        assert!(interface.to_string().contains("type-function"));
        assert!(!compiled.generated.typescript.contains("unknown as"));
    }
}

#[test]
fn alpha_renamed_alias_chains_are_unused_duplicates_with_the_original_declaration_position() {
    let source = "// 雫🙂\ntype Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>\nalias Other<X,Y> = Flip<X,Y>\ntrait Tag<F<_,_>> { fn tag<A,B> value:F<A,B> -> String }\ninstance Tag<Flip> { fn tag<A,B> value:Pair<B,A> -> String = \"first\" }\ninstance Tag<Other> { fn tag<A,B> value:Pair<B,A> -> String = \"second\" }\n";
    let error = compile_module(CompileInput::new(
        "duplicates.ssrg",
        "fixture/type-functions",
        source,
    ))
    .unwrap_err();
    let duplicate = error
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message_key == "trait.instance-duplicate")
        .unwrap_or_else(|| panic!("{error:#?}"));
    assert!(
        source[duplicate.primary.start..duplicate.primary.end].starts_with("instance Tag<Other>")
    );
    assert!(!duplicate.related.is_empty());
    assert!(duplicate
        .related
        .iter()
        .all(|related| related.primary.end <= source.len()));
}

#[test]
fn free_instance_parameter_loss_wrong_owner_kind_and_method_contract_stay_positioned_refusals() {
    for (source, key) in [
        ("type Box<A> = | Box A\nalias Ignore<T,A> = Box<A>\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance<T> Tag<Ignore<T>> { fn tag<A> value:Box<A> -> String = \"bad\" }\n", "trait.instance-head-unresolved"),
        ("type Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>\ntrait Tag<F<_,_>> { fn tag<A,B> value:F<A,B> -> String }\ninstance Tag<Flip> { fn tag<A,B> value:Pair<A,B> -> String = \"bad\" }\n", "trait.instance-method-signature-mismatch"),
        ("type Pair<A,B> = | Pair (A,B)\ntype Other<A,B> = | Other (A,B)\nalias Repeat<A> = Pair<A,A>\ntrait Tag<F<_>> { fn tag<A> value:F<A> -> String }\ninstance Tag<Repeat> { fn tag<A> value:Other<A,A> -> String = \"bad\" }\n", "trait.instance-method-signature-mismatch"),
        ("alias Erase<A> = Int\ntrait Tag<F<_,_>> { fn tag<A,B> value:F<A,B> -> String }\ninstance Tag<Erase> { fn tag<A,B> value:Int -> String = \"bad\" }\n", "trait.instance-method-signature-mismatch"),
    ] {
        let error = compile_module(CompileInput::new("refused-雫.ssrg", "fixture/type-functions", source)).unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.message_key == key
            && diagnostic.primary.start < diagnostic.primary.end && diagnostic.primary.end <= source.len()), "{error:#?}");
    }
}

#[test]
fn renamed_facades_transport_a_structural_body_without_creating_an_alias_owner() {
    for profile in profiles() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/type-functions::model".into(), [])
            .unwrap();
        graph
            .add_module(
                "fixture/type-functions::facade".into(),
                [("./model".into(), "fixture/type-functions::model".into())],
            )
            .unwrap();
        graph
            .add_module(
                "fixture/type-functions::main".into(),
                [("./facade".into(), "fixture/type-functions::facade".into())],
            )
            .unwrap();
        let model = include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-type-functions/src/model.ssrg");
        let facade = include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-type-functions/src/facade.ssrg");
        let main = include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-type-functions/src/main.ssrg");
        let compiled = compile_project(
            graph,
            [("model", model), ("facade", facade), ("main", main)].map(|(name, source)| {
                ProjectModuleInput::new(
                    format!("{name}.ssrg"),
                    format!("fixture/type-functions::{name}"),
                    source,
                    format!("dist/{name}.js"),
                )
                .with_profile(profile)
            }),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        let model = &compiled.modules["fixture/type-functions::model"];
        assert!(model.typed_hir.instances[0].argument_identities[0]
            .contains("fixture/type-functions::model::Pair"));
        assert!(!model.typed_hir.instances[0].argument_identities[0].contains("::Flip"));
        let main = &compiled.modules["fixture/type-functions::main"];
        assert!(!main.generated.typescript.contains("unknown as"));
        assert_eq!(model.typed_hir.instances.len(), 3);
        for instance in &model.typed_hir.instances {
            let TypedInstanceImplementation::UserDefined { methods } = &instance.implementation
            else {
                panic!("{instance:#?}")
            };
            assert_eq!(methods[0].parameters.len(), 2);
        }
        assert!(main
            .core_ir
            .functions
            .iter()
            .any(|function| function.symbol.ends_with("::main")));
    }
}
