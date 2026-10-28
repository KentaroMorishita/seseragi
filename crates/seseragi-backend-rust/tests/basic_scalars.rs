use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("basic-雫.ssrg", "native/basic", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
        .core_ir
}

#[test]
fn canonical_bool_and_unit_equality_and_not_keep_original_programs() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/primitive-equality-literals.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-scalar-effects.ssrg"),
        "fn same<A> left: A -> right: A -> Bool where Eq<A> = eq left right\npub effect fn main = do { println (show (same True False)); println (show (same () ())) }\n",
    ] {
        emit_rust_module(&checked(source), "main").expect("checked canonical primitive operations");
    }
    let core = checked("pub fn probe -> Bool = eq () ()\n");
    assert_eq!(
        core.functions[0].parameters.len(),
        1,
        "entry retains its anonymous Unit source parameter"
    );
    emit_rust_module(&core, "probe").unwrap();
}

#[test]
fn bool_and_unit_standard_evidence_never_substitutes_another_owner_or_kind() {
    for expression in ["True == False", "() == ()"] {
        let core = checked(&format!("// 雫🙂\npub fn probe -> Bool = {expression}\n"));
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..10 {
            let mut changed = core.clone();
            let origin = changed.functions[0].origin.clone();
            let CoreExpr::Binary {
                evidence,
                type_ref,
                right,
                ..
            } = &mut changed.functions[0].body
            else {
                panic!()
            };
            match mutation {
                0 => evidence[0].constraint.trait_identity = Some("fake::Eq".into()),
                1 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                2 => {
                    evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                        name: if expression.starts_with('T') {
                            "Bool"
                        } else {
                            "Unit"
                        }
                        .into(),
                        canonical: "fake::Scalar".into(),
                        arguments: vec![],
                    }
                }
                3 => {
                    let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = if expression.starts_with('T') {
                        "std/unit::Eq"
                    } else {
                        "std/bool::Eq"
                    }
                    .into();
                }
                4 => {
                    let CoreInstanceEvidence::Standard { type_arguments, .. } =
                        &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    type_arguments.push(CoreType::Hole);
                }
                5 => {
                    let child = evidence[0].clone();
                    let CoreInstanceEvidence::Standard {
                        evidence_arguments, ..
                    } = &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    evidence_arguments.push(child);
                }
                6 => evidence.push(evidence[0].clone()),
                7 => *type_ref = CoreType::Hole,
                8 => {
                    *right = Box::new(CoreExpr::Integer {
                        value: "0".into(),
                        origin,
                    })
                }
                _ => {
                    evidence[0].constraint.arguments[0] = CoreType::Named {
                        name: "std/prelude::Bool".into(),
                        arguments: vec![CoreType::Named {
                            name: "std/prelude::Unit".into(),
                            arguments: vec![],
                        }],
                    }
                }
            }
            let error = emit_rust_module(&changed, "probe")
                .expect_err("counterfeit primitive evidence is rejected before generation");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap().source, "basic-雫.ssrg");
        }
    }
}

#[test]
fn prefix_not_rejects_wrong_input_result_owner_and_kind_before_emission() {
    let core = checked("pub fn probe -> Bool = !False\n");
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..6 {
        let mut changed = core.clone();
        let origin = changed.functions[0].origin.clone();
        let CoreExpr::Unary {
            operand, type_ref, ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *operand = Box::new(CoreExpr::Unit { origin }),
            1 => *type_ref = CoreType::Hole,
            2 => {
                *type_ref = CoreType::ExternalNamed {
                    name: "Bool".into(),
                    canonical: "fake::Bool".into(),
                    arguments: vec![],
                }
            }
            3 => {
                *type_ref = CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![CoreType::Named {
                        name: "std/prelude::Unit".into(),
                        arguments: vec![],
                    }],
                }
            }
            4 => {
                *operand = Box::new(CoreExpr::Integer {
                    value: "1".into(),
                    origin,
                })
            }
            _ => {
                *type_ref = CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").expect_err("! is canonical Bool to Bool");
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap().source, "basic-雫.ssrg");
    }
}

#[test]
fn user_owned_scalar_names_and_float_do_not_acquire_primitive_operations() {
    for source in [
        "struct Bool {value: Int}\npub fn probe -> Bool = !(Bool {value: 1})\n",
        "struct Unit {value: Int}\npub fn probe -> Bool = eq () (Unit {value: 1})\n",
        "pub fn probe -> Bool = eq 1.0 1.0\n",
    ] {
        compile_core_module(CompileInput::new("invalid.ssrg", "native/basic", source))
            .expect_err("source checking cannot invent a standard primitive instance");
    }
}

#[test]
fn unit_equality_keeps_both_checked_source_applications() {
    let core = checked("pub fn probe -> Bool = eq () ()\n");
    emit_rust_module(&core, "probe").unwrap();
    for count in [0, 1, 3] {
        let mut changed = core.clone();
        let CoreExpr::Call { arguments, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let unit = arguments[0].clone();
        *arguments = vec![unit; count];
        let error = emit_rust_module(&changed, "probe")
            .expect_err("the checked Bool result requires two Unit source arguments");
        assert_eq!(error.code, "SES-RUST-0001");
    }
}

#[test]
fn scoped_scalar_binders_never_become_standard_runtime_values() {
    let mut not =
        checked("fn identity<Bool> value: Bool -> Bool = value\npub fn probe -> Bool = False\n");
    let identity = not
        .functions
        .iter_mut()
        .find(|function| function.symbol.ends_with("::identity"))
        .unwrap();
    identity.body = CoreExpr::Unary {
        operator: "!".into(),
        operand: Box::new(identity.body.clone()),
        type_ref: CoreType::Named {
            name: "std/prelude::Bool".into(),
            arguments: vec![],
        },
        origin: identity.origin.clone(),
    };
    emit_rust_module(&not, "probe")
        .expect_err("a scoped binder named Bool is not an intrinsic Bool");

    let mut equal = checked("fn same<Unit> left: Unit -> right: Unit -> Bool where Eq<Unit> = eq left right\npub fn probe -> Bool = True\n");
    let same = equal
        .functions
        .iter_mut()
        .find(|function| function.symbol.ends_with("::same"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &mut same.body else {
        panic!()
    };
    evidence[0].evidence = CoreInstanceEvidence::Standard {
        identity: "std/unit::Eq".into(),
        type_arguments: vec![],
        evidence_arguments: vec![],
    };
    emit_rust_module(&equal, "probe")
        .expect_err("a scoped Eq parameter cannot be replaced with standard Unit Eq");
}
