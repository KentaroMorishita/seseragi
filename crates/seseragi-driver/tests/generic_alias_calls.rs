use seseragi_driver::{
    compile_module, compile_project, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;

const ORIGINAL: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-regressions/validation-inferred-return.ssrg"
);

#[test]
fn inferred_validation_alias_calls_keep_checked_type_arguments_in_both_profiles() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("alias-雪🙂.ssrg", "fixture/alias-calls", ORIGINAL)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        assert!(
            compiled
                .generated
                .typescript
                .contains("extract<string, number>"),
            "{profile:?}:\n{}",
            compiled.generated.typescript,
        );
    }
}

#[test]
fn chained_permuted_repeated_and_callable_aliases_keep_checked_type_arguments() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend-regressions/generic-alias-calls.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("aliases.ssrg", "fixture/alias-calls", source).with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{profile:?}: {error:#?}"));
        for call in [
            "extract<string, number>",
            "relay<string, number>",
            "repeated<number>",
            "first<string>",
            "apply<number>",
        ] {
            assert!(
                compiled.generated.typescript.contains(call),
                "{profile:?}: {call}\n{}",
                compiled.generated.typescript,
            );
        }
    }
}

#[test]
fn imported_renamed_aliases_keep_caller_binders_and_canonical_array_arguments() {
    let model = include_str!(
        "../../../examples/spec/fixtures/rust-backend-projects/generic-alias-calls/src/model.ssrg"
    );
    let main = include_str!(
        "../../../examples/spec/fixtures/rust-backend-projects/generic-alias-calls/src/main.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/alias-calls::model".into(), [])
            .unwrap();
        graph
            .add_module(
                "fixture/alias-calls::main".into(),
                [("./model".into(), "fixture/alias-calls::model".into())],
            )
            .unwrap();
        let compiled = compile_project(
            graph,
            [("model", model), ("main", main)].map(|(name, source)| {
                ProjectModuleInput::new(
                    format!("{name}.ssrg"),
                    format!("fixture/alias-calls::{name}"),
                    source,
                    format!("dist/{name}.js"),
                )
                .with_profile(profile)
            }),
        )
        .unwrap_or_else(|error| panic!("{profile:?}: {error:#?}"));
        let ts = &compiled.modules["fixture/alias-calls::main"]
            .generated
            .typescript;
        assert!(ts.contains("extract<string, number>"), "{profile:?}: {ts}");
        assert!(ts.contains("first<string>"), "{profile:?}: {ts}");
    }
}

#[test]
fn alias_bodies_keep_fixed_types_apart_from_caller_binders() {
    for source in [
        "alias Values<A> = (A, Int)\npub fn keep<Int> value:Int -> Values<Int> = (value, 0)\n",
        "type Label = | Label Int\nalias Values<A> = (A, Label)\npub fn keep<A> value:A -> Values<A> = (value, Label 0)\n",
        "alias Ignore<A, B> = B\nfn keep<A, B> unused:A -> value:Ignore<A, B> -> B = value\npub fn probe -> Int = keep \"ignored\" 42\n",
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile_module(
                CompileInput::new("binders.ssrg", "fixture/alias-calls", source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{profile:?}: {source}\n{error:#?}"));
        }
    }
}

#[test]
fn alias_argument_mismatches_keep_their_positioned_source_diagnostics() {
    for (source, key) in [
        ("type Box<A> = | Box A\ntype Other<A> = | Other A\nalias Value<A> = Box<A>\nfn consume<A> value:Value<A> -> A = match value { Box found -> found }\npub fn wrong -> Int = consume (Other 1)\n", "call.argument-type-mismatch"),
        ("alias Same<A> = (A, A)\nfn first<A> value:Same<A> -> A = match value { (a, _) -> a }\npub fn wrong -> Int = first (1, True)\n", "call.argument-type-mismatch"),
        ("import { Validation } from \"std/validation\"\nimport * as validation from \"std/validation\"\nalias Check<E, A> = Validation<E, A>\nfn require value:Check<String, Int> -> Int = 0\npub fn wrong -> Int = require (validation.valid True)\n", "call.argument-type-mismatch"),
        ("alias Values<A> = (A, Int)\npub fn wrong<Int> value:Int -> Values<Int> = (value, value)\n", "function.return-type-mismatch"),
        ("type Label = | Label Int\nalias Values<A> = (A, Label)\npub fn wrong<Label> value:Label -> Values<Label> = (value, value)\n", "function.return-type-mismatch"),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let error = compile_module(
                CompileInput::new("wrong-雪🙂.ssrg", "fixture/alias-calls", source)
                    .with_profile(profile),
            )
            .unwrap_err();
            assert!(
                error.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == "SES-T0101"
                        && diagnostic.message_key == key
                        && diagnostic.primary.start < diagnostic.primary.end
                        && diagnostic.primary.end <= source.len()
                }),
                "{profile:?}: {source}\n{error:#?}",
            );
        }
    }
}
