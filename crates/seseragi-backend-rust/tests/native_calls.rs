#[path = "support/native.rs"]
mod support;

use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_free_variables, known_pure_call_shape, CoreCallShape};
use std::process::Command;
use support::{emit, run, rust_id, Scratch};

#[test]
fn direct_calls_keep_partial_and_returned_function_abi() {
    let source = "fn add a: Int -> b: Int -> Int = a + b\nfn returned seed: Int -> (Int -> Int) = \\value -> seed + value\npub fn probe -> (Int, Int, Int, Int) = {\n let partially = add 35\n let alias = add\n (add 35 7, partially 7, returned 35 7, alias 35 7)\n}\n";
    run(source, "(42, 42, 42, 42)");
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert_eq!(
        known_pure_call_shape(&core, "native/main::add", 1),
        Some(CoreCallShape::Partial)
    );
    assert_eq!(
        known_pure_call_shape(&core, "native/main::add", 2),
        Some(CoreCallShape::Saturated)
    );
    assert_eq!(
        known_pure_call_shape(&core, "native/main::returned", 2),
        Some(CoreCallShape::ReturnedApplication)
    );
    assert_eq!(known_pure_call_shape(&core, "unresolved", 2), None);
}

#[test]
fn captures_bindings_and_hidden_evidence_are_analyzed_separately() {
    let core = compile_core_module(CompileInput::new(
        "main.ssrg",
        "native/main",
        include_str!("../../../examples/spec/fixtures/rust-backend/calls.ssrg"),
    ))
    .unwrap()
    .core_ir;
    let capture = core
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::captureOne"))
        .unwrap();
    let free = core_free_variables(&capture.body).unwrap();
    assert!(free.names.contains("seed"));
    assert!(!free.names.contains("unused"));
    let source = "pub fn probe -> Int = { let seed = 40; let value = { let seed = seed + 1; seed + 1 }; value }\n";
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert!(core_free_variables(&core.functions[0].body)
        .unwrap()
        .names
        .is_empty());
    run(source, "42");
}

#[test]
fn escaped_callback_does_not_retain_an_unused_array() {
    let source = format!(
        "{}\npub fn probe -> (Unit -> Int) = captureOne 42 [1, 2, 3]\n",
        include_str!("../../../examples/spec/fixtures/rust-backend/calls.ssrg")
    );
    let artifact = emit(&source);
    let main = artifact.source.rfind("fn main()").unwrap();
    let generated = format!("{}\nfn main() {{ let callback = {}(vec![]).apply(V::Unit); assert_eq!(LIVE.with(Cell::get), 1, \"only the escaped function remains\"); let V::Fun(function) = &callback else {{ panic!() }}; assert_eq!(function.get().captures.len(), 1); assert_eq!(callback.apply(V::Unit).render(), \"42\"); drop(callback); assert_eq!(LIVE.with(Cell::get), 0); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
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
