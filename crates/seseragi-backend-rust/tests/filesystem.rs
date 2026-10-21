use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use std::{fs, process::Command};

#[test]
fn file_transform_preserves_utf8_input_and_typed_failures() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-transform.ssrg");
    let core = compile_core_module(CompileInput::new("filesystem.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "filesystem-sync"));
    let temp =
        std::env::temp_dir().join(format!("seseragi-filesystem-native-{}", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fs::create_dir(&temp).unwrap();
    let _cleanup = Cleanup(temp.clone());
    let rust = temp.join("program.rs");
    fs::write(&rust, artifact.source).unwrap();
    let binary = temp.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let compiled = Command::new("rustc")
        .args(["--edition=2021", "-o"])
        .arg(&binary)
        .arg(&rust)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    for (name, input, error) in [
        ("utf8", Some("初期値\n42\n".as_bytes()), None),
        ("missing", None, Some("FileNotFound")),
        (
            "invalid",
            Some(&[0x61, 0xff][..]),
            Some("InvalidUtf8 { offset: 1 }"),
        ),
        ("directory", Some(&b"hello"[..]), Some("FileFailure")),
    ] {
        let cwd = temp.join(name);
        fs::create_dir(&cwd).unwrap();
        if let Some(input) = input {
            fs::write(cwd.join("input.txt"), input).unwrap();
        }
        let old = b"unchanged old output\nunchanged old output\n";
        if name == "directory" {
            fs::create_dir(cwd.join("output.txt")).unwrap();
        } else {
            fs::write(cwd.join("output.txt"), old).unwrap();
        }
        let output = Command::new(&binary)
            .current_dir(&cwd)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("managed-live=0"), "{name}: {stderr}");
        assert!(output.stdout.is_empty());
        if let Some(error) = error {
            assert_eq!(output.status.code(), Some(1), "{name}: {stderr}");
            assert!(stderr.contains(error), "{name}: {stderr}");
            if name != "directory" {
                assert_eq!(fs::read(cwd.join("output.txt")).unwrap(), old);
            }
        } else {
            assert!(output.status.success(), "{stderr}");
            assert_eq!(
                fs::read(cwd.join("output.txt")).unwrap(),
                [b"processed:\n".as_slice(), input.unwrap()].concat()
            );
        }
        if let Some(input) = input {
            assert_eq!(fs::read(cwd.join("input.txt")).unwrap(), input);
        }
    }
}

#[test]
fn filesystem_rejects_forged_owners_results_requirements_and_stages() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-transform.ssrg");
    let core = compile_core_module(CompileInput::new("filesystem.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
    fn alter(value: &mut serde_json::Value, case: usize) -> bool {
        if let Some(object) = value.as_object_mut() {
            let callee = object.get("callee").and_then(|x| x.as_str()).unwrap_or("");
            if (case <= 5 && callee == "std/fs::readTextUtf8")
                || (case == 6 && callee == "std/path::child")
                || (case == 7 && callee == "std/effect::fromEither")
            {
                match case {
                    0 => {
                        object["arguments"][0]["type"]["canonical"] =
                            serde_json::json!("counterfeit::Path")
                    }
                    1 => {
                        object["failure"]["canonical"] =
                            serde_json::json!("counterfeit::FileTextError")
                    }
                    2 => {
                        object["success"] =
                            serde_json::json!({"kind":"named","name":"Bool","arguments":[]})
                    }
                    3 => {
                        object["requirements"]["fields"][0]["type"]["canonical"] =
                            serde_json::json!("counterfeit::FileSystem")
                    }
                    4 => object["requirements"]["closed"] = serde_json::json!(false),
                    5 => {
                        object
                            .get_mut("arguments")
                            .unwrap()
                            .as_array_mut()
                            .unwrap()
                            .clear();
                    }
                    6 => object["type"] = serde_json::json!({"kind":"hole"}),
                    7 => {
                        object["failure"]["canonical"] = serde_json::json!("counterfeit::PathError")
                    }
                    _ => unreachable!(),
                }
                return true;
            }
            return object.values_mut().any(|v| alter(v, case));
        }
        value
            .as_array_mut()
            .is_some_and(|values| values.iter_mut().any(|v| alter(v, case)))
    }
    for case in 0..8 {
        let mut changed = core.clone();
        let mut found = false;
        for function in &mut changed.functions {
            let mut value = serde_json::to_value(&function.body).unwrap();
            if alter(&mut value, case) {
                function.body = serde_json::from_value(value).unwrap();
                found = true;
                break;
            }
        }
        assert!(found, "missing mutation {case}");
        let error = emit_rust_module(&changed, "main").unwrap_err();
        assert!(error.origin.is_some(), "mutation {case}: {error}");
    }
}

#[test]
fn either_conversion_checks_scoped_value_kinds_before_erasure() {
    let source = "import * as effects from \"std/effect\"\nfn lifted<A,B> value: Either<A,B> -> Effect<{},A,B> = effects.fromEither value\npub fn main -> Int = 42\n";
    let mut core = compile_core_module(CompileInput::new("kinds.ssrg", "native/kinds", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
    core.functions
        .iter_mut()
        .find(|f| f.symbol == "native/kinds::lifted")
        .unwrap()
        .type_parameters[0]
        .arity = 1;
    let error = emit_rust_module(&core, "main").unwrap_err();
    assert!(error.origin.is_some());
    assert!(
        error.message.contains("fromEither argument/result kinds"),
        "{error}"
    );
}
