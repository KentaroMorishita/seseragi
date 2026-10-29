use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
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
        let path = std::env::temp_dir().join(format!(
            "seseragi-closed-display-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn run(&self, source: &str) -> Vec<u8> {
        let file = self.0.join("program.rs");
        let exe = self
            .0
            .join(format!("program{}", std::env::consts::EXE_SUFFIX));
        fs::write(&file, source).unwrap();
        let output = Command::new("rustc")
            .args(["--edition=2021", "-O"])
            .arg(&file)
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = Command::new(exe).env("PATH", "").output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn symbol(name: &str) -> String {
    format!(
        "v_{}",
        format!("display/main::{name}")
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

#[test]
fn standard_tuple_factories_stay_lazy_reuse_only_the_dictionary_and_clear_every_root() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/tuple-show-kernel.ssrg");
    let core = compile_core_module(CompileInput::new("main.ssrg", "display/main", source)).unwrap();
    let artifact = emit_rust_module(&core.core_ir, "probe").unwrap();
    assert!(artifact.metadata.closed_dictionary_sites > 0);
    assert!(artifact.source.contains("closed-standard-display:"));
    // Observe the actual factory separately from timing. Keep all generated
    // callable/runtime bodies and add only the counter plus a test entrypoint.
    let signature = "fn native_tuple_display_dictionary(elements: Vec<V>, debug: bool) -> V {";
    assert_eq!(artifact.source.matches(signature).count(), 1);
    let mut rust = artifact.source[..artifact.source.rfind("\nfn main()").unwrap()].replace(
        signature,
        &format!("{signature} FACTORIES.with(|n| n.set(n.get()+1));"),
    );
    rust.push_str(&format!(
        r#"
thread_local! {{static FACTORIES: Cell<usize> = const {{Cell::new(0)}};}}
fn main() {{
    let kernel={kernel}(vec![]);
    FACTORIES.with(|n|assert_eq!(n.get(),0));
    for value in 41..51 {{
        let result=kernel.apply(V::Int(value));
        assert_eq!(result.render(),format!("({{}}, {{}}, True)",value,value+1));
        drop(result);
        FACTORIES.with(|n|assert_eq!(n.get(),1));
    }}
    clear_dictionaries();
    let result=kernel.apply(V::Int(99));
    assert_eq!(result.render(),"(99, 100, True)");
    FACTORIES.with(|n|assert_eq!(n.get(),2));
    drop(result); drop(kernel);clear_dictionaries();clear_globals();
    assert_eq!(LIVE.with(Cell::get),0);
}}
"#,
        kernel = symbol("kernel")
    ));
    Scratch::new().run(&rust);
}

#[test]
fn user_and_parameter_dependent_display_factories_keep_the_original_path() {
    for (source,expected) in [
        (include_str!("../../../examples/spec/fixtures/rust-backend/tuple-nominal.ssrg"),include_str!("../../../examples/spec/fixtures/rust-backend/tuple-nominal.stdout")),
        ("fn selected<A> value: A -> String where Show<A> = show (value, True)\npub fn probe -> String = selected 41", "(41, True)\n"),
    ] {
        let core=compile_core_module(CompileInput::new("main.ssrg","display/main",source)).unwrap();
        let artifact=emit_rust_module(&core.core_ir,"probe").unwrap();
        assert!(!artifact.source.contains("closed-standard-display:"));
        assert_eq!(Scratch::new().run(&artifact.source),expected.as_bytes());
    }
}
