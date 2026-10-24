use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
use std::{fs, process::Command};

fn execute(source: &str, name: &str, expected: &str, code: i32) {
    execute_entry(source, "probe", name, expected, code);
}

fn execute_entry(source: &str, entry: &str, name: &str, expected: &str, code: i32) {
    execute_trace(source, entry, name, expected, code, None);
}

fn execute_trace(
    source: &str,
    entry: &str,
    name: &str,
    expected: &str,
    code: i32,
    trace: Option<&str>,
) {
    let checked = compile_core_module(CompileInput::new("int-雪🙂.ssrg", "native/main", source))
        .expect("valid source division contract");
    let artifact = emit_rust_module(&checked.core_ir, entry).expect("native Int division");
    execute_rust(&artifact.source, name, expected, code, trace);
}

fn execute_rust(source: &str, name: &str, expected: &str, code: i32, trace: Option<&str>) {
    let directory = std::env::temp_dir().join(format!(
        "seseragi-int-division-{name}-{}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    let input = directory.join("program.rs");
    let executable = directory.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, source).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&executable)
        .arg(&input)
        .output()
        .unwrap();
    let result = built.status.success().then(|| {
        Command::new(&executable)
            .current_dir(&directory)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap()
    });
    let observed_trace = trace.map(|_| fs::read_to_string(directory.join("trace.txt")));
    fs::remove_dir_all(directory).unwrap();
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
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    if let Some(expected_trace) = trace {
        assert_eq!(observed_trace.unwrap().unwrap(), expected_trace);
        assert!(String::from_utf8_lossy(&result.stderr).contains("Int division by zero"));
    }
}

#[test]
fn native_int_division_accepts_checked_canonical_division() {
    execute("pub fn probe -> Int = 7 / 2\n", "simple", "3\n", 0);
}

#[test]
fn native_division_does_not_expand_program_wasm_arithmetic_admission() {
    let source = "pub fn quotient n: Int -> Int = n / 2\npub fn probe -> Int = quotient 7\n";
    let core = compile_core_module(CompileInput::new(
        "div-target-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").expect("ordinary native target supports checked Int Div");
    let error =
        seseragi_backend_rust::emit_program_wasm(&[&core], "native/main", &["quotient".into()])
            .expect_err("program WASM arithmetic still requires its own proof");
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "div-target-雪.ssrg");
}

#[test]
fn canonical_division_keeps_boundaries_methods_partials_and_generic_evidence() {
    execute_entry(
        include_str!("../../../examples/spec/fixtures/rust-backend/int-division.ssrg"),
        "main",
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/int-division.stdout"),
        0,
    );
    execute_entry(
        include_str!("../../../examples/spec/fixtures/rust-backend/int-division-effects.ssrg"),
        "main",
        "effects",
        include_str!("../../../examples/spec/fixtures/rust-backend/int-division-effects.stdout"),
        0,
    );
}

#[test]
fn zero_division_is_a_defect_and_a_partial_does_not_run_early() {
    execute("pub effect fn probe = do { let saved: Int -> Int = div 125; println \"saved\"; println (show (saved 0)) }", "zero", "saved\n", 70);
    execute(
        "pub effect fn probe = do { let saved: Int -> Int = div (7 / 0); println \"unreachable\" }",
        "prefix-zero",
        "",
        70,
    );
}

#[test]
fn zero_division_awaits_the_resource_finalizer_before_exit() {
    execute_trace(
        include_str!("../../../examples/spec/fixtures/rust-backend/int-division-defect.ssrg"),
        "main",
        "release-zero",
        "",
        70,
        Some("acquire\nsaved\nrelease\n"),
    );
}

#[test]
fn division_does_not_substitute_a_lexical_binder_named_int() {
    let source = "fn generic<A> n: A -> (A -> A) where Div<A,A,A> = div n\npub fn probe -> Int = { let saved = generic 7; saved 2 }";
    let mut core = compile_core_module(CompileInput::new(
        "div-binder-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let function = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    fn rename(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(name) if name == "A" => *name = "Int".into(),
            serde_json::Value::Object(fields) => fields.values_mut().for_each(rename),
            serde_json::Value::Array(values) => values.iter_mut().for_each(rename),
            _ => (),
        }
    }
    let mut encoded = serde_json::to_value(&*function).unwrap();
    rename(&mut encoded);
    *function = serde_json::from_value(encoded).unwrap();
    emit_rust_module(&core, "probe").expect("alpha-renamed lexical dictionary remains valid");
    let function = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &mut function.body else {
        panic!("method")
    };
    evidence[0].evidence = CoreInstanceEvidence::Standard {
        identity: "std/int::Div".into(),
        type_arguments: vec![],
        evidence_arguments: vec![],
    };
    let error =
        emit_rust_module(&core, "probe").expect_err("lexical Int is not the canonical primitive");
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "div-binder-雪.ssrg");
}

#[test]
fn primitive_division_refuses_forged_binary_evidence_and_operand_types() {
    let core = compile_core_module(CompileInput::new(
        "div-雪🙂.ssrg",
        "native/main",
        "pub fn probe -> Int = 7 / 2\n",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..12 {
        let mut changed = core.clone();
        let CoreExpr::Binary {
            evidence,
            left,
            right,
            type_ref,
            origin,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("checked binary")
        };
        let boolean = CoreType::Named {
            name: "Bool".into(),
            arguments: vec![],
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Div".into()),
            1 => evidence[0].constraint.name = "Mul".into(),
            2 => {
                evidence[0].constraint.arguments.pop();
            }
            3 => evidence[0].constraint.arguments[0] = boolean,
            4 => *type_ref = CoreType::Hole,
            5 => {
                if let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                {
                    type_arguments.push(boolean);
                }
            }
            6 => {
                let child = evidence[0].clone();
                if let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                {
                    evidence_arguments.push(child);
                }
            }
            7 => evidence.push(evidence[0].clone()),
            8 => evidence.clear(),
            9 => {
                if let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence {
                    *identity = "std/int::Add".into();
                }
            }
            10 => {
                **left = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            _ => {
                **right = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").expect_err(&format!("mutation {mutation}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "div-雪🙂.ssrg");
    }
}

#[test]
fn division_method_refuses_forged_source_stages_and_dispatch() {
    let source = "pub fn probe -> (Int -> Int) = div 7\n";
    let core = compile_core_module(CompileInput::new(
        "div-method-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..7 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("checked method")
        };
        match mutation {
            0 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Div".into(),
            1 => trait_dispatch.as_mut().unwrap().method = "mul".into(),
            2 => *type_ref = CoreType::Hole,
            3 => {
                arguments.push(arguments[0].clone());
                arguments.push(arguments[0].clone());
            }
            4 => deferred_evidence_parameters.push(CoreType::Hole),
            5 => *evidence_argument_index = Some(0),
            _ => {
                evidence[0].constraint.arguments[1] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").expect_err(&format!("mutation {mutation}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "div-method-雪.ssrg");
    }
}

#[test]
fn division_callee_operands_and_captured_prefix_are_evaluated_once_in_source_order() {
    let source = r#"
type Operand = | Operand Int
fn unwrap value: Operand -> Int = match value { Operand n -> n }
fn left -> Operand = Operand 125
fn right -> Operand = Operand 60
fn selected marker: Operand -> Operand = marker
fn select marker: Operand -> (Int -> Int -> Int) = match selected marker { Operand _ -> (/) }
pub fn probe -> Int = {
  let direct = unwrap (left ()) / unwrap (right ())
  let full = select (Operand 0) (unwrap (left ())) (unwrap (right ()))
  let saved = select (Operand 0) (unwrap (left ()))
  direct + full + saved (unwrap (right ())) + saved (unwrap (right ()))
}
"#;
    let core = compile_core_module(CompileInput::new(
        "div-order-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    // Instrument only this private observation copy. Normal emitted programs
    // above run without hooks; these markers reveal strict call order/counts.
    let mut generated = artifact.source[..artifact.source.rfind("fn main()").unwrap()].to_owned();
    let symbol = |name: &str| {
        format!(
            "v_{}",
            name.bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        )
    };
    // The curried wrapper and direct entry each contain the source body.
    // A saturated helper inside that body observes both invocation paths.
    for (name, event) in [("left", "left"), ("right", "right"), ("selected", "select")] {
        let start = generated
            .find(&format!(
                "fn {}_direct(",
                symbol(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(
            body,
            &format!(" TRACE.with(|trace| trace.borrow_mut().push({event:?})); "),
        );
    }
    generated.push_str(&format!(r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
    let result = {}(vec![]).apply(V::Unit);
    assert_eq!(result.render(), "8");
    TRACE.with(|trace| assert_eq!(&*trace.borrow(), &["left", "right", "select", "left", "right", "select", "left", "right", "right"]));
    drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
    eprintln!("managed-live={{}}", LIVE.with(Cell::get));
}}
"#, symbol(&artifact.metadata.entry)));
    execute_rust(&generated, "order", "", 0, None);
}
