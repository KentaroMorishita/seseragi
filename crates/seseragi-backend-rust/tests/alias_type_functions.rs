use seseragi_backend_rust::emit_program_wasm;
use seseragi_driver::{compile_core_module, compile_project, CompileInput, ProjectModuleInput};
use seseragi_lowering::{CoreModule, CoreType};
use seseragi_project::ModuleGraph;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);

struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_program_wasm_probe(modules: &[&CoreModule], entry_module: &str, export: &str) -> String {
    let mut source = emit_program_wasm(modules, entry_module, &[export.into()])
        .unwrap_or_else(|error| panic!("{entry_module}::{export}: {error}"))
        .source;
    source.push_str(
        r#"
fn token() -> u32 {
    PROGRAM.with(|s| u32::from_le_bytes(s.borrow().output.as_slice().try_into().unwrap()))
}
fn response() -> String {
    PROGRAM.with(|s| {
        let state = s.borrow();
        let n = u32::from_le_bytes(state.output[..4].try_into().unwrap()) as usize;
        assert_eq!(n + 4, state.output.len());
        std::str::from_utf8(&state.output[4..]).unwrap().to_owned()
    })
}
fn main() {
    assert_eq!(ssrg_init(), 0);
    assert_eq!(ssrg_export(0), 0);
    let callback = token();
    PROGRAM.with(|s| s.borrow_mut().input.clear());
    assert_eq!(ssrg_call(callback), 0);
    print!("{}", response());
    ssrg_detach(callback);
    assert_eq!(ssrg_dispose(), 0);
    assert_eq!(ssrg_live(), 0);
}
"#,
    );
    let path = std::env::temp_dir().join(format!(
        "seseragi-typefn-wasm-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let scratch = Scratch(path);
    let input = scratch.0.join("program.rs");
    let binary = scratch.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, source).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-C", "debuginfo=0"])
        .arg(&input)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new(&binary).env("PATH", "").output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).unwrap()
}

#[test]
fn compile_time_type_functions_execute_in_program_wasm() {
    let source = r#"
type Pair<A, B> = | Pair (A, B)
alias Flip<A, B> = Pair<B, A>
alias Repeat<A> = Pair<A, A>
alias Erase<A> = Int
trait Tag2<F<_, _>> { fn tag2<A, B> value: F<A, B> -> String }
trait Tag1<F<_>> { fn tag1<A> value: F<A> -> String }
instance Tag2<Flip> { fn tag2<A, B> value: Pair<B, A> -> String = "flip" }
instance Tag1<Repeat> { fn tag1<A> value: Pair<A, A> -> String = "repeat" }
instance Tag1<Erase> { fn tag1<A> value: Int -> String = "erased" }
pub fn probe -> String =
  tag2<Flip, Int, Bool> (Pair (True, 42))
    + ":" + tag1<Repeat, Int> (Pair (1, 2))
    + ":" + tag1<Erase, Bool> 9
"#;
    let compiled = compile_core_module(CompileInput::new(
        "type-functions.ssrg",
        "fixture/type-functions",
        source,
    ))
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:#?}"));
    let output = run_program_wasm_probe(&[&compiled.core_ir], &compiled.core_ir.module, "probe");
    assert_eq!(output, "flip:repeat:erased");
}

#[test]
fn structural_alias_heads_execute_in_program_wasm() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/structural-alias-functions.ssrg"
    )
    .replace("pub effect fn main = println (probe ())", "");
    let compiled = compile_core_module(CompileInput::new(
        "structural.ssrg",
        "fixture/structural",
        &source,
    ))
    .unwrap_or_else(|e| panic!("{e:#?}"));
    let output = run_program_wasm_probe(&[&compiled.core_ir], &compiled.core_ir.module, "probe");
    assert_eq!(output, "43:7:9");
}

#[test]
fn reexported_public_alias_type_functions_execute_in_program_wasm() {
    let mut graph = ModuleGraph::new();
    graph.add_module("fixture::model".into(), []).unwrap();
    graph
        .add_module(
            "fixture::facade".into(),
            [("./model".into(), "fixture::model".into())],
        )
        .unwrap();
    graph
        .add_module(
            "fixture::main".into(),
            [("./facade".into(), "fixture::facade".into())],
        )
        .unwrap();
    let project = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "model.ssrg",
                "fixture::model",
                r#"
pub type Pair<A, B> = | Pair (A, B)
pub alias Flip<A, B> = Pair<B, A>
pub alias Reader<A> = Int -> A
pub trait Tag2<F<_, _>> { fn tag2<A, B> value: F<A, B> -> String }
pub trait Extract<F<_>> { fn extract<A> value: F<A> -> A }
instance Tag2<Flip> { fn tag2<A, B> value: Pair<B, A> -> String = "reexported-flip" }
instance Extract<Reader> { fn extract<A> value: (Int -> A) -> A = value 40 }
pub fn label2<F<_, _>, A, B> value: F<A, B> -> String where Tag2<F> = tag2<F, A, B> value
pub fn runExtract<F<_>, A> value: F<A> -> A where Extract<F> = extract<F, A> value
"#,
                "dist/model.js",
            ),
            ProjectModuleInput::new(
                "facade.ssrg",
                "fixture::facade",
                "pub import { Pair, Flip as Reversed, Reader as FnReader, Tag2, Extract, label2, runExtract } from \"./model\"\n",
                "dist/facade.js",
            ),
            ProjectModuleInput::new(
                "main.ssrg",
                "fixture::main",
                r#"
import { Pair, Reversed, FnReader, Tag2, Extract, label2, runExtract } from "./facade"
fn bump n: Int -> Int = n + 2
pub fn probe -> String =
  label2<Reversed, Int, Bool> (Pair (True, 1))
    + ":" + show (runExtract<FnReader, Int> bump)
"#,
                "dist/main.js",
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("{error:#?}"));
    let modules = project
        .order
        .iter()
        .map(|id| &project.modules[id].core_ir)
        .collect::<Vec<_>>();
    let output = run_program_wasm_probe(&modules, "fixture::main", "probe");
    assert_eq!(output, "reexported-flip:42");
}

#[test]
fn unapplied_type_function_in_public_export_signature_is_still_rejected() {
    let source = "pub fn value input: Int -> Int = input\n";
    let mut module = compile_core_module(CompileInput::new("main.ssrg", "fixture/main", source))
        .unwrap()
        .core_ir;
    module.functions[0].value_type = Some(CoreType::Function {
        parameter: Box::new(CoreType::TypeFunction {
            parameter_kinds: vec![0],
            body: Box::new(CoreType::TypeFunctionParameter { depth: 0, index: 0 }),
        }),
        result: Box::new(CoreType::Named {
            name: "Int".into(),
            arguments: vec![],
        }),
    });
    let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.origin.is_some(), "{error}");
}
