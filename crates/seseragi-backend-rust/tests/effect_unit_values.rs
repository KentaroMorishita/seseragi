use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn execute(source: &str, entry: &str, stdout: &str, exit: i32, trace: Option<&str>) {
    let checked =
        compile_core_module(CompileInput::new("unit-雪🙂.ssrg", "fixture/unit", source)).unwrap();
    let artifact = emit_rust_module(&checked.core_ir, entry).expect("checked Unit success");
    let directory = Scratch(std::env::temp_dir().join(format!(
        "seseragi-unit-value-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&directory.0).unwrap();
    let input = directory.0.join("program.rs");
    let binary = directory.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, artifact.source).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = Command::new(binary)
        .current_dir(&directory.0)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), stdout);
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    if let Some(expected) = trace {
        assert_eq!(
            fs::read_to_string(directory.0.join("trace.txt")).unwrap(),
            expected
        );
    }
}

#[test]
fn literal_stored_annotated_and_shadowed_units_execute_as_the_same_value() {
    execute("pub effect fn main = succeed ()\n", "main", "", 0, None);
    for body in ["succeed ()", "{ let unit = (); succeed unit }"] {
        execute(&format!("struct Unit {{value: Int}}\neffect fn genuine = {body}\npub effect fn main = do {{ value <- genuine (); println (show value) }}\n"), "main", "()\n", 0, None);
    }
    execute("effect fn genuine -> Unit = succeed ()\npub effect fn main = do { value <- genuine (); println (show value) }\n", "main", "()\n", 0, None);
    execute("effect fn genuine<Unit> value: Unit = succeed ()\npub effect fn main = do { value <- genuine 7; println (show value) }\n", "main", "()\n", 0, None);
}

#[test]
fn literal_unit_keeps_cold_ref_replay_and_scope_finalization() {
    execute(
        include_str!("../../../examples/spec/fixtures/rust-backend/effect-unit-values.ssrg"),
        "main",
        include_str!("../../../examples/spec/fixtures/rust-backend/effect-unit-values.stdout"),
        0,
        None,
    );
}

#[test]
fn unit_success_keeps_real_file_finalizers_on_failure_and_defect() {
    for (entry, exit) in [("failed", 1), ("crash", 70)] {
        execute(
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/effect-unit-finalizers.ssrg"
            ),
            entry,
            "",
            exit,
            Some("acquire\nbody: ()\nrelease\n"),
        );
    }
}

#[test]
fn unit_success_refuses_missing_value_and_forged_result_owner_or_kind() {
    let source = "// 雪🙂\npub effect fn literal<Unit> value: Unit = succeed ()\npub effect fn main = literal 7\n";
    let original = compile_core_module(CompileInput::new(
        "unit-owner-雪🙂.ssrg",
        "fixture/unit-owner",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&original, "main").unwrap();
    for mutation in 0..7 {
        let mut changed = original.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::literal"))
            .unwrap();
        let CoreExpr::EffectOperation {
            arguments,
            success,
            operation,
            ..
        } = &mut function.body
        else {
            panic!("checked Unit operation")
        };
        match mutation {
            0 => arguments.clear(),
            1 => {
                *success = CoreType::ExternalNamed {
                    name: "Unit".into(),
                    canonical: "forged::Unit".into(),
                    arguments: vec![],
                }
            }
            2 => {
                *success = CoreType::ExternalNamed {
                    name: "Unit".into(),
                    canonical: "std/prelude::Unit".into(),
                    arguments: vec![CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    }],
                }
            }
            3 => *success = CoreType::Hole,
            4 => {
                *success = CoreType::Named {
                    name: "Unit".into(),
                    arguments: vec![],
                }
            }
            5 => *operation = "forged::succeed".into(),
            6 => arguments.push(arguments[0].clone()),
            _ => unreachable!(),
        }
        let error =
            emit_rust_module(&changed, "main").expect_err("unproved Unit success must not emit");
        assert_eq!(error.code, "SES-RUST-0001");
        let origin = error.origin.expect("source-positioned refusal");
        assert_eq!(origin.source, "unit-owner-雪🙂.ssrg");
        assert_eq!(&source[origin.start..origin.end], "succeed ()");
    }
}
