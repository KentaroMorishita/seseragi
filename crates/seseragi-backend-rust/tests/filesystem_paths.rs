use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};

fn source(name: &str) -> String {
    let definition = match name {
        "list" => "fn operation path: paths.Path -> streams.Stream<{fileSystem:fs.FileSystem},fs.FileSystemError,fs.DirectoryEntry> = fs.list path".into(),
        "writeAtomic" => "effect fn operation value: bytes.Bytes -> path: paths.Path -> Unit with FileSystem fails fs.FileSystemError = fs.writeAtomic value path".into(),
        "move" => "effect fn operation destination: paths.Path -> source: paths.Path -> Unit with FileSystem fails fs.FileSystemError = fs.move destination source".into(),
        "withTemporaryDirectory" | "withTemporaryFile" => format!("fn operation<R,E,A> prefix: String -> use: (paths.Path -> Effect<R,E,A>) -> Effect<R & {{fileSystem:fs.FileSystem}},Either<fs.FileSystemError,E>,A> = fs.{name} prefix use"),
        _ => format!("effect fn operation path: paths.Path -> Unit with FileSystem fails fs.FileSystemError = fs.{name} path"),
    };
    format!("import * as fs from \"std/fs\"\nimport * as paths from \"std/path\"\nimport * as bytes from \"std/bytes\"\nimport * as streams from \"std/stream\"\n{definition}\npub fn main -> Int = 0\n")
}

#[test]
fn filesystem_path_fixture_compiles_with_checked_saved_and_partial_functions() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-paths.ssrg");
    let core = compile_core_module(CompileInput::new("paths.ssrg", "native/paths", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn filesystem_paths_reject_forged_arguments_results_requirements_and_source_stages() {
    fn mutate(value: &mut serde_json::Value, name: &str, case: usize) -> bool {
        if let Some(object) = value.as_object_mut() {
            if object.get("callee").and_then(|value| value.as_str()) == Some(name) {
                let effect = object["kind"] == "effect-invoke";
                match case {
                    0 => {
                        object["arguments"][0]["type"] = serde_json::json!({"kind":"named","name":"fake::Argument","arguments":[]})
                    }
                    1 => {
                        object[if effect { "success" } else { "type" }] =
                            serde_json::json!({"kind":"hole"})
                    }
                    2 => {
                        object["arguments"].as_array_mut().unwrap().pop();
                    }
                    3 => {
                        let argument = object["arguments"][0].clone();
                        object["arguments"].as_array_mut().unwrap().push(argument);
                    }
                    4 if effect => {
                        object["requirements"] =
                            serde_json::json!({"kind":"record","closed":true,"fields":[]})
                    }
                    5 if effect => {
                        object["failure"] =
                            serde_json::json!({"kind":"named","name":"fake::Error","arguments":[]})
                    }
                    4 => object["type"]["arguments"][0] = serde_json::json!({"kind":"hole"}),
                    5 => {
                        object["type"]["arguments"][1] =
                            serde_json::json!({"kind":"named","name":"fake::Error","arguments":[]})
                    }
                    _ => unreachable!(),
                }
                return true;
            }
            return object.values_mut().any(|value| mutate(value, name, case));
        }
        value
            .as_array_mut()
            .is_some_and(|array| array.iter_mut().any(|value| mutate(value, name, case)))
    }
    for name in [
        "writeAtomic",
        "list",
        "createDirectory",
        "createDirectories",
        "removeFile",
        "removeDirectory",
        "move",
        "withTemporaryDirectory",
        "withTemporaryFile",
    ] {
        let source = source(name);
        let core = compile_core_module(CompileInput::new("paths.ssrg", "native/paths", &source))
            .unwrap_or_else(|error| panic!("{name}: {error:?}"))
            .core_ir;
        emit_rust_module(&core, "main").unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(
            emit_program_wasm(&[&core], &core.module, &["main".into()]).is_err(),
            "{name}"
        );
        for case in 0..6 {
            let mut changed = core.clone();
            let function = changed
                .functions
                .iter_mut()
                .find(|function| function.symbol.ends_with("::operation"))
                .unwrap();
            let mut body = serde_json::to_value(&function.body).unwrap();
            assert!(
                mutate(&mut body, &format!("std/fs::{name}"), case),
                "{name}/{case}"
            );
            function.body = serde_json::from_value(body).unwrap();
            let Err(error) = emit_rust_module(&changed, "main") else {
                panic!("{name}/{case}: accepted forged filesystem contract")
            };
            assert!(error.origin.is_some(), "{name}/{case}: {error}");
        }
        if name.starts_with("withTemporary") {
            for binder in 0..3 {
                let mut changed = core.clone();
                changed
                    .functions
                    .iter_mut()
                    .find(|function| function.symbol.ends_with("::operation"))
                    .unwrap()
                    .type_parameters[binder]
                    .arity = 1;
                assert!(
                    emit_rust_module(&changed, "main").is_err(),
                    "{name}/kind/{binder}"
                );
            }
        }
    }
}

#[test]
fn temporary_requirements_merge_identical_services_and_reject_conflicting_or_optional_services() {
    for name in ["withTemporaryDirectory", "withTemporaryFile"] {
        let source = format!("import * as fs from \"std/fs\"\nimport * as paths from \"std/path\"\nimport * as effects from \"std/effect\"\neffect fn callback path: paths.Path -> String with FileSystem fails String = effects.succeed \"used\"\neffect fn operation prefix: String -> String with FileSystem fails Either<fs.FileSystemError,String> = fs.{name} prefix callback\npub fn main -> Int = 0\n");
        let core = compile_core_module(CompileInput::new(
            "requirements.ssrg",
            "native/requirements",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "main").unwrap();
        for case in 0..4 {
            let mut changed = core.clone();
            let operation = changed
                .functions
                .iter_mut()
                .find(|function| function.symbol.ends_with("::operation"))
                .unwrap();
            let mut body = serde_json::to_value(&operation.body).unwrap();
            assert_eq!(body["kind"], "effect-invoke");
            let mut field = body["requirements"]["fields"][0].clone();
            match case {
                0 => (),
                1 => {
                    field["type"] =
                        serde_json::json!({"kind":"named","name":"fake::FileSystem","arguments":[]})
                }
                2 => field["optional"] = true.into(),
                3 => field["name"] = "unexpected".into(),
                _ => unreachable!(),
            }
            body["requirements"]["fields"]
                .as_array_mut()
                .unwrap()
                .push(field);
            operation.body = serde_json::from_value(body).unwrap();
            let result = emit_rust_module(&changed, "main");
            assert_eq!(result.is_ok(), case == 0, "{name}/{case}");
            if let Err(error) = result {
                assert!(error.origin.is_some());
            }
        }
    }
}
