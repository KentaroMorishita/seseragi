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
