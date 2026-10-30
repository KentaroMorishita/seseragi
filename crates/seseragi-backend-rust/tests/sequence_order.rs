use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "sequence-order-雪🙂.ssrg",
        "native/sequence-order",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}
fn cases() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("sort", "sort [3,1,2]", "Array<Int>"),
        ("sortBy", "sortBy key [3,1,2]", "Array<Int>"),
        ("groupBy", "groupBy key [3,1,2]", "maps.Map<Int,Array<Int>>"),
    ]
}
fn source(owner: &str, expression: &str, result: &str) -> String {
    let expression = if owner == "list" {
        expression.replace('[', "`[")
    } else {
        expression.into()
    };
    let result = if owner == "list" {
        result.replace("Array", "List")
    } else {
        result.into()
    };
    format!("// 雪🙂\nimport * as sequence from \"std/{owner}\"\nimport * as maps from \"std/map\"\nfn key n:Int -> Int = n % 2\npub fn probe -> {result} = sequence.{expression}\n")
}

#[test]
fn sequence_order_accepts_original_custom_primitive_generic_saved_and_cold_inputs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-sequence-order.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-sequence-order-effects.ssrg"
        ),
    ] {
        let artifact = emit_rust_module(&checked(source), "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == "sequence-order"));
    }
}

#[test]
fn sequence_order_rejects_forged_types_dictionary_owner_key_children_and_source_stages_at_origin() {
    for owner in ["array", "list"] {
        for (name, expression, result) in cases() {
            let core = checked(&source(owner, expression, result));
            let index = core
                .functions
                .iter()
                .position(|f| f.symbol.ends_with("::probe"))
                .unwrap();
            emit_rust_module(&core, "probe").unwrap();
            let origin = core_expr_origin(&core.functions[index].body).clone();
            let CoreExpr::Call { arguments, .. } = &core.functions[index].body else {
                panic!()
            };
            for mutation in 0..16 + arguments.len() {
                let mut bad = core.clone();
                let CoreExpr::Call {
                    arguments,
                    type_ref,
                    evidence,
                    trait_dispatch,
                    deferred_evidence_parameters,
                    deferred_evidence_type_constructor_parameters,
                    evidence_argument_index,
                    ..
                } = &mut bad.functions[index].body
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
                    5 => evidence.clear(),
                    6 => deferred_evidence_parameters.push(CoreType::Hole),
                    7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                    8 => *evidence_argument_index = Some(0),
                    9 => {
                        *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                            trait_identity: "std/prelude::Ord".into(),
                            method: "compare".into(),
                            method_contract: None,
                        })
                    }
                    10 => {
                        evidence[0].constraint.arguments[0] = CoreType::Named {
                            name: "std/prelude::Bool".into(),
                            arguments: vec![],
                        }
                    }
                    11 => {
                        let CoreInstanceEvidence::Standard { identity, .. } =
                            &mut evidence[0].evidence
                        else {
                            panic!()
                        };
                        *identity = if name == "groupBy" {
                            "std/int::Ord"
                        } else {
                            "std/int::Eq"
                        }
                        .into();
                    }
                    12 => evidence[0].constraint.trait_identity = Some("fake::Ord".into()),
                    13 => {
                        let CoreInstanceEvidence::Standard { type_arguments, .. } =
                            &mut evidence[0].evidence
                        else {
                            panic!()
                        };
                        type_arguments.push(CoreType::Hole);
                    }
                    14 => {
                        evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                            name: "Int".into(),
                            canonical: "fake::Int".into(),
                            arguments: vec![],
                        }
                    }
                    15 => evidence.push(evidence[0].clone()),
                    index => {
                        let origin = core_expr_origin(&arguments[index - 16]).clone();
                        arguments[index - 16] = CoreExpr::Boolean {
                            value: true,
                            origin,
                        };
                    }
                }
                let error = emit_rust_module(&bad, "probe")
                    .err()
                    .unwrap_or_else(|| panic!("accepted {owner}/{name}/{mutation}"));
                assert_eq!(error.code, "SES-RUST-0001", "{owner}/{name}/{mutation}");
                assert_eq!(error.origin.unwrap(), origin, "{owner}/{name}/{mutation}");
            }
        }
    }
}

#[test]
fn sequence_order_keeps_saved_references_and_rejects_counterfeit_function_suffixes() {
    for owner in ["array", "list"] {
        let collection = if owner == "array" { "Array" } else { "List" };
        for (name, _, _) in cases() {
            let signature = match name {
                "sort" => format!("{collection}<Int> -> {collection}<Int>"),
                "sortBy" => format!("(Int -> Int) -> {collection}<Int> -> {collection}<Int>"),
                _ => {
                    format!("(Int -> Int) -> {collection}<Int> -> maps.Map<Int,{collection}<Int>>")
                }
            };
            let core=checked(&format!("import * as sequence from \"std/{owner}\"\nimport * as maps from \"std/map\"\nfn operation -> ({signature}) = sequence.{name}\npub fn main -> Int = 0\n"));
            emit_rust_module(&core, "main").unwrap();
            let index = core
                .functions
                .iter()
                .position(|f| f.symbol.ends_with("::operation"))
                .unwrap();
            let origin = core_expr_origin(&core.functions[index].body).clone();
            for mutation in 0..3 {
                let mut bad = core.clone();
                let ty = match &mut bad.functions[index].body {
                    CoreExpr::Call { type_ref, .. } | CoreExpr::Variable { type_ref, .. } => {
                        type_ref
                    }
                    CoreExpr::Lambda { body, .. } => {
                        fn selected(expr: &mut CoreExpr) -> &mut CoreExpr {
                            match expr {
                                CoreExpr::Lambda { body, .. } => selected(body),
                                _ => expr,
                            }
                        }
                        let CoreExpr::Call {
                            arguments,
                            type_ref,
                            evidence,
                            ..
                        } = selected(body)
                        else {
                            panic!()
                        };
                        if mutation == 0 {
                            *type_ref = CoreType::Hole;
                        } else if mutation == 1 {
                            let origin = core_expr_origin(&arguments[0]).clone();
                            arguments[0] = CoreExpr::Boolean {
                                value: true,
                                origin,
                            };
                        } else {
                            evidence[0].constraint.arguments[0] = CoreType::Hole;
                        }
                        let error = emit_rust_module(&bad, "main").unwrap_err();
                        assert_eq!(error.code, "SES-RUST-0001");
                        assert_eq!(error.origin.unwrap(), origin);
                        continue;
                    }
                    other => panic!("{other:?}"),
                };
                if mutation == 0 {
                    *ty = CoreType::Hole;
                } else {
                    let CoreType::Function { parameter, result } = ty else {
                        panic!()
                    };
                    if mutation == 1 {
                        **parameter = CoreType::Hole;
                    } else {
                        **result = CoreType::Hole;
                    }
                }
                let error = emit_rust_module(&bad, "main").unwrap_err();
                assert_eq!(error.code, "SES-RUST-0001");
                assert_eq!(error.origin.unwrap(), origin);
            }
        }
    }
}

#[test]
fn sequence_order_does_not_admit_unused_program_wasm_operations() {
    for owner in ["array", "list"] {
        let collection = if owner == "array" { "Array" } else { "List" };
        for (name, _, _) in cases() {
            let signature = match name {
                "sort" => format!("{collection}<Int> -> {collection}<Int>"),
                "sortBy" => format!("(Int -> Int) -> {collection}<Int> -> {collection}<Int>"),
                _ => {
                    format!("(Int -> Int) -> {collection}<Int> -> maps.Map<Int,{collection}<Int>>")
                }
            };
            let core=checked(&format!("import * as sequence from \"std/{owner}\"\nimport * as maps from \"std/map\"\npub fn probe value:Int -> Int = {{let saved:{signature} = sequence.{name};value}}\npub fn main -> Int = 0\n"));
            emit_rust_module(&core, "main").unwrap();
            let error = emit_program_wasm(&[&core], "native/sequence-order", &["probe".into()])
                .unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap().source, "sequence-order-雪🙂.ssrg");
        }
    }
}
