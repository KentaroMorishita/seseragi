use seseragi_backend_rust::{emit_rust_module_with_options, RustSourceFile, RustTargetOptions};
use seseragi_driver::{compile_core_module, CompileInput};

#[test]
fn source_locations_leave_recursive_calls_and_comparisons_unwrapped() {
    for source in [
        "fn fib n: Int -> Int = if n < 2 then n else fib (n - 1) + fib (n - 2)\npub fn main -> Int = fib 36\n",
        "fn count n: Int -> total: Int -> Int = if n < 1 then total else count (n - 1) (total + n)\npub fn main -> Int = count 100 0\n",
    ] {
        let checked = compile_core_module(CompileInput::new("probe.ssrg", "fixture/probe", source)).unwrap();
        let mut options = RustTargetOptions::default();
        options.sources.insert(checked.core_ir.module.clone(), RustSourceFile {
            path: "probe.ssrg".into(),
            text: source.into(),
        });
        let artifact = emit_rust_module_with_options(&checked.core_ir, "main", &options).unwrap();
        let (_, generated) = artifact.source.split_once("fn initialize_globals()").unwrap();
        assert!(!generated.contains("native_source("), "{generated}");
        assert!(!generated.contains("native_source_value("), "{generated}");
        assert!(generated.contains("Some(&NativeSource"), "{generated}");
        assert!(artifact.metadata.direct_calls > 0);
    }
}

#[test]
fn runtime_paths_are_portable_and_do_not_embed_absolute_directories() {
    let source = "pub fn main -> Int = 1 / 0\n";
    let checked =
        compile_core_module(CompileInput::new("main.ssrg", "fixture/paths", source)).unwrap();
    for (path, expected) in [
        ("/private/tmp/checkout/main.ssrg", "main.ssrg"),
        ("C:\\checkout\\main.ssrg", "main.ssrg"),
        ("src\\errors\\main.ssrg", "src/errors/main.ssrg"),
    ] {
        let mut options = RustTargetOptions::default();
        options.sources.insert(
            checked.core_ir.module.clone(),
            RustSourceFile {
                path: path.into(),
                text: source.into(),
            },
        );
        let artifact = emit_rust_module_with_options(&checked.core_ir, "main", &options).unwrap();
        assert!(
            artifact.source.contains(&format!(
                "NativeSource {{ file: {expected:?}, line: 1, column: 22 }}"
            )),
            "{}",
            artifact.source
        );
        if path != expected {
            assert!(!artifact.source.contains(path));
        }
    }
}
