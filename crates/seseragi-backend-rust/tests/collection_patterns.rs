use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_module, CompileInput};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "seseragi-collection-patterns-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn compare(source: &str, expected: &str, trace_scrutinee: bool) {
    let compiled = compile_module(CompileInput::new(
        "collection-patterns.ssrg",
        "native/patterns",
        source,
    ))
    .unwrap_or_else(|error| panic!("{error:?}"));
    let mut rust = emit_rust_module(&compiled.core_ir, "main").unwrap().source;
    let mut typescript = compiled.generated.typescript;
    if trace_scrutinee {
        // Trace the fixture's producer without adding effects to its source type.
        let producer = compiled
            .core_ir
            .functions
            .iter()
            .find(|function| function.symbol.ends_with("::scrutinee"))
            .unwrap();
        let name = producer
            .symbol
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let start = rust.find(&format!("fn v_{name}_direct(")).unwrap();
        let body = start + rust[start..].find('{').unwrap() + 1;
        rust.insert_str(body, " eprintln!(\"scrutinee\"); ");
        assert_eq!(typescript.matches("const scrutinee =").count(), 1);
        typescript = typescript.replacen("const scrutinee =", "const producer =", 1);
        typescript.push_str(
            "\nconst scrutinee = (...args: Parameters<typeof producer>) => { console.error('scrutinee'); return producer(...args); };\n",
        );
    }
    let scratch = Scratch::new();
    let program = scratch.0.join("program.rs");
    let binary = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    fs::write(&program, rust).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O"])
        .arg(&program)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let native = Command::new(binary).env("PATH", "").output().unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(native.stdout, expected.as_bytes());
    let trace = if trace_scrutinee { "scrutinee\n" } else { "" };
    assert_eq!(native.stderr, trace.as_bytes());

    let runtime = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../runtime/ts/src")
        .canonicalize()
        .unwrap();
    typescript.push_str(
        "\nimport { run as runFixture } from '@seseragi/runtime/effect';\nimport { liveConsole as fixtureConsole } from '@seseragi/runtime/console';\nconst result = await runFixture(main(undefined), { console: fixtureConsole });\nif (result.kind !== 'success') throw new Error(JSON.stringify(result));\n",
    );
    let typescript = typescript
        .replace("\"@seseragi/runtime/", &format!("\"{}/", runtime.display()))
        .replace("'@seseragi/runtime/", &format!("'{}/", runtime.display()));
    let program = scratch.0.join("program.ts");
    fs::write(&program, typescript).unwrap();
    let ts = Command::new("bun").arg(program).output().unwrap();
    assert!(
        ts.status.success(),
        "{}",
        String::from_utf8_lossy(&ts.stderr)
    );
    assert_eq!(ts.stdout, expected.as_bytes());
    assert_eq!(native.stdout, ts.stdout);
    assert_eq!(ts.stderr, trace.as_bytes());
}

#[test]
fn fixed_rest_nested_and_guarded_array_and_list_patterns_match_ts_and_evaluate_once() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/collection-match.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/collection-match.stdout"),
        true,
    );
}

#[test]
fn large_array_rest_and_failed_guard_match_ts() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/collection-match-large.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/collection-match-large.stdout"),
        false,
    );
}

#[test]
fn collection_patterns_sample_matches_ts() {
    compare(
        include_str!("../../../examples/samples/collection-patterns/main.ssrg"),
        include_str!("../../../examples/samples/collection-patterns/stdout.txt"),
        false,
    );
}

#[test]
fn rest_before_fixed_elements_remains_invalid() {
    for (ty, open) in [("Array<Int>", "["), ("List<Int>", "`[")] {
        for pattern in [
            format!("{open}...rest, last]"),
            format!("{open}first, ...rest, last]"),
        ] {
            let source = format!(
                "pub fn probe values: {ty} -> Int = match values {{ {pattern} -> 1; _ -> 0 }}"
            );
            assert!(compile_module(CompileInput::new(
                "invalid-rest.ssrg",
                "native/patterns",
                &source
            ))
            .is_err());
        }
    }
}
