#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr};
use std::process::Command;
use support::{emit, rust_id, Scratch};

#[test]
fn comprehensions_preserve_iterable_order_patterns_guards_and_scopes() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/comprehensions.ssrg");
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
    let artifact = emit_rust_module(&core.core_ir, "main").unwrap();
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/comprehensions.stdout")
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn comprehensions_evaluate_sources_guards_and_elements_once_in_order() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/comprehension-evaluation.ssrg");
    let trace: Vec<String> = serde_json::from_str(include_str!(
        "../../../examples/spec/fixtures/rust-backend/comprehension-evaluation.trace.json"
    ))
    .unwrap();
    for list in [false, true] {
        let source = if list {
            source
                .replace("pub fn probe -> Array<Int>", "pub fn probe -> List<Int>")
                .replace("[element x y |", "`[element x y |")
        } else {
            source.to_owned()
        };
        let artifact = emit(&source);
        let main = artifact.source.rfind("fn main()").unwrap();
        let mut generated = artifact.source[..main].to_owned();
        for name in ["outer", "keep", "inner", "start", "accept", "element"] {
            let start = generated
                .find(&format!(
                    "fn {}_direct(",
                    rust_id(&format!("native/main::{name}"))
                ))
                .unwrap();
            let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
            generated.insert_str(
                body,
                &format!(" TRACE.with(|t| t.borrow_mut().push({name:?})); "),
            );
        }
        generated.push_str(&format!(
            r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
    let result = {}(vec![]).apply(V::Unit);
    assert_eq!(result.render(), {:?});
    TRACE.with(|trace| assert_eq!(*trace.borrow(), {:?}));
    drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#,
            rust_id(&artifact.metadata.entry),
            if list { "`[11, 33]" } else { "[11, 33]" },
            trace,
        ));
        let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&generated))
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
}

#[test]
fn comprehensions_bound_live_cursor_memory_and_frames_on_a_small_stack() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/comprehension-large.ssrg");
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
    let artifact = emit_rust_module(&core.core_ir, "main").unwrap();
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    let start = generated
        .find(&format!("fn {}_direct(", rust_id("native/main::reject")))
        .unwrap();
    let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
    generated.insert_str(
        body,
        " assert!(LIVE.with(Cell::get) < 128); CALLS.with(|c| c.set(c.get() + 1)); ",
    );
    generated.push_str(&format!(
        r#"
thread_local! {{ static CALLS: Cell<usize> = const {{ Cell::new(0) }}; }}
fn main() {{
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{
        enable_cycle_collection();
        let mut context = NativeContext::default();
        let result = context.root({}(vec![]).apply(V::Unit));
        assert!(matches!(result, Ok(V::Unit)));
        assert!(context.peak_frames <= 8, "{{}}", context.peak_frames);
        CALLS.with(|c| assert_eq!(c.get(), 200_000));
        clear_dictionaries(); collect_cycles().unwrap(); assert_eq!(LIVE.with(Cell::get), 0);
    }}).unwrap().join().unwrap();
}}
"#,
        rust_id(&artifact.metadata.entry)
    ));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/comprehension-large.stdout")
    );
}

#[test]
fn comprehensions_reject_unproven_result_source_element_guard_and_iterable_evidence() {
    use seseragi_lowering::{CoreComprehensionClause, CorePattern, CoreType};
    for list in [false, true] {
        let source = format!(
            "pub fn probe -> {}<Int> = {}[x | x <- [1,2], x > 0]",
            if list { "List" } else { "Array" },
            if list { "`" } else { "" }
        );
        let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", &source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..10 {
            let mut bad = core.clone();
            let (clauses, type_ref) = match &mut bad.functions[0].body {
                CoreExpr::ArrayComprehension {
                    clauses, type_ref, ..
                }
                | CoreExpr::ListComprehension {
                    clauses, type_ref, ..
                } => (clauses, type_ref),
                _ => panic!(),
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: if list { "List" } else { "Array" }.into(),
                        canonical: "fake::Collection".into(),
                        arguments: vec![CoreType::Named {
                            name: "Int".into(),
                            arguments: vec![],
                        }],
                    }
                }
                2 => clauses.clear(),
                3 => clauses.swap(0, 1),
                4 => {
                    let CoreComprehensionClause::Guard { condition, .. } = &mut clauses[1] else {
                        panic!()
                    };
                    *condition = CoreExpr::Integer {
                        value: "1".into(),
                        origin: origin.clone(),
                    };
                }
                _ => {
                    let CoreComprehensionClause::Generator {
                        pattern,
                        source,
                        evidence,
                        ..
                    } = &mut clauses[0]
                    else {
                        panic!()
                    };
                    match mutation {
                        5 => {
                            *source = CoreExpr::Boolean {
                                value: true,
                                origin: origin.clone(),
                            }
                        }
                        6 => {
                            let CorePattern::Binding { type_ref, .. } = pattern else {
                                panic!()
                            };
                            *type_ref = CoreType::Named {
                                name: "Bool".into(),
                                arguments: vec![],
                            };
                        }
                        7 => evidence.constraint.trait_identity = Some("fake::Iterable".into()),
                        8 => evidence.constraint.arguments.pop().map(|_| ()).unwrap(),
                        9 => {
                            let seseragi_lowering::CoreInstanceEvidence::Standard {
                                identity, ..
                            } = &mut evidence.evidence
                            else {
                                panic!()
                            };
                            *identity = "std/list::Iterable".into();
                        }
                        _ => unreachable!(),
                    }
                }
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{mutation}: {error:?}");
            assert_eq!(error.origin.unwrap(), origin, "{mutation}");
        }
    }
}
