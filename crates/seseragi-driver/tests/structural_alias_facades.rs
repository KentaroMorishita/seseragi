use seseragi_driver::{
    compile_module, compile_project, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;
use seseragi_semantics::TypedInstanceImplementation;

#[test]
fn structural_alias_facades_preserve_type_only_names_and_method_source_arity() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/structural::model".into(), [])
            .unwrap();
        graph
            .add_module(
                "fixture/structural::facade".into(),
                [("./model".into(), "fixture/structural::model".into())],
            )
            .unwrap();
        graph
            .add_module(
                "fixture/structural::main".into(),
                [("./facade".into(), "fixture/structural::facade".into())],
            )
            .unwrap();
        let inputs = [
            ("model", include_str!("../../../examples/spec/fixtures/rust-backend-projects/structural-aliases/src/model.ssrg")),
            ("facade", include_str!("../../../examples/spec/fixtures/rust-backend-projects/structural-aliases/src/facade.ssrg")),
            ("main", include_str!("../../../examples/spec/fixtures/rust-backend-projects/structural-aliases/src/main.ssrg")),
        ];
        let compiled = compile_project(
            graph,
            inputs.map(|(name, source)| {
                ProjectModuleInput::new(
                    format!("{name}.ssrg"),
                    format!("fixture/structural::{name}"),
                    source,
                    format!("dist/{name}.js"),
                )
                .with_profile(profile)
            }),
        )
        .unwrap_or_else(|e| panic!("{e:#?}"));
        let model = &compiled.modules["fixture/structural::model"];
        assert_eq!(model.typed_hir.instances.len(), 3);
        for instance in &model.typed_hir.instances {
            let TypedInstanceImplementation::UserDefined { methods } = &instance.implementation
            else {
                panic!("{instance:#?}")
            };
            assert_eq!(methods[0].parameters.len(), 1);
            assert_eq!(methods[0].scheme.type_parameters.len(), 1);
        }
        for module in compiled.modules.values() {
            assert!(!module.generated.typescript.contains("unknown as"));
        }
        let facade = &compiled.modules["fixture/structural::facade"]
            .generated
            .typescript;
        for name in ["Query", "Row", "Pair"] {
            assert!(facade.contains(name), "{facade}");
        }
    }
}

#[test]
fn returned_structural_aliases_reject_different_owners_and_free_caller_binders() {
    for (source, key) in [
        ("alias Reader<A> = Int -> A\nfn read value: Bool -> Int = 1\npub fn wrong -> Reader<Int> = read\n", "function.return-type-mismatch"),
        ("alias Reader<A> = Int -> A\nfn read value: Int -> Bool = True\npub fn wrong -> Reader<Int> = read\n", "function.return-type-mismatch"),
        ("alias Reader<A> = Int -> A\nfn source<A> value:A -> A = value\npub fn wrong<A> value:A -> (Reader<Int> -> A) = \\_reader:Reader<Int> -> source 42\n", "lambda.body-type-mismatch"),
        ("type Int = | Local\nalias Reader<A> = Int -> A\nfn read value:Bool -> Bool = value\npub fn wrong -> Reader<Bool> = read\n", "function.return-type-mismatch"),
        ("alias Shape<A> = { value: A, callback: Unit -> A }\nfn use<A> value:Shape<A> -> A = value.value\npub fn wrong -> Int = use { value:42, callback: \\_:Unit -> True }\n", "call.argument-type-mismatch"),
        ("alias Shape<A> = { value: A, callback: Unit -> A }\nfn use<A> value:Shape<A> -> A = value.value\npub fn wrong -> Int = use<Int> { value:42, callback: \\_:Unit -> True }\n", "call.argument-type-mismatch"),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let error = compile_module(CompileInput::new("wrong-雫.ssrg", "fixture/structural", source).with_profile(profile)).unwrap_err();
            assert!(error.diagnostics.iter().any(|d| d.message_key == key && d.primary.start < d.primary.end && d.primary.end <= source.len()), "{source}\n{error:#?}");
        }
    }
}
