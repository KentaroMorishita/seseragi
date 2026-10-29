use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("bytes-雪🙂.ssrg", "native/bytes", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}

#[test]
fn basic_bytes_accept_closed_signatures_saved_functions_and_buffers() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-bytes.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-bytes-effects.ssrg"),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact.metadata.capabilities.iter().any(|c| c == "bytes"));
    }
}

#[test]
fn basic_bytes_reject_forged_types_evidence_and_source_stages_at_the_original_span() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (parameters, result, expression) in [
        (
            "",
            "Either<bytes.ByteError, bytes.Bytes>",
            "bytes.fromInts [0, 255]",
        ),
        (" value: bytes.Bytes", "Array<Int>", "bytes.toInts value"),
        (
            " value: bytes.Bytes",
            "Maybe<bytes.Byte>",
            "bytes.get 0 value",
        ),
        (" value: bytes.Byte", "Int", "bytes.toInt value"),
    ] {
        let source = format!("// 雪🙂\nimport * as bytes from \"std/bytes\"\npub fn probe{parameters} -> {result} = {expression}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let expected_origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..10 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[probe].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: result.into(),
                        canonical: "fake::Result".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/bytes::Bytes".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 3]),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                _ => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven Bytes call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(
                error.origin.unwrap(),
                expected_origin,
                "{expression}/{mutation}"
            );
        }
    }
}

#[test]
fn bytes_arguments_do_not_substitute_lexical_opaque_type_binders() {
    let mut core =
        checked("fn identity<Bytes> value: Bytes -> Bytes = value\npub fn main -> Int = 0\n");
    let call_core = checked("import * as bytes from \"std/bytes\"\nfn probe value: bytes.Bytes -> Array<Int> = bytes.toInts value\npub fn main -> Int = 0\n");
    let mut call = call_core
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::probe"))
        .unwrap()
        .body
        .clone();
    let identity = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::identity"))
        .unwrap();
    let CoreExpr::Call { arguments, .. } = &mut call else {
        panic!()
    };
    arguments[0] = identity.body.clone();
    let expected_origin = core_expr_origin(&call).clone();
    identity.body = call;
    let error = emit_rust_module(&core, "main")
        .expect_err("lexical Bytes is not the opaque canonical Bytes");
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap(), expected_origin);
}

#[test]
fn basic_bytes_do_not_silently_expand_program_wasm_admission() {
    for expression in [
        "bytes.fromInts [0, 255]",
        "bytes.toInts",
        "bytes.get",
        "bytes.toInt",
    ] {
        let source = format!("import * as bytes from \"std/bytes\"\npub fn probe value: Int -> Int = {{ let result = {expression}; value }}\npub fn main -> Int = probe 1\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/bytes", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific"),
            "{expression}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "bytes-雪🙂.ssrg");
    }
}
