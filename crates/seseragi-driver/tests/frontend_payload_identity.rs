use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[path = "support/frontend_evidence.rs"]
mod evidence;

fn compile(source: &str, profile: BuildProfile) -> seseragi_driver::CompiledModule {
    compile_module(
        CompileInput::new("payload-雫.ssrg", "fixture/payload-identity", source)
            .with_profile(profile),
    )
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
}

#[test]
fn imported_closed_adt_payload_keeps_its_owner_in_nested_matches() {
    for (declaration, pattern) in [
        (
            "type Failure = | Wrapped floats.FloatConversionError",
            "Wrapped value",
        ),
        (
            "alias Conversion = floats.FloatConversionError\ntype Failure = | Wrapped Conversion",
            "Wrapped value",
        ),
        (
            "type Failure = | Wrapped (String, floats.FloatConversionError)",
            "Wrapped (_, value)",
        ),
        (
            "struct Envelope { error: floats.FloatConversionError }\ntype Failure = | Wrapped Envelope",
            "Wrapped (Envelope { error: value })",
        ),
    ] {
        let source = format!(
            "import * as floats from \"std/float\"\n\
             {declaration}\n\
             fn describe failure: Failure -> String = match failure {{\n\
               {pattern} -> match value {{\n\
                 floats.FloatNotFinite -> \"not-finite\"\n\
                 floats.FloatOutsideIntRange -> \"outside-int-range\"\n\
               }}\n\
             }}\n"
        );
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile(&source, profile);
        }
    }
}

#[test]
fn imported_closed_adt_payload_matches_the_explicit_helper_control() {
    let source = r#"
import * as floats from "std/float"
type FloatConversionError = | LocalError
type Failure = | Wrapped floats.FloatConversionError
fn describeConversion value: floats.FloatConversionError -> String = match value {
  floats.FloatNotFinite -> "not-finite"
  floats.FloatOutsideIntRange -> "outside-int-range"
}
fn nested failure: Failure -> String = match failure {
  Wrapped value -> match value {
    floats.FloatNotFinite -> "not-finite"
    floats.FloatOutsideIntRange -> "outside-int-range"
  }
}
fn helper failure: Failure -> String = match failure {
  Wrapped value -> describeConversion value
}
pub effect fn main = do {
  println (nested (Wrapped floats.FloatNotFinite))
  println (helper (Wrapped floats.FloatNotFinite))
  println (nested (Wrapped floats.FloatOutsideIntRange))
  println (helper (Wrapped floats.FloatOutsideIntRange))
}
"#;
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile(source, profile);
    }
}

#[test]
fn imported_payload_rejects_a_constructor_from_a_different_canonical_owner() {
    for (payload, branch) in [
        ("floats.FloatConversionError", "LocalError"),
        ("FloatConversionError", "floats.FloatNotFinite"),
        ("Conversion", "LocalError"),
    ] {
        let source = format!(
            "import * as floats from \"std/float\"\n\
             type FloatConversionError = | LocalError\n\
             alias Conversion = floats.FloatConversionError\n\
             type Failure = | Wrapped {payload}\n\
             fn describe failure: Failure -> String = match failure {{\n\
               Wrapped value -> match value {{ {branch} -> \"invalid\"; _ -> \"other\" }}\n\
             }}\n"
        );
        let error = compile_module(CompileInput::new(
            "payload-wrong-owner.ssrg",
            "fixture/payload-identity",
            &source,
        ))
        .unwrap_err();
        assert!(
            format!("{error:#?}").contains("does not belong to the matched type"),
            "{source}\n{error:#?}"
        );
    }
}

#[test]
fn imported_adt_aliases_share_one_canonical_owner_across_namespace_and_direct_imports() {
    let source = r#"
import { FloatConversionError as Direct, FloatNotFinite as NotFinite } from "std/float"
import * as floats from "std/float"
alias Conversion = Direct
type Failure = | Wrapped Conversion
fn describe failure: Failure -> String = match failure {
  Wrapped value -> match value {
    floats.FloatNotFinite -> "not-finite"
    floats.FloatOutsideIntRange -> "outside-int-range"
  }
}
pub effect fn main = println (describe (Wrapped NotFinite))
"#;
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile(source, profile);
    }
}

#[test]
fn imported_payload_execution_fixture_keeps_the_original_generated_typescript() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/imported-error-payload-match.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile(source, profile);
        evidence::record(
            "imported-error-payload-match",
            profile,
            "main",
            "not-finite\nnot-finite\noutside-int-range\noutside-int-range\n",
            [("main", &compiled)],
        );
    }
}
