use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "standard-雪🙂.ssrg",
        "native/standard",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn basic_standard_operations_keep_saved_callables_and_the_former_sqrt_refusal() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-text-math.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    for capability in ["text", "math"] {
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|c| c == capability));
    }
    emit_rust_module(
        &checked(include_str!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-math-effects.ssrg"
        )),
        "main",
    )
    .unwrap();
    // Preserve the exact former unsupported source from native.rs.
    emit_rust_module(
        &checked("import * as math from \"std/math\"\npub fn probe -> Float = math.sqrt 4.0\n"),
        "probe",
    )
    .unwrap();
}

#[test]
fn text_boundaries_keep_saved_stages_and_reject_unused_program_wasm_references() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-text-boundaries.ssrg");
    emit_rust_module(&checked(source), "main").unwrap();
    for operation in ["startsWith", "endsWith"] {
        let source = format!("// 雪🙂\nimport * as text from \"std/text\"\npub fn probe value: Int -> Int = {{ let saved: String -> String -> Bool = text.{operation}; value }}\n");
        let core = checked(&source);
        let error = emit_program_wasm(&[&core], "native/standard", &["probe".into()]).unwrap_err();
        assert!(
            error
                .message
                .contains("native text extension requires a target-specific proof"),
            "{error}"
        );
        assert_eq!(error.origin.unwrap().source, "standard-雪🙂.ssrg");
    }
}

#[test]
fn basic_standard_calls_reject_forged_types_evidence_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (module, expression, result) in [
        ("math", "sqrt 4.0", "Float"),
        ("math", "sin 0.5", "Float"),
        ("math", "cos 0.5", "Float"),
        ("math", "log 2.0", "Float"),
        ("text", "split \",\" \"雫🙂\"", "Array<String>"),
        ("text", "trim \"雫🙂\"", "String"),
        ("text", "contains \"雫\" \"雫🙂\"", "Bool"),
        ("text", "startsWith \"雫\" \"雫🙂\"", "Bool"),
        ("text", "endsWith \"🙂\" \"雫🙂\"", "Bool"),
        ("text", "replace \"雫\" \"川\" \"雫🙂\"", "String"),
    ] {
        let source = format!(
            "import * as api from \"std/{module}\"\npub fn probe -> {result} = api.{expression}\n"
        );
        let core = checked(&source);
        emit_rust_module(&core, "probe").unwrap();
        let expected_origin = core_expr_origin(&core.functions[0].body).clone();
        let CoreExpr::Call { arguments, .. } = &core.functions[0].body else {
            panic!()
        };
        for mutation in 0..9 + arguments.len() {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[0].body
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
                        name: format!("std/prelude::{result}"),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 3]),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                _ => {
                    let index = mutation - 9;
                    arguments[index] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[index]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "probe").expect_err("unproven standard call");
            assert_eq!(
                error.code, "SES-RUST-0001",
                "{module}/{expression}/{mutation}"
            );
            let origin = error.origin.unwrap();
            assert_eq!(origin.source, "standard-雪🙂.ssrg");
            assert_eq!(origin, expected_origin, "{module}/{expression}/{mutation}");
        }
    }
}

#[test]
fn canonical_standard_arguments_do_not_substitute_lexical_float_or_string_binders() {
    for (scalar, module, expression) in [
        ("Float", "math", "sqrt 4.0"),
        ("String", "text", "trim \"雫\""),
    ] {
        let mut core = checked(&format!("fn identity<{scalar}> value: {scalar} -> {scalar} = value\npub fn probe -> {scalar} = {}\n", if scalar == "Float" { "1.0" } else { "\"雫\"" }));
        let mut call = checked(&format!(
            "import * as api from \"std/{module}\"\npub fn probe -> {scalar} = api.{expression}\n"
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
        arguments[0] = identity.body.clone();
        identity.body = call;
        let error = emit_rust_module(&core, "probe")
            .expect_err("lexical scalar is not a canonical standard argument");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "standard-雪🙂.ssrg");
    }
}

#[test]
fn native_basic_extensions_do_not_silently_expand_program_wasm_admission() {
    for (module, expression) in [
        ("math", "sqrt 4.0"),
        ("math", "sin 0.5"),
        ("math", "cos 0.5"),
        ("math", "log 2.0"),
        ("text", "split \",\" \"雫🙂\""),
        ("text", "trim \"雫🙂\""),
        ("text", "contains \"雫\" \"雫🙂\""),
        ("text", "startsWith \"雫\" \"雫🙂\""),
        ("text", "endsWith \"🙂\" \"雫🙂\""),
        ("text", "replace \"雫\" \"川\" \"雫🙂\""),
    ] {
        let source = format!("import * as api from \"std/{module}\"\npub fn probe value: Int -> Int = {{ let result = api.{expression}; value }}\npub fn main -> Int = probe 1\n");
        let core = checked(&source);
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/standard", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific"),
            "{error}"
        );
        assert_eq!(error.origin.unwrap().source, "standard-雪🙂.ssrg");
    }
}

#[test]
fn prelude_collections_keep_selected_dictionaries_and_saved_stages() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/prelude-collections.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn prelude_collections_reject_forged_source_types_dictionaries_kinds_and_stages() {
    for (result, expression) in [
        ("Int", "sum [1,2]"),
        ("String", "join \",\" [\"a\",\"b\"]"),
        (
            "Effect<{},Never,Unit>",
            "forEach (\\n:Int -> effects.succeed ()) [1,2]",
        ),
    ] {
        let core = checked(&format!(
            "import * as effects from \"std/effect\"\npub fn probe -> {result} = {expression}\n"
        ));
        emit_rust_module(&core, "probe").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..12 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                type_ref,
                evidence,
                arguments,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[0].body
            else {
                panic!("{expression}")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => evidence.clear(),
                2 => evidence[0].constraint.trait_identity = Some("fake::Reducible".into()),
                3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                4 => {
                    evidence[0].constraint.arguments[1] = CoreType::Named {
                        name: "std/prelude::String".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                5 => evidence[0].constraint.arguments.swap(0, 1),
                6 => {
                    let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                        &mut evidence[0].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/set::Reducible".into();
                }
                7 => evidence.push(evidence[0].clone()),
                8 => deferred_evidence_parameters.push(CoreType::Hole),
                9 => deferred_evidence_type_constructor_parameters.push("F".into()),
                10 => *evidence_argument_index = Some(0),
                _ => {
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "probe")
                .expect_err(&format!("{expression}: mutation {mutation}"));
            assert_eq!(
                error.origin.as_ref(),
                Some(&origin),
                "{expression}: {mutation}: {error}"
            );
        }
    }
}

#[test]
fn prelude_collections_keep_reducible_parent_and_rank_polymorphic_method_proofs() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/prelude-collections.ssrg"
    ));
    let index = core
        .instances
        .iter()
        .position(|instance| instance.identity.starts_with("std/prelude::Reducible<"))
        .unwrap();
    for mutation in 0..6 {
        let mut changed = core.clone();
        let instance = &mut changed.instances[index];
        match mutation {
            0 => instance.supertrait_count = 0,
            1 => instance.constraints[0].trait_identity = Some("fake::Iterable".into()),
            2 => instance.constraints[0].arguments.swap(0, 1),
            _ => {
                let seseragi_lowering::CoreInstanceImplementation::UserDefined { methods } =
                    &mut instance.implementation
                else {
                    panic!()
                };
                match mutation {
                    3 => methods[0].type_parameters[0].arity = 1,
                    4 => methods[0].name = "iterate".into(),
                    _ => methods[0].parameters[0].type_ref = CoreType::Hole,
                }
            }
        }
        emit_rust_module(&changed, "main").expect_err(&format!("mutation {mutation}"));
    }
}

#[test]
fn prelude_collections_do_not_expand_program_wasm_admission() {
    for (result, expression) in [("Int", "sum [1]"), ("String", "join \",\" [\"a\"]")] {
        let core = checked(&format!("pub fn probe -> {result} = {expression}\n"));
        let error = emit_program_wasm(&[&core], "native/standard", &["probe".into()]).unwrap_err();
        assert!(
            error
                .message
                .contains("native Prelude collection requires a target-specific ABI proof"),
            "{error}"
        );
    }
}
