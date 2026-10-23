use seseragi_backend_rust::emit_program_wasm;
use seseragi_driver::{compile_core_module, CompileInput};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("effect.ssrg", "wasm/effect", source))
        .unwrap_or_else(|diagnostics| panic!("{source}\n{diagnostics:?}"))
        .core_ir
}

#[test]
fn checked_cold_effect_uses_a_distinct_resumable_program_contract() {
    let module = checked(
        "import * as effects from \"std/effect\"\n\
         pub effect fn value input: String -> String fails String = effects.succeed input\n",
    );
    let artifact = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap();
    assert_eq!(artifact.metadata.abi, "seseragi-program-wasm-effect-v1");
    assert_eq!(artifact.metadata.exports[0].source_arity, 1);
    assert!(artifact
        .declarations
        .contains("OwnedEffect<string, string>"));
    assert!(artifact.declarations.contains("dispose(): Promise<void>"));
}

#[test]
fn a_hidden_arithmetic_defect_is_rejected_before_producing_an_effect_artifact() {
    let module = checked(
        "import * as effects from \"std/effect\"\n\
         fn hidden value: Int -> Int = value + 1\n\
         pub effect fn value input: String -> String = effects.succeed input\n",
    );
    let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
    assert!(error.message.contains("target-safe"), "{error}");
    let origin = error
        .origin
        .expect("the unsupported operation has a source location");
    assert_eq!(origin.source, "effect.ssrg");
    assert!(origin.start > 30);
}

#[test]
fn actual_aborting_runtime_drains_scopes_before_publishing_cancel_or_defect() {
    use std::{fs, process::Command};
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/program-wasm-effects.ssrg"
    ));
    let mut artifact = emit_program_wasm(
        &[&module],
        &module.module,
        &[
            "value",
            "rejection",
            "defect",
            "callback",
            "pending",
            "cleanup",
            "cancelCleanup",
            "saved",
            "staged",
            "transport",
            "late",
        ]
        .map(String::from),
    )
    .unwrap();
    assert_eq!(artifact.metadata.exports[8].source_arity, 2);
    artifact.source.push_str(r#"
fn token() -> u32 { PROGRAM.with(|s| u32::from_le_bytes(s.borrow().output.as_slice().try_into().unwrap())) }
fn output() -> Vec<u8> { PROGRAM.with(|s| s.borrow().output.clone()) }
fn text(value: &str) { let mut bytes=(value.len() as u32).to_le_bytes().to_vec(); bytes.extend_from_slice(value.as_bytes()); PROGRAM.with(|s| s.borrow_mut().input=bytes); }
fn make(index: u32) -> (u32,u32) { assert_eq!(ssrg_export(index),0); let root=token(); text("雪"); assert_eq!(ssrg_call(root),0); (root,token()) }
fn drain() -> Vec<u8> { for _ in 0..10000 { assert_eq!(ssrg_poll(1),0); let bytes=output(); if bytes[0]!=0 { assert_eq!(ssrg_runs(),0); return bytes; } } panic!("bounded test Effect failed to complete") }
fn messages(bytes:&[u8]) -> Vec<String> { assert_eq!(bytes[0],3); let mut input=&bytes[1..]; let n=program_u32(&mut input).unwrap(); let mut out=Vec::new(); for _ in 0..n { let n=program_u32(&mut input).unwrap(); out.push(std::str::from_utf8(program_take(&mut input,n as usize).unwrap()).unwrap().to_owned()); } assert!(input.is_empty()); out }
fn acquired() { for _ in 0..100 { if PROGRAM_RUN.with(|r| r.borrow().as_ref().unwrap().machine.context.scopes.iter().any(|scope| !scope.finalizers.is_empty())) { return; } assert_eq!(ssrg_poll(1),0); } panic!("resource not acquired") }
fn main() {
 assert_eq!(ssrg_init(),0);
 let (root,plan)=make(0);
 for _ in 0..2 { assert_eq!(ssrg_start(plan),0); assert_eq!(ssrg_start(plan),1); let bytes=drain(); assert_eq!(bytes[0],1); assert_eq!(&bytes[5..],"雪".as_bytes()); }
 ssrg_detach(root); ssrg_detach(plan);
 let (root,plan)=make(1); assert_eq!(ssrg_start(plan),0); ssrg_detach(plan); let bytes=drain(); assert_eq!(bytes[0],2); assert_eq!(&bytes[5..],"雪".as_bytes()); ssrg_detach(root);
 let (root,plan)=make(2); assert_eq!(ssrg_start(plan),0); assert_eq!(messages(&drain()),["雪"]); ssrg_detach(root); ssrg_detach(plan);
 let (root,plan)=make(3); assert_eq!(ssrg_start(plan),0); let bytes=drain(); assert_eq!(bytes[0],1); let callback=u32::from_le_bytes(bytes[1..].try_into().unwrap()); ssrg_detach(root); ssrg_detach(plan); text("!"); assert_eq!(ssrg_call(callback),0); assert_eq!(&output()[4..],"雪!".as_bytes()); ssrg_detach(callback);
 let (root,plan)=make(4); assert_eq!(ssrg_start(plan),0); acquired(); assert_eq!(ssrg_dispose(),usize::MAX); assert_eq!(ssrg_cancel(),0); assert_eq!(drain(),[4]); ssrg_detach(root); ssrg_detach(plan);
 let (root,plan)=make(5); assert_eq!(ssrg_start(plan),0); assert_eq!(messages(&drain()),["inner released","outer released"]); ssrg_detach(root); ssrg_detach(plan);
 let (root,plan)=make(6); assert_eq!(ssrg_start(plan),0); acquired(); assert_eq!(ssrg_cancel(),0); assert_eq!(messages(&drain()),["cancel released"]); ssrg_detach(root); ssrg_detach(plan);
 let (root,plan)=make(10); assert_eq!(ssrg_start(plan),0);
 let mut committed=false; for _ in 0..100 { if PROGRAM_RUN.with(|r| { let r=r.borrow(); let c=&r.as_ref().unwrap().machine.context; c.masked && c.scopes.last().is_some_and(|s| s.root) }) { committed=true; break; } assert_eq!(ssrg_poll(1),0); }
 assert!(committed); assert_eq!(ssrg_cancel(),0); let bytes=drain(); assert_eq!(bytes[0],1); assert_eq!(&bytes[5..],"雪".as_bytes()); ssrg_detach(root); ssrg_detach(plan);
 assert_eq!(ssrg_callbacks(),0); assert_eq!(ssrg_dispose(),0); assert_eq!(ssrg_dispose(),0); assert_eq!(ssrg_live(),0); println!("cold effects, cancellation and finalizers released");
}
"#);
    let directory =
        std::env::temp_dir().join(format!("seseragi-program-effect-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(directory);
    let input = scratch.0.join("program.rs");
    let binary = scratch.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, artifact.source).unwrap();
    let built = Command::new("rustc")
        .args([
            "--edition=2021",
            "-O",
            "-C",
            "panic=abort",
            "-C",
            "debuginfo=0",
        ])
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
    assert_eq!(
        run.stdout,
        b"cold effects, cancellation and finalizers released\n"
    );
}
