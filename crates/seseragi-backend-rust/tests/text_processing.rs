use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

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
