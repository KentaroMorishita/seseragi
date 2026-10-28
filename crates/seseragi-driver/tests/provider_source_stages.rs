use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_lowering::{
    CoreExpr, CoreStatement, CoreType, TypeScriptExpr, TypeScriptFunction, TypeScriptStatement,
};

const IMPORTS: &str = "import * as clock from \"std/clock\"\nimport {Clock} from \"std/clock\"\nimport * as time from \"std/time\"\n";

fn source_type_is_a_clock_callable(ty: &CoreType) {
    let CoreType::Function { parameter, result } = ty else {
        panic!("{ty:?}");
    };
    assert!(matches!(parameter.as_ref(), CoreType::ExternalNamed {
        canonical, arguments, ..
    } if canonical == "std/time::Duration" && arguments.is_empty()));
    let CoreType::Named { name, arguments } = result.as_ref() else {
        panic!("{result:?}");
    };
    // Checked Core keeps intrinsic Effect/Never/Unit in its intrinsic encoding;
    // the external parameter and service below retain their canonical owners.
    assert_eq!(name, "Effect");
    assert_eq!(arguments.len(), 3);
    let CoreType::Record { closed, fields } = &arguments[0] else {
        panic!("{arguments:?}");
    };
    assert!(*closed);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "clock");
    assert!(matches!(&fields[0].type_ref, CoreType::ExternalNamed {
        canonical, arguments, ..
    } if canonical == "std/clock::Clock" && arguments.is_empty()));
    assert!(matches!(&arguments[1], CoreType::Named { name, arguments }
        if name == "Never" && arguments.is_empty()));
    assert!(matches!(&arguments[2], CoreType::Named { name, arguments }
        if name == "Unit" && arguments.is_empty()));
}

fn source_reference(value: &CoreExpr) {
    let (callee, evidence, type_ref) = match value {
        CoreExpr::Call {
            callee,
            arguments,
            evidence,
            type_ref,
            ..
        } => {
            assert!(arguments.is_empty());
            (callee, evidence, type_ref)
        }
        CoreExpr::Variable {
            name,
            evidence,
            type_ref,
            ..
        } => (name, evidence, type_ref),
        _ => panic!("{value:?}"),
    };
    assert_eq!(callee, "std/clock::sleep");
    assert!(evidence.is_empty());
    source_type_is_a_clock_callable(type_ref);
}

fn cold_callback(value: &TypeScriptExpr) {
    let TypeScriptExpr::Lambda { parameter, body } = value else {
        panic!("constructing a saved function must not call its provider: {value:?}");
    };
    let TypeScriptExpr::RuntimeCall { callee, arguments } = body.as_ref() else {
        panic!("{body:?}");
    };
    assert_eq!(callee, "_ssrg_clock_sleep");
    assert_eq!(arguments.len(), 1);
    let TypeScriptExpr::Identifier { name } = &arguments[0] else {
        panic!("{arguments:?}");
    };
    assert_eq!(parameter.split_once(':').unwrap().0, name);
}

#[test]
fn saved_clock_operation_keeps_its_checked_source_argument_until_application() {
    let source = format!(
        "{IMPORTS}pub effect fn wait pause: time.Duration -> Unit with Clock = do {{ let saved = clock.sleep; saved pause }}\n"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("provider-stage.ssrg", "fixture/provider-stage", &source)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        let CoreExpr::Sequence { statements, .. } = &compiled.core_ir.functions[0].body else {
            panic!("{:?}", compiled.core_ir);
        };
        let CoreStatement::PureLet {
            type_ref, value, ..
        } = &statements[0]
        else {
            panic!("{statements:?}");
        };
        source_type_is_a_clock_callable(type_ref);
        source_reference(value);
        let TypeScriptFunction::ConstFunction { body, .. } = &compiled.typescript_ir.functions[0];
        let TypeScriptExpr::Sequence { statements, .. } = body else {
            panic!("{body:?}");
        };
        let TypeScriptStatement::PureLet { initializer, .. } = &statements[0] else {
            panic!("{statements:?}");
        };
        cold_callback(initializer);

        let direct = compile_module(
            CompileInput::new(
                "provider-direct.ssrg",
                "fixture/provider-direct",
                &format!("{IMPORTS}pub effect fn wait pause: time.Duration -> Unit with Clock = clock.sleep pause\n"),
            )
            .with_profile(profile),
        )
        .unwrap();
        let TypeScriptFunction::ConstFunction { body, .. } = &direct.typescript_ir.functions[0];
        assert!(
            matches!(body, TypeScriptExpr::RuntimeCall { callee, arguments }
            if callee == "_ssrg_clock_sleep" && arguments.len() == 1)
        );
    }
}

#[test]
fn returned_clock_callable_does_not_consume_the_factory_unit_stage() {
    let source = format!(
        "{IMPORTS}fn factory -> (time.Duration -> Effect<{{clock: Clock}}, Never, Unit>) = clock.sleep\npub effect fn wait pause: time.Duration -> Unit with Clock = (factory ()) pause\n"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("provider-factory.ssrg", "fixture/provider-factory", &source)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        let factory = &compiled.core_ir.functions[0];
        assert_eq!(factory.parameters.len(), 1);
        assert_eq!(factory.parameters[0].kind, "implicit");
        assert!(matches!(factory.body, CoreExpr::Variable { .. }));
        source_reference(&factory.body);
        let TypeScriptFunction::ConstFunction {
            parameters, body, ..
        } = &compiled.typescript_ir.functions[0];
        assert_eq!(parameters.len(), 1);
        assert!(parameters[0].implicit);
        // This bare reference already has the same single source/host argument;
        // it stays a reference while the factory retains its own Unit stage.
        assert!(matches!(body, TypeScriptExpr::RuntimeReference { name }
            if name == "_ssrg_clock_sleep"));

        let invalid = source.replace("(factory ()) pause", "factory pause");
        let error = compile_module(
            CompileInput::new(
                "provider-factory.ssrg",
                "fixture/provider-factory",
                &invalid,
            )
            .with_profile(profile),
        )
        .expect_err("factory source Unit may not be omitted");
        assert!(error.diagnostics.iter().any(|d| d.code == "SES-T0101"
            && d.message_key == "call.argument-type-mismatch"
            && &invalid[d.primary.start..d.primary.end] == "pause"));
    }
}

#[test]
fn clock_callable_keeps_nominal_argument_identity_and_user_function_ownership() {
    for (declarations, argument) in [
        ("", "()"),
        ("", "0"),
        ("struct Duration {}\n", "Duration {}"),
    ] {
        let source = format!(
            "{IMPORTS}{declarations}pub effect fn wrong -> Unit with Clock = do {{ let saved = clock.sleep; saved ({argument}) }}\n"
        );
        let error = compile_module(CompileInput::new(
            "provider-owner.ssrg",
            "fixture/provider-owner",
            &source,
        ))
        .expect_err("source arguments require canonical std/time Duration");
        assert!(error
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0101" && d.message_key == "call.argument-type-mismatch"));
    }
    let source = "fn sleep first: Int -> (Int -> Int) = \\second -> first + second\npub fn factory value: Int -> (Int -> Int) = sleep value\n";
    let compiled = compile_module(CompileInput::new(
        "provider-owned.ssrg",
        "fixture/provider-owned",
        source,
    ))
    .unwrap();
    let factory = compiled
        .typescript_ir
        .functions
        .iter()
        .find(|function| matches!(function, TypeScriptFunction::ConstFunction { name, .. } if name == "factory"))
        .unwrap();
    let TypeScriptFunction::ConstFunction { body, .. } = factory;
    assert!(
        matches!(body, TypeScriptExpr::Call { callee, arguments, .. }
        if callee == "sleep" && arguments.len() == 1)
    );
}
