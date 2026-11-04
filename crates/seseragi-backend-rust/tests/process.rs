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

#[test]
fn signal_streams_accept_direct_saved_and_generic_references() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/process-signals.ssrg"
    ));
    let artifact = seseragi_backend_rust::emit_rust_module_with_options(
        &core,
        "main",
        &seseragi_backend_rust::RustTargetOptions {
            clock_provider: Some(seseragi_backend_rust::NativeClockProvider::BuiltinSleep),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(artifact
        .source
        .contains("native_process_operation(\"std/process::signals\""));
    for capability in ["process", "process-signals", "stream"] {
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|value| value == capability));
    }
    for body in [
        "process.signals watched",
        "{ let saved = process.signals; saved watched }",
    ] {
        let core = checked(&format!("import * as process from \"std/process\"\nimport * as streams from \"std/stream\"\nimport * as nonEmpty from \"std/non-empty-list\"\nfn hidden watched: nonEmpty.NonEmptyList<process.ProcessSignal> -> streams.Stream<{{ process: process.Process }}, process.ProcessError, process.ProcessSignal> = {body}\npub fn value -> Int = 42\n"));
        let error = emit_program_wasm(&[&core], &core.module, &["value".into()]).unwrap_err();
        assert!(error.origin.is_some(), "{error}");
        assert!(error.message.contains("native Process"), "{error}");
    }
}

fn alter_signal(value: &mut serde_json::Value, case: usize) -> bool {
    if let Some(object) = value.as_object_mut() {
        if object.get("callee").and_then(|value| value.as_str()) == Some("std/process::signals") {
            match case {
                0 => {
                    object["arguments"][0]["type"]["canonical"] = "counterfeit::NonEmptyList".into()
                }
                1 => {
                    object["arguments"][0]["type"]["arguments"][0]["canonical"] =
                        "counterfeit::ProcessSignal".into()
                }
                2 => object["arguments"][0]["type"]["arguments"] = serde_json::json!([]),
                3 => object["type"]["canonical"] = "counterfeit::Stream".into(),
                4 => {
                    object["type"]["arguments"][0]["fields"][0]["type"]["canonical"] =
                        "counterfeit::Process".into()
                }
                5 => object["type"]["arguments"][0]["fields"][0]["optional"] = true.into(),
                6 => object["type"]["arguments"][0]["closed"] = false.into(),
                7 => {
                    object["type"]["arguments"][1]["canonical"] = "counterfeit::ProcessError".into()
                }
                8 => {
                    object["type"]["arguments"][2]["canonical"] =
                        "counterfeit::ProcessSignal".into()
                }
                9 => object["arguments"] = serde_json::json!([]),
                10 => {
                    let argument = object["arguments"][0].clone();
                    object["arguments"].as_array_mut().unwrap().push(argument);
                }
                11 => {
                    let origin = object["arguments"][0]["origin"].clone();
                    object["arguments"][0] =
                        serde_json::to_value(seseragi_lowering::CoreExpr::Boolean {
                            value: true,
                            origin: serde_json::from_value(origin).unwrap(),
                        })
                        .unwrap();
                }
                12 => {
                    let argument = object["type"]["arguments"][2].clone();
                    object["type"]["arguments"]
                        .as_array_mut()
                        .unwrap()
                        .push(argument);
                }
                13 => {
                    let argument = object["arguments"][0]["type"]["arguments"][0].clone();
                    object["arguments"][0]["type"]["arguments"]
                        .as_array_mut()
                        .unwrap()
                        .push(argument);
                }
                14 => object["type"]["arguments"][0]["fields"][0]["name"] = "other".into(),
                _ => unreachable!(),
            }
            return true;
        }
        return object.values_mut().any(|value| alter_signal(value, case));
    }
    value
        .as_array_mut()
        .is_some_and(|values| values.iter_mut().any(|value| alter_signal(value, case)))
}

#[test]
fn signals_reject_unproven_list_signal_environment_stream_failure_and_arity() {
    let core = checked("import * as process from \"std/process\"\nimport * as nonEmpty from \"std/non-empty-list\"\nimport * as streams from \"std/stream\"\npub fn watch watched: nonEmpty.NonEmptyList<process.ProcessSignal> -> streams.Stream<{ process: process.Process }, process.ProcessError, process.ProcessSignal> = process.signals watched\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    for case in 0..15 {
        let mut changed = core.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::watch"))
            .unwrap();
        let mut value = serde_json::to_value(&function.body).unwrap();
        assert!(alter_signal(&mut value, case));
        function.body = serde_json::from_value(value).unwrap();
        let error = emit_rust_module(&changed, "main").unwrap_err();
        assert!(error.origin.is_some(), "mutation {case}: {error}");
        assert!(
            error.message.contains("Process"),
            "mutation {case}: {error}"
        );
    }
}
