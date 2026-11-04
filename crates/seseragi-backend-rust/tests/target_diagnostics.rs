use seseragi_backend_rust::{
    diagnose_rust_project, emit_program_wasm, emit_rust_module, RustDiagnosticTarget,
    RustTargetOptions,
};
use seseragi_driver::{compile_core_module, compile_module, CompileInput};

#[test]
fn diagnostics_match_native_refusals_without_changing_checked_core_or_ts() {
    for source in [
        "import * as effects from \"std/effect\"\npub effect fn main -> Int = effects.race (effects.succeed 1) (effects.succeed 2)\n",
        "import * as effects from \"std/effect\"\npub let saved: Task<Int> -> Task<Int> -> Task<Int> = effects.race\npub effect fn main -> Int = saved (effects.succeed 1) (effects.succeed 2)\n",
    ] {
        let input = CompileInput::new("target-雪🙂.ssrg", "target/main", source);
        let core = compile_core_module(input).unwrap().core_ir;
        let ts = compile_module(input).unwrap();
        assert_eq!(core, ts.core_ir);
        let before = core.clone();
        let expected = emit_rust_module(&core, "main").unwrap_err();
        let diagnostics = diagnose_rust_project(
            &[&core],
            &core.module,
            RustDiagnosticTarget::Native,
            &RustTargetOptions::default(),
        );
        assert_eq!(diagnostics, vec![expected]);
        assert_eq!(core, before);
        assert_eq!(compile_module(input).unwrap(), ts);
        assert!(emit_rust_module(&core, "main").is_err());
    }
}

#[test]
fn diagnostics_collect_multiple_uses_in_bindings_and_functions_without_an_entry() {
    let source = concat!(
        "import { race as compete, succeed } from \"std/effect\"\n",
        "pub let first: Task<Int> = compete (succeed 1) (succeed 2)\n",
        "pub fn pair -> (Task<Int>, Task<Int>) = { let _prefix = \"雪🙂\"; (compete (succeed 3) (succeed 4), compete (succeed 5) (succeed 6)) }\n",
        "pub fn race value: Int -> Int = value\n",
        "pub fn ordinary -> Int = race 42\n",
    );
    let core = compile_core_module(CompileInput::new("uses.ssrg", "target/uses", source))
        .unwrap()
        .core_ir;
    let diagnostics = diagnose_rust_project(
        &[&core],
        &core.module,
        RustDiagnosticTarget::Native,
        &RustTargetOptions::default(),
    );
    assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
    let starts = source
        .match_indices("compete (succeed")
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    for (diagnostic, start) in diagnostics.iter().zip(starts) {
        assert_eq!(diagnostic.code, "SES-RUST-0001");
        assert!(diagnostic.message.contains("std/effect::race"));
        assert_eq!(diagnostic.module.as_deref(), Some("target/uses"));
        let origin = diagnostic.origin.as_ref().unwrap();
        assert_eq!(origin.source, "uses.ssrg");
        assert_eq!(origin.start, start);
        assert!(source[origin.start..origin.end].starts_with("compete (succeed"));
    }
}

#[test]
fn native_support_is_not_confused_with_program_wasm_support() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-transform.ssrg"),
        "pub fn main -> Int = 2 ** 3\n",
    ] {
        let core = compile_core_module(CompileInput::new("supported.ssrg", "target/main", source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "main").unwrap();
        assert!(diagnose_rust_project(
            &[&core],
            &core.module,
            RustDiagnosticTarget::Native,
            &RustTargetOptions::default(),
        )
        .is_empty());
    }
    let core = compile_core_module(CompileInput::new(
        "power.ssrg",
        "target/power",
        "pub fn main -> Int = 2 ** 3\n",
    ))
    .unwrap()
    .core_ir;
    let expected = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
    let wasm = diagnose_rust_project(
        &[&core],
        &core.module,
        RustDiagnosticTarget::ProgramWasm,
        &RustTargetOptions::default(),
    );
    assert_eq!(wasm, vec![expected]);
}

#[test]
fn batch_native_apis_keep_their_checked_support_in_diagnostics() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process-config.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process-callables.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/std-benchmark.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/std-test.ssrg"),
    ] {
        let input = CompileInput::new("batch.ssrg", "target/batch", source);
        let core = compile_core_module(input).unwrap().core_ir;
        assert_eq!(compile_module(input).unwrap().core_ir, core);
        emit_rust_module(&core, "main").unwrap();
        assert_eq!(
            diagnose_rust_project(
                &[&core],
                &core.module,
                RustDiagnosticTarget::Native,
                &RustTargetOptions::default(),
            ),
            [],
        );
    }
}

#[test]
fn wasm_diagnostics_match_batch_api_refusals_at_calls_and_saved_references() {
    for (imports, declaration, expression, message) in [
        (
            "import * as c from \"std/child-process\"",
            "fn probe -> c.CaptureLimit",
            "c.defaultCaptureLimit ()",
            "native child process",
        ),
        (
            "import * as c from \"std/child-process\"",
            "fn probe -> (Unit -> c.CaptureLimit)",
            "c.defaultCaptureLimit",
            "native child process",
        ),
        (
            "import * as c from \"std/child-process\"",
            "effect fn probe command:c.Command -> c.ChildExitStatus with childProcesses:c.ChildProcesses fails c.ChildProcessError",
            "c.runInherited command",
            "native child process",
        ),
        (
            "import * as b from \"std/benchmark\"",
            "fn probe -> Int",
            "b.blackBox 42",
            "std/benchmark",
        ),
        (
            "import * as b from \"std/benchmark\"",
            "fn probe -> (Int -> Int)",
            "b.blackBox",
            "std/benchmark",
        ),
        (
            "import * as b from \"std/benchmark\"",
            "fn probe -> Effect<{},b.BenchmarkFailure,Unit>",
            "b.fail \"boom\"",
            "std/benchmark",
        ),
        (
            "import * as t from \"std/test\"",
            "fn probe -> Effect<{},t.TestFailure,Unit>",
            "t.equal 1 1",
            "std/test",
        ),
        (
            "import * as t from \"std/test\"",
            "fn probe -> (Bool -> Effect<{},t.TestFailure,Unit>)",
            "t.isTrue",
            "std/test",
        ),
        (
            "import * as s from \"std/signal\"",
            "fn probe -> Task<s.MutableSignal<Int>>",
            "s.make 1",
            "Signal",
        ),
        (
            "import * as p from \"std/process\"\nimport * as n from \"std/non-empty-list\"\nimport * as s from \"std/stream\"",
            "fn probe watched:n.NonEmptyList<p.ProcessSignal> -> s.Stream<{process:p.Process},p.ProcessError,p.ProcessSignal>",
            "p.signals watched",
            "native Process",
        ),
    ] {
        let source = format!("// 雪🙂\n{imports}\n{declaration} = {expression}\npub fn main -> Int = 42\n");
        let input = CompileInput::new("batch-雪.ssrg", "target/batch", &source);
        let core = compile_core_module(input).unwrap().core_ir;
        assert_eq!(compile_module(input).unwrap().core_ir, core);
        emit_rust_module(&core, "main").unwrap();
        assert!(diagnose_rust_project(
            &[&core],
            &core.module,
            RustDiagnosticTarget::Native,
            &RustTargetOptions::default(),
        ).is_empty());
        let expected = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert!(expected.message.contains(message), "{expression}: {expected}");
        let start = source.rfind(expression).unwrap();
        let origin = expected.origin.as_ref().unwrap();
        assert_eq!(origin.source, "batch-雪.ssrg");
        assert_eq!(origin.start, start, "{expression}");
        assert_eq!(origin.end, start + expression.len(), "{expression}");
        assert_eq!(diagnose_rust_project(
            &[&core],
            &core.module,
            RustDiagnosticTarget::ProgramWasm,
            &RustTargetOptions::default(),
        ), vec![expected], "{expression}");
    }
}

#[test]
fn foreign_declaration_diagnostics_keep_their_source_locations() {
    let source = "pub foreign \"typescript\" from \"./host.mjs\" { pure fn first -> Int }\npub foreign \"typescript\" from \"./other.mjs\" { pure fn second -> Int }\npub fn main -> Int = 42\n";
    let core = compile_core_module(CompileInput::new("foreign.ssrg", "target/main", source))
        .unwrap()
        .core_ir;
    let diagnostics = diagnose_rust_project(
        &[&core],
        &core.module,
        RustDiagnosticTarget::Native,
        &RustTargetOptions::default(),
    );
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    for (diagnostic, foreign) in diagnostics.iter().zip(&core.foreign_modules) {
        assert_eq!(diagnostic.origin.as_ref(), Some(&foreign.origin));
        assert_eq!(
            diagnostic.message,
            "Rust backend: foreign modules are unsupported"
        );
    }
    assert!(emit_rust_module(&core, "main").is_err());
}
