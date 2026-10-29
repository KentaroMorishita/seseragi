use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("array-雪🙂.ssrg", "native/array", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}

#[test]
fn basic_array_accepts_generic_saved_and_partial_callables_with_captured_elements() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-ordering.ssrg"),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "array-operations"));
    }
}

#[test]
fn basic_array_preserves_the_original_former_refusals() {
    for (expression, result) in [
        ("filter keep [1, 2]", "Array<Int>"),
        ("find keep [1, 2]", "Maybe<Int>"),
        ("get 0 [1, 2]", "Maybe<Int>"),
        ("length [1, 2]", "Int"),
        ("take 1 [1, 2]", "Array<Int>"),
        ("drop 1 [1, 2]", "Array<Int>"),
    ] {
        let source = format!("import * as arrays from \"std/array\"\nfn keep _value: Int -> Bool = True\npub fn probe -> {result} = arrays.{expression}\n");
        emit_rust_module(&checked(&source), "probe").unwrap();
    }
}

#[test]
fn basic_array_rejects_forged_owner_element_kind_evidence_and_staging_at_the_original_span() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (expression, result) in [
        ("filter keep [1, 2]", "Array<Int>"),
        ("find keep [1, 2]", "Maybe<Int>"),
        ("get 0 [1, 2]", "Maybe<Int>"),
        ("length [1, 2]", "Int"),
        ("take 1 [1, 2]", "Array<Int>"),
        ("drop 1 [1, 2]", "Array<Int>"),
    ] {
        let source = format!("// 雪🙂\nimport * as arrays from \"std/array\"\nfn keep _value: Int -> Bool = True\npub fn probe -> {result} = arrays.{expression}\n");
        let core = checked(&source);
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..if expression.starts_with("length") {
            12
        } else {
            13
        } {
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
                        name: "std/prelude::Int".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 3]),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "equals".into(),
                        method_contract: None,
                    })
                }
                10 => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                11 => {
                    let CoreExpr::Array { type_ref, .. } = arguments.last_mut().unwrap() else {
                        panic!()
                    };
                    *type_ref = CoreType::ExternalNamed {
                        name: "Array".into(),
                        canonical: "fake::Array".into(),
                        arguments: vec![CoreType::Named {
                            name: "Int".into(),
                            arguments: vec![],
                        }],
                    };
                }
                _ => {
                    let CoreExpr::Array { type_ref, .. } = arguments.last_mut().unwrap() else {
                        panic!()
                    };
                    *type_ref = CoreType::Named {
                        name: "Array".into(),
                        arguments: vec![CoreType::Named {
                            name: "String".into(),
                            arguments: vec![],
                        }],
                    };
                }
            }
            let error = emit_rust_module(&changed, "probe")
                .err()
                .unwrap_or_else(|| panic!("unproven Array call accepted: {expression}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn basic_array_does_not_replace_lexical_array_or_int_binders() {
    for (binder, kind, parameter, result, expression) in [
        ("Array", "<_>", "Array<Int>", "Int", "arrays.length [1, 2]"),
        ("Int", "", "Int", "Maybe<Int>", "arrays.get 0 [1, 2]"),
    ] {
        let mut core = checked(&format!("fn identity<{binder}{kind}> value: {parameter} -> {parameter} = value\npub fn main -> Int = 0\n"));
        let mut call = checked(&format!(
            "import * as arrays from \"std/array\"\npub fn probe -> {result} = {expression}\n"
        ))
        .functions[0]
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
        let index = if binder == "Array" {
            arguments.len() - 1
        } else {
            0
        };
        arguments[index] = identity.body.clone();
        let origin = core_expr_origin(&call).clone();
        identity.body = call;
        let error = emit_rust_module(&core, "main")
            .expect_err("lexical binder is not a canonical ABI type");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
}

#[test]
fn basic_array_keeps_program_wasm_admission_explicit() {
    for (name, signature) in [
        ("filter", "(Int -> Bool) -> Array<Int> -> Array<Int>"),
        ("find", "(Int -> Bool) -> Array<Int> -> Maybe<Int>"),
        ("get", "Int -> Array<Int> -> Maybe<Int>"),
        ("length", "Array<Int> -> Int"),
        ("take", "Int -> Array<Int> -> Array<Int>"),
        ("drop", "Int -> Array<Int> -> Array<Int>"),
    ] {
        let source = format!("import * as arrays from \"std/array\"\npub fn probe value: Int -> Int = {{ let operation: {signature} = arrays.{name}; value }}\npub fn main -> Int = probe 1\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/array", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific"),
            "{name}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "array-雪🙂.ssrg");
    }
}
