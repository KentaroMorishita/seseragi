#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use std::process::Command;
use support::{emit, rust_id, Scratch};

#[test]
fn monomorphic_globals_keep_values_patterns_closures_and_source_arity() {
    let artifact = emit(
        r#"
struct Pair { left: Int, right: Int }
let base: Int = laterFunction ()
let (left, right): (Int, Int) = (20, 22)
let pair: Pair = Pair { left: left, right: right }
let retained: Unit -> Int = \_ -> later
let partial: Int -> Int = returned pair.left
let later: Int = 42
fn laterFunction -> Int = 7
fn returned n: Int -> (Int -> Int) = \m -> n + m
pub fn probe -> (Int, Int, Int) = (base, retained (), partial pair.right)
"#,
    );
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"(7, 42, 42)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn polymorphic_globals_keep_erased_values_and_per_use_dictionaries() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/polymorphic-globals.ssrg");
    let core = compile_core_module(CompileInput::new("global.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/polymorphic-globals.stdout")
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn polymorphic_globals_share_one_initialized_value_across_types() {
    let artifact = emit(
        "let identity = \\value -> value\nlet retained = identity\nfn number -> (Int -> Int) = identity\nfn text -> (String -> String) = retained\npub fn probe -> Int = 42\n",
    );
    let main = artifact.source.rfind("fn main()").unwrap();
    let generated = format!(
        r#"{}
fn main() {{
    initialize_globals();
    let original = get_global(0).node_id().unwrap();
    assert_eq!(get_global(1).node_id(), Some(original));
    for _ in 0..100 {{
        let number = {}(vec![]).apply(V::Unit);
        let text = {}(vec![]).apply(V::Unit);
        assert_eq!(number.node_id(), Some(original));
        assert_eq!(text.node_id(), Some(original));
        assert_eq!(number.apply(V::Int(42)).render(), "42");
        assert_eq!(text.apply(V::text("hello")).render(), "hello");
    }}
    clear_globals();
    clear_dictionaries();
    assert_eq!(LIVE.with(Cell::get), 0);
}}
"#,
        &artifact.source[..main],
        rust_id("native/main::number"),
        rust_id("native/main::text"),
    );
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn polymorphic_globals_link_in_dependency_order_and_keep_dictionary_owners() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/polymorphic-globals"
    ))
    .unwrap();
    let checked = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    let modules = checked
        .compiled
        .order
        .iter()
        .map(|id| &checked.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let artifact =
        seseragi_backend_rust::emit_rust_project(&modules, &checked.entry_module, "main").unwrap();
    assert_eq!(artifact.metadata.modules.len(), 5);
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, include_bytes!("../../../examples/spec/fixtures/rust-backend-projects/polymorphic-globals/expected.stdout"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn globals_reject_erased_source_parameters_before_emission() {
    // Ordinary constrained aliases keep their source parameters and evidence
    // ABI. Erasing them cannot turn the body into a certified method factory.
    let source = "fn render<A> value: A -> String where Show<A> = show value\nlet retainedValue = render\npub fn probe -> String = retainedValue 42\n";
    let mut core = compile_core_module(CompileInput::new("global.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "probe").unwrap();
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"42\n");
    core.functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::retainedValue"))
        .unwrap()
        .parameters
        .clear();
    let error = emit_rust_module(&core, "probe").err().unwrap();
    assert!(
        error
            .message
            .contains("method factory introduces source arguments"),
        "{error:?}"
    );
    assert!(error.origin.is_some());
}

#[test]
fn global_cleanup_handles_partial_initialization_and_deep_ref_graphs() {
    let artifact = emit("pub fn probe -> Int = 42\n");
    let main = artifact.source.rfind("fn main()").unwrap();
    let harness = r#"
fn main() {
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {
        let code = run_native_pure_entry(|| {
            let mut chain = V::Unit;
            for _ in 0..100_000 { chain = V::adt("node", chain); }
            set_global(0, chain);
            let cell = V::reference(V::Unit);
            cell.write_reference(V::fun(vec![cell.clone()], |c, _| c[0].clone()));
            set_global(1, cell);
            assert_eq!(collect_cycles().unwrap().reclaimed, 0);
            panic!("initializer stopped");
        }, true);
        assert_eq!(code, 70);
        assert_eq!(LIVE.with(Cell::get), 0);
        assert!(GLOBALS.with(|v| v.borrow().is_empty()));
        assert_eq!(run_native_pure_entry(|| { set_global(0, V::text("again")); get_global(0) }, true), 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }).unwrap().join().unwrap();
}
"#;
    let scratch = Scratch::new();
    let source = format!("{}\n{harness}", &artifact.source[..main]);
    let output = Command::new(scratch.compile(&source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"again\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("initializer stopped"));
}
