use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "array-rest-雪🙂.ssrg",
        "native/array-rest",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn array_rest_accepts_original_saved_generic_nested_and_cold_inputs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-rest.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-rest-effects.ssrg"),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|value| value == "array-operations"));
    }
}
#[test]
fn array_rest_rejects_forged_owner_element_kind_evidence_and_staging_at_the_original_span() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (expression, result) in [
        ("empty ()", "Array<Int>"),
        ("singleton 1", "Array<Int>"),
        ("append [3,4] [1,2]", "Array<Int>"),
        ("reverse [1,2]", "Array<Int>"),
        ("isEmpty [1,2]", "Bool"),
        ("head [1,2]", "Maybe<Int>"),
        ("tail [1,2]", "Maybe<Array<Int>>"),
        ("filterMap choose [1,2]", "Array<String>"),
        ("flatMap expand [1,2]", "Array<String>"),
    ] {
        let source = format!("// 雪🙂\nimport * as arrays from \"std/array\"\nfn choose value:Int -> Maybe<String> = Just (show value)\nfn expand value:Int -> Array<String> = [show value]\npub fn probe -> {result} = arrays.{expression}\n");
        let core = checked(&source);
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        emit_rust_module(&core, "probe").unwrap();
        let CoreExpr::Call { arguments, .. } = &core.functions[probe].body else {
            panic!()
        };
        for mutation in 0..10 + arguments.len() {
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
                index => {
                    let origin = core_expr_origin(&arguments[index - 10]).clone();
                    arguments[index - 10] = CoreExpr::Boolean {
                        value: true,
                        origin,
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
fn array_rest_saved_references_reject_counterfeit_function_stages() {
    for (name, signature) in [
        ("empty", "Unit -> Array<Int>"),
        ("singleton", "Int -> Array<Int>"),
        ("append", "Array<Int> -> Array<Int> -> Array<Int>"),
        ("reverse", "Array<Int> -> Array<Int>"),
        ("isEmpty", "Array<Int> -> Bool"),
        ("head", "Array<Int> -> Maybe<Int>"),
        ("tail", "Array<Int> -> Maybe<Array<Int>>"),
        (
            "filterMap",
            "(Int -> Maybe<String>) -> Array<Int> -> Array<String>",
        ),
        (
            "flatMap",
            "(Int -> Array<String>) -> Array<Int> -> Array<String>",
        ),
    ] {
        let core=checked(&format!("import * as arrays from \"std/array\"\nfn operation -> ({signature}) = arrays.{name}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::operation"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..3 {
            let mut bad = core.clone();
            let type_ref = match &mut bad.functions[index].body {
                CoreExpr::Call { type_ref, .. } | CoreExpr::Variable { type_ref, .. } => type_ref,
                other => panic!("{name}: expected saved callable, got {other:?}"),
            };
            if mutation == 0 {
                *type_ref = CoreType::Hole;
            } else {
                let CoreType::Function { parameter, result } = type_ref else {
                    panic!()
                };
                if mutation == 1 {
                    **parameter = CoreType::Hole;
                } else {
                    **result = CoreType::Hole;
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin);
        }
    }
}
#[test]
fn array_rest_does_not_expand_unused_program_wasm_references() {
    for (name, signature) in [
        ("empty", "Unit -> Array<Int>"),
        ("singleton", "Int -> Array<Int>"),
        ("append", "Array<Int> -> Array<Int> -> Array<Int>"),
        ("reverse", "Array<Int> -> Array<Int>"),
        ("isEmpty", "Array<Int> -> Bool"),
        ("head", "Array<Int> -> Maybe<Int>"),
        ("tail", "Array<Int> -> Maybe<Array<Int>>"),
        (
            "filterMap",
            "(Int -> Maybe<String>) -> Array<Int> -> Array<String>",
        ),
        (
            "flatMap",
            "(Int -> Array<String>) -> Array<Int> -> Array<String>",
        ),
    ] {
        let core=checked(&format!("import * as arrays from \"std/array\"\npub fn probe value:Int -> Int = {{let saved:{signature} = arrays.{name};value}}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/array-rest", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "array-rest-雪🙂.ssrg");
    }
}
