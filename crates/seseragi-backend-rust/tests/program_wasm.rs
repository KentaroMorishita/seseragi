use seseragi_backend_rust::{emit_program_wasm, ProgramCodec};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreModule, CoreType};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("program.ssrg", "wasm/program", source))
        .unwrap_or_else(|diagnostics| panic!("{source}\n{diagnostics:?}"))
        .core_ir
}
fn fixture() -> CoreModule {
    checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/program-wasm.ssrg"
    ))
}

#[test]
fn public_contract_retains_source_stages_and_returned_function_types() {
    let module = fixture();
    let names = [
        "label", "validate", "enabled", "optional", "staged", "ready",
    ]
    .map(String::from);
    let artifact = emit_program_wasm(&[&module], &module.module, &names).unwrap();
    assert_eq!(
        artifact
            .metadata
            .exports
            .iter()
            .map(|e| e.source_arity)
            .collect::<Vec<_>>(),
        [2, 1, 1, 1, 1, 1]
    );
    for export in &artifact.metadata.exports {
        assert!(matches!(
            artifact.metadata.codecs[export.codec],
            ProgramCodec::Function { .. }
        ));
        assert_eq!(export.symbol, format!("wasm/program::{}", export.name));
        assert_eq!(export.origin.source, "program.ssrg");
    }
    assert!(!artifact.source.contains("fn main()"));
    assert!(artifact
        .declarations
        .contains("OwnedCallback<string, OwnedCallback<number, OwnedCallback<number, string>>>"));
    assert!(artifact.declarations.contains("'Left'"));
    assert!(artifact
        .javascript
        .contains("checked contract does not match"));
    assert_eq!(artifact.metadata.trap_policy, "retire-instance");
}

#[test]
fn refuses_unproved_public_types_and_staging_before_any_artifact() {
    let original = checked("pub fn value input: Int -> Int = input\n");
    for mutation in 0..11 {
        let mut module = original.clone();
        let function = &mut module.functions[0];
        match mutation {
            0 => function.value_type = None,
            1 => function.value_type = Some(CoreType::Hole),
            2 => function.parameters.clear(),
            3 => function.parameters[0].kind = "dictionary".into(),
            4 => {
                function.parameters[0].type_ref = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "other::Int".into(),
                    arguments: vec![],
                }
            }
            5 => function.visibility = seseragi_syntax::Visibility::Private,
            6 => function.is_effect = true,
            7 => function.parameters.push(function.parameters[0].clone()),
            8 => {
                function.value_type = Some(CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                })
            }
            9 => {
                if let CoreType::Function { result, .. } = function.value_type.as_mut().unwrap() {
                    **result = CoreType::ExternalNamed {
                        name: "Int".into(),
                        canonical: "other::Int".into(),
                        arguments: vec![],
                    };
                }
            }
            _ => {
                function.body = CoreExpr::String {
                    value: "wrong checked result".into(),
                    origin: function.origin.clone(),
                }
            }
        }
        let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some(), "mutation{mutation}: {error}");
    }
    for source in [
        "pub fn value<A> input: A -> String where Show<A> = show input\n",
        "import * as refs from \"std/ref\"\npub effect fn value<A> input: A -> A = do { cell <- refs.make input; refs.get cell }\n",
        "import * as decimal from \"std/decimal\"\npub fn value input: decimal.DecimalContext -> Int = 1\n",
        "import * as decimal from \"std/decimal\"\npub fn value input: {x: Int, y: decimal.DecimalContext} -> Int = input.x\n",
        "import * as decimal from \"std/decimal\"\npub fn value input: Array<decimal.DecimalContext> -> Int = 1\n",
    ] {
        let module = checked(source);
        let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
        assert!(error.origin.is_some(), "{source}: {error}");
    }
    // A function argument is a synchronous host function, so it needs the
    // host function ABI.
    let module = checked("pub fn value input: (Int -> Int) -> Int = input 1\n");
    let artifact = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap();
    assert_eq!(
        artifact.metadata.abi,
        "seseragi-program-wasm-host-effect-v3"
    );
    // A user type named `Int` is that ADT, never the prelude Int.
    let module = checked("type Int = | Number\npub fn value input: Int -> Int = input\n");
    let artifact = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap();
    assert!(artifact.metadata.codecs.iter().any(|codec| matches!(
        codec,
        seseragi_backend_rust::ProgramCodec::Adt { variants, .. }
            if variants.len() == 1 && variants[0].tag == "Number" && variants[0].payload.is_none()
    )));
    assert!(!artifact
        .metadata
        .codecs
        .contains(&seseragi_backend_rust::ProgramCodec::Int));
}

#[test]
fn a_cold_effect_in_an_unselected_declaration_still_requires_another_abi() {
    let module = checked("import * as effects from \"std/effect\"\nfn unused -> Effect<{}, Never, Int> = effects.succeed 1\npub fn value input: Int -> Int = input\n");
    let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
    assert!(
        error.message.contains("separate unwind/cancellation ABI"),
        "{error}"
    );
    assert!(error.origin.is_some());
}

static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn actual_generated_runtime_keeps_detached_and_reused_callback_roots_correct() {
    let module = fixture();
    let mut source = emit_program_wasm(
        &[&module],
        &module.module,
        &["label".into(), "validate".into(), "transport".into()],
    )
    .unwrap()
    .source;
    // Native execution of the exact generated runtime independently checks the
    // root graph. Real WASM/JS transport is covered by the CLI boundary suite.
    source.push_str(r#"
fn token() -> u32 { PROGRAM.with(|s| u32::from_le_bytes(s.borrow().output.as_slice().try_into().unwrap())) }
fn argument(bytes: &[u8]) { PROGRAM.with(|s| s.borrow_mut().input = bytes.to_vec()); }
fn text(value: &str) { let mut bytes = (value.len() as u32).to_le_bytes().to_vec(); bytes.extend_from_slice(value.as_bytes()); argument(&bytes); }
fn response() -> String { PROGRAM.with(|s| { let state = s.borrow(); let n = u32::from_le_bytes(state.output[..4].try_into().unwrap()) as usize; assert_eq!(n + 4,state.output.len()); std::str::from_utf8(&state.output[4..]).unwrap().to_owned() }) }
fn main() {
 assert_eq!(ssrg_init(),0);
 assert_eq!(ssrg_export(0),0); let root=token();
 text("合計: "); assert_eq!(ssrg_call(root),0); let prefix=token();
 argument(&40_i64.to_le_bytes()); assert_eq!(ssrg_call(prefix),0); let callback=token();
 ssrg_detach(prefix); ssrg_detach(root);
 argument(&2_i64.to_le_bytes()); assert_eq!(ssrg_call(callback),0); assert_eq!(response(),"合計: 42");
 let live=ssrg_live();
 for _ in 0..1000 { assert_eq!(ssrg_call(callback),0); assert_eq!(response(),"合計: 42"); assert_eq!(ssrg_live(),live); }
 argument(&9_007_199_254_740_992_i64.to_le_bytes()); assert_eq!(ssrg_call(callback),1); assert_eq!(ssrg_live(),live);
 ssrg_detach(callback); assert_eq!(ssrg_call(callback),1); assert_eq!(ssrg_callbacks(),0);
 assert_eq!(ssrg_export(0),0); assert!(token()>callback); ssrg_detach(token());
 assert_eq!(ssrg_export(1),0); let validate=token(); argument(&2_i64.to_le_bytes()); assert_eq!(ssrg_call(validate),0);
 PROGRAM.with(|s| assert_eq!(s.borrow().output[0],1));
 argument(&(-1_i64).to_le_bytes()); assert_eq!(ssrg_call(validate),0); PROGRAM.with(|s| assert_eq!(s.borrow().output[0],0));
 assert_eq!(ssrg_export(2),0); let transport=token();
 let mut bytes=("絵😀".len() as u32).to_le_bytes().to_vec(); bytes.extend_from_slice("絵😀".as_bytes());
 bytes.push(1); bytes.extend_from_slice(&5_i64.to_le_bytes()); bytes.push(0);
 bytes.extend_from_slice(&("失敗".len() as u32).to_le_bytes()); bytes.extend_from_slice("失敗".as_bytes());
 argument(&bytes); assert_eq!(ssrg_call(transport),0); PROGRAM.with(|s| assert_eq!(s.borrow().output,bytes));
 let mut trailing=bytes.clone(); trailing.push(0); argument(&trailing); assert_eq!(ssrg_call(transport),1);
 assert_eq!(ssrg_dispose(),0); assert_eq!(ssrg_live(),0); assert_eq!(ssrg_callbacks(),0); assert_eq!(ssrg_dispose(),0);
 assert_eq!(ssrg_call(validate),1); assert_eq!(ssrg_init(),1); println!("program roots released");
}
"#);
    let path = std::env::temp_dir().join(format!(
        "seseragi-program-abi-{}-{}",
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
    assert_eq!(run.stdout, b"program roots released\n");
}

#[test]
fn a_type_nested_in_itself_with_other_arguments_is_a_diagnostic() {
    // Each level adds an Array, so the codecs would never end.
    let module = checked("type Nested<A> =\n  | End\n  | Next (Nested<Array<A>>)\n\npub fn value input: Nested<Int> -> Nested<Int> = input\n");
    let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(
        error.message.contains("unsupported public codec"),
        "{error}"
    );
    assert!(error.origin.is_some());
    // A finite change of arguments still has codecs.
    let module = checked("type Pair<A> =\n  | Pair (A, Maybe<Pair<Int>>)\n\npub fn value input: Pair<String> -> Pair<String> = input\n");
    emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap();
}

#[test]
fn a_type_nested_in_itself_many_times_by_hand_has_codecs() {
    // Each level is written in the source and is smaller than the one
    // around it, so twenty levels of Box still have a finite set of codecs.
    let deep = (0..20).fold("Int".to_string(), |inner, _| format!("Box<{inner}>"));
    let module = checked(&format!(
        "type Box<A> =\n  | Box A\n\nstruct Holder {{ value: {deep} }}\n\npub fn value input: {deep} -> {deep} = input\npub fn held input: Holder -> Holder = input\n"
    ));
    emit_program_wasm(&[&module], &module.module, &["value".into(), "held".into()]).unwrap();
}
