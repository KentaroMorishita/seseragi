use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreModule, CoreStatement};

fn core(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("child.ssrg", "native/child", source))
        .unwrap_or_else(|error| panic!("{error:?}"))
        .core_ir
}

#[test]
fn child_apis_emit_checked_native_calls_and_reject_program_wasm() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process-config.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/child-process-callables.ssrg"),
    ] {
        let core = core(source);
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|value| value == "child-process"));
        assert!(artifact.source.contains("native_child_process_operation"));
        assert!(emit_program_wasm(&[&core], &core.module, &["main".into()]).is_err());
    }
}

#[test]
fn child_calls_reject_forged_owners_results_requirements_and_stages() {
    let core = core(include_str!(
        "../../../examples/spec/fixtures/rust-backend/child-process.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
    fn alter(value: &mut serde_json::Value, case: usize) -> bool {
        if let Some(object) = value.as_object_mut() {
            let callee = object
                .get("callee")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if (case < 5 && callee == "std/child-process::runCaptured")
                || (case >= 5 && callee == "std/child-process::runStreaming")
            {
                match case {
                    0 => {
                        object["arguments"][0]["type"]["canonical"] =
                            serde_json::json!("forged::CaptureLimit")
                    }
                    1 => {
                        object["failure"]["canonical"] =
                            serde_json::json!("forged::ChildProcessError")
                    }
                    2 => {
                        object["requirements"]["fields"][0]["type"]["canonical"] =
                            serde_json::json!("forged::ChildProcesses")
                    }
                    3 => object["requirements"]["closed"] = serde_json::json!(false),
                    4 => object["arguments"].as_array_mut().unwrap().clear(),
                    5 => {
                        object["arguments"][0]["type"]["canonical"] =
                            serde_json::json!("forged::Stream")
                    }
                    6 => object["type"] = serde_json::json!({"kind":"hole"}),
                    7 => {
                        object.insert("evidenceArgumentIndex".into(), serde_json::json!(0));
                    }
                    _ => {
                        let extra = object["arguments"][0].clone();
                        object["arguments"].as_array_mut().unwrap().push(extra);
                    }
                }
                return true;
            }
            return object.values_mut().any(|value| alter(value, case));
        }
        value
            .as_array_mut()
            .is_some_and(|values| values.iter_mut().any(|value| alter(value, case)))
    }
    for case in 0..9 {
        let mut changed = core.clone();
        let mut found = false;
        for function in &mut changed.functions {
            let mut value = serde_json::to_value(&function.body).unwrap();
            if alter(&mut value, case) {
                function.body = serde_json::from_value(value)
                    .unwrap_or_else(|error| panic!("case {case}: {error}"));
                found = true;
                break;
            }
        }
        assert!(found, "missing case {case}");
        assert!(
            emit_rust_module(&changed, "main").is_err(),
            "accepted case {case}"
        );
    }
}

#[test]
fn child_streams_preserve_generic_requirements_and_saved_stages() {
    for body in [
        "fn operation<E> source: streams.Stream<{process: process.Process},E,children.ChildInput> -> command: children.Command -> streams.Stream<{childProcesses: children.ChildProcesses, process: process.Process},Either<E,children.ChildProcessError>,children.ChildEvent> = children.runStreaming source command",
        "fn operation<E> source: streams.Stream<{process: process.Process},E,children.ChildInput> -> (children.Command -> streams.Stream<{childProcesses: children.ChildProcesses, process: process.Process},Either<E,children.ChildProcessError>,children.ChildEvent>) = children.runStreaming source",
        "fn operation<E> -> (streams.Stream<{process: process.Process},E,children.ChildInput> -> children.Command -> streams.Stream<{childProcesses: children.ChildProcesses, process: process.Process},Either<E,children.ChildProcessError>,children.ChildEvent>) = children.runStreaming",
    ] {
        let source = format!("import * as children from \"std/child-process\"\nimport * as streams from \"std/stream\"\nimport * as process from \"std/process\"\n{body}\npub fn main -> Int = 0");
        let core = core(&source);
        emit_rust_module(&core, "main").unwrap();
        assert!(emit_program_wasm(&[&core], &core.module, &["main".into()]).is_err());
    }
}

#[test]
fn saved_child_stream_references_require_bound_value_type_parameters() {
    let core = core(include_str!(
        "../../../examples/spec/fixtures/rust-backend/child-process-callables.ssrg"
    ));
    for case in 0..4 {
        let mut changed = core.clone();
        let parameters = changed
            .functions
            .iter_mut()
            .find_map(|function| {
                let CoreExpr::Sequence { statements, .. } = &mut function.body else {
                    return None;
                };
                statements.iter_mut().find_map(|statement| match statement {
                    CoreStatement::PureLet {
                        name,
                        type_parameters,
                        ..
                    } if name == "runStreaming" => Some(type_parameters),
                    _ => None,
                })
            })
            .expect("saved generic runStreaming reference");
        match case {
            0 => parameters.clear(),
            1 | 2 => parameters[case - 1].arity = 1,
            _ => parameters.push(parameters[0].clone()),
        }
        assert!(
            emit_rust_module(&changed, "main").is_err(),
            "accepted invalid type parameter case {case}"
        );
    }
}
