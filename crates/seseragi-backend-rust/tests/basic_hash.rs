use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("hash-雪🙂.ssrg", "native/hash", source))
        .unwrap_or_else(|e| panic!("{e:?}\n{source}"))
        .core_ir
}

#[test]
fn native_hash_preserves_standard_user_scoped_and_saved_dictionaries() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-hash.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "hash"));
}

#[test]
fn native_hash_accepts_the_five_original_official112_unit_inputs() {
    for value in ["7", "True", "'雫'", "\"雫🙂\"", "()"] {
        let source = format!("pub fn probe -> Unit = {{\n  let result = hash {value}\n  ()\n}}\n");
        emit_rust_module(&checked(&source), "probe").unwrap();
    }
}

#[test]
fn native_hash_does_not_substitute_primitive_dictionaries_for_same_named_binders() {
    for (binder, value) in [("Int", "42"), ("String", "\"snow\"")] {
        let mut core = checked(&format!(
            "fn identity<{binder}> value: {binder} -> {binder} = value\npub fn main -> Int = 0\n"
        ));
        let other = checked(&format!("pub fn probe -> Int = hash {value}\n"));
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
fn existing_user_hash_program_wasm_remains_accepted() {
    // This exact source already emits native and program WASM on official112.
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-hash-user.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
    emit_program_wasm(&[&core], "native/hash", &["probe".into()]).unwrap();
}

#[test]
fn new_primitive_hash_is_refused_even_when_a_constrained_callee_ignores_it() {
    let core=checked("fn ignore<A> _value: A -> Int where Hash<A> = 1\npub fn probe value: Int -> Int = ignore value\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    let error = emit_program_wasm(&[&core], "native/hash", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("requires a target-specific"));
}

#[test]
fn native_hash_rejects_unproved_identity_owner_kind_source_stages_and_results() {
    let core = checked("// 雪🙂\npub fn probe -> Int = hash 42\n");
    let origin = core_expr_origin(&core.functions[0].body).clone();
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..17 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Hash".into()),
            1 => evidence[0].constraint.name = "Eq".into(),
            2 => evidence[0].constraint.arguments.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
            5 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            6 => {
                let extra = evidence[0].clone();
                let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            7 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "std/string::Hash".into();
            }
            8 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            9 => trait_dispatch.as_mut().unwrap().method = "eq".into(),
            10 => trait_dispatch.as_mut().unwrap().trait_identity = "std/prelude::Show".into(),
            11 => {
                *type_ref = CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                }
            }
            12 => arguments.clear(),
            13 => arguments.push(arguments[0].clone()),
            14 => deferred_evidence_parameters.push(CoreType::Hole),
            15 => deferred_evidence_type_constructor_parameters.push("F".into()),
            _ => *evidence_argument_index = Some(0),
        }
        let error = emit_rust_module(&changed, "probe")
            .err()
            .unwrap_or_else(|| panic!("mutation{mutation} accepted"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn native_hash_keeps_saved_primitive_methods_out_of_program_wasm() {
    for ty in ["Int", "Bool", "Char", "String", "Unit"] {
        let core=checked(&format!("pub fn probe value: Int -> Int = {{ let saved: {ty} -> Int = hash; value }}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/hash", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("requires a target-specific"));
        assert_eq!(error.origin.unwrap().source, "hash-雪🙂.ssrg");
    }
}
