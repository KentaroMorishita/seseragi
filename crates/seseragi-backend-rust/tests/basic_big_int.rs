use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("big-雪🙂.ssrg", "native/big", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}

#[test]
fn basic_big_int_accepts_saved_functions_exact_values_and_captured_opaque_storage() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-big-int-effects.ssrg"),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "big-int"));
    }
}

#[test]
fn basic_big_int_rejects_forged_owner_kind_evidence_and_stages_at_the_original_byte_span() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (parameter, result, expression) in [
        (
            "",
            "Either<big.BigIntParseError, big.BigInt>",
            "big.parse \"900719925474099200000001\"",
        ),
        (" value: big.BigInt", "String", "big.format value"),
        ("", "big.BigInt", "big.fromInt 42"),
    ] {
        let source = format!("// 雪🙂\nimport * as big from \"std/big-int\"\nfn probe{parameter} -> {result} = {expression}\npub fn main -> Int = 0\n");
        let core = checked(&source);
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..11 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
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
                        name: "std/big-int::BigInt".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                _ => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main")
                .err()
                .unwrap_or_else(|| panic!("{expression}/{mutation} accepted"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn basic_big_int_does_not_replace_scoped_opaque_or_int_binders() {
    for (binder, result, expression) in [
        ("BigInt", "String", "big.format (big.fromInt 42)"),
        ("Int", "big.BigInt", "big.fromInt 42"),
    ] {
        let mut core = checked(&format!(
            "fn identity<{binder}> value: {binder} -> {binder} = value\npub fn main -> Int = 0\n"
        ));
        let other = checked(&format!(
            "import * as big from \"std/big-int\"\npub fn probe -> {result} = {expression}\n"
        ));
        let mut call = other.functions[0].body.clone();
        let identity = core
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::identity"))
            .unwrap();
        let CoreExpr::Call { arguments, .. } = &mut call else {
            panic!()
        };
        arguments[0] = identity.body.clone();
        let origin = core_expr_origin(&call).clone();
        identity.body = call;
        let error = emit_rust_module(&core, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
}

#[test]
fn basic_big_int_keeps_program_wasm_admission_explicit() {
    for (name, signature) in [
        (
            "parse",
            "String -> Either<big.BigIntParseError, big.BigInt>",
        ),
        ("format", "big.BigInt -> String"),
        ("fromInt", "Int -> big.BigInt"),
    ] {
        let source=format!("import * as big from \"std/big-int\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = big.{name}; value }}\npub fn main -> Int = probe 1\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/big", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("requires a target-specific"));
        assert_eq!(error.origin.unwrap().source, "big-雪🙂.ssrg");
    }
}
