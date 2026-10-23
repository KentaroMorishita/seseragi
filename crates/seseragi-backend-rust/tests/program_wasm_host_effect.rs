use seseragi_backend_rust::{emit_program_wasm, ProgramCodec, PROGRAM_WASM_HOST_EFFECT_ABI};
use seseragi_driver::{compile_core_module, CompileInput};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new("host-effect.ssrg", "wasm/host", source))
        .unwrap_or_else(|diagnostics| panic!("{source}\n{diagnostics:?}"))
        .core_ir
}

const TRANSFER: &str = "import * as effects from \"std/effect\"\n\
pub effect fn transfer request: (String -> Effect<{}, String, String>) -> release: (String -> Effect<{}, Never, Unit>) -> input: String -> String fails String = effects.scoped $ do {\n\
resource <- effects.acquireRelease (request (\"open:\" + input)) release\n\
result <- request (\"read:\" + resource)\n\
effects.succeed (\"processed:\" + result)\n\
}\n";

#[test]
fn checked_host_effect_callbacks_keep_one_source_argument_and_a_separate_transport() {
    let module = checked(TRANSFER);
    let artifact = emit_program_wasm(&[&module], &module.module, &["transfer".into()]).unwrap();
    assert_eq!(artifact.metadata.abi, PROGRAM_WASM_HOST_EFFECT_ABI);
    assert_eq!(artifact.metadata.exports[0].source_arity, 3);
    assert_eq!(
        artifact
            .metadata
            .codecs
            .iter()
            .filter(|code| matches!(code, ProgramCodec::HostEffect { .. }))
            .count(),
        2
    );
    assert!(artifact
        .declarations
        .contains("HostEffect<string, string, string>"));
    assert!(artifact
        .declarations
        .contains("HostEffect<string, never, undefined>"));
    assert!(artifact.declarations.contains("hostCallbacks: number"));
}

#[test]
fn synchronous_multi_argument_and_generic_host_functions_do_not_gain_a_codec() {
    for source in [
        "import * as effects from \"std/effect\"\npub effect fn entry callback: (String -> String) -> input: String -> String = effects.succeed (callback input)\n",
        "import * as effects from \"std/effect\"\npub effect fn entry callback: (String -> String -> Effect<{}, Never, String>) -> input: String -> String = callback input input\n",
        "import * as effects from \"std/effect\"\npub effect fn entry<A> callback: (A -> Effect<{}, Never, A>) -> input: A -> A = callback input\n",
    ] {
        let module = checked(source);
        let error = emit_program_wasm(&[&module], &module.module, &["entry".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "host-effect.ssrg");
    }
}

#[test]
fn host_effect_codec_requires_closed_canonical_row_and_channels() {
    use seseragi_lowering::CoreType;
    let original = checked(TRANSFER);
    emit_program_wasm(&[&original], &original.module, &["transfer".into()]).unwrap();
    for mutation in 0..4 {
        let mut module = original.clone();
        let declaration = module
            .functions
            .iter_mut()
            .find(|function| function.symbol == "wasm/host::transfer")
            .unwrap();
        let CoreType::Function { result, .. } = &mut declaration.parameters[0].type_ref else {
            panic!("checked request callback must have one source stage")
        };
        let (owner, arguments) = match result.as_mut() {
            CoreType::Named { name, arguments } => (name, arguments),
            CoreType::ExternalNamed {
                canonical,
                arguments,
                ..
            } => (canonical, arguments),
            _ => panic!("checked callback must return an Effect"),
        };
        match mutation {
            0 => {
                let CoreType::Record { closed, .. } = &mut arguments[0] else {
                    panic!("checked requirements row")
                };
                *closed = false;
            }
            1 => arguments[0] = CoreType::Hole,
            2 => *owner = "fixture/nominal::Effect".into(),
            3 => arguments.push(CoreType::Hole),
            _ => unreachable!(),
        }
        // Keep the declaration's source parameter and public signature in
        // agreement. Neither an open/Hole row nor a nominal Effect with the
        // same display spelling may become a runtime transport contract.
        let CoreType::Function { parameter, .. } = declaration.value_type.as_mut().unwrap() else {
            panic!("checked public source signature")
        };
        **parameter = declaration.parameters[0].type_ref.clone();
        let error = emit_program_wasm(&[&module], &module.module, &["transfer".into()])
            .expect_err("unproven host Effect contract produced a program artifact");
        let origin = error.origin.expect("rejection retains source location");
        assert_eq!(origin.source, "host-effect.ssrg");
        assert!(origin.end > origin.start);
    }
}

#[test]
fn host_leaf_suspends_before_register_and_masked_cleanup_releases_its_last_leases() {
    use std::{fs, process::Command};
    let module = checked(TRANSFER);
    let mut artifact = emit_program_wasm(&[&module], &module.module, &["transfer".into()]).unwrap();
    artifact.source.push_str(r#"
fn result_bytes() -> Vec<u8> { PROGRAM.with(|s| s.borrow().output.clone()) }
fn root_token() -> u32 { u32::from_le_bytes(result_bytes().try_into().unwrap()) }
fn put(bytes: Vec<u8>) { PROGRAM.with(|s| s.borrow_mut().input = bytes); }
fn text(value: &str) { let mut bytes=(value.len() as u32).to_le_bytes().to_vec(); bytes.extend_from_slice(value.as_bytes()); put(bytes); }
fn plan(first:u32, second:u32) -> u32 {
 assert_eq!(ssrg_export(0),0); let root=root_token();
 put(first.to_le_bytes().to_vec()); assert_eq!(ssrg_call(root),0); let partial=root_token();
 put(second.to_le_bytes().to_vec()); assert_eq!(ssrg_call(partial),0); let applied=root_token();
 text("snow"); assert_eq!(ssrg_call(applied),0); let plan=root_token();
 ssrg_detach(root); ssrg_detach(partial); ssrg_detach(applied); plan
}
fn poll() -> Vec<u8> { for _ in 0..1000 { assert_eq!(ssrg_poll(1),0); let bytes=result_bytes(); if bytes[0] != 0 { return bytes; } } panic!("bounded host checkpoint"); }
fn request(token:u32, masked:u8, argument:&str) {
 let bytes=poll(); assert_eq!(bytes[0],5); assert_eq!(u32::from_le_bytes(bytes[1..5].try_into().unwrap()),token);
 assert_eq!(bytes[9],masked); assert_eq!(&bytes[14..],argument.as_bytes());
 // Repeated polling must neither call the host twice nor advance a placeholder.
 assert_eq!(ssrg_poll(1),0); assert_eq!(result_bytes(),bytes);
}
fn main() {
 assert_eq!(ssrg_init(),0);
 let root=plan(1,2); assert_eq!(ssrg_start(root),0); ssrg_detach(root);
 request(1,1,"open:snow");
 assert!(PROGRAM_RUN.with(|slot| { let slot=slot.borrow(); let run=slot.as_ref().unwrap(); matches!(run.machine.state,Some(NativeState::Return(Ok(V::Unit)))) && run.machine.context.scopes.iter().all(|scope|scope.finalizers.is_empty()) }));
 assert_eq!(ssrg_hosts(),0); assert_eq!(u32::from_le_bytes(result_bytes()[..4].try_into().unwrap()),2);
 // Cancellation while acquisition is pending waits for its real resource.
 assert_eq!(ssrg_cancel(),0); text("actual-resource"); assert_eq!(ssrg_host_reply(1),0);
 request(2,1,"actual-resource");
 put(Vec::new()); assert_eq!(ssrg_host_reply(1),0); assert_eq!(poll(),[4]);
 assert_eq!(ssrg_hosts(),0); assert_eq!(result_bytes(),0u32.to_le_bytes());
 let root=plan(3,4); assert_eq!(ssrg_start(root),0); ssrg_detach(root);
 request(3,1,"open:snow"); text("resource"); assert_eq!(ssrg_host_reply(1),0);
 request(3,0,"read:resource"); text("data"); assert_eq!(ssrg_host_reply(1),0);
 request(4,1,"resource"); put(Vec::new()); assert_eq!(ssrg_host_reply(1),0);
 let bytes=poll(); assert_eq!(bytes[0],1); assert_eq!(&bytes[5..],b"processed:data");
 assert_eq!(ssrg_hosts(),0); assert_eq!(result_bytes(),0u32.to_le_bytes());
 let root=plan(5,6); assert_eq!(ssrg_start(root),0); ssrg_detach(root);
 request(5,1,"open:snow"); text("resource"); assert_eq!(ssrg_host_reply(1),0);
 request(5,0,"read:resource"); put(vec![255]); assert_eq!(ssrg_host_reply(1),0);
 request(6,1,"resource"); put(Vec::new()); assert_eq!(ssrg_host_reply(1),0);
 assert_eq!(poll()[0],3); // Bad typed response is a Defect AFTER release.
 assert_eq!(ssrg_hosts(),0); assert_eq!(result_bytes(),0u32.to_le_bytes());
 assert_eq!(ssrg_callbacks(),0); assert_eq!(ssrg_dispose(),0); assert_eq!(ssrg_live(),0);
 println!("host suspension, acquisition masking, typed resume, cleanup and leases verified");
}
"#);
    let directory =
        std::env::temp_dir().join(format!("seseragi-host-effect-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(directory);
    let source = scratch.0.join("program.rs");
    let binary = scratch.0.join("program");
    fs::write(&source, artifact.source).unwrap();
    fn bounded(
        command: &mut Command,
        directory: &std::path::Path,
        label: &str,
    ) -> std::process::Output {
        let out = directory.join(format!("{label}.out"));
        let err = directory.join(format!("{label}.err"));
        let mut child = command
            .stdout(fs::File::create(&out).unwrap())
            .stderr(fs::File::create(&err).unwrap())
            .spawn()
            .unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(60);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if std::time::Instant::now() >= until {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("bounded {label} timed out")
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        std::process::Output {
            status,
            stdout: fs::read(out).unwrap(),
            stderr: fs::read(err).unwrap(),
        }
    }
    let built = bounded(
        Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()))
            .args([
                "--edition=2021",
                "-O",
                "-C",
                "panic=abort",
                "-C",
                "debuginfo=0",
            ])
            .arg(source)
            .arg("-o")
            .arg(&binary),
        &scratch.0,
        "compile",
    );
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let ran = bounded(Command::new(binary).env("PATH", ""), &scratch.0, "run");
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    assert_eq!(
        ran.stdout,
        b"host suspension, acquisition masking, typed resume, cleanup and leases verified\n"
    );
}
