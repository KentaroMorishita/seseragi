#[path = "../../test-support/native_runtime.rs"]
mod link;

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
            "seseragi-let-type-scope-{}-{}",
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

fn compare(source: &str, expected: &str) {
    let compiled = compile_module(CompileInput::new(
        "let-type-scope.ssrg",
        "native/let-type-scope",
        source,
    ))
    .unwrap_or_else(|error| panic!("{error:?}"));
    let rust = emit_rust_module(&compiled.core_ir, "main")
        .unwrap_or_else(|error| panic!("{error:?}"))
        .source;
    let mut typescript = compiled.generated.typescript;
    let scratch = Scratch::new();
    let program = scratch.0.join("program.rs");
    let binary = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    link::write_program(&program, rust).unwrap();
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
    let native = Command::new(binary)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(native.stdout, expected.as_bytes(), "native");
    assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));

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
    assert_eq!(ts.stdout, expected.as_bytes(), "typescript");
    assert_eq!(native.stdout, ts.stdout);
    assert!(
        ts.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&ts.stderr)
    );
}

#[test]
fn overlapping_generalized_let_type_parameter_matches_typescript() {
    compare(
        r#"fn tag<A> value: A -> suffix: String -> String where Show<A> = show value + suffix
fn outer<A> x: A -> String = {
  let p = tag
  p 1 "?"
}
pub effect fn main = println (outer True)
"#,
        "1?\n",
    );
}

#[test]
fn generalized_let_type_parameter_shapes_match_typescript() {
    compare(
        r#"fn tag<A> value: A -> suffix: String -> String where Show<A> = show value + suffix
fn tagB<B> value: B -> suffix: String -> String where Show<B> = show value + suffix
fn renamedBound<A> x: A -> String = {
  let p = tagB
  p 1 "?"
}
fn renamedOuter<B> x: B -> String = {
  let p = tag
  p 1 "?"
}
fn annotated<A> x: A -> String = {
  let p: Int -> String -> String = tag
  p 1 "?"
}
fn wrapped<A> x: A -> String = {
  let p = \value -> tag value
  p 1 "?"
}
fn direct<A> x: A -> String = {
  tag 1 "?"
}
fn beside<A> x: A -> String where Show<A> = {
  let p = tag
  show x + p 1 "?"
}
fn atOuter<A> x: A -> String where Show<A> = {
  let p = tag
  p x "?"
}
fn shown<A> x: A -> String = {
  let p = show
  p 1
}
fn identity<A> value: A -> A = value
fn reused<A> x: A -> String where Show<A> = {
  let p = identity
  show x + show (p 1)
}
pub effect fn main = println (
  renamedBound True + "\n" +
  renamedOuter True + "\n" +
  annotated True + "\n" +
  wrapped True + "\n" +
  direct True + "\n" +
  beside True + "\n" +
  atOuter 1 + "\n" +
  shown True + "\n" +
  reused True
)
"#,
        "1?\n1?\n1?\n1?\n1?\nTrue1?\n1?\n1\nTrue1\n",
    );
}
