use seseragi_driver::{compile_module, BuildProfile, CompileInput};

const ORIGINAL: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-regressions/standard-returned-application.ssrg"
);

#[test]
fn inferred_function_results_retain_the_runtime_source_boundary_in_both_profiles() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for source in [
            ORIGINAL.to_owned(),
            "import * as maybe from \"std/maybe\"\npub fn probe -> Int = maybe.withDefault (\\x:Int -> \\y:Int -> x + y) (Just (\\x:Int -> \\y:Int -> x * y)) 6 7\n".into(),
            "fn choose<A> fallback:A -> value:Maybe<A> -> A = match value { Nothing -> fallback; Just found -> found }\npub fn probe -> Int = choose (\\value:Int -> value + 1) Nothing 41\n".into(),
        ] {
            let compiled = compile_module(
                CompileInput::new("returned-雪🙂.ssrg", "fixture/returned", &source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{error:#?}\n{source}"));
            assert!(compiled.diagnostics.diagnostics.is_empty());
            if source.contains("maybe.withDefault") {
                let ts = &compiled.generated.typescript;
                assert!(ts.contains("__ssrg$collection$returned"), "{ts}");
            }
        }
    }
}

#[test]
fn noncallable_results_and_incompatible_returned_arguments_remain_source_errors() {
    for expression in [
        "maybe.withDefault 1 (Just 2) 3",
        "maybe.withDefault (\\value:Int -> value + 1) Nothing true",
        "maybe.withDefault (\\value:Int -> value + 1) Nothing 41 0",
    ] {
        let source =
            format!("import * as maybe from \"std/maybe\"\npub fn probe -> Int = {expression}\n");
        assert!(
            compile_module(CompileInput::new(
                "returned-invalid.ssrg",
                "fixture/returned-invalid",
                &source,
            ))
            .is_err(),
            "{expression}"
        );
    }
}
