use seseragi_backend_rust::{emit_program_wasm, emit_rust_module, emit_rust_tests};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreModule, CoreType};

fn checked(module: &str, source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("std-test-雪.ssrg", module, source))
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
        .core_ir
}

#[test]
fn std_test_accepts_direct_saved_partial_generic_and_user_dictionaries() {
    let module = checked(
        "native/test",
        include_str!("../../../examples/spec/fixtures/rust-backend/std-test.ssrg"),
    );
    let artifact = emit_rust_module(&module, "main").unwrap();
    assert!(artifact.metadata.capabilities.contains(&"test".into()));
    assert!(artifact.source.contains("native_test_operation"));
}

#[test]
fn std_test_instantiates_checked_declarations_without_losing_caller_type_identity() {
    for (declaration, actual) in [
        (
            "fn probe -> Effect<{},t.TestFailure,Unit> = t.equal 1 2",
            "Int",
        ),
        (
            "fn probe<A> value:A -> Effect<{},t.TestFailure,Unit> where Eq<A>, Debug<A> = t.equal value value",
            "A",
        ),
        (
            "fn probe<Unit> value:Unit -> Assertion where Eq<Unit>, Debug<Unit> = t.equal value value",
            "Unit",
        ),
    ] {
        let source = format!(
            "import * as t from \"std/test\"\nalias Assertion = Effect<{{}},t.TestFailure,Unit>\n{declaration}\npub fn main -> Int = 42\n"
        );
        let mut core = checked("native/test", &source);
        let CoreExpr::Call {
            type_ref,
            callee_signature,
            ..
        } = &mut core.functions[0].body
        else {
            panic!("checked std/test call")
        };
        let value = |name: &str| CoreType::Named {
            name: name.into(),
            arguments: vec![],
        };
        let mut result = type_ref.clone();
        let (CoreType::Named { arguments, .. }
        | CoreType::ExternalNamed { arguments, .. }) = &mut result
        else {
            panic!("checked Effect result")
        };
        arguments[2] = value("Unit");
        *callee_signature = Some(seseragi_lowering::CoreCallSignature {
            type_parameters: vec![seseragi_syntax::TypeParameter {
                name: "A".into(),
                arity: 0,
            }],
            parameters: vec![value("A"), value("A")],
            result,
            argument_types: vec![value(actual), value(actual)],
            instantiated_type_arguments: vec![value(actual)],
        });
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..7 {
            let mut forged = core.clone();
            let CoreExpr::Call {
                callee_signature: Some(signature),
                ..
            } = &mut forged.functions[0].body
            else {
                panic!("checked std/test declaration")
            };
            match mutation {
                0 => signature.instantiated_type_arguments.clear(),
                1 => signature.instantiated_type_arguments.push(value(actual)),
                2 => signature.instantiated_type_arguments[0] = CoreType::Hole,
                3 => signature.instantiated_type_arguments[0] = value("Array"),
                4 => signature.instantiated_type_arguments[0] = value("String"),
                5 => signature.type_parameters[0].arity = 1,
                6 => {
                    signature.type_parameters.push(signature.type_parameters[0].clone());
                    signature.instantiated_type_arguments.push(value(actual));
                }
                _ => unreachable!(),
            }
            assert!(
                emit_rust_module(&forged, "main").is_err(),
                "accepted {actual} mutation {mutation}"
            );
        }
    }
}

#[test]
fn native_test_entry_links_independent_test_modules_and_checks_export_types() {
    let first = checked("native/first", "import * as t from \"std/test\"\npub let tests:t.Test = t.test \"first\" (t.isTrue True)\n");
    let second = checked("native/second", "import * as t from \"std/test\"\npub let tests:t.Test = t.skip \"later\" (t.test \"second\" (t.isFalse False))\n");
    let tests = vec![
        ("first".into(), "native/first::tests".into()),
        ("second".into(), "native/second::tests".into()),
    ];
    let artifact = emit_rust_tests(&[&first, &second], &tests).unwrap();
    assert_eq!(artifact.metadata.entry_kind, "test");
    assert_eq!(artifact.metadata.modules, ["native/first", "native/second"]);
    assert_eq!(artifact.source.matches("fn main()").count(), 1);
    assert!(artifact.source.contains("get_global(0)"));
    assert!(artifact.source.contains("get_global(1)"));
    assert!(artifact.source.contains("run_native_tests_entry"));
    let invalid = checked("native/first", "pub let tests:Int = 1\n");
    assert!(emit_rust_tests(&[&invalid, &second], &tests).is_err());
}

#[test]
fn std_test_rejects_forged_results_dictionaries_kinds_source_stages_and_declarations() {
    let source = "import * as t from \"std/test\"\nfn probe -> Effect<{},t.TestFailure,Unit> = t.equal 1 2\npub fn main -> Int = 42\n";
    let core = checked("native/test", source);
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..13 {
        let mut module = core.clone();
        let probe = module
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&probe.body).clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            type_ref,
            trait_dispatch: _,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            callee_signature,
            ..
        } = &mut probe.body
        else {
            panic!("checked std/test call")
        };
        if mutation >= 11 {
            *callee_signature = Some(seseragi_lowering::CoreCallSignature {
                type_parameters: vec![],
                parameters: vec![
                    CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![]
                    };
                    2
                ],
                result: type_ref.clone(),
                argument_types: vec![],
                instantiated_type_arguments: vec![],
            });
        }
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => arguments.push(arguments[0].clone()),
            2 => evidence.clear(),
            3 => evidence.reverse(),
            4 => evidence[0].constraint.trait_identity = Some("forged::Eq".into()),
            5 => evidence[1].constraint.trait_identity = Some("forged::Debug".into()),
            6 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "std/string::Eq".into();
            }
            7 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[1].evidence
                else {
                    panic!()
                };
                *identity = "std/string::Debug".into();
            }
            8 => *evidence_argument_index = Some(0),
            9 => deferred_evidence_parameters.push(CoreType::Hole),
            10 => deferred_evidence_type_constructor_parameters.push("F".into()),
            11 => callee_signature
                .as_mut()
                .unwrap()
                .parameters
                .pop()
                .map(|_| ())
                .unwrap(),
            12 => callee_signature.as_mut().unwrap().result = CoreType::Hole,
            _ => unreachable!(),
        }
        let error = emit_rust_module(&module, "main").unwrap_err();
        assert_eq!(error.origin, Some(origin), "mutation {mutation}: {error}");
    }
    let mut generic = checked("native/test", "import * as t from \"std/test\"\nfn probe<A> a:A -> Effect<{},t.TestFailure,Unit> where Eq<A>, Debug<A> = t.equal a a\npub fn main -> Int = 42\n");
    emit_rust_module(&generic, "main").unwrap();
    generic.functions[0].type_parameters[0].arity = 1;
    assert!(emit_rust_module(&generic, "main").is_err());
}

#[test]
fn native_test_values_remain_rejected_in_program_wasm_even_when_unused() {
    for (expression, output) in [
        ("t.isTrue True", "Effect<{},t.TestFailure,Unit>"),
        ("t.fail \"boom\"", "Effect<{},t.TestFailure,Unit>"),
        ("t.equal 1 1", "Effect<{},t.TestFailure,Unit>"),
        ("t.test \"case\" (t.isTrue True)", "t.Test"),
    ] {
        let core = checked("native/test", &format!("import * as t from \"std/test\"\nfn unused -> {output} = {expression}\npub fn main -> Int = 42\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/test", &["main".into()]).unwrap_err();
        assert!(error.origin.is_some(), "{error}");
    }
}

#[test]
fn test_bodies_cannot_add_unprovided_services_or_other_failure_and_success_types() {
    let core = checked("native/test", "import * as t from \"std/test\"\nfn probe -> t.Test = t.test \"case\" (t.isTrue True)\npub fn main -> Int = 42\n");
    emit_rust_module(&core, "main").unwrap();
    for slot in 0..3 {
        let mut forged = core.clone();
        let probe = forged
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let CoreExpr::Call { arguments, .. } = &mut probe.body else {
            panic!("test call")
        };
        let CoreExpr::Call { type_ref, .. } = &mut arguments[1] else {
            panic!("assertion call")
        };
        let CoreType::Named { arguments, .. } = type_ref else {
            panic!("Effect type")
        };
        arguments[slot] = if slot == 0 {
            CoreType::Record {
                closed: true,
                fields: vec![seseragi_lowering::CoreRecordField {
                    name: "fileSystem".into(),
                    optional: false,
                    type_ref: CoreType::Named {
                        name: "std/fs::FileSystem".into(),
                        arguments: vec![],
                    },
                }],
            }
        } else {
            CoreType::Named {
                name: "String".into(),
                arguments: vec![],
            }
        };
        let error = emit_rust_module(&forged, "main").unwrap_err();
        assert!(error.origin.is_some(), "{error}");
    }
}
