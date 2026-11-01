#![allow(dead_code)]

use seseragi_backend_rust::{emit_rust_module, RustArtifact};
use seseragi_driver::{compile_core_module, CompileInput};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);

pub struct Scratch(PathBuf);
impl Scratch {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "seseragi-rust-target-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub fn compile(&self, source: &str) -> PathBuf {
        let input = self.0.join("generated.rs");
        let binary = self.0.join(if cfg!(windows) {
            "native.exe"
        } else {
            "native"
        });
        fs::write(&input, source).unwrap();
        let result = Command::new("rustc")
            .args(["--edition=2021", "-O", "-o"])
            .arg(&binary)
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        binary
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub fn emit(source: &str) -> RustArtifact {
    let core = compile_core_module(CompileInput::new("main.ssrg", "native/main", source)).unwrap();
    emit_rust_module(&core.core_ir, "probe").unwrap()
}

pub fn run(source: &str, expected: &str) {
    let artifact = emit(source);
    let scratch = Scratch::new();
    // Keep static dictionaries between invocations, but ensure their retained
    // graph is bounded. Explicit teardown must release the entire runtime.
    let main = artifact.source.rfind("fn main()").unwrap();
    let generated = format!("{}\nfn main() {{ let mut warm = None; for n in 0..10000 {{ let result = {}(vec![]).apply(V::Unit); if n == 0 {{ println!(\"{{}}\", result.render()); }} drop(result); let live = LIVE.with(Cell::get); if let Some(first) = warm {{ assert_eq!(live, first); }} else {{ warm = Some(live); }} }} clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
}
pub fn rust_id(name: &str) -> String {
    format!(
        "v_{}",
        name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    )
}
