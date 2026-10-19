use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreStatement, CoreType};
use std::{fs, process::Command};

fn execute(source: &str, name: &str, stdout: &str, code: i32) {
    let core = compile_core_module(CompileInput::new("operator-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    execute_generated(&artifact.source, name, stdout, code);
}

fn execute_generated(source: &str, name: &str, stdout: &str, code: i32) {
    let directory =
        std::env::temp_dir().join(format!("seseragi-operator-{name}-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let input = directory.join("program.rs");
    let exe = directory.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, source).unwrap();
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
fn primitive_and_selected_operator_partials_keep_values_and_reuse() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partials.ssrg"),
        "values",
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partials.stdout"),
        0,
    );
}

#[test]
fn a_saved_operator_runs_its_selected_first_stage_before_the_next_statement() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-defect.ssrg"),
        "selected-defect",
        "",
        70,
    );
    execute("pub effect fn main = do { let saved: Int -> Int = add (9007199254740991 + 1); println \"unreachable\" }", "primitive-defect", "", 70);
}

#[test]
fn selected_operator_edges_preserve_cold_effects_finalizers_and_cycles() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-effects.ssrg"),
        "effects",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/operator-partial-effects.stdout"
        ),
        0,
    );
}

#[test]
fn a_selected_higher_kinded_child_dictionary_runs_in_the_first_stage() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-children.ssrg"),
        "hkt-child",
        "",
        70,
    );
}

const MONAD_BOX: &str = r#"
type Box<A> = | Box A
trait Monad<F<_>> {
  fn flatMap<A, B> f: (A -> F<B>) -> value: F<A> -> F<B>
}
instance Monad<Box> {
  fn flatMap<A, B> f: (A -> Box<B>) -> value: Box<A> -> Box<B> = match value { Box item -> f item }
}
"#;

#[test]
fn trait_operator_values_preserve_forward_and_reversed_abis_and_saved_captures() {
    for (name, source, stdout) in [
        (
            "values",
            include_str!("../../../examples/spec/fixtures/rust-backend/trait-operator-order.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/trait-operator-order.stdout"
            ),
        ),
        (
            "effects",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/trait-operator-effects.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/trait-operator-effects.stdout"
            ),
        ),
    ] {
        execute(source, &format!("trait-{name}"), stdout, 0);
    }
}

#[test]
fn trait_operator_partial_retains_conditional_children_through_resource_defect() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-operator-children.ssrg"),
        "trait-children",
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-operator-children.stdout"),
        70,
    );
}

#[test]
fn full_trait_operator_preserves_method_owned_evidence_and_declared_arity() {
    for (name, signature, implementation) in [
        (
            "two",
            "f: (A -> M<B>) -> value: M<A> -> M<B>",
            "f: (A -> Box<B>) -> value: Box<A> -> Box<B> where Show<A> = match value { Box item -> { let text = show item; f item } }",
        ),
        (
            "one",
            "f: (A -> M<B>) -> (M<A> -> M<B>)",
            "f: (A -> Box<B>) -> (Box<A> -> Box<B>) where Show<A> = \\value -> match value { Box item -> { let text = show item; f item } }",
        ),
    ] {
        let source = format!(
            "type Box<A> = | Box A\ntrait Monad<M<_>> {{ fn flatMap<A, B> {signature} where Show<A> }}\ninstance Monad<Box> {{ fn flatMap<A, B> {implementation} }}\npub effect fn main = do {{ let result = Box 40 >>= (\\n: Int -> Box (n + 2)); println (match result {{ Box n -> show n }}) }}\n"
        );
        execute(&source, &format!("method-evidence-{name}"), "42\n", 0);
    }
}

#[test]
fn canonical_hkt_parameter_method_values_require_the_full_checked_signature() {
    let source = "fn mapped<F<_>, A, B> f: (A -> B) -> value: F<A> -> F<B> where Functor<F> = (<$>) f value\nfn applied<F<_>, A, B> f: F<A -> B> -> value: F<A> -> F<B> where Applicative<F> = (<*>) f value\nfn bound<F<_>, A, B> value: F<A> -> f: (A -> F<B>) -> F<B> where Monad<F> = (>>=) value f\npub fn probe -> Unit = ()\n";
    let core = compile_core_module(CompileInput::new("hkt-order-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    execute_generated(&artifact.source, "hkt-signatures", "()\n", 0);
    for function in 0..3 {
        for mutation in 0..9 {
            let mut changed = core.clone();
            let CoreExpr::Sequence { statements, .. } = &mut changed.functions[function].body
            else {
                panic!("operator sequence")
            };
            let CoreStatement::PureLet {
                value:
                    CoreExpr::Call {
                        evidence,
                        trait_dispatch,
                        type_ref,
                        deferred_evidence_parameters,
                        evidence_argument_index,
                        ..
                    },
                ..
            } = &mut statements[0]
            else {
                panic!("selected method")
            };
            let dispatch = trait_dispatch.as_mut().unwrap();
            match mutation {
                0 => dispatch.trait_identity = "unrelated::Functor".into(),
                1 => dispatch.method = "unknown".into(),
                2 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                3 => evidence[0].constraint.arguments.clear(),
                4 => evidence[0].constraint.arguments.push(CoreType::Hole),
                5 => *type_ref = CoreType::Hole,
                6 => {
                    let CoreType::Function { result, .. } = type_ref else {
                        panic!()
                    };
                    **result = CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    };
                }
                7 => deferred_evidence_parameters.push(CoreType::Hole),
                _ => *evidence_argument_index = Some(0),
            }
            assert!(
                emit_rust_module(&changed, "probe").is_err(),
                "function {function}, mutation {mutation}"
            );
        }
    }
}

#[test]
fn reversed_trait_operator_partial_evaluates_its_supplied_source_operand() {
    execute(
        &format!("{MONAD_BOX}\nfn left -> Box<Int> = {{ let bad = 9007199254740991 + 1; Box 1 }}\npub effect fn main = do {{ let saved: (Int -> Box<Int>) -> Box<Int> = (>>=) (left ()); println \"unreachable\" }}"),
        "reversed-source-prefix",
        "",
        70,
    );
}

#[test]
fn reversed_trait_operator_reference_keeps_source_arguments_and_method_abi_distinct() {
    execute(
        &format!("{MONAD_BOX}\nfn unbox<A> box: Box<A> -> A = match box {{ Box item -> item }}\npub effect fn main = do {{ let bind: Box<Int> -> (Int -> Box<Int>) -> Box<Int> = (>>=); let saved = bind (Box 40); println (show (unbox (saved (\\n -> Box (n + 2))))) }}"),
        "reversed-source-reference",
        "42\n",
        0,
    );
}

#[test]
fn reversed_trait_operator_runs_source_expressions_once_before_method_stages() {
    let header = MONAD_BOX.replace(
        "fn flatMap<A, B> f: (A -> Box<B>) -> value: Box<A> -> Box<B> = match value { Box item -> f item }",
        "fn flatMap<A, B> f: (A -> Box<B>) -> (Box<A> -> Box<B>) = { let observed = firstStage (); \\value -> { let observed = lastStage (); match value { Box item -> f item } } }",
    );
    for (name, expression) in [
        ("infix", "left () >>= right ()"),
        ("bare", "(>>=) (left ()) (right ())"),
        ("saved", "{ let selected: Box<Int> -> (Int -> Box<Int>) -> Box<Int> = (>>=); selected (left ()) (right ()) }"),
        ("partial", "{ let saved: (Int -> Box<Int>) -> Box<Int> = (>>=) (left ()); saved (right ()) }"),
    ] {
        let source = format!("{header}\nfn firstStage -> Unit = ()\nfn lastStage -> Unit = ()\nfn left -> Box<Int> = Box 40\nfn right -> (Int -> Box<Int>) = \\n -> Box (n + 2)\npub fn probe -> Int = match ({expression}) {{ Box result -> result }}");
        let core = compile_core_module(CompileInput::new("order-λ.ssrg", "native/main", &source)).unwrap().core_ir;
        let artifact = emit_rust_module(&core, "probe").unwrap();
        let end = artifact.source.rfind("fn main()").unwrap();
        let mut generated = artifact.source[..end].to_owned();
        let symbol = |name: &str| format!("v_{}", name.bytes().map(|b| format!("{b:02x}")).collect::<String>());
        // Only this test copy records calls. Separate CLI probes execute the
        // normal emitted body; these traces establish order and call counts.
        for function in ["left", "right", "firstStage", "lastStage"] {
            let start = generated.find(&format!("fn {}_direct(", symbol(&format!("native/main::{function}")))).unwrap();
            let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
            generated.insert_str(body, &format!(" TRACE.with(|trace| trace.borrow_mut().push({function:?})); "));
        }
        generated.push_str(&format!(r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let result = {}(vec![]).apply(V::Unit);
 assert_eq!(result.render(), "42");
 TRACE.with(|trace| assert_eq!(&*trace.borrow(), &["left", "right", "firstStage", "lastStage"]));
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
 eprintln!("managed-live={{}}", LIVE.with(Cell::get));
}}
"#, symbol(&artifact.metadata.entry)));
        execute_generated(&generated, &format!("reversed-trace-{name}"), "", 0);
    }
}

#[test]
fn primitive_arithmetic_refuses_unproved_identity_types_children_and_stages() {
    let source = "// 雫🙂\npub fn probe -> Int = { let saved: Int -> Int = add 40; saved 2 }";
    let core = compile_core_module(CompileInput::new("operator-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..19 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!()
        };
        let CoreStatement::PureLet {
            value:
                CoreExpr::Call {
                    arguments,
                    evidence,
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
        let bool_type = CoreType::Named {
            name: "Bool".into(),
            arguments: vec![],
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Add".into()),
            1 => evidence[0].constraint.name = "Sub".into(),
            2 => evidence[0].constraint.arguments.pop().map(|_| ()).unwrap(),
            3 => evidence[0].constraint.arguments[0] = bool_type,
            4 => evidence[0].constraint.arguments[2] = CoreType::Hole,
            5 => {
                if let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                {
                    type_arguments.push(bool_type);
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
            7 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Add".into(),
            8 => trait_dispatch.as_mut().unwrap().method = "sub".into(),
            9 => *type_ref = bool_type,
            10 => {
                if let CoreType::Function { parameter, .. } = type_ref {
                    *parameter = Box::new(bool_type);
                }
            }
            11 => {
                if let CoreType::Function { result, .. } = type_ref {
                    *result = Box::new(bool_type);
                }
            }
            12 => deferred_evidence_parameters.push(bool_type),
            13 => deferred_evidence_type_constructor_parameters.push("F".into()),
            14 => *evidence_argument_index = Some(0),
            15 => arguments.push(arguments[0].clone()),
            16 => {
                evidence[0].constraint.arguments[1] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
            17 => evidence.push(evidence[0].clone()),
            _ => {
                evidence.push(evidence[0].clone());
                let mut constraints = evidence
                    .iter()
                    .map(|e| e.constraint.clone())
                    .collect::<Vec<_>>();
                for constraint in &mut constraints {
                    constraint.trait_identity = Some("std/prelude::Add".into());
                }
                trait_dispatch.as_mut().unwrap().method_contract =
                    Some(seseragi_lowering::CoreTraitMethodContract {
                        parameters: vec![
                            CoreType::Named {
                                name: "Int".into(),
                                arguments: vec![]
                            };
                            2
                        ],
                        constraints,
                    });
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap().source, "operator-λ.ssrg");
    }
}

#[test]
fn primitive_arithmetic_does_not_replace_a_scoped_binder_named_int() {
    let source = "fn generic<A> value: A -> (A -> A) where Add<A, A, A> = add value\npub fn probe -> Int = { let saved = generic 40; saved 2 }";
    let core = compile_core_module(CompileInput::new("operator-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let mut changed = core;
    let function = changed
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    // Alpha-rename the checked lexical binder, including its parameter and
    // constraint uses. Its spelling must never select the primitive ABI.
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
    emit_rust_module(&changed, "probe").unwrap();
    let function = changed
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &mut function.body else {
        panic!()
    };
    evidence[0].evidence = CoreInstanceEvidence::Standard {
        identity: "std/int::Add".into(),
        type_arguments: vec![],
        evidence_arguments: vec![],
    };
    let error = emit_rust_module(&changed, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "operator-λ.ssrg");
}
