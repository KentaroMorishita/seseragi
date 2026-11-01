#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use std::process::Command;
use support::{rust_id, Scratch};

#[test]
fn checked_effect_programs_run_on_a_small_stack_and_release_the_native_heap() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend-research/deep-effect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend-research/ref-cycle.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/list-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/list-prefix-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/local-dictionary-effects.ssrg"),
    ] {
        let core =
            compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
        let artifact = emit_rust_module(&core.core_ir, "main").unwrap();
        let main = artifact.source.rfind("fn main()").unwrap();
        let generated = format!("{}\nfn main() {{ let code = std::thread::Builder::new().stack_size(256 * 1024).spawn(|| run_native_entry(|| {}(vec![]).apply(V::Unit), {})).unwrap().join().unwrap(); assert_eq!(code, 0); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry), artifact.metadata.capabilities.iter().any(|x| x == "ref"));
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
        assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
    }
}
