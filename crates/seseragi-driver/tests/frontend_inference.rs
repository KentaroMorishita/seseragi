use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[path = "support/frontend_evidence.rs"]
mod evidence;

#[test]
fn same_type_operator_applications_use_source_and_remaining_function_types() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-source-partials.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("eq-source-雫.ssrg", "fixture/eq-source-partial", source)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        evidence::record(
            "equality-source-partial",
            profile,
            "main",
            "True\nTrue\nTrue\nTrue\n",
            [("main", &compiled)],
        );
    }
}

#[test]
fn source_inference_preserves_wrong_owner_argument_and_missing_instance_rejection() {
    for source in [
        "let same: Int -> Bool = (==) \"雫\"\n",
        "let same: String -> Bool = (==) 42\n",
        "let same: Float -> Bool = (==) 1.5\n",
        "let same: String -> Int = (==) \"雫\"\n",
        "let same: String -> Int -> Bool = (==) \"雫\"\n",
        "type String = | LocalString\nlet same: String -> Bool = (==) \"雫\"\n",
        "fn wrong<A, B> left: A -> (B -> Bool) where Eq<A>, Eq<B> = (==) left\n",
    ] {
        let error = compile_module(CompileInput::new(
            "eq-source-wrong-owner.ssrg",
            "fixture/eq-source-partial",
            source,
        ))
        .unwrap_err();
        assert!(
            error.diagnostics.iter().any(|diagnostic| {
                matches!(diagnostic.code.as_str(), "SES-T0101" | "SES-T0201")
            }),
            "{source}\n{error:#?}"
        );
    }
}

#[test]
fn custom_operator_partial_keeps_its_declared_parameter_and_result_types() {
    let source = "operator infixl 4 <=> left: Int -> right: String -> String = right\nlet partial: String -> String = (<=>) 42\npub effect fn main = println (partial \"custom\")\n";
    compile_module(CompileInput::new(
        "custom-operator-partial.ssrg",
        "fixture/custom-operator",
        source,
    ))
    .unwrap();
}

#[test]
fn ref_get_keeps_builtin_array_identity_under_a_same_named_nominal() {
    for declaration in [
        "",
        "type Array<A> = | LocalArray A\ninstance<A> Show<Array<A>> where Show<A> { fn show value: Array<A> -> String = match value { LocalArray item -> \"local=\" + show item } }\n",
    ] {
        let source = format!(
            "import * as refs from \"std/ref\"\n\
             {declaration}\n\
             pub effect fn main = do {{\n\
               value <- refs.make [1, 2]\n\
               current <- refs.get value\n\
               println (show current)\n\
               refs.set [3, 4] value\n\
               updated <- refs.get value\n\
               println (show updated)\n\
             }}\n"
        );
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile_module(
                CompileInput::new("ref-array-identity.ssrg", "fixture/ref-array", &source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        }
    }
}

#[test]
fn ref_array_execution_fixture_keeps_both_owners_and_updated_values() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/ref-array-shadow.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("ref-array-雫.ssrg", "fixture/ref-array", source)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        evidence::record(
            "ref-array-shadow",
            profile,
            "main",
            "[1, 2]\n[3, 4]\nlocal=5\n",
            [("main", &compiled)],
        );
    }
}
