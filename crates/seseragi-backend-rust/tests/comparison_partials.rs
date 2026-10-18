use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};

#[test]
fn canonical_int_comparison_partial_is_a_bool_callable() {
    let source = "pub fn probe -> Bool = { let less: Int -> Bool = (<) 1; less 2 }";
    let core = compile_core_module(CompileInput::new(
        "comparison-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
}

#[test]
fn comparison_refuses_counterfeit_ord_owners_types_children_and_source_stages() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreStatement, CoreType};
    let source = "// 雫🙂\npub fn probe -> Bool = { let less: Int -> Bool = (<) 1; less 2 }";
    let core = compile_core_module(CompileInput::new(
        "comparison-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    for mutation in 0..16 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let CoreStatement::PureLet {
            value:
                CoreExpr::Call {
                    evidence,
                    arguments,
                    trait_dispatch,
                    type_ref,
                    deferred_evidence_parameters,
                    deferred_evidence_type_constructor_parameters,
                    evidence_argument_index,
                    ..
                },
            ..
        } = &mut statements[0]
        else {
            panic!()
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Ord".into()),
            1 => evidence[0].constraint.name = "Eq".into(),
            2 => evidence[0].constraint.arguments.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
            5 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            6 => {
                let child = evidence[0].clone();
                let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(child);
            }
            7 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "std/char::Ord".into();
            }
            8 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Ord".into(),
            9 => trait_dispatch.as_mut().unwrap().method = "eq".into(),
            10 => *type_ref = CoreType::Hole,
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => deferred_evidence_type_constructor_parameters.push("F".into()),
            13 => *evidence_argument_index = Some(0),
            14 => {
                arguments.push(arguments[0].clone());
                arguments.push(arguments[0].clone());
            }
            _ => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "comparison-λ.ssrg");
    }
}

fn execute(source: &str, name: &str, expected: &str, exit: i32) {
    use std::{fs, process::Command};
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let core = compile_core_module(CompileInput::new(
        "comparison-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let dir = Scratch(
        std::env::temp_dir().join(format!("seseragi-comparison-{}-{name}", std::process::id())),
    );
    fs::create_dir(&dir.0).unwrap();
    let file = dir.0.join("program.rs");
    fs::write(&file, artifact.source).unwrap();
    let exe = dir.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&exe)
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = Command::new(exe)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
}

#[test]
fn saved_comparisons_keep_selected_ord_bool_conversion_generic_calls_and_runtime_values() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/comparison-partials.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/comparison-partials.stdout"),
        0,
    );
}

#[test]
fn runtime_prefix_closures_keep_cold_effects_ref_cycles_and_finalizers() {
    execute(
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/comparison-partial-effects.ssrg"
        ),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/comparison-partial-effects.stdout"
        ),
        0,
    );
}

#[test]
fn selected_compare_first_stage_defects_before_the_unused_right_argument() {
    execute("type Mark = | Mark Int\ninstance Eq<Mark> { fn eq left: Mark -> right: Mark -> Bool = True }\ninstance Ord<Mark> { fn compare left: Mark -> (Mark -> Ordering) = { let bad = 9007199254740991 + 1; \\right -> Less } }\npub effect fn main = do { let saved: Mark -> Bool = (<) (Mark 1); println \"unreachable\" }", "first-stage", "",70);
}

#[test]
fn saved_operator_reference_executes_its_first_method_stage_once_before_the_marker() {
    use std::{fs, process::Command};
    let source = r#"
type Mark = | Mark Int
instance Eq<Mark> { fn eq left: Mark -> right: Mark -> Bool = True }
instance Ord<Mark> { fn compare left: Mark -> (Mark -> Ordering) = { let captured = firstStage left; \right -> Less } }
fn leftValue -> Mark = Mark 1
fn firstStage value: Mark -> Mark = value
fn marker -> Unit = ()
fn rightValue -> Mark = Mark 2
pub fn probe -> Bool = {
  let operator: Mark -> Mark -> Bool = (<)
  let saved = operator (leftValue ())
  let ignored = marker ()
  let first = saved (rightValue ())
  let repeated = saved (rightValue ())
  (leftValue ()) < (rightValue ())
}
"#;
    let core = compile_core_module(CompileInput::new("trace.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    let id = |name: &str| {
        format!(
            "v_{}",
            name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
        )
    };
    for name in ["leftValue", "firstStage", "marker", "rightValue"] {
        // Instrument this test copy's source helpers, keeping the ordinary
        // generated comparison expression and its capture graph unchanged.
        let start = generated
            .find(&format!(
                "fn {}_direct(",
                id(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(
            body,
            &format!(" TRACE.with(|t| t.borrow_mut().push({name:?})); "),
        );
    }
    generated.push_str(&format!(r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let value = {}(vec![]).apply(V::Unit); assert!(value.boolean());
 TRACE.with(|t| assert_eq!(&*t.borrow(), &["leftValue","firstStage","marker","rightValue","rightValue","leftValue","firstStage","rightValue"]));
 drop(value); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get),0);
}}
"#,id(&artifact.metadata.entry)));
    let d = std::env::temp_dir().join(format!("seseragi-comparison-trace-{}", std::process::id()));
    fs::create_dir(&d).unwrap();
    let input = d.join("program.rs");
    fs::write(&input, generated).unwrap();
    let exe = d.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&exe)
        .arg(input)
        .output()
        .unwrap();
    let output = if built.status.success() {
        Some(Command::new(exe).env("PATH", "").output().unwrap())
    } else {
        None
    };
    fs::remove_dir_all(d).unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = output.unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn selected_ord_requires_the_proved_eq_parent_and_checked_method_type() {
    use seseragi_lowering::{CoreInstanceImplementation, CoreType};
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/comparison-partials.ssrg");
    let core = compile_core_module(CompileInput::new(
        "ord-parent-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    for mutation in 0..8 {
        let mut changed = core.clone();
        let instance = changed
            .instances
            .iter_mut()
            .find(|i| i.trait_name == "Ord")
            .unwrap();
        match mutation {
            0 => instance.supertrait_count = 2,
            1 => instance.constraints[0].trait_identity = Some("fake::Eq".into()),
            2 => {
                instance.constraints[0].arguments = vec![CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                }]
            }
            3 => instance.identity = "fake::Ord<Mark>".into(),
            4 => instance.constraints.clear(),
            5 => {
                let CoreInstanceImplementation::UserDefined { methods } =
                    &mut instance.implementation
                else {
                    panic!()
                };
                methods[0].parameters.clear();
            }
            6 => {
                let CoreInstanceImplementation::UserDefined { methods } =
                    &mut instance.implementation
                else {
                    panic!()
                };
                methods[0].name = "eq".into();
            }
            _ => {
                let CoreInstanceImplementation::UserDefined { methods } =
                    &mut instance.implementation
                else {
                    panic!()
                };
                methods.push(methods[0].clone());
            }
        }
        assert!(
            emit_rust_module(&changed, "probe").is_err(),
            "case {mutation}"
        );
    }
}

#[test]
fn all_int_comparison_sections_preserve_equal_negative_and_extreme_operands() {
    let mut expressions = vec![];
    let mut expected = vec![];
    for left in [-9007199254740991_i64, 0, 9007199254740991_i64] {
        for right in [-9007199254740991_i64, 0, 9007199254740991_i64] {
            for (operator, result) in [
                ("<", left < right),
                ("<=", left <= right),
                (">", left > right),
                (">=", left >= right),
            ] {
                expressions.push(format!("(({operator}) ({left})) ({right})"));
                expected.push(if result { "True" } else { "False" });
            }
        }
    }
    let source = format!(
        "pub effect fn main = println (show [{}])",
        expressions.join(", ")
    );
    execute(
        &source,
        "int-boundaries",
        &format!("[{}]\n", expected.join(", ")),
        0,
    );
}

#[test]
fn primitive_ord_does_not_replace_scoped_binders_named_int_or_char() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence};
    for (binder, value, identity) in [
        ("Int", "3", "std/int::Ord"),
        ("Char", "'λ'", "std/char::Ord"),
    ] {
        let source=format!("fn generic<{binder}> value: {binder} -> Bool where Ord<{binder}> = (<) value value\npub fn probe -> Bool = generic {value}");
        let core = compile_core_module(CompileInput::new("binder-λ.ssrg", "native/main", &source))
            .unwrap()
            .core_ir;
        assert!(emit_rust_module(&core, "probe").is_ok());
        let mut changed = core.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::generic"))
            .unwrap();
        let CoreExpr::Call { evidence, .. } = &mut function.body else {
            panic!()
        };
        evidence[0].evidence = CoreInstanceEvidence::Standard {
            identity: identity.into(),
            type_arguments: vec![],
            evidence_arguments: vec![],
        };
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "binder-λ.ssrg");
    }
}

#[test]
fn ord_uses_the_selected_eq_parent_even_for_identical_payloads() {
    execute(
        "type Mark = | Mark Int\ninstance Eq<Mark> { fn eq left: Mark -> right: Mark -> Bool = False }\ninstance Ord<Mark> { fn compare left: Mark -> (Mark -> Ordering) = \\right -> if eq left right then Equal else Greater }\npub effect fn main = println (show (((>) (Mark 1)) (Mark 1)))",
        "parent-policy",
        "True\n",
        0,
    );
}
