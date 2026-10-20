use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
use std::{fs, process::Command};

fn execute(source: &str, name: &str, stdout: &str, code: i32) {
    let core = compile_core_module(CompileInput::new(
        "method-reference-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let directory = std::env::temp_dir().join(format!(
        "seseragi-method-reference-{name}-{}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let input = directory.join("program.rs");
    let exe = directory.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, &artifact.source).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&exe)
        .arg(&input)
        .output()
        .unwrap();
    let result = built.status.success().then(|| {
        Command::new(&exe)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap()
    });
    fs::remove_dir_all(&directory).unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = result.unwrap();
    assert_eq!(
        result.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), stdout);
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
}

#[test]
fn method_owned_where_direct_application_keeps_actual_provider_stages() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/method-direct-where-defect.ssrg"
    );
    execute(source, "where-direct-first-stage", "", 70);
    for (name, body) in [
        ("let", "  let saved = pick left\n  saved"),
        (
            "annotated-let",
            "  let saved: F<A> -> Bool = pick left\n  saved",
        ),
        ("nested-let", "  let saved = { pick left }\n  saved"),
    ] {
        assert!(source.contains("  pick left"));
        execute(
            &source.replace("  pick left", body),
            &format!("where-direct-{name}"),
            "",
            70,
        );
    }
    let two_parameters = source
        .replace(
            "fn pick<A> left: Maybe<A> -> (Maybe<A> -> Bool)",
            "fn pick<A> left: Maybe<A> -> right: Maybe<A> -> Bool",
        )
        .replace("\\right -> True", "True");
    execute(
        &two_parameters,
        "where-direct-two-source-stages",
        "created\n",
        0,
    );
    for (source, name, stdout, code) in [
        (
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-operator.ssrg"
            ),
            "operator",
            "42\n",
            0,
        ),
        (
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-trace.ssrg"
            ),
            "trace",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-trace.stdout"
            ),
            0,
        ),
        (
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-values.ssrg"
            ),
            "values",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-values.stdout"
            ),
            0,
        ),
        (
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-effects.ssrg"
            ),
            "effects",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-effects.stdout"
            ),
            0,
        ),
        (
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-cold.ssrg"
            ),
            "cold",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-cold.stdout"
            ),
            0,
        ),
        (
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-children.ssrg"
            ),
            "children",
            "",
            70,
        ),
    ] {
        execute(source, &format!("where-direct-{name}"), stdout, code);
    }
}

#[test]
fn method_owned_where_alias_runs_the_selected_first_stage() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-defect.ssrg");
    for (name, source) in [
        ("captured", source.to_string()),
        (
            "generalized",
            source.replace(
                "let chosen: F<A> -> F<A> -> Bool = pick",
                "let chosen = pick",
            ),
        ),
        (
            "closed",
            source.replace(
                "let saved = selected<Maybe, Int> (Just 1)",
                "let chosen: Maybe<Int> -> Maybe<Int> -> Bool = pick; let saved = chosen (Just 1)",
            ),
        ),
    ] {
        execute(&source, &format!("where-{name}"), "", 70);
    }
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-values.ssrg"),
        "where-values",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-values.stdout"),
        0,
    );
}

#[test]
fn method_owned_where_values_keep_managed_edges_and_evidence_frames() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-effects.ssrg"),
        "where-effects",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-effects.stdout"),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-children.ssrg"),
        "where-children",
        "",
        70,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-cold.ssrg"),
        "where-cold",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-where-cold.stdout"),
        0,
    );
}

#[test]
fn constrained_method_values_keep_never_nominal_and_binder_owners() {
    execute(
        "type Never = | Never Int\ntrait Pick<T> { fn pick<A> left: T -> right: A -> String where Show<A> }\ninstance Pick<Never> { fn pick<A> left: Never -> (A -> String) where Show<A> = \\right -> show right }\nfn capture<T, A> left: T -> (A -> String) where Pick<T>, Show<A> = { let chosen: T -> A -> String = pick; chosen left }\npub effect fn main = do { let held = capture<Never, Int> (Never 1); println (held 7) }",
        "where-never-nominal", "7\n", 0,
    );
    execute(
        "type Tag = | Tag\ntrait Pick<T> { fn pick<A> left: T -> right: A -> String where Show<A> }\ninstance Pick<Tag> { fn pick<A> left: Tag -> (A -> String) where Show<A> = \\right -> show right }\nfn capture<Never> value: Never -> (Never -> String) where Show<Never> = { let chosen: Tag -> Never -> String = pick; chosen Tag }\npub effect fn main = do { let held = capture 1; println (held 7) }",
        "where-never-binder", "7\n", 0,
    );
}

#[test]
fn method_owned_where_values_reject_forged_contracts_kinds_and_frames() {
    use seseragi_lowering::CoreStatement;
    for mode in ["captured", "generalized", "direct"] {
        let mut source =
            include_str!("../../../examples/spec/fixtures/rust-backend/method-where-defect.ssrg")
                .to_string();
        if mode == "generalized" {
            source = source.replace(
                "let chosen: F<A> -> F<A> -> Bool = pick",
                "let chosen = pick",
            );
        }
        if mode == "direct" {
            source = include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-direct-where-defect.ssrg"
            )
            .into();
        }
        let core = compile_core_module(CompileInput::new(
            "method-where-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..34 {
            let mut changed = core.clone();
            let declaration = &mut changed.trait_methods[0];
            match mutation {
                0 => declaration.constraints.clear(),
                1 => declaration
                    .constraints
                    .push(declaration.constraints[0].clone()),
                2 => declaration.constraints[0].trait_identity = Some("std/prelude::Debug".into()),
                3 => declaration.signature.type_parameters[0].arity = 1,
                4 => declaration.trait_parameters[0].arity = 0,
                5 => declaration.signature.type_parameters[0].name = "F".into(),
                6 => declaration.signature.parameters[0] = CoreType::Hole,
                7 => declaration.signature.result = CoreType::Hole,
                8 => declaration.signature.parameters.clear(),
                9 => declaration.signature.argument_types.push(CoreType::Hole),
                10 => changed.trait_methods.clear(),
                26..=33 => {
                    let seseragi_lowering::CoreInstanceImplementation::UserDefined { methods } =
                        &mut changed.instances[0].implementation
                    else {
                        panic!("provider")
                    };
                    let method = &mut methods[0];
                    match mutation {
                        26 => method.constraints.clear(),
                        27 => {
                            method.constraints[0].trait_identity = Some("std/prelude::Debug".into())
                        }
                        28 => {
                            method.constraints[0].arguments[0] = CoreType::Named {
                                name: "String".into(),
                                arguments: vec![],
                            }
                        }
                        29 => method.type_parameters[0].arity = 1,
                        30 => method
                            .type_parameters
                            .push(method.type_parameters[0].clone()),
                        31 => method.parameters.clear(),
                        32 => method.constraints.push(method.constraints[0].clone()),
                        _ => method.parameters[0].type_ref = CoreType::Hole,
                    }
                }
                _ => {
                    let function = changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::selected"))
                        .unwrap();
                    let CoreExpr::Sequence {
                        statements, result, ..
                    } = &mut function.body
                    else {
                        panic!("body")
                    };
                    let statements = if mode == "direct" {
                        let CoreExpr::Sequence { statements, .. } = result.as_mut() else {
                            panic!("direct method value")
                        };
                        statements
                    } else {
                        statements
                    };
                    let value = match &mut statements[0] {
                        CoreStatement::PureLet { value, .. } => value,
                        CoreStatement::LocalFunction { body, .. } => body,
                        _ => panic!("factory"),
                    };
                    let CoreExpr::Call {
                        evidence,
                        trait_dispatch,
                        type_ref,
                        deferred_evidence_parameters,
                        evidence_argument_index,
                        ..
                    } = value
                    else {
                        panic!("method")
                    };
                    let dispatch = trait_dispatch.as_mut().unwrap();
                    match mutation {
                        11 => dispatch.method = "invented".into(),
                        12 => dispatch.trait_identity = "counterfeit::trait(Pick)".into(),
                        13 => dispatch.method_contract = None,
                        14 => dispatch
                            .method_contract
                            .as_mut()
                            .unwrap()
                            .parameters
                            .clear(),
                        15 => {
                            dispatch.method_contract.as_mut().unwrap().parameters[0] =
                                CoreType::Hole
                        }
                        16 => {
                            dispatch.method_contract.as_mut().unwrap().constraints[1]
                                .trait_identity = Some("std/prelude::Debug".into())
                        }
                        17 => {
                            dispatch.method_contract.as_mut().unwrap().constraints[1].arguments[0] =
                                CoreType::Named {
                                    name: "Bool".into(),
                                    arguments: vec![],
                                }
                        }
                        18 => evidence[1].evidence = CoreInstanceEvidence::Parameter { index: 0 },
                        19 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
                        20 => {
                            evidence[1].constraint.trait_identity =
                                Some("std/prelude::Debug".into())
                        }
                        21 => {
                            evidence.pop();
                        }
                        22 => *type_ref = CoreType::Hole,
                        23 => *evidence_argument_index = Some(0),
                        24 => deferred_evidence_parameters.push(CoreType::Hole),
                        _ => {
                            let fake = CoreType::Named {
                                name: "Bool".into(),
                                arguments: vec![],
                            };
                            evidence[1].constraint.arguments[0] = fake.clone();
                            dispatch.method_contract.as_mut().unwrap().constraints[1].arguments
                                [0] = fake;
                        }
                    }
                }
            }
            let error = match emit_rust_module(&changed, "main") {
                Err(error) => error,
                Ok(_) => {
                    panic!("accepted method-where mutation {mutation}, mode {mode}")
                }
            };
            assert_eq!(
                error.code, "SES-RUST-0001",
                "mutation {mutation}, mode {mode}"
            );
            assert_eq!(error.origin.as_ref().unwrap().source, "method-where-λ.ssrg");
        }
    }
}

#[test]
fn parameter_method_providers_cannot_add_undeclared_dictionary_stages() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-defect.ssrg");
    let mut core = compile_core_module(CompileInput::new(
        "undeclared-where-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let seseragi_lowering::CoreInstanceImplementation::UserDefined { methods } =
        &mut core.instances[0].implementation
    else {
        panic!("provider")
    };
    methods[0]
        .constraints
        .push(seseragi_lowering::CoreInstanceConstraint {
            name: "Show".into(),
            trait_identity: Some("std/prelude::Show".into()),
            arguments: vec![CoreType::Named {
                name: "A".into(),
                arguments: vec![],
            }],
        });
    let result = emit_rust_module(&core, "main");
    assert!(
        result.is_err(),
        "accepted undeclared provider dictionary stage"
    );
}

#[test]
fn selected_bare_methods_run_their_first_stage_when_applied() {
    for (method, implementation, result) in [
        ("add", "instance Add<Mark, Mark, Mark> { fn add left: Mark -> (Mark -> Mark) = { let bad = 9007199254740991 + 1; \\right -> left } }", "Mark"),
        ("eq", "instance Eq<Mark> { fn eq left: Mark -> (Mark -> Bool) = { let bad = 9007199254740991 + 1; \\right -> True } }", "Bool"),
    ] {
        let source = format!("type Mark = | Mark Int\n{implementation}\npub effect fn main = do {{ let selected: Mark -> Mark -> {result} = {method}; let held = selected (Mark 1); println \"unreachable\" }}");
        execute(&source, method, "", 70);
    }
}

fn scoped_method_defect(method: &str, instance: &str, constraint: &str, result: &str) {
    let source = format!(
        "type Mark = | Mark Int\n{instance}\n\
         pub effect fn main = do {{\n\
           let held = {{\n\
             fn saved<A> left: A -> (A -> {result}) where {constraint} = {{\n\
               let chosen: A -> A -> {result} = {method}\n\
               chosen left\n\
             }}\n\
             saved (Mark 1)\n\
           }}\n\
           println \"unreachable\"\n\
         }}\n"
    );
    execute(&source, &format!("scoped-{method}"), "", 70);
}

#[test]
fn scoped_eq_method_alias_keeps_its_available_dictionary_stage() {
    scoped_method_defect(
        "eq",
        "instance Eq<Mark> { fn eq left: Mark -> (Mark -> Bool) = { let bad = 9007199254740991 + 1; \\right -> True } }",
        "Eq<A>",
        "Bool",
    );
}

#[test]
fn scoped_add_method_alias_keeps_its_available_dictionary_stage() {
    scoped_method_defect(
        "add",
        "instance Add<Mark, Mark, Mark> { fn add left: Mark -> (Mark -> Mark) = { let bad = 9007199254740991 + 1; \\right -> left } }",
        "Add<A, A, A>",
        "A",
    );
}

#[test]
fn user_trait_parameter_methods_preserve_the_selected_first_stage() {
    for initializer in ["let chosen: A -> A -> Bool = pick", "let chosen = pick"] {
        execute(
            &format!(
                "type Mark = | Mark Int\n\
                 trait Pick<A> {{ fn pick left: A -> right: A -> Bool }}\n\
                 instance Pick<Mark> {{ fn pick left: Mark -> (Mark -> Bool) = {{\n\
                   let bad = 9007199254740991 + 1; \\right -> True\n\
                 }} }}\n\
                 fn selected<A> left: A -> (A -> Bool) where Pick<A> = {{\n\
                   {initializer}; chosen left\n\
                 }}\n\
                 pub effect fn main = do {{ let saved = selected (Mark 1); println \"unreachable\" }}"
            ),
            if initializer.contains(':') { "user-scoped" } else { "user-generalized" },
            "",
            70,
        );
    }
}

#[test]
fn generalized_eq_alias_runs_the_selected_method_first_stage() {
    execute(
        "type Mark = | Mark Int\n\
         instance Eq<Mark> { fn eq left: Mark -> (Mark -> Bool) = {\n\
           let bad = 9007199254740991 + 1; \\right -> True\n\
         } }\n\
         fn selected<A> left: A -> (A -> Bool) where Eq<A> = {\n\
           let chosen = eq; chosen left\n\
         }\n\
         pub effect fn main = do { let saved = selected (Mark 1); println \"unreachable\" }",
        "generalized-eq-stage",
        "",
        70,
    );
}

#[test]
fn constrained_global_method_aliases_preserve_existing_programs_and_first_stage() {
    execute(
        "let equal = eq\npub effect fn main = println (show (equal 40 42))",
        "global-eq-control",
        "False\n",
        0,
    );
    execute(
        "type Mark = | Mark Int\n\
         instance Eq<Mark> { fn eq left: Mark -> (Mark -> Bool) = { let bad = 9007199254740991 + 1; \\right -> True } }\n\
         let equal = eq\nlet copied = equal\n\
         pub effect fn main = do { let saved = copied (Mark 1); println \"unreachable\" }",
        "global-eq-stage", "", 70,
    );
}

#[test]
fn constrained_global_method_factories_reject_forged_signature_and_alias_cycles() {
    let source = "// 雫🙂\nlet equal = eq\nlet copied = equal\npub fn probe -> Bool = copied 40 42";
    let core = compile_core_module(CompileInput::new(
        "global-factory-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..8 {
        let mut changed = core.clone();
        let f = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::equal"))
            .unwrap();
        let CoreExpr::Call {
            callee,
            evidence,
            trait_dispatch,
            type_ref,
            ..
        } = &mut f.body
        else {
            panic!("factory")
        };
        match mutation {
            0 => {
                *callee = "native/main::copied".into();
                *trait_dispatch = None;
            }
            1 => f.type_parameters[0].arity = 1,
            2 => f.constraints.clear(),
            3 => f.constraints[0].trait_identity = Some("counterfeit::Eq".into()),
            4 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
            5 => *type_ref = CoreType::Hole,
            6 => trait_dispatch.as_mut().unwrap().method = "add".into(),
            _ => f.is_effect = true,
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "global-factory-λ.ssrg");
    }
}

#[test]
fn method_factory_arguments_preserve_nominal_owner_instead_of_display_name() {
    let source = "type Mark = | Mark Int\ntype Other = | Other Int\ninstance Eq<Mark> { fn eq left: Mark -> right: Mark -> Bool = True }\nlet equal = eq\npub fn probe -> Bool = equal (Mark 1) (Mark 2)";
    let mut core = compile_core_module(CompileInput::new("owner-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let probe = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::probe"))
        .unwrap();
    let CoreExpr::Call { arguments, .. } = &mut probe.body else {
        panic!("factory call")
    };
    let CoreExpr::Call { type_ref, .. } = &mut arguments[0] else {
        panic!("constructor")
    };
    *type_ref = CoreType::ExternalNamed {
        name: "Mark".into(),
        canonical: "native/main::Other".into(),
        arguments: vec![],
    };
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "owner-λ.ssrg");
}

#[test]
fn method_factory_result_substitutions_preserve_returned_function_stages() {
    let prefix = "type Mark = | Mark Int\ninstance Add<Mark, Mark, (Int -> Int)> { fn add left: Mark -> right: Mark -> (Int -> Int) = \\value -> value + 1 }\n";
    execute(&format!("{prefix}let chosen = add\npub effect fn main = println (show (chosen<Mark, Mark, (Int -> Int)> (Mark 1) (Mark 2) 41))"), "global-function-result", "42\n", 0);
    execute(&format!("{prefix}pub effect fn main = do {{ let chosen = add; println (show (chosen<Mark, Mark, (Int -> Int)> (Mark 1) (Mark 2) 41)) }}"), "local-function-result", "42\n", 0);
}

#[test]
fn generalized_method_factories_preserve_values_children_and_lifetimes() {
    for (name, source, expected, exit) in [
        (
            "values",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-values.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-values.stdout"
            ),
            0,
        ),
        (
            "defect",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-defect.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-defect.stdout"
            ),
            70,
        ),
        (
            "effects",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-effects.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-effects.stdout"
            ),
            0,
        ),
        (
            "children",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/generalized-method-children.ssrg"
            ),
            "",
            70,
        ),
    ] {
        execute(source, &format!("generalized-{name}"), expected, exit);
    }
}

#[test]
fn generalized_method_factories_reject_forged_frames_types_and_stages() {
    use seseragi_lowering::CoreStatement;
    let source = "// 雫🙂\npub fn probe -> Bool = { let chosen = eq; chosen 42 43 }";
    let core = compile_core_module(CompileInput::new("factory-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..18 {
        let mut changed = core.clone();
        let CoreExpr::Sequence {
            statements, result, ..
        } = &mut changed.functions[0].body
        else {
            panic!("sequence")
        };
        let CoreStatement::LocalFunction {
            type_parameters,
            constraints,
            parameters,
            body,
            ..
        } = &mut statements[0]
        else {
            panic!("method factory")
        };
        assert!(parameters.is_empty());
        let CoreExpr::Call {
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            evidence_argument_index,
            ..
        } = body
        else {
            panic!("selected method")
        };
        match mutation {
            0 => type_parameters[0].name = "counterfeit".into(),
            1 => type_parameters[0].arity = 1,
            2 => constraints.clear(),
            3 => constraints[0].trait_identity = Some("counterfeit::Eq".into()),
            4 => constraints[0].arguments[0] = CoreType::Hole,
            5 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
            6 => trait_dispatch.as_mut().unwrap().method = "add".into(),
            7 => trait_dispatch.as_mut().unwrap().trait_identity = "counterfeit::Eq".into(),
            8 => *type_ref = CoreType::Hole,
            9 => {
                let CoreType::Function { parameter, .. } = type_ref else {
                    panic!("arrow")
                };
                **parameter = CoreType::Hole;
            }
            10 => *evidence_argument_index = Some(0),
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => constraints.push(constraints[0].clone()),
            _ => {
                let CoreExpr::Call {
                    evidence,
                    evidence_argument_index,
                    type_ref,
                    arguments,
                    ..
                } = result.as_mut()
                else {
                    panic!("factory call")
                };
                match mutation {
                    13 => *evidence_argument_index = Some(1),
                    14 => evidence[0].constraint.trait_identity = Some("counterfeit::Eq".into()),
                    15 => *type_ref = CoreType::Hole,
                    16 => arguments.clear(),
                    _ => *evidence_argument_index = None,
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "factory-λ.ssrg");
    }
}

#[test]
fn generalized_method_values_preserve_source_stages_and_single_evaluation() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/generalized-method-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-trace.ssrg"),
    ] {
        let core = compile_core_module(CompileInput::new("trace.ssrg", "native/main", source))
            .unwrap()
            .core_ir;
        let artifact = emit_rust_module(&core, "probe").unwrap();
        let mut rust = artifact.source[..artifact.source.rfind("fn main()").unwrap()].to_owned();
        let id = |name: &str| {
            format!(
                "v_{}",
                name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
            )
        };
        // Observe ordinary helper entries in a test copy. The implementation's
        // one-source-parameter method and two-parameter method must keep distinct
        // stages; a saved partial must not repeat its left argument or first stage.
        for name in ["left", "right", "first", "lateLeft", "lateRight", "later"] {
            let start = rust
                .find(&format!(
                    "fn {}_direct(",
                    id(&format!("native/main::{name}"))
                ))
                .unwrap();
            let offset = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
            rust.insert_str(
                offset,
                &format!(" TRACE.with(|t| t.borrow_mut().push({name:?})); "),
            );
        }
        rust.push_str(&format!(r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let result = {}(vec![]).apply(V::Unit);
 assert_eq!(result.render(), "[True, True, True]");
 TRACE.with(|t| assert_eq!(&*t.borrow(), &["left", "first", "right", "right", "lateLeft", "lateRight", "later"]));
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#, id(&artifact.metadata.entry)));
        let dir =
            std::env::temp_dir().join(format!("seseragi-generalized-stage-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        let input = dir.join("program.rs");
        let executable = dir.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        fs::write(&input, rust).unwrap();
        let built = Command::new("rustc")
            .args(["--edition=2021", "-O", "-o"])
            .arg(&executable)
            .arg(&input)
            .output()
            .unwrap();
        let ran = built
            .status
            .success()
            .then(|| Command::new(&executable).env("PATH", "").output().unwrap());
        fs::remove_dir_all(&dir).unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let ran = ran.unwrap();
        assert!(
            ran.status.success(),
            "{}",
            String::from_utf8_lossy(&ran.stderr)
        );
    }
}

#[test]
fn bare_methods_preserve_stored_values_and_generalized_aliases() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-references.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-references.stdout"),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-trace.ssrg"),
        "trace",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-trace.stdout"),
        0,
    );
}

#[test]
fn bare_method_captures_keep_refs_cold_effects_and_finalizers() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-effects.ssrg"),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/method-reference-effects.stdout"
        ),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-children.ssrg"),
        "children",
        "",
        70,
    );
}

#[test]
fn selected_bare_method_rejects_forged_owner_types_children_and_stages() {
    let source = "type Mark = | Mark Int\ntrait Pick<A> { fn pick left: A -> (A -> String) }\ninstance Pick<Mark> { fn pick left: Mark -> (Mark -> String) = \\right -> \"selected\" }\npub fn probe -> (Mark -> Mark -> String) = pick";
    let core = compile_core_module(CompileInput::new("forged-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..18 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("selected method must remain a value")
        };
        let original = evidence[0].clone();
        match mutation {
            0 => trait_dispatch.as_mut().unwrap().trait_identity = "elsewhere::trait(Pick)".into(),
            1 => trait_dispatch.as_mut().unwrap().method = "unknown".into(),
            2 => evidence[0].constraint.name = "Other".into(),
            3 => evidence[0].constraint.trait_identity = Some("elsewhere::trait(Pick)".into()),
            4 => evidence[0].constraint.arguments.clear(),
            5 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Mark".into(),
                    canonical: "elsewhere::Mark".into(),
                    arguments: vec![],
                }
            }
            6 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            7 => {
                *type_ref = CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                }
            }
            8 => {
                let CoreType::Function { parameter, .. } = type_ref else {
                    panic!()
                };
                *parameter = Box::new(CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                });
            }
            9 => {
                let CoreType::Function { result, .. } = type_ref else {
                    panic!()
                };
                *result = Box::new(CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                });
            }
            10 => evidence.push(original.clone()),
            11 => evidence.clear(),
            12 => deferred_evidence_parameters.push(CoreType::Named {
                name: "A".into(),
                arguments: vec![],
            }),
            13 => deferred_evidence_type_constructor_parameters.push("F".into()),
            14 => *evidence_argument_index = Some(0),
            15 => {
                let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                });
            }
            16 => {
                let CoreInstanceEvidence::Local {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(original.clone());
            }
            17 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            _ => unreachable!(),
        }
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "accepted counterfeit {mutation}"
        );
    }
}

#[test]
fn bare_method_constructor_arguments_keep_kind_and_fixed_prefix() {
    for constructor in ["Maybe", "Either<Int>"] {
        let source = format!("type Higher<F<_>> = | Higher F<Int>\ntrait Inspect<A> {{ fn inspect left: A -> (A -> Bool) }}\ninstance<F<_>> Inspect<Higher<F>> {{ fn inspect left: Higher<F> -> (Higher<F> -> Bool) = \\right -> True }}\npub fn probe -> (Higher<{constructor}> -> Higher<{constructor}> -> Bool) = inspect");
        let core = compile_core_module(CompileInput::new("kind-λ.ssrg", "native/main", &source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence, type_ref, ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        let integer = CoreType::Named {
            name: "Int".into(),
            arguments: vec![],
        };
        let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence[0].evidence else {
            panic!()
        };
        type_arguments[0] = integer.clone();
        let higher = match &mut evidence[0].constraint.arguments[0] {
            CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                arguments[0] = integer;
                evidence[0].constraint.arguments[0].clone()
            }
            _ => panic!(),
        };
        *type_ref = CoreType::Function {
            parameter: Box::new(higher.clone()),
            result: Box::new(CoreType::Function {
                parameter: Box::new(higher),
                result: Box::new(CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }),
            }),
        };
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "accepted a value in a constructor slot"
        );
    }
}

#[test]
fn scoped_method_captures_keep_frames_children_and_effect_lifetimes() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-values.ssrg"),
        "scoped-method-values",
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-values.stdout"),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-defect.ssrg"),
        "scoped-method-defect",
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-defect.stdout"),
        70,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-effects.ssrg"),
        "scoped-method-effects",
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-effects.stdout"),
        0,
    );
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-children.ssrg"),
        "scoped-method-children",
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-children.stdout"),
        70,
    );
}

#[test]
fn parameter_method_values_reject_forged_frames_and_signatures() {
    for (name, constraint, result) in [("eq", "Eq<A>", "Bool"), ("add", "Add<A, A, A>", "A")] {
        let source = format!("fn selected<A> -> (A -> A -> {result}) where {constraint} = {name}\ntype Mark = | Mark\ninstance Eq<Mark> {{ fn eq left: Mark -> (Mark -> Bool) = \\right -> True }}\ninstance Add<Mark, Mark, Mark> {{ fn add left: Mark -> (Mark -> Mark) = \\right -> left }}\npub fn probe -> (Mark -> Mark -> {}) = selected ()", if name == "eq" { "Bool" } else { "Mark" });
        let core = compile_core_module(CompileInput::new(
            "scoped-forged-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        // An explicit specialization reaches the helper without relying on a
        // generalized root binding, which remains outside the native slice.
        emit_rust_module(&core, "probe").unwrap();
        let mut core = core;
        let selected = core
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::selected"))
            .unwrap();
        assert!(
            matches!(selected.body, CoreExpr::Call { ref arguments, .. } if arguments.is_empty())
        );
        for mutation in 0..9 {
            let mut changed = core.clone();
            let selected = changed
                .functions
                .iter_mut()
                .find(|f| f.symbol.ends_with("::selected"))
                .unwrap();
            let CoreExpr::Call {
                evidence,
                trait_dispatch,
                type_ref,
                evidence_argument_index,
                ..
            } = &mut selected.body
            else {
                panic!()
            };
            match mutation {
                0 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 9 },
                1 => evidence[0].constraint.trait_identity = Some("other::Eq".into()),
                2 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                3 => trait_dispatch.as_mut().unwrap().method = "unknown".into(),
                4 => trait_dispatch.as_mut().unwrap().trait_identity = "other::Eq".into(),
                5 => {
                    *type_ref = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    }
                }
                6 => {
                    if let CoreType::Function { result, .. } = type_ref {
                        *result = Box::new(CoreType::Named {
                            name: "String".into(),
                            arguments: vec![],
                        });
                    }
                }
                7 => *evidence_argument_index = Some(0),
                8 => evidence.clear(),
                _ => unreachable!(),
            }
            assert!(
                emit_rust_module(&changed, "probe").is_err(),
                "accepted {name} mutation {mutation}"
            );
        }
    }
}

#[test]
fn declared_method_values_preserve_types_children_and_lifetimes() {
    for (name, source, expected, exit) in [
        (
            "values",
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-values.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-values.stdout"),
            0,
        ),
        (
            "trace",
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-trace.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-trace.stdout"),
            0,
        ),
        (
            "defect",
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-defect.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-defect.stdout"),
            70,
        ),
        (
            "effects",
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-effects.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/trait-method-effects.stdout"
            ),
            0,
        ),
        (
            "children",
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-children.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/trait-method-children.stdout"
            ),
            70,
        ),
    ] {
        execute(source, &format!("declared-{name}"), expected, exit);
    }
}

#[test]
fn abstract_method_values_require_the_independent_declaration_and_exact_frame() {
    use seseragi_lowering::CoreStatement;
    let source = "// 雫🙂\ntrait Pick<A> { fn pick left: A -> right: A -> Bool }\nfn saved<A> left: A -> (A -> Bool) where Pick<A> = { let chosen: A -> A -> Bool = pick; chosen left }\npub fn probe -> Unit = ()";
    let core = compile_core_module(CompileInput::new("declared-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert!(core.instances.is_empty());
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..14 {
        let mut changed = core.clone();
        match mutation {
            0 => changed.trait_methods.clear(),
            1 => changed.trait_methods[0].trait_identity = "counterfeit::Pick".into(),
            2 => changed.trait_methods[0].method = "invented".into(),
            3 => changed.trait_methods.push(changed.trait_methods[0].clone()),
            4 => changed.trait_methods[0].trait_parameters[0].arity = 1,
            5 => changed.trait_methods[0].trait_parameters.clear(),
            6 => changed.trait_methods[0].signature.parameters[0] = CoreType::Hole,
            7 => {
                changed.trait_methods[0].signature.result = CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                }
            }
            8 => {
                let parameter = changed.trait_methods[0].trait_parameters[0].clone();
                changed.trait_methods[0]
                    .signature
                    .type_parameters
                    .push(parameter);
            }
            9 => changed.trait_methods[0]
                .signature
                .argument_types
                .push(CoreType::Hole),
            _ => {
                let function = changed
                    .functions
                    .iter_mut()
                    .find(|f| f.symbol.ends_with("::saved"))
                    .unwrap();
                let constraint = function.constraints[0].clone();
                let CoreExpr::Sequence { statements, .. } = &mut function.body else {
                    panic!("body")
                };
                let CoreStatement::PureLet { value, .. } = &mut statements[0] else {
                    panic!("binding")
                };
                let CoreExpr::Call {
                    evidence,
                    trait_dispatch,
                    type_ref,
                    ..
                } = value
                else {
                    panic!("selected value")
                };
                match mutation {
                    10 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
                    11 => trait_dispatch.as_mut().unwrap().method = "invented".into(),
                    12 => *type_ref = CoreType::Hole,
                    _ => changed.trait_methods[0].constraints.push(constraint),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(
            error.origin.unwrap().source,
            "declared-λ.ssrg",
            "case {mutation}"
        );
    }
}

#[test]
fn constructor_trait_parameter_methods_preserve_the_selected_first_stage() {
    for initializer in [
        "let chosen: F<A> -> F<A> -> Bool = pick",
        "let chosen = pick",
    ] {
        execute(
            &format!(
                "trait Pick<F<_>> {{ fn pick<A> left: F<A> -> right: F<A> -> Bool }}\n\
                 instance Pick<Maybe> {{ fn pick<A> left: Maybe<A> -> (Maybe<A> -> Bool) = {{\n\
                   let bad = 9007199254740991 + 1; \\right -> True\n\
                 }} }}\n\
                 fn selected<F<_>, A> left: F<A> -> (F<A> -> Bool) where Pick<F> = {{\n\
                   {initializer}; chosen left\n\
                 }}\n\
                 pub effect fn main = do {{ let saved = selected (Just 1); println \"unreachable\" }}"
            ),
            if initializer.contains(':') { "constructor-scoped" } else { "constructor-generalized" },
            "",
            70,
        );
    }
}

#[test]
fn method_owned_constructor_binders_preserve_the_selected_first_stage() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-defect.ssrg");
    for (name, source) in [
        ("method-constructor-captured", source.to_string()),
        (
            "method-constructor-generalized",
            source.replace(
                "let chosen: G<F<A>> -> G<F<A>> -> Bool = pick",
                "let chosen = pick",
            ),
        ),
        (
            "method-constructor-child-stage",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/method-constructor-children.ssrg"
            )
            .to_string(),
        ),
    ] {
        execute(&source, name, "", 70);
    }
}

#[test]
fn method_owned_constructors_keep_prefixes_dictionaries_and_managed_lifetimes() {
    for (name, source, expected) in [
        (
            "values",
            include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-values.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-values.stdout"),
        ),
        (
            "effects",
            include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-effects.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-effects.stdout"),
        ),
        (
            "prefix-kind",
            include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-prefix-kind.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/method-constructor-prefix-kind.stdout"),
        ),
    ] {
        execute(source, &format!("method-constructor-{name}"), expected, 0);
    }
}

#[test]
fn method_owned_constructor_values_reject_forged_binders_kinds_and_frames() {
    use seseragi_lowering::CoreStatement;
    for initializer in [
        "let chosen: G<F<A>> -> G<F<A>> -> Bool = pick",
        "let chosen = pick",
    ] {
        let source = format!("// 雫🙂\ntrait Pick<F<_>> {{ fn pick<G<_>, A> left: G<F<A>> -> right: G<F<A>> -> Bool }}\nfn saved<F<_>, G<_>, A> left: G<F<A>> -> (G<F<A>> -> Bool) where Pick<F> = {{ {initializer}; chosen left }}\npub fn probe -> Unit = ()");
        let core = compile_core_module(CompileInput::new(
            "method-kind-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..21 {
            let mut changed = core.clone();
            let declaration = &mut changed.trait_methods[0];
            match mutation {
                0 => declaration.signature.type_parameters[0].arity = 0,
                1 => declaration.signature.type_parameters[0].arity = 2,
                2 => declaration.signature.type_parameters[1].arity = 1,
                3 => declaration.signature.type_parameters[0].name = "F".into(),
                4 => declaration.signature.type_parameters[1].name = "G".into(),
                5 => declaration.signature.type_parameters.clear(),
                6 => declaration.trait_parameters[0].arity = 0,
                7 => declaration.signature.parameters[0] = CoreType::Hole,
                8 => {
                    declaration.signature.result = CoreType::Named {
                        name: "G".into(),
                        arguments: vec![],
                    }
                }
                9 => declaration.constraints.push(
                    core.functions
                        .iter()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap()
                        .constraints[0]
                        .clone(),
                ),
                10 => changed.trait_methods.clear(),
                20 => {
                    let mut unused = declaration.signature.type_parameters[0].clone();
                    unused.name = "UnusedConstructor".into();
                    declaration.signature.type_parameters.push(unused);
                }
                _ => {
                    let function = changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap();
                    if mutation == 11 {
                        function.type_parameters[1].arity = 0;
                    } else if mutation == 12 {
                        function.type_parameters[1].arity = 2;
                    } else {
                        let CoreExpr::Sequence { statements, .. } = &mut function.body else {
                            panic!("body")
                        };
                        let value = match &mut statements[0] {
                            CoreStatement::PureLet { value, .. } => value,
                            CoreStatement::LocalFunction { body, .. } => body,
                            _ => panic!("alias"),
                        };
                        let CoreExpr::Call {
                            evidence,
                            trait_dispatch,
                            type_ref,
                            deferred_evidence_parameters,
                            evidence_argument_index,
                            ..
                        } = value
                        else {
                            panic!("method")
                        };
                        match mutation {
                            13 => {
                                evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 }
                            }
                            14 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                            15 => trait_dispatch.as_mut().unwrap().method = "other".into(),
                            16 => {
                                trait_dispatch.as_mut().unwrap().trait_identity =
                                    "counterfeit::trait(Pick)".into()
                            }
                            17 => *type_ref = CoreType::Hole,
                            18 => *evidence_argument_index = Some(1),
                            _ => deferred_evidence_parameters.push(CoreType::Hole),
                        }
                    }
                }
            }
            let error = emit_rust_module(&changed, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
            assert_eq!(
                error.origin.unwrap().source,
                "method-kind-λ.ssrg",
                "mutation {mutation}"
            );
        }
    }
}

#[test]
fn constructor_method_values_require_canonical_kinds_slots_and_evidence_frames() {
    use seseragi_lowering::CoreStatement;
    for initializer in [
        "let chosen: F<A> -> F<A> -> Bool = pick",
        "let chosen = pick",
    ] {
        let source = format!("// 雫🙂\ntrait Pick<F<_>> {{ fn pick<A> left: F<A> -> right: F<A> -> Bool }}\nfn saved<F<_>, A> left: F<A> -> (F<A> -> Bool) where Pick<F> = {{ {initializer}; chosen left }}\npub fn probe -> Unit = ()");
        let core = compile_core_module(CompileInput::new(
            "constructor-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..24 {
            let mut changed = core.clone();
            match mutation {
                0 => changed.trait_methods[0].trait_parameters[0].arity = 0,
                1 => changed.trait_methods[0].trait_parameters[0].arity = 2,
                2 => changed.trait_methods[0].signature.type_parameters[0].arity = 1,
                3 => changed.trait_methods[0].signature.type_parameters[0].name = "F".into(),
                4 => changed.trait_methods[0].trait_parameters.clear(),
                5 => changed.trait_methods[0].trait_identity = "counterfeit::Pick".into(),
                6 => changed.trait_methods[0].method = "invented".into(),
                7 => changed.trait_methods[0].constraints.push(
                    changed
                        .functions
                        .iter()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap()
                        .constraints[0]
                        .clone(),
                ),
                8 => changed.trait_methods[0].signature.parameters[0] = CoreType::Hole,
                9 => changed.trait_methods[0].signature.parameters.clear(),
                10 => {
                    changed.trait_methods[0].signature.result = CoreType::Named {
                        name: "String".into(),
                        arguments: vec![],
                    }
                }
                11 => changed.trait_methods[0]
                    .signature
                    .argument_types
                    .push(CoreType::Hole),
                12 => changed.trait_methods.clear(),
                13 => changed.trait_methods.push(changed.trait_methods[0].clone()),
                14 => changed.trait_methods[0].signature.type_parameters.clear(),
                _ => {
                    let function = changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap();
                    let CoreExpr::Sequence { statements, .. } = &mut function.body else {
                        panic!("body")
                    };
                    let value = match &mut statements[0] {
                        CoreStatement::PureLet { value, .. } => value,
                        CoreStatement::LocalFunction { body, .. } => body,
                        _ => panic!("method alias"),
                    };
                    let CoreExpr::Call {
                        evidence,
                        trait_dispatch,
                        type_ref,
                        deferred_evidence_parameters,
                        evidence_argument_index,
                        ..
                    } = value
                    else {
                        panic!("selected method")
                    };
                    match mutation {
                        15 => evidence[0].constraint.arguments[0] = CoreType::Hole,
                        16 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
                        17 => trait_dispatch.as_mut().unwrap().method = "invented".into(),
                        18 => {
                            trait_dispatch.as_mut().unwrap().trait_identity =
                                "counterfeit::Pick".into()
                        }
                        19 => *type_ref = CoreType::Hole,
                        20 => deferred_evidence_parameters.push(CoreType::Hole),
                        21 => *evidence_argument_index = Some(0),
                        22 => {
                            evidence[0].constraint.trait_identity = Some("counterfeit::Pick".into())
                        }
                        _ => evidence[0].constraint.arguments.clear(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "probe").unwrap_err();
            assert_eq!(
                error.code, "SES-RUST-0001",
                "{initializer} mutation {mutation}"
            );
            assert_eq!(error.origin.unwrap().source, "constructor-λ.ssrg");
        }
    }
}

#[test]
fn constructor_method_values_preserve_fixed_prefixes_children_and_lifetimes() {
    for (name, source, expected, exit) in [
        (
            "values",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/constructor-method-values.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/constructor-method-values.stdout"
            ),
            0,
        ),
        (
            "defect",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/constructor-method-defect.ssrg"
            ),
            "",
            70,
        ),
        (
            "effects",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/constructor-method-effects.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/constructor-method-effects.stdout"
            ),
            0,
        ),
        (
            "children",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/constructor-method-children.ssrg"
            ),
            "",
            70,
        ),
    ] {
        execute(source, &format!("constructor-{name}"), expected, exit);
    }
}

#[test]
fn constructor_module_aliases_keep_the_dictionary_factory_and_method_stages() {
    execute(
        "trait Pick<F<_>> { fn pick<A> left: F<A> -> right: F<A> -> Bool }\n\
         instance Pick<Maybe> { fn pick<A> left: Maybe<A> -> (Maybe<A> -> Bool) = { let bad = 9007199254740991 + 1; \\right -> True } }\n\
         let selectedMethod = pick\nlet copiedMethod = selectedMethod\n\
         fn selected<F<_>, A> left: F<A> -> (F<A> -> Bool) where Pick<F> = {\n\
           let chosen: F<A> -> F<A> -> Bool = copiedMethod; chosen left\n\
         }\n\
         pub effect fn main = do { let saved = selected (Just 1); println \"unreachable\" }",
        "constructor-global-alias", "", 70,
    );
}

#[test]
fn constructor_module_factories_reject_forged_kinds_frames_and_alias_cycles() {
    let source = "// 雫🙂\ntrait Pick<F<_>> { fn pick<A> left: F<A> -> right: F<A> -> Bool }\ninstance Pick<Maybe> { fn pick<A> left: Maybe<A> -> (Maybe<A> -> Bool) = \\right -> True }\nlet selected = pick\nlet copied = selected\npub fn probe -> Bool = copied (Just 1) (Just 2)";
    let core = compile_core_module(CompileInput::new(
        "constructor-global-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..8 {
        let mut changed = core.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::selected"))
            .unwrap();
        let CoreExpr::Call {
            callee,
            evidence,
            trait_dispatch,
            type_ref,
            ..
        } = &mut function.body
        else {
            panic!("factory")
        };
        match mutation {
            0 => function.type_parameters[0].arity = 0,
            1 => function.type_parameters[0].arity = 2,
            2 => function.constraints[0].arguments.clear(),
            3 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 },
            4 => trait_dispatch.as_mut().unwrap().method = "invented".into(),
            5 => *type_ref = CoreType::Hole,
            6 => {
                *callee = "native/main::copied".into();
                *trait_dispatch = None;
            }
            _ => function.is_effect = true,
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "constructor-global-λ.ssrg");
    }
}

#[test]
fn fixed_constructor_prefixes_follow_declared_slot_kinds() {
    let source = "trait Pick<F<_>> { fn pick<A> left: F<A> -> right: F<A> -> Bool }\n\
        type Container<G<_>, A> = | Container G<A>\n\
        instance Pick<Container<Maybe>> { fn pick<A> left: Container<Maybe, A> -> (Container<Maybe, A> -> Bool) = \\right -> True }\n\
        fn selected<G<_>, A> left: Container<G, A> -> (Container<G, A> -> Bool) where Pick<Container<G>> = {\n\
          let chosen: Container<G, A> -> Container<G, A> -> Bool = pick; chosen left\n\
        }\n\
        pub effect fn main = do {\n\
          let saved = selected<Maybe, Int> (Container<Maybe, Int> (Just 1))\n\
          println (show (saved (Container<Maybe, Int> (Just 2))))\n\
        }";
    execute(source, "constructor-kind-prefix", "True\n", 0);
}

#[test]
fn constructor_prefix_slot_kinds_reject_forged_adt_and_struct_declarations() {
    for declaration in [
        "type Holder<G<_>, A> = | Holder G<A>",
        "struct Holder<G<_>, A> { value: G<A> }",
    ] {
        let source = format!("// 雫🙂\ntrait Pick<F<_>> {{ fn pick<A> left: F<A> -> right: F<A> -> Bool }}\n{declaration}\nfn saved<G<_>, A> left: Holder<G, A> -> (Holder<G, A> -> Bool) where Pick<Holder<G>> = {{ let chosen: Holder<G, A> -> Holder<G, A> -> Bool = pick; chosen left }}\npub fn probe -> Unit = ()");
        let core = compile_core_module(CompileInput::new(
            "slot-kinds-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..10 {
            let mut changed = core.clone();
            let parameters = if let Some(adt) = changed.adts.first_mut() {
                &mut adt.type_parameters
            } else {
                &mut changed.structs[0].type_parameters
            };
            match mutation {
                0 => parameters[0].arity = 0,
                1 => parameters[0].arity = 2,
                2 => {
                    parameters.swap(0, 1);
                }
                3 => {
                    parameters.pop();
                }
                4 => {
                    let mut next = parameters[0].clone();
                    next.name = "Unfilled".into();
                    parameters.push(next);
                }
                5 => changed.trait_methods[0].trait_parameters[0].arity = 2,
                6 => {
                    changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap()
                        .type_parameters[0]
                        .arity = 0
                }
                7 => {
                    changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap()
                        .type_parameters[0]
                        .arity = 2
                }
                8 => parameters[1].arity = 1,
                _ => {
                    parameters.clear();
                }
            }
            let Err(error) = emit_rust_module(&changed, "probe") else {
                panic!("accepted {declaration} mutation {mutation}");
            };
            assert_eq!(
                error.code, "SES-RUST-0001",
                "{declaration} mutation {mutation}"
            );
            assert!(error.origin.is_some());
        }
    }
}

#[test]
fn method_owned_constructor_prefixes_reject_forged_slot_kinds() {
    for declaration in [
        "type Holder<G<_>, A> = | Holder G<A>",
        "struct Holder<G<_>, A> { value: G<A> }",
    ] {
        let source = format!("// 雫🙂\ntrait Pick<A> {{ fn pick<F<_>> left: F<A> -> right: F<A> -> Bool }}\n{declaration}\nfn saved<G<_>, A> left: Holder<G, A> -> (Holder<G, A> -> Bool) where Pick<A> = {{ let chosen: Holder<G, A> -> Holder<G, A> -> Bool = pick; chosen left }}\npub fn probe -> Unit = ()");
        let core = compile_core_module(CompileInput::new(
            "slot-kinds-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..10 {
            let mut changed = core.clone();
            let parameters = if let Some(adt) = changed.adts.first_mut() {
                &mut adt.type_parameters
            } else {
                &mut changed.structs[0].type_parameters
            };
            match mutation {
                0 => parameters[0].arity = 0,
                1 => parameters[0].arity = 2,
                2 => {
                    parameters.swap(0, 1);
                }
                3 => {
                    parameters.pop();
                }
                4 => {
                    let mut next = parameters[0].clone();
                    next.name = "Unfilled".into();
                    parameters.push(next);
                }
                5 => changed.trait_methods[0].trait_parameters[0].arity = 2,
                6 => {
                    changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap()
                        .type_parameters[0]
                        .arity = 0
                }
                7 => {
                    changed
                        .functions
                        .iter_mut()
                        .find(|f| f.symbol.ends_with("::saved"))
                        .unwrap()
                        .type_parameters[0]
                        .arity = 2
                }
                8 => parameters[1].arity = 1,
                _ => {
                    parameters.clear();
                }
            }
            let Err(error) = emit_rust_module(&changed, "probe") else {
                panic!("accepted {declaration} mutation {mutation}");
            };
            assert_eq!(
                error.code, "SES-RUST-0001",
                "{declaration} mutation {mutation}"
            );
            assert!(error.origin.is_some());
        }
    }
}

#[test]
fn method_owned_constructors_cannot_hide_higher_order_slots_in_value_binders() {
    use seseragi_lowering::CoreStatement;
    for (declaration, name, arguments) in [
        (
            "type Holder<H<_>, A> = | Holder H<A>",
            "Holder",
            vec![
                CoreType::Named {
                    name: "Maybe".into(),
                    arguments: vec![],
                },
                CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                },
            ],
        ),
        (
            "type Holder<P, H<_>, A> = | Holder (P, H<A>)",
            "Holder",
            vec![
                CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                },
                CoreType::Named {
                    name: "Maybe".into(),
                    arguments: vec![],
                },
                CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                },
            ],
        ),
    ] {
        let source = format!("// 雫🙂\ntrait Select<Tag> {{ fn select<G<_, _>, A, B> left: G<A, B> -> right: G<A, B> -> Bool }}\n{declaration}\nfn unused<Tag, G<_, _>, A, B> left: G<A, B> -> Unit where Select<Tag> = {{ let chosen: G<A, B> -> G<A, B> -> Bool = select; () }}\npub fn probe -> Unit = ()");
        let core = compile_core_module(CompileInput::new(
            "higher-slot-λ.ssrg",
            "native/main",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        let mut changed = core;
        let function = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::unused"))
            .unwrap();
        let CoreExpr::Sequence { statements, .. } = &mut function.body else {
            panic!("body")
        };
        let CoreStatement::PureLet { value, .. } = &mut statements[0] else {
            panic!("alias")
        };
        let CoreExpr::Call { type_ref, .. } = value else {
            panic!("method")
        };
        let argument = CoreType::Named {
            name: name.into(),
            arguments,
        };
        *type_ref = CoreType::Function {
            parameter: Box::new(argument.clone()),
            result: Box::new(CoreType::Function {
                parameter: Box::new(argument),
                result: Box::new(CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }),
            }),
        };
        let Err(error) = emit_rust_module(&changed, "probe") else {
            panic!("accepted a higher-order slot in a value-kind method binder")
        };
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "higher-slot-λ.ssrg");
    }
}

#[test]
fn closed_method_values_reject_higher_order_method_constructor_bindings() {
    let source = "// 雫🙂\ntrait Pick<Tag> { fn pick<G<_, _>, A, B> left: G<A, B> -> right: G<A, B> -> Bool }\ntype Pair<A, B> = | Pair (A, B)\ntype Holder<H<_>, A> = | Holder H<A>\ninstance Pick<Int> { fn pick<G<_, _>, A, B> left: G<A, B> -> (G<A, B> -> Bool) = \\right -> True }\npub fn probe -> (Pair<Int, String> -> Pair<Int, String> -> Bool) = pick<Int, Pair, Int, String>";
    let mut core = compile_core_module(CompileInput::new(
        "higher-local-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let CoreExpr::Call { type_ref, .. } = &mut core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::probe"))
        .unwrap()
        .body
    else {
        panic!("method")
    };
    let argument = CoreType::Named {
        name: "Holder".into(),
        arguments: vec![
            CoreType::Named {
                name: "Maybe".into(),
                arguments: vec![],
            },
            CoreType::Named {
                name: "Int".into(),
                arguments: vec![],
            },
        ],
    };
    *type_ref = CoreType::Function {
        parameter: Box::new(argument.clone()),
        result: Box::new(CoreType::Function {
            parameter: Box::new(argument),
            result: Box::new(CoreType::Named {
                name: "Bool".into(),
                arguments: vec![],
            }),
        }),
    };
    let Err(error) = emit_rust_module(&core, "probe") else {
        panic!("accepted a higher-order closed method constructor")
    };
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "higher-local-λ.ssrg");
}

#[test]
fn closed_method_values_reject_higher_order_instance_constructor_arguments() {
    let source = "// 雫🙂\ntrait Inspect<A> { fn inspect left: Int -> right: Int -> Bool }\ntype Pair<A, B> = | Pair (A, B)\ntype Outer<F<_, _>> = | Outer Unit\ntype Holder<H<_>, A> = | Holder H<A>\ninstance<F<_, _>> Inspect<Outer<F>> { fn inspect left: Int -> (Int -> Bool) = \\right -> True }\npub fn probe -> (Int -> Int -> Bool) = inspect<Outer<Pair>>";
    let mut core = compile_core_module(CompileInput::new(
        "higher-instance-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let CoreExpr::Call { evidence, .. } = &mut core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::probe"))
        .unwrap()
        .body
    else {
        panic!("method")
    };
    let constructor = CoreType::Named {
        name: "Holder".into(),
        arguments: vec![],
    };
    evidence[0].constraint.arguments[0] = CoreType::Named {
        name: "Outer".into(),
        arguments: vec![constructor.clone()],
    };
    let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence[0].evidence else {
        panic!("instance")
    };
    type_arguments[0] = constructor;
    let Err(error) = emit_rust_module(&core, "probe") else {
        panic!("accepted a higher-order instance constructor")
    };
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "higher-instance-λ.ssrg");
}

#[test]
fn closed_method_values_keep_cold_effect_types_and_returned_values() {
    for (name, source) in [
        (
            "closed-value-effect-kind",
            r#"import * as effects from "std/effect"
trait Keep<Tag> { fn keep<A> left: A -> right: A -> A }
instance Keep<Int> { fn keep<A> left: A -> (A -> A) = \right -> left }
effect fn cold = do { println "cold"; effects.succeed 42 }
pub effect fn main = do {
 let chosen = keep<Int, Effect<{console: Console}, ConsoleError, Int>>
 let saved = chosen (cold ())
 println "created"
 let plan: Effect<{console: Console}, ConsoleError, Int> = saved (cold ())
 value <- plan
 println (show value)
}
"#,
        ),
        (
            "closed-constructor-effect-kind",
            r#"import * as effects from "std/effect"
trait Keep<Tag> { fn keep<G<_>, A> left: G<A> -> right: G<A> -> G<A> }
instance Keep<Int> { fn keep<G<_>, A> left: G<A> -> (G<A> -> G<A>) = \right -> left }
effect fn cold = do { println "cold"; effects.succeed 42 }
pub effect fn main = do {
 let chosen = keep<Int, Maybe, Effect<{console: Console}, ConsoleError, Int>>
 let saved = chosen (Just (cold ()))
 println "created"
 let plan: Effect<{console: Console}, ConsoleError, Int> = match (saved (Just (cold ()))) { Just held -> held; Nothing -> cold () }
 value <- plan
 println (show value)
}
"#,
        ),
    ] {
        execute(source, name, "created\ncold\n42\n", 0);
    }
}
