use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreStatement, CoreType};

const PARTIAL: &str = "fn mapped<F<_>> callback:(Int -> Int) -> values:F<Int> -> F<Int> where Functor<F> = map callback values\npub fn probe -> (Array<Int> -> Array<Int>) = mapped (\\x:Int -> x + 1)\n";
const LOCAL: &str = "pub fn probe -> (Array<Int> -> Array<Int>) = { fn mapped<F<_>> callback:(Int -> Int) -> values:F<Int> -> F<Int> where Functor<F> = map callback values; mapped (\\x:Int -> x + 1) }\n";
const ALIAS: &str = "fn mapped<F<_>> callback:(Int -> Int) -> values:F<Int> -> F<Int> where Functor<F> = map callback values\npub fn probe -> (Array<Int> -> Array<Int>) = { let saved = mapped; saved (\\x:Int -> x + 1) }\n";

#[test]
fn constrained_values_keep_all_first_class_positions() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/constrained-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/constrained-hkt-partials.ssrg"),
    ] {
        let core = compile_core_module(CompileInput::new(
            "constrained-雪.ssrg",
            "native/constrained",
            source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "main").unwrap();
    }
}

#[test]
fn constructor_partials_reject_unproved_types_evidence_and_stages() {
    for source in [PARTIAL, LOCAL, ALIAS] {
        let core = compile_core_module(CompileInput::new(
            "partial-雪.ssrg",
            "native/constrained",
            source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..14 {
            let mut bad = core.clone();
            let function = bad
                .functions
                .iter_mut()
                .find(|function| function.symbol.ends_with("::probe"))
                .unwrap();
            let (body, parameters) = if let CoreExpr::Sequence {
                statements, result, ..
            } = &mut function.body
            {
                let CoreStatement::LocalFunction {
                    type_parameters, ..
                } = &mut statements[0]
                else {
                    panic!("local declaration")
                };
                (result.as_mut(), type_parameters)
            } else {
                let declaration = core
                    .functions
                    .iter()
                    .position(|function| function.symbol.ends_with("::mapped"))
                    .unwrap();
                let (before, after) = bad.functions.split_at_mut(declaration + 1);
                (&mut after[0].body, &mut before[declaration].type_parameters)
            };
            let CoreExpr::Call {
                arguments,
                evidence,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                type_ref,
                ..
            } = body
            else {
                panic!("partial call")
            };
            match mutation {
                0 => deferred_evidence_type_constructor_parameters.clear(),
                1 => deferred_evidence_type_constructor_parameters.push("F".into()),
                2 => deferred_evidence_type_constructor_parameters[0] = "G".into(),
                3 => parameters[0].arity = 0,
                4 => parameters[0].arity = 2,
                5 => deferred_evidence_parameters.clear(),
                6 => deferred_evidence_parameters[0] = CoreType::Hole,
                7 => *type_ref = CoreType::Hole,
                8 => evidence.clear(),
                9 => evidence[0].constraint.trait_identity = Some("fake::Functor".into()),
                10 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                11 => *evidence_argument_index = Some(1),
                12 => {
                    let CoreExpr::Lambda { type_ref, .. } = &mut arguments[0] else {
                        panic!("callback")
                    };
                    *type_ref = CoreType::Hole;
                }
                13 => {
                    let CoreType::Function { result, .. } = type_ref else {
                        panic!("remaining source function")
                    };
                    **result = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    };
                }
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "probe")
                .err()
                .unwrap_or_else(|| panic!("accepted mutation {mutation}: {source}"));
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap().source, "partial-雪.ssrg");
        }
    }
}
