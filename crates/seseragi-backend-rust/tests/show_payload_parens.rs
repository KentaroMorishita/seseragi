#[path = "../../test-support/native_runtime.rs"]
mod link;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_module, CompileInput};
use std::{fs, path::PathBuf, process::Command};

const SOURCE: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/show-payload-parens.ssrg");
const EXPECTED: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/show-payload-parens.stdout");

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("seseragi-payload-parens-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

// Payload parentheses cover a Unit payload of a standard constructor and stop
// at the shared depth bound, where the elided payload is a bare "…" on both
// targets.
#[test]
fn payload_parentheses_match_typescript_for_unit_payloads_and_the_depth_bound() {
    let compiled = compile_module(CompileInput::new("parens.ssrg", "native/parens", SOURCE))
        .unwrap_or_else(|e| panic!("{e:?}"));
    let artifact = emit_rust_module(&compiled.core_ir, "probe").unwrap_or_else(|e| panic!("{e:?}"));
    let scratch = Scratch::new();
    let rust = scratch.path().join("program.rs");
    let binary = scratch
        .path()
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    link::write_program(&rust, artifact.source).unwrap();
    let build = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(&rust)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let native = Command::new(&binary).env("PATH", "").output().unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let ts = scratch.path().join("program.ts");
    fs::write(
        &ts,
        format!(
            "{}\nconsole.log(probe());\n",
            compiled.generated.typescript.replace(
                "\"@seseragi/runtime/",
                &format!("\"{}/runtime/ts/src/", root.display())
            )
        ),
    )
    .unwrap();
    let output = Command::new("bun").arg(&ts).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), EXPECTED);
    assert_eq!(String::from_utf8_lossy(&native.stdout), EXPECTED);
    let elided = EXPECTED.matches('…').count();
    assert_eq!(elided, 7, "each deep value is elided once");
    assert!(!EXPECTED.contains("(…)"));
}
