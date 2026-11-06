use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

#[test]
fn text_error_constructors_support_direct_and_saved_display() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/text-error-show.ssrg");
    let core = compile_core_module(CompileInput::new("text-errors.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn text_error_constructor_matches_all_record_field_orders() {
    for fields in [
        ["start", "end", "length"],
        ["start", "length", "end"],
        ["end", "start", "length"],
        ["end", "length", "start"],
        ["length", "start", "end"],
        ["length", "end", "start"],
    ] {
        let annotation = fields.map(|field| format!("{field}: Int")).join(", ");
        let payload = fields.map(|field| format!("{field}: 0")).join(", ");
        let source = format!(
            "import * as text from \"std/text\"\npub fn probe -> String = {{\n let create: {{ {annotation} }} -> text.TextSliceError = text.InvalidScalarRange\n show (create {{ {payload} }})\n}}\n"
        );
        let core = compile_core_module(CompileInput::new(
            "text-errors.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
    }
}

#[test]
fn text_error_constructor_rejects_unproven_record_payloads() {
    let core = compile_core_module(CompileInput::new(
        "text-errors-雫🙂.ssrg",
        "native/main",
        "import * as text from \"std/text\"\npub fn probe -> String = show (text.InvalidScalarRange { start: -1, end: 2, length: 6 })",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..9 {
        let mut changed = core.clone();
        let CoreExpr::Call { arguments, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let constructor = &mut arguments[0];
        let origin = core_expr_origin(constructor).clone();
        let CoreExpr::Call { arguments, .. } = constructor else {
            panic!()
        };
        let CoreExpr::Record { type_ref, .. } = &mut arguments[0] else {
            panic!()
        };
        let CoreType::Record { closed, fields } = type_ref else {
            panic!()
        };
        match mutation {
            0 => *closed = false,
            1 => {
                fields.pop();
            }
            2 => {
                let mut extra = fields[0].clone();
                extra.name = "extra".into();
                fields.push(extra);
            }
            3 => fields[0].name = "counterfeit".into(),
            4 => fields[0].optional = true,
            5 => fields[0].type_ref = CoreType::Hole,
            6 => {
                fields[0].type_ref = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "counterfeit::Int".into(),
                    arguments: vec![],
                }
            }
            7 => {
                fields[0].type_ref = CoreType::ExternalNamed {
                    name: "String".into(),
                    canonical: "std/prelude::String".into(),
                    arguments: vec![],
                }
            }
            _ => fields[0].name = fields[1].name.clone(),
        }
        let error = emit_rust_module(&changed, "probe").expect_err("unproven record payload");
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation {mutation}");
    }
}

#[test]
fn checked_text_line_filter_and_file_program_have_proven_native_operations() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/text-line-filter.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/filesystem-text-line-filter.ssrg"
        ),
    ] {
        let core = compile_core_module(CompileInput::new("text-雫🙂.ssrg", "native/main", source))
            .unwrap()
            .core_ir;
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact.metadata.capabilities.iter().any(|c| c == "text"));
    }
}

#[test]
fn checked_text_operations_keep_first_class_and_saved_source_stages() {
    let source = r#"
import * as text from "std/text"
pub effect fn main = do {
  let lines: String -> Array<String> = text.lines
  let join: String -> Array<String> -> String = text.join
  let saved = join "|"
  println (saved (lines "a\n\n🙂\n"))
}
"#;
    let core = compile_core_module(CompileInput::new("text-stages.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/text-scalars.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/text-scalars-effects.ssrg"),
    ] {
        let core = compile_core_module(CompileInput::new(
            "text-scalars.ssrg",
            "native/main",
            source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "main").unwrap();
    }
}

#[test]
fn program_wasm_admits_string_text_operations_and_rejects_bytes_and_unicode() {
    for (operation, signature) in [
        ("isEmpty", "String -> Bool"),
        ("concat", "Array<String> -> String"),
        ("lengthScalars", "String -> Int"),
        ("lengthBytes", "String -> Int"),
        ("scalarAt", "Int -> String -> Maybe<Char>"),
        ("words", "String -> Array<String>"),
        ("trimStart", "String -> String"),
        ("trimEnd", "String -> String"),
        ("replaceAll", "String -> String -> String -> String"),
    ] {
        let source = format!("import * as text from \"std/text\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = text.{operation}; value }}\npub fn nativeProbe -> Int = probe 42\n");
        let core = compile_core_module(CompileInput::new("text-wasm.ssrg", "native/main", &source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "nativeProbe").unwrap();
        emit_program_wasm(&[&core], "native/main", &["probe".into()])
            .unwrap_or_else(|error| panic!("{operation}: {error}"));
    }
    // UTF-8 Bytes conversions and the Unicode tables need their own proof.
    for (import, value) in [
        ("std/text", "let saved = text.encodeUtf8"),
        (
            "std/text",
            "let saved: Int -> Int -> String -> Either<text.TextSliceError, String> = text.sliceScalars",
        ),
        (
            "std/text/unicode",
            "let saved: String -> String = text.fullCaseFold",
        ),
    ] {
        let source = format!("import * as text from \"{import}\"\npub fn probe value: Int -> Int = {{ {value}; value }}\npub fn nativeProbe -> Int = probe 42\n");
        let core = compile_core_module(CompileInput::new("text-wasm.ssrg", "native/main", &source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "nativeProbe").unwrap();
        let error = emit_program_wasm(&[&core], "native/main", &["probe".into()]).unwrap_err();
        assert!(
            error
                .message
                .contains("native text extension requires a target-specific proof"),
            "{import}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "text-wasm.ssrg");
    }
}

#[test]
fn text_operations_reject_unproven_types_evidence_and_arity_at_source() {
    let core = compile_core_module(CompileInput::new(
        "text-雫🙂.ssrg",
        "native/main",
        "import * as text from \"std/text\"\npub fn probe -> Array<String> = text.lines \"雫\\n\"",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let eq = compile_core_module(CompileInput::new(
        "other.ssrg",
        "native/other",
        "pub fn probe -> Bool = 1 == 1",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Binary {
        evidence: other_evidence,
        ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for mutation in 0..8 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence,
            deferred_evidence_parameters,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => {
                *type_ref = CoreType::Named {
                    name: "Array".into(),
                    arguments: vec![CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    }],
                }
            }
            2 => {
                *type_ref = CoreType::ExternalNamed {
                    name: "Array".into(),
                    canonical: "fake::Array".into(),
                    arguments: vec![CoreType::Named {
                        name: "String".into(),
                        arguments: vec![],
                    }],
                }
            }
            3 => arguments.clear(),
            4 => arguments.push(arguments[0].clone()),
            5 => evidence.push(other_evidence[0].clone()),
            6 => deferred_evidence_parameters.push(CoreType::Hole),
            _ => {
                arguments[0] = CoreExpr::Integer {
                    value: "1".into(),
                    origin: core_expr_origin(&arguments[0]).clone(),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "text-雫🙂.ssrg");
    }
}
