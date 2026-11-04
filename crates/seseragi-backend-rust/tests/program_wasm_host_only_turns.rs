use seseragi_backend_rust::{
    emit_program_wasm, PROGRAM_WASM_EFFECT_ABI, PROGRAM_WASM_HOST_EFFECT_ABI,
};
use seseragi_driver::{compile_core_module, CompileInput};
use std::{fs, process::Command};

#[path = "../../test-support/temporary.rs"]
mod temporary;

struct Scratch(std::path::PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn run_bounded(command: &mut Command, directory: &std::path::Path) -> std::process::Output {
    let stdout = directory.join("run.stdout");
    let stderr = directory.join("run.stderr");
    let mut child = command
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("unsupported host operation did not terminate");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    std::process::Output {
        status,
        stdout: fs::read(stdout).unwrap(),
        stderr: fs::read(stderr).unwrap(),
    }
}

#[test]
fn host_only_turns_report_defects_and_release_scopes_in_both_effect_abis() {
    for (source, abi) in [
        (
            "import * as effects from \"std/effect\"\npub effect fn value -> Unit = effects.succeed ()\n",
            PROGRAM_WASM_EFFECT_ABI,
        ),
        (
            "pub effect fn value callback: (Unit -> Task<Unit>) -> Unit = callback ()\n",
            PROGRAM_WASM_HOST_EFFECT_ABI,
        ),
    ] {
        let module = compile_core_module(CompileInput::new("signals.ssrg", "wasm/signals", source))
            .unwrap().core_ir;
        let artifact = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap();
        assert_eq!(artifact.metadata.abi, abi);
        let source = artifact.source + r#"
thread_local! { static RELEASED: Cell<usize> = const { Cell::new(0) }; }
fn release(_: &[V], _: &mut NativeContext) -> NativeOutcome {
    RELEASED.with(|count| count.set(count.get() + 1));
    Ok(V::Unit)
}
fn unsupported_host_operation(instruction: Instruction) -> V {
    let acquired = V::effect(Instruction::AcquireRelease, vec![
        V::leaf(vec![], |_, _| Ok(V::Unit)),
        V::fun(vec![], |_, _| V::leaf(vec![], release)),
    ]);
    let operation = V::effect(instruction, vec![V::Int(0)]);
    V::bind(acquired, V::fun(vec![operation], |captures, _| captures[0].clone()))
}
fn main() {
    assert_eq!(ssrg_init(), 0);
    let codec = program_codecs().iter().position(|code| matches!(code, ProgramCode::Effect(..))).unwrap();
    let operations = [
        (Instruction::ProcessSignalPull, "program ABI: process signals are unavailable in WASM"),
        (Instruction::ChildProcess(NativeChildOperation::Captured), "program ABI: child processes are unavailable in WASM"),
        (Instruction::ChildProcess(NativeChildOperation::Inherited), "program ABI: child processes are unavailable in WASM"),
        (Instruction::ChildProcess(NativeChildOperation::Event), "program ABI: child processes are unavailable in WASM"),
        (Instruction::ChildProcess(NativeChildOperation::Input), "program ABI: child processes are unavailable in WASM"),
        (Instruction::ChildProcess(NativeChildOperation::Close), "program ABI: child processes are unavailable in WASM"),
    ];
    let mut released = 0;
    for (instruction, message) in operations.into_iter().cycle().take(operations.len() * 2) {
        let token = program_register(unsupported_host_operation(instruction), codec).unwrap();
        assert_eq!(ssrg_start(token), 0);
        let mut completed = false;
        for _ in 0..1000 {
            assert_eq!(ssrg_poll(1), 0);
            let bytes = PROGRAM.with(|state| state.borrow().output.clone());
            if bytes[0] == 0 { continue; }
            assert_eq!(bytes[0], 3);
            let mut input = &bytes[1..];
            assert_eq!(program_u32(&mut input).unwrap(), 1);
            let length = program_u32(&mut input).unwrap() as usize;
            assert_eq!(program_take(&mut input, length).unwrap(), message.as_bytes());
            assert!(input.is_empty());
            completed = true;
            break;
        }
        assert!(completed, "unsupported host operation must terminate within bounded polling");
        assert_eq!(ssrg_runs(), 0);
        released += 1;
        assert_eq!(RELEASED.with(Cell::get), released);
        ssrg_detach(token);
    }
    assert_eq!(ssrg_callbacks(), 0);
    assert_eq!(ssrg_dispose(), 0);
    assert_eq!(ssrg_live(), 0);
}
"#;
        let scratch = Scratch(temporary::temporary_directory("seseragi-wasm-host-only-turns"));
        let rust = scratch.0.join("program.rs");
        let binary = scratch.0.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        fs::write(&rust, source).unwrap();
        let built = Command::new("rustc")
            .args(["--edition=2021", "-C", "panic=abort"])
            .arg(&rust).arg("-o").arg(&binary).output().unwrap();
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        let ran = run_bounded(Command::new(binary).env("PATH", ""), &scratch.0);
        assert!(ran.status.success(), "{ran:?}");
        assert!(ran.stdout.is_empty(), "{ran:?}");
        assert!(ran.stderr.is_empty(), "{ran:?}");
    }
}
