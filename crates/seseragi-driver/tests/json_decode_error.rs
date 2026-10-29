use seseragi_driver::{analyze_module, compile_module, CompileInput};
use seseragi_project::BuildProfile;

#[test]
fn public_decode_error_supports_handwritten_decoder_with_exact_path_and_kind() {
    let source = "import * as json from \"std/json\"\nimport { DecodeError } from \"std/json\"\npub fn custom _value: json.Json -> Either<json.DecodeError, Int> = Left (DecodeError { path: [json.JsonField \"雪🙂\", json.JsonIndex 1], kind: json.InvalidJsonValue \"domain\" })\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("decoder-雪🙂.ssrg", "test/decoder", source).with_profile(profile),
        )
        .unwrap();
        assert!(compiled.generated.typescript.contains("path"));
        assert!(compiled.generated.typescript.contains("domain"));
    }
}

#[test]
fn public_decode_error_still_rejects_missing_or_wrong_fields_and_opaque_nominals() {
    for (expression, key) in [
        (
            "DecodeError { path: [], kind: \"wrong\" }",
            "struct.field-type-mismatch",
        ),
        (
            "DecodeError { path: [1], kind: json.InvalidJsonValue \"domain\" }",
            "array.element-type-mismatch",
        ),
        ("DecodeError { path: [] }", "struct.field-missing"),
        (
            "Json { path: [], kind: json.InvalidJsonValue \"domain\" }",
            "struct.construction-on-non-struct",
        ),
    ] {
        let source = format!("// 雪🙂\nimport * as json from \"std/json\"\nimport {{ DecodeError, Json }} from \"std/json\"\npub fn custom -> json.DecodeError = {expression}\n");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let errors = compile_module(
                CompileInput::new("decoder-雪🙂.ssrg", "test/decoder", &source)
                    .with_profile(profile),
            )
            .unwrap_err();
            assert!(
                errors.diagnostics.iter().any(|d| d.message_key == key),
                "{errors:?}"
            );
            assert_eq!(
                analyze_module(
                    CompileInput::new("decoder-雪🙂.ssrg", "test/decoder", &source)
                        .with_profile(profile)
                )
                .diagnostics,
                errors
            );
            assert!(errors
                .diagnostics
                .iter()
                .all(|d| source.is_char_boundary(d.primary.start)
                    && source.is_char_boundary(d.primary.end)));
        }
    }
}
