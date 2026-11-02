use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreModule;

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("process.ssrg", "native/process", source))
        .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"))
        .core_ir
}

#[test]
fn process_calls_and_saved_references_use_the_cached_native_runtime() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/process-context.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "process"));
    assert_eq!(artifact.metadata.abi, "rust-erased-sync-v2");
    assert!(artifact.source.contains("use seseragi_native_runtime::*;"));
    for name in ["arguments", "environment", "currentDirectory"] {
        assert!(artifact
            .source
            .contains(&format!("native_process_operation(\"std/process::{name}\"")));
    }
}

fn alter(value: &mut serde_json::Value, name: &str, case: usize) -> bool {
    if let Some(object) = value.as_object_mut() {
        if object.get("callee").and_then(|x| x.as_str()) == Some(name)
            && object.contains_key("requirements")
        {
            match case {
                0 => {
                    object["requirements"]["fields"][0]["type"]["canonical"] =
                        serde_json::json!("counterfeit::Process")
                }
                1 => {
                    object["requirements"]["fields"][0]["type"]["arguments"] =
                        serde_json::json!([{"kind":"named","name":"Int","arguments":[]}])
                }
                2 => object["requirements"]["closed"] = serde_json::json!(false),
                3 => object["requirements"]["fields"][0]["optional"] = serde_json::json!(true),
                4 => object["requirements"]["fields"][0]["name"] = serde_json::json!("other"),
                5 => {
                    object["failure"]["canonical"] = serde_json::json!("counterfeit::ProcessError")
                }
                6 => object["success"] = serde_json::json!({"kind":"hole"}),
                7 => {
                    let origin = object["arguments"][0]["origin"].clone();
                    object["arguments"][0] =
                        serde_json::to_value(seseragi_lowering::CoreExpr::Boolean {
                            value: true,
                            origin: serde_json::from_value(origin).unwrap(),
                        })
                        .unwrap();
                }
                8 => object["arguments"] = serde_json::json!([]),
                9 => {
                    let argument = object["arguments"][0].clone();
                    object["arguments"].as_array_mut().unwrap().push(argument);
                }
                _ => unreachable!(),
            }
            return true;
        }
        return object.values_mut().any(|v| alter(v, name, case));
    }
    value
        .as_array_mut()
        .is_some_and(|values| values.iter_mut().any(|v| alter(v, name, case)))
}

#[test]
fn process_rejects_unproven_owners_requirements_results_arguments_and_arity() {
    for name in ["arguments", "environment", "currentDirectory"] {
        let argument = if name == "environment" {
            "\"key\""
        } else {
            "()"
        };
        let core = checked(&format!(
            "import * as process from \"std/process\"\npub effect fn main = process.{name} {argument}\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        for case in 0..10 {
            let mut changed = core.clone();
            let mut value = serde_json::to_value(&changed.functions[0].body).unwrap();
            assert!(alter(&mut value, &format!("std/process::{name}"), case));
            changed.functions[0].body = serde_json::from_value(value).unwrap();
            let error = emit_rust_module(&changed, "main").unwrap_err();
            assert!(error.origin.is_some(), "{name}, mutation {case}: {error}");
            assert!(
                error.message.contains("Process"),
                "{name}, mutation {case}: {error}"
            );
        }
    }
}

#[test]
fn unused_process_operations_remain_unavailable_in_program_wasm() {
    for (name, success, argument) in [
        ("arguments", "Array<String>", "()"),
        ("environment", "Maybe<String>", "\"key\""),
        ("currentDirectory", "paths.Path", "()"),
    ] {
        for body in [
            format!("process.{name} {argument}"),
            format!("{{ let saved = process.{name}; saved {argument} }}"),
        ] {
            let core = checked(&format!(
                "import * as process from \"std/process\"\nimport * as paths from \"std/path\"\nfn hidden -> Effect<{{ process: process.Process }}, process.ProcessError, {success}> = {body}\npub fn value -> Int = 42\n"
            ));
            let error = emit_program_wasm(&[&core], &core.module, &["value".into()]).unwrap_err();
            assert!(error.origin.is_some(), "{body}: {error}");
            assert!(error.message.contains("native Process"), "{body}: {error}");
        }
    }
}
