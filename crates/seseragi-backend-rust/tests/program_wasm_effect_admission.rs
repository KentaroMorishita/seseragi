use seseragi_backend_rust::emit_program_wasm;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreFunction, CoreModule, CoreType};

const FILE: &str = "effect-admission-雪.ssrg";
const MODULE: &str = "fixture/effect-admission";
const IMPORT: &str = "import * as effects from \"std/effect\"\n";
const VALUE: &str = "pub effect fn value input: String -> String = effects.succeed input\n";

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new(FILE, MODULE, source))
        .unwrap_or_else(|diagnostics| {
            panic!("legal control failed frontend checking:\n{source}\n{diagnostics:?}")
        })
        .core_ir
}
fn program(hidden: &str) -> CoreModule {
    checked(&format!("{IMPORT}{hidden}\n{VALUE}"))
}
fn accepts(module: &CoreModule) {
    let artifact = emit_program_wasm(&[module], MODULE, &["value".into()])
        .unwrap_or_else(|error| panic!("legal control failed admission: {error}"));
    assert_eq!(artifact.metadata.abi, "seseragi-program-wasm-effect-v1");
    assert_eq!(artifact.metadata.exports[0].source_arity, 1);
}
fn refuses(module: &CoreModule) {
    // Err is the API's no-artifact result. Both native evidence validation and
    // target admission run before any program source/loader is returned.
    let error = emit_program_wasm(&[module], MODULE, &["value".into()])
        .expect_err("unsafe/unproven Core returned a program artifact");
    assert_eq!(error.code, "SES-RUST-0001");
    let origin = error.origin.expect("rejection must retain its source span");
    assert_eq!(origin.source, FILE);
    assert!(origin.end > origin.start, "{origin:?}");
}
fn named(name: &str) -> CoreType {
    CoreType::Named {
        name: format!("std/prelude::{name}"),
        arguments: Vec::new(),
    }
}
fn requirements() -> CoreType {
    CoreType::Record {
        closed: true,
        fields: Vec::new(),
    }
}
fn effect(failure: CoreType, success: CoreType) -> CoreType {
    CoreType::Named {
        name: "std/prelude::Effect".into(),
        arguments: vec![requirements(), failure, success],
    }
}
fn function<'a>(module: &'a mut CoreModule, name: &str) -> &'a mut CoreFunction {
    let symbol = format!("{MODULE}::{name}");
    module
        .functions
        .iter_mut()
        .find(|function| function.symbol == symbol)
        .unwrap()
}
fn result_type(function: &mut CoreFunction) -> &mut CoreType {
    let mut ty = function
        .value_type
        .as_mut()
        .expect("checked declaration type");
    for _ in &function.parameters {
        let CoreType::Function { result, .. } = ty else {
            panic!("missing source stage")
        };
        ty = result;
    }
    ty
}
fn arguments(expr: &mut CoreExpr) -> &mut Vec<CoreExpr> {
    match expr {
        CoreExpr::Call { arguments, .. }
        | CoreExpr::EffectInvoke { arguments, .. }
        | CoreExpr::EffectOperation { arguments, .. } => arguments,
        other => panic!("expected checked operation, got {other:?}"),
    }
}
fn channels(expr: &mut CoreExpr) -> (&mut CoreType, &mut CoreType, &mut CoreType) {
    match expr {
        CoreExpr::EffectInvoke {
            requirements,
            failure,
            success,
            ..
        }
        | CoreExpr::EffectOperation {
            requirements,
            failure,
            success,
            ..
        } => (requirements, failure, success),
        CoreExpr::Call { type_ref, .. } => {
            let (CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. }) =
                type_ref
            else {
                panic!("expected Effect result")
            };
            let [requirements, failure, success] = arguments.as_mut_slice() else {
                panic!("expected three Effect channels")
            };
            (requirements, failure, success)
        }
        other => panic!("expected checked Effect, got {other:?}"),
    }
}
fn lambda_result(expr: &mut CoreExpr) -> &mut CoreType {
    let CoreExpr::Lambda {
        type_ref: CoreType::Function { result, .. },
        ..
    } = expr
    else {
        panic!("expected checked lambda")
    };
    result
}

#[test]
fn hidden_integer_literals_stay_in_the_language_range() {
    let original = program("fn hidden input: String -> Int = 42");
    accepts(&original);
    for value in [
        "9007199254740992",
        "-9007199254740992",
        "9223372036854775807",
    ] {
        let mut module = original.clone();
        let CoreExpr::Integer { value: literal, .. } = &mut function(&mut module, "hidden").body
        else {
            panic!("expected checked integer")
        };
        *literal = value.into();
        refuses(&module);
    }
}

#[test]
fn hidden_if_cannot_reach_boolean_extraction_with_a_scalar_of_another_type() {
    let original =
        program("fn hidden enabled: Bool -> String = if enabled then \"yes\" else \"no\"");
    accepts(&original);
    for integer in [false, true] {
        let mut module = original.clone();
        let CoreExpr::If {
            condition, origin, ..
        } = &mut function(&mut module, "hidden").body
        else {
            panic!("expected checked conditional")
        };
        **condition = if integer {
            CoreExpr::Integer {
                value: "1".into(),
                origin: origin.clone(),
            }
        } else {
            CoreExpr::String {
                value: "not Bool".into(),
                origin: origin.clone(),
            }
        };
        refuses(&module);
    }
}

#[test]
fn variable_identity_is_proved_against_its_lexical_parameter() {
    let original = program("fn hidden input: Bool -> Bool = input");
    accepts(&original);
    let mut module = original;
    let hidden = function(&mut module, "hidden");
    hidden.parameters[0].type_ref = named("String");
    let CoreType::Function { parameter, .. } = hidden.value_type.as_mut().unwrap() else {
        panic!("source signature")
    };
    **parameter = named("String");
    // Body and declaration still claim Bool. Only the lexical binding reveals
    // that this V would actually contain a String when the function is called.
    refuses(&module);
}

#[test]
fn returned_lambda_proves_both_its_input_and_its_actual_result() {
    let original = program(
        "fn hidden prefix: String -> (String -> String) = \\suffix: String -> prefix + suffix",
    );
    accepts(&original);
    for mutation in 0..2 {
        let mut module = original.clone();
        let CoreExpr::Lambda {
            parameter,
            body,
            origin,
            ..
        } = &mut function(&mut module, "hidden").body
        else {
            panic!("checked returned lambda")
        };
        if mutation == 0 {
            parameter.type_ref = named("Bool");
        } else {
            **body = CoreExpr::Unit {
                origin: origin.clone(),
            };
        }
        refuses(&module);
    }
}

#[test]
fn tuple_codec_annotations_cannot_hide_wrong_payloads_or_arity() {
    let original = program("fn hidden input: String -> (String, Bool) = (input, True)");
    accepts(&original);
    for mutation in 0..2 {
        let mut module = original.clone();
        let CoreExpr::Tuple {
            elements, origin, ..
        } = &mut function(&mut module, "hidden").body
        else {
            panic!("checked tuple")
        };
        if mutation == 0 {
            elements[1] = CoreExpr::String {
                value: "not Bool".into(),
                origin: origin.clone(),
            };
        } else {
            elements.push(CoreExpr::Unit {
                origin: origin.clone(),
            });
        }
        refuses(&module);
    }
}

#[test]
fn succeed_cannot_claim_a_success_type_different_from_its_argument() {
    let original = program("");
    accepts(&original);
    let mut module = original;
    let selected = function(&mut module, "value");
    *channels(&mut selected.body).2 = named("Bool");
    *result_type(selected) = effect(named("Never"), named("Bool"));
    refuses(&module);
}

#[test]
fn fail_cannot_claim_a_failure_type_different_from_its_argument() {
    let original = checked(&format!(
        "{IMPORT}pub effect fn value input: String -> String fails String = effects.fail input\n"
    ));
    accepts(&original);
    let mut module = original;
    let selected = function(&mut module, "value");
    *channels(&mut selected.body).1 = named("Bool");
    *result_type(selected) = effect(named("Bool"), named("String"));
    refuses(&module);
}

fn acquiring() -> CoreModule {
    checked(&format!("{IMPORT}let unit = ()\npub effect fn value input: String -> String = effects.acquireRelease (effects.succeed input) (\\_resource: String -> effects.succeed unit)\n"))
}
#[test]
fn registered_release_cannot_introduce_a_typed_failure() {
    let original = acquiring();
    accepts(&original);
    let mut module = original;
    let release = &mut arguments(&mut function(&mut module, "value").body)[1];
    *lambda_result(release) = effect(named("String"), named("Unit"));
    let CoreExpr::Lambda { body, origin, .. } = release else {
        panic!("release lambda")
    };
    **body = CoreExpr::EffectOperation {
        operation: "effect.fail".into(),
        requirements: requirements(),
        failure: named("String"),
        success: named("Never"),
        arguments: vec![CoreExpr::String {
            value: "release failed".into(),
            origin: origin.clone(),
        }],
        origin: origin.clone(),
    };
    refuses(&module);
}

#[test]
fn registered_release_cannot_replace_its_unit_success_with_a_string() {
    let original = acquiring();
    accepts(&original);
    let mut module = original;
    let release = &mut arguments(&mut function(&mut module, "value").body)[1];
    *lambda_result(release) = effect(named("Never"), named("String"));
    let CoreExpr::Lambda { body, origin, .. } = release else {
        panic!("release lambda")
    };
    arguments(body)[0] = CoreExpr::String {
        value: "not Unit".into(),
        origin: origin.clone(),
    };
    *channels(body).2 = named("String");
    refuses(&module);
}

fn deferred() -> CoreModule {
    checked(&format!("{IMPORT}pub effect fn value input: String -> String = effects.defer (\\_trigger: Unit -> effects.succeed input)\n"))
}
#[test]
fn defer_requires_a_unit_parameter_even_when_the_lambda_is_self_consistent() {
    let original = deferred();
    accepts(&original);
    let mut module = original;
    let thunk = &mut arguments(&mut function(&mut module, "value").body)[0];
    let CoreExpr::Lambda {
        parameter,
        type_ref: CoreType::Function {
            parameter: input, ..
        },
        ..
    } = thunk
    else {
        panic!("checked defer thunk")
    };
    parameter.type_ref = named("String");
    **input = named("String");
    refuses(&module);
}

#[test]
fn defer_requires_a_plan_even_when_the_lambda_body_matches_its_annotation() {
    let original = deferred();
    accepts(&original);
    let mut module = original;
    let thunk = &mut arguments(&mut function(&mut module, "value").body)[0];
    *lambda_result(thunk) = named("String");
    let CoreExpr::Lambda { body, origin, .. } = thunk else {
        panic!("checked defer thunk")
    };
    **body = CoreExpr::String {
        value: "not an Effect plan".into(),
        origin: origin.clone(),
    };
    refuses(&module);
}

#[test]
fn eager_global_cycles_are_rejected_while_direct_defer_cycles_are_admitted() {
    let original = program("fn wait -> Effect<{}, Never, Unit> = effects.defer (\\_ -> wait ())");
    accepts(&original);
    let mut module = original;
    let wait = function(&mut module, "wait");
    wait.body = CoreExpr::EffectInvoke {
        callee: wait.symbol.clone(),
        callee_signature: None,
        requirements: requirements(),
        failure: named("Never"),
        success: named("Unit"),
        arguments: vec![CoreExpr::Unit {
            origin: wait.origin.clone(),
        }],
        evidence: Vec::new(),
        origin: wait.origin.clone(),
    };
    refuses(&module);
}

#[test]
fn saved_standard_effects_need_another_proof_but_saved_source_functions_work() {
    let source = format!("{IMPORT}fn lift input: String -> Effect<{{}}, Never, String> = effects.succeed input\nfn saved input: String -> Effect<{{}}, Never, String> = {{ let create: String -> Effect<{{}}, Never, String> = lift; create input }}\n{VALUE}");
    accepts(&checked(&source));
    // Preserve the checked monomorphic context and change only the initializer
    // authority from an ordinary source declaration to the standard operation.
    let unsupported = source.replace("= lift;", "= effects.succeed;");
    refuses(&checked(&unsupported));
}

#[test]
fn partial_standard_effects_are_rejected_without_losing_source_partials() {
    let source = format!("{IMPORT}fn combine prefix: String -> suffix: String -> String = prefix + suffix\nfn saved prefix: String -> Effect<{{}}, Never, (String -> String)> = {{ let next = combine prefix; effects.succeed next }}\n{VALUE}");
    accepts(&checked(&source));
    let unsupported = format!("{IMPORT}fn saved acquire: Effect<{{}}, Never, String> -> ((String -> Effect<{{}}, Never, Unit>) -> Effect<{{}}, Never, String>) = effects.acquireRelease acquire\n{VALUE}");
    refuses(&checked(&unsupported));
}

#[test]
fn do_block_keeps_failures_from_effects_before_its_terminal_succeed() {
    let original = checked(&format!("{IMPORT}effect fn reject input: String -> String fails String = effects.fail input\npub effect fn value input: String -> String fails String = do {{ reject input; effects.succeed input }}\n"));
    accepts(&original);
    let mut module = original;
    // The tail is still succeed, whose own failure is Never. It cannot erase
    // the earlier executed Effect's String failure from the public contract.
    *result_type(function(&mut module, "value")) = effect(named("Never"), named("String"));
    refuses(&module);
}

#[test]
fn open_provider_rows_and_unknown_channel_types_never_produce_an_artifact() {
    let original = program("");
    accepts(&original);
    for mutation in 0..2 {
        let mut module = original.clone();
        let (requirements, failure, _) = channels(&mut function(&mut module, "value").body);
        if mutation == 0 {
            *requirements = CoreType::Record {
                closed: false,
                fields: Vec::new(),
            };
        } else {
            *failure = CoreType::Hole;
        }
        refuses(&module);
    }
}
