use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreModule;

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("log.ssrg", "native/log", source))
        .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"))
        .core_ir
}

#[test]
fn log_calls_and_constructors_use_the_native_runtime() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/structured-log.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact.metadata.capabilities.contains(&"logger".into()));
    assert!(artifact
        .metadata
        .capabilities
        .contains(&"effect-sync".into()));
    assert!(artifact.source.contains("use seseragi_native_runtime::*;"));
    for name in [
        "log",
        "LogTrace",
        "LogDebug",
        "LogInfo",
        "LogWarn",
        "LogFailure",
        "LogString",
        "LogInt",
        "LogFloat",
        "LogBool",
    ] {
        assert!(
            artifact
                .source
                .contains(&format!("native_log_operation(\"std/log::{name}\"")),
            "{name}"
        );
    }
}

#[test]
fn log_and_value_constructors_support_saved_references() {
    for (name, argument) in [
        (
            "log",
            "(LogEvent { level: logs.LogInfo, message: \"saved\", fields: `[] })",
        ),
        ("LogString", "\"text\""),
        ("LogInt", "42"),
        ("LogFloat", "42.5"),
        ("LogBool", "True"),
    ] {
        let result = if name == "log" {
            "Effect<{ logger: logs.Logger }, logs.LogError, Unit>"
        } else {
            "logs.LogValue"
        };
        let source = format!("import * as logs from \"std/log\"\nimport {{ LogEvent }} from \"std/log\"\nfn save<A> value: A -> A = value\npub fn probe -> {result} = {{ let saved = save logs.{name}; saved {argument} }}\n");
        let artifact = emit_rust_module(&checked(&source), "probe").unwrap();
        assert!(
            artifact
                .source
                .contains(&format!("native_log_operation(\"std/log::{name}\"")),
            "{name}"
        );
    }
}

fn alter(value: &mut serde_json::Value, case: usize) -> bool {
    if let Some(object) = value.as_object_mut() {
        if object.get("callee").and_then(|v| v.as_str()) == Some("std/log::log")
            && object.contains_key("requirements")
        {
            if case >= 9 {
                object.insert("calleeSignature".into(), serde_json::json!({
                    "typeParameters": [],
                    "parameters": [{"kind":"named","name":"std/log::LogEvent","arguments":[]}],
                    "result": {"kind":"named","name":"std/prelude::Effect","arguments":[object["requirements"], object["failure"], object["success"]]}
                }));
            }
            match case {
                0 => {
                    object["requirements"]["fields"][0]["type"]["canonical"] =
                        "counterfeit::Logger".into()
                }
                1 => object["requirements"]["closed"] = false.into(),
                2 => object["requirements"]["fields"][0]["optional"] = true.into(),
                3 => object["requirements"]["fields"][0]["name"] = "other".into(),
                4 => object["failure"]["canonical"] = "counterfeit::LogError".into(),
                5 => object["success"] = serde_json::json!({"kind":"hole"}),
                6 => {
                    let origin = object["arguments"][0]["origin"].clone();
                    object["arguments"][0] =
                        serde_json::json!({"kind":"boolean","value":true,"origin":origin});
                }
                7 => object["arguments"] = serde_json::json!([]),
                8 => {
                    let argument = object["arguments"][0].clone();
                    object["arguments"].as_array_mut().unwrap().push(argument);
                }
                9 => object["calleeSignature"]["parameters"] = serde_json::json!([]),
                10 => object["calleeSignature"]["result"] = serde_json::json!({"kind":"hole"}),
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

#[test]
fn log_rejects_unproven_requirements_results_arguments_and_source_arity() {
    let original = checked("import * as logs from \"std/log\"\nimport { LogEvent } from \"std/log\"\npub effect fn main = logs.log (LogEvent { level: logs.LogInfo, message: \"test\", fields: `[] })\n");
    emit_rust_module(&original, "main").unwrap();
    for case in 0..11 {
        let mut changed = original.clone();
        let mut value = serde_json::to_value(&changed.functions[0].body).unwrap();
        assert!(alter(&mut value, case));
        changed.functions[0].body = serde_json::from_value(value).unwrap();
        let error = emit_rust_module(&changed, "main").unwrap_err();
        assert!(error.origin.is_some(), "mutation {case}: {error}");
        assert!(error.message.contains("Logger"), "mutation {case}: {error}");
    }
}

#[test]
fn unused_log_operations_are_rejected_in_program_wasm() {
    for body in ["logs.log event", "{ let saved = logs.log; saved event }"] {
        let core = checked(&format!("import * as logs from \"std/log\"\nfn hidden event: logs.LogEvent -> Effect<{{ logger: logs.Logger }}, logs.LogError, Unit> = {body}\npub fn value -> Int = 42\n"));
        let error = emit_program_wasm(&[&core], &core.module, &["value".into()]).unwrap_err();
        assert!(error.origin.is_some(), "{body}: {error}");
        assert!(error.message.contains("native Logger"), "{body}: {error}");
    }
}
