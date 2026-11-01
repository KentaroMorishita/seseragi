#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr};
use std::process::Command;
use support::{emit, run, rust_id, Scratch};

#[test]
fn recursive_named_locals_cannot_capture_a_same_named_global_or_outer_local() {
    for initial in [
        "let loop = \\n: Int -> n + 7\npub fn probe -> Int = {",
        "pub fn probe -> Int = { let loop = \\n: Int -> n + 7; {",
    ] {
        let source = format!(
            "{initial} fn loop n: Int -> Int = if n == 0 then 0 else loop (n - 1); loop 1 }}{}",
            if initial.contains("let loop =") && initial.starts_with("pub") {
                " }"
            } else {
                ""
            }
        );
        run(&source, "0");
    }
}

#[test]
fn recursive_locals_keep_self_mutual_captures_and_partial_calls() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/local-recursion.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/local-recursion.stdout")
            .trim_end(),
    );
}

#[test]
fn recursive_local_tail_calls_keep_snapshots_and_evidence_on_a_small_stack() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/local-tail-recursion.ssrg"
    ));
    assert!(artifact.metadata.self_tail_calls >= 3);
    let main = artifact.source.rfind("fn main()").unwrap();
    let expected =
        include_str!("../../../examples/spec/fixtures/rust-backend/local-tail-recursion.stdout")
            .trim_end();
    let generated = format!("{}\nfn main() {{ std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), {expected:?}); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}).unwrap().join().unwrap(); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn escaped_recursive_locals_release_their_environments() {
    for source in [
        r#"
pub fn probe -> (Int -> String) = {
  let payload = "retained"
  fn loop n: Int -> String = if n == 0 then payload else loop (n - 1)
  loop
}
"#,
        r#"
pub fn probe -> (Int -> String) = {
  let payload = "retained"
  rec {
    fn even n: Int -> String = if n == 0 then payload else odd (n - 1)
    fn odd n: Int -> String = if n == 0 then payload else even (n - 1)
  }
  even
}
"#,
    ] {
        let artifact = emit(source);
        let main = artifact.source.rfind("fn main()").unwrap();
        let harness = format!(
            r#"
fn main() {{
    enable_cycle_collection();
    let callback = {}(vec![]).apply(V::Unit);
    assert_eq!(callback.apply(checked_int(10)).render(), "retained");
    assert_eq!(collect_cycles().unwrap().reclaimed, 0);
    drop(callback);
    clear_dictionaries();
    assert_eq!(LIVE.with(Cell::get), 0, "recursive closures have no ownership cycle");
}}
"#,
            rust_id(&artifact.metadata.entry)
        );
        let scratch = Scratch::new();
        let output =
            Command::new(scratch.compile(&format!("{}\n{harness}", &artifact.source[..main])))
                .env("PATH", "")
                .output()
                .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn recursive_local_captures_participate_in_ref_cycle_collection() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/local-recursion-effects.ssrg");
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
    let artifact = emit_rust_module(&core.core_ir, "main").unwrap();
    let main = artifact.source.rfind("fn main()").unwrap();
    let generated = format!("{}\nfn main() {{ std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{ assert_eq!(run_native_entry(|| {}(vec![]).apply(V::Unit), true), 0); assert_eq!(LIVE.with(Cell::get), 0); }}).unwrap().join().unwrap(); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/local-recursion-effects.stdout"
        )
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("managed-live=0"), "{stderr}");
    let reclaimed: usize = stderr
        .split("collected=")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        reclaimed >= 3,
        "must reclaim the Ref, recursive closure and captured environment: {stderr}"
    );
}

#[test]
fn recursive_local_tail_calls_reject_forged_arguments_and_staging() {
    use seseragi_lowering::{CoreStatement, CoreType};
    let source = "pub fn probe -> Int = { fn loop n: Int -> Int = if n == 0 then 0 else loop (n - 1); loop 5 }\n";
    let core = compile_core_module(CompileInput::new("recursive-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    for case in 0..4 {
        let mut changed = core.clone();
        let CoreExpr::Sequence { statements, .. } = &mut changed.functions[0].body else {
            panic!("sequence")
        };
        let CoreStatement::LocalFunction { body, .. } = &mut statements[0] else {
            panic!("local")
        };
        let CoreExpr::If { else_branch, .. } = body else {
            panic!("if")
        };
        let CoreExpr::Call {
            arguments,
            evidence_argument_index,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            ..
        } = &mut **else_branch
        else {
            panic!("self call")
        };
        match case {
            0 => *evidence_argument_index = Some(99),
            1 => deferred_evidence_parameters.push(CoreType::Hole),
            2 => deferred_evidence_type_constructor_parameters.push("F".into()),
            3 => {
                arguments[0] = CoreExpr::Boolean {
                    value: true,
                    origin: core_expr_origin(&arguments[0]).clone(),
                }
            }
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {case}");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "recursive-λ.ssrg");
        assert!(origin.start < origin.end && origin.end <= source.len());
    }
}
