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
