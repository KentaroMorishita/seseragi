use super::*;

const SUM: &str = "fn sum remaining: Int -> total: Int -> Int = if remaining == 0 then total else sum (remaining - 1) (total + remaining)\n";

fn function(source: &str) -> CoreFunction {
    let typed = seseragi_semantics::type_module("scalar/main.ssrg", source);
    crate::lower_typed_module(typed).functions.remove(0)
}

#[test]
fn certifies_checked_int_arithmetic_and_boolean_state() {
    let sum = scalar_self_tail_loop(&function(SUM)).unwrap();
    assert_eq!(sum.parameters, vec![CoreScalarKind::Int; 2]);
    assert_eq!(sum.result, CoreScalarKind::Int);
    assert_eq!(sum.self_tail_calls, 1);
    let boolean = function(
        "fn loop n: Int -> answer: Bool -> Bool = if n == 0 then answer else loop (n - 1) answer\n",
    );
    assert_eq!(
        scalar_self_tail_loop(&boolean).unwrap().result,
        CoreScalarKind::Bool
    );
}

#[test]
fn rejects_scope_capture_generic_non_tail_and_returned_function_shapes() {
    for source in [
        "fn add n: Int -> Int = n + 1\n",
        "fn sum n: Int -> Int = if n == 0 then 0 else sum (n - 1) + 1\n",
        "fn sum n: Int -> total: Int -> Int = { let current = n; if current == 0 then total else sum (current - 1) (total + current) }\n",
        "fn sum<A> n: Int -> total: A -> A = if n == 0 then total else sum (n - 1) total\n",
        "fn make n: Int -> (Int -> Int) = \\value -> value + n\n",
    ] {
        assert!(scalar_self_tail_loop(&function(source)).is_none(), "{source}");
    }
    let mut effect = function(SUM);
    effect.is_effect = true;
    assert!(scalar_self_tail_loop(&effect).is_none());
}

#[test]
fn rejects_nominal_primitive_spelling_in_parameters_and_operands() {
    let nominal = CoreType::ExternalNamed {
        name: "Int".into(),
        canonical: "user::Int".into(),
        arguments: vec![],
    };
    let mut parameter = function(SUM);
    parameter.parameters[0].type_ref = nominal.clone();
    assert!(scalar_self_tail_loop(&parameter).is_none());
    let mut operand = function(SUM);
    let CoreExpr::If { condition, .. } = &mut operand.body else {
        panic!()
    };
    let CoreExpr::Binary { left, .. } = &mut **condition else {
        panic!()
    };
    let CoreExpr::Variable { type_ref, .. } = &mut **left else {
        panic!()
    };
    *type_ref = nominal;
    assert!(scalar_self_tail_loop(&operand).is_none());
}

#[test]
fn requires_the_selected_standard_dictionary_and_constraint_identity() {
    for mutation in 0..4 {
        let mut input = function(SUM);
        let CoreExpr::If { condition, .. } = &mut input.body else {
            panic!()
        };
        let CoreExpr::Binary { evidence, .. } = &mut **condition else {
            panic!()
        };
        match mutation {
            0 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            1 => evidence[0].constraint.trait_identity = Some("user::Eq".into()),
            2 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "user::Int".into(),
                    arguments: vec![],
                }
            }
            _ => evidence.clear(),
        }
        assert!(scalar_self_tail_loop(&input).is_none());
    }
}

#[test]
fn requires_exact_source_arity_and_direct_unstaged_self_calls() {
    for mutation in 0..5 {
        let mut input = function(SUM);
        let CoreExpr::If { else_branch, .. } = &mut input.body else {
            panic!()
        };
        let CoreExpr::Call {
            callee,
            arguments,
            evidence_argument_index,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            ..
        } = &mut **else_branch
        else {
            panic!()
        };
        match mutation {
            0 => {
                arguments.pop();
            }
            1 => *callee = "alias".into(),
            2 => *evidence_argument_index = Some(0),
            3 => deferred_evidence_parameters.push(CoreType::Hole),
            _ => deferred_evidence_type_constructor_parameters.push("F".into()),
        }
        assert!(scalar_self_tail_loop(&input).is_none());
    }
}

fn global_module(path: &str) -> CoreModule {
    crate::lower_typed_module(seseragi_semantics::type_module(
        path,
        &format!(
            "let increment: Int = 3\n{}",
            SUM.replace("total + remaining", "total + increment")
        ),
    ))
}

fn global_operand(function: &mut CoreFunction) -> &mut CoreExpr {
    let CoreExpr::If { else_branch, .. } = &mut function.body else {
        panic!()
    };
    let CoreExpr::Call { arguments, .. } = &mut **else_branch else {
        panic!()
    };
    let CoreExpr::Binary { right, .. } = &mut arguments[1] else {
        panic!()
    };
    right
}

#[test]
fn certifies_literal_globals_without_rewriting_the_read_or_initializer() {
    let module = global_module("scalar/main.ssrg");
    let before = module.clone();
    let facts = CoreScalarGlobals::from_modules([&module]);
    let function = &module.functions[0];
    assert!(scalar_self_tail_loop(function).is_none());
    let plan = scalar_self_tail_loop_with_globals(function, &module.module, &facts).unwrap();
    assert_eq!(plan.global_reads.len(), 1);
    let read = plan.global_reads.values().next().unwrap();
    assert_eq!(read.symbol, module.bindings[0].symbol);
    assert_eq!(read.kind, CoreScalarKind::Int);
    assert_eq!(module, before);
}

#[test]
fn canonical_global_owner_and_lexical_parameter_take_precedence_over_spelling() {
    let module = global_module("scalar/main.ssrg");
    let other = global_module("other/lib.ssrg");
    let facts = CoreScalarGlobals::from_modules([&module, &other]);
    let mut f = module.functions[0].clone();
    let CoreExpr::Variable { name, .. } = global_operand(&mut f) else {
        panic!()
    };
    *name = other.bindings[0].symbol.clone();
    let proof = scalar_self_tail_loop_with_globals(&f, &module.module, &facts).unwrap();
    assert_eq!(
        proof.global_reads.values().next().unwrap().symbol,
        other.bindings[0].symbol
    );
    assert!(scalar_self_tail_loop_with_globals(&f, &other.module, &facts).is_none());
    assert!(scalar_self_tail_loop_with_globals(
        &f,
        &module.module,
        &CoreScalarGlobals::from_modules([&module])
    )
    .is_none());

    let shadow = crate::lower_typed_module(seseragi_semantics::type_module("scalar/shadow.ssrg", "let increment: Int = 3\nfn sum remaining: Int -> increment: Int -> Int = if remaining == 0 then increment else sum (remaining - 1) (increment + remaining)\n"));
    let plan = scalar_self_tail_loop_with_globals(
        &shadow.functions[0],
        &shadow.module,
        &CoreScalarGlobals::from_modules([&shadow]),
    )
    .unwrap();
    assert!(plan.global_reads.is_empty());
}

#[test]
fn unproved_duplicate_polymorphic_and_invalid_global_declarations_do_not_certify() {
    let module = global_module("scalar/main.ssrg");
    assert!(scalar_self_tail_loop_with_globals(
        &module.functions[0],
        &module.module,
        &CoreScalarGlobals::from_modules([&module, &module])
    )
    .is_none());
    for mutation in 0..4 {
        let mut changed = module.clone();
        match mutation {
            0 => changed.bindings[0].symbol = "wrong/owner::increment".into(),
            1 => {
                changed.bindings[0].type_parameters =
                    function("fn id<A> value: A -> A = value\n").type_parameters
            }
            2 => {
                let CoreExpr::Integer { value, .. } = &mut changed.bindings[0].value else {
                    panic!()
                };
                *value = "9007199254740992".into();
            }
            _ => changed.bindings[0].value = changed.functions[0].body.clone(),
        }
        assert!(
            scalar_self_tail_loop_with_globals(
                &changed.functions[0],
                &changed.module,
                &CoreScalarGlobals::from_modules([&changed])
            )
            .is_none(),
            "mutation={mutation}"
        );
    }
}

#[test]
fn global_facts_do_not_bypass_nominal_identity_evidence_or_source_arity() {
    let module = global_module("scalar/main.ssrg");
    let facts = CoreScalarGlobals::from_modules([&module]);
    for mutation in 0..4 {
        let mut f = module.functions[0].clone();
        if mutation == 0 {
            let CoreExpr::Variable { type_ref, .. } = global_operand(&mut f) else {
                panic!()
            };
            *type_ref = CoreType::ExternalNamed {
                name: "Int".into(),
                canonical: "user::Int".into(),
                arguments: vec![],
            };
        } else if mutation == 1 {
            let CoreExpr::If { condition, .. } = &f.body else {
                panic!()
            };
            let CoreExpr::Binary { evidence, .. } = &**condition else {
                panic!()
            };
            let selected = evidence.clone();
            let CoreExpr::Variable { evidence, .. } = global_operand(&mut f) else {
                panic!()
            };
            *evidence = selected;
        } else {
            let CoreExpr::If { else_branch, .. } = &mut f.body else {
                panic!()
            };
            let CoreExpr::Call {
                arguments,
                deferred_evidence_parameters,
                ..
            } = &mut **else_branch
            else {
                panic!()
            };
            if mutation == 2 {
                arguments.pop();
            } else {
                deferred_evidence_parameters.push(CoreType::Hole);
            }
        }
        assert!(
            scalar_self_tail_loop_with_globals(&f, &module.module, &facts).is_none(),
            "mutation={mutation}"
        );
    }
}
