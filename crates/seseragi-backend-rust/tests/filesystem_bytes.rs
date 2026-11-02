use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};

fn source(name: &str) -> String {
    let definition = match name {
        "readBytes" => "effect fn operation path: paths.Path -> bytes.Bytes with FileSystem fails fs.FileSystemError = fs.readBytes path",
        "readChunks" => "fn operation size: streams.BufferCapacity -> path: paths.Path -> streams.Stream<{fileSystem:fs.FileSystem},fs.FileSystemError,bytes.Bytes> = fs.readChunks size path",
        "writeBytes" => "effect fn operation mode: fs.WriteMode -> value: bytes.Bytes -> path: paths.Path -> Unit with FileSystem fails fs.FileSystemError = fs.writeBytes mode value path",
        "writeChunks" => "fn operation<R,E> mode: fs.WriteMode -> source: streams.Stream<R,E,bytes.Bytes> -> path: paths.Path -> Effect<R & {fileSystem:fs.FileSystem},Either<E,fs.FileSystemError>,Unit> = fs.writeChunks mode source path",
        _ => unreachable!(),
    };
    format!("import * as fs from \"std/fs\"\nimport * as paths from \"std/path\"\nimport * as bytes from \"std/bytes\"\nimport * as streams from \"std/stream\"\n{definition}\npub fn main -> Int = 0\n")
}

#[test]
fn binary_filesystem_checks_owners_results_requirements_kinds_and_source_stages() {
    fn mutate(value: &mut serde_json::Value, name: &str, case: usize) -> bool {
        if let Some(object) = value.as_object_mut() {
            if object.get("callee").and_then(|value| value.as_str()) == Some(name) {
                match case {
                    0 => {
                        object["arguments"][0]["type"] = serde_json::json!({"kind":"named","name":"fake::Argument","arguments":[]})
                    }
                    1 => {
                        let field = if object["kind"] == "effect-invoke" {
                            "success"
                        } else {
                            "type"
                        };
                        object[field] = serde_json::json!({"kind":"hole"});
                    }
                    2 => {
                        object["arguments"].as_array_mut().unwrap().pop();
                    }
                    3 => {
                        let argument = object["arguments"][0].clone();
                        object["arguments"].as_array_mut().unwrap().push(argument);
                    }
                    4 if object["kind"] == "effect-invoke" => {
                        object["requirements"] =
                            serde_json::json!({"kind":"record","closed":true,"fields":[]})
                    }
                    5 if object["kind"] == "effect-invoke" => {
                        object["failure"] = serde_json::json!({"kind":"named","name":"fake::FileSystemError","arguments":[]})
                    }
                    4 => object["type"]["arguments"][0] = serde_json::json!({"kind":"hole"}),
                    5 => {
                        object["type"]["arguments"][1] =
                            serde_json::json!({"kind":"named","name":"fake::Error","arguments":[]})
                    }
                    _ => {
                        let field = if object["kind"] == "effect-invoke" {
                            "success"
                        } else {
                            "type"
                        };
                        if field == "success" {
                            object[field] = serde_json::json!({"kind":"named","name":"fake::Result","arguments":[]});
                        } else {
                            object[field]["arguments"][2] = serde_json::json!({"kind":"named","name":"fake::Result","arguments":[]});
                        }
                    }
                }
                return true;
            }
            return object.values_mut().any(|value| mutate(value, name, case));
        }
        value
            .as_array_mut()
            .is_some_and(|array| array.iter_mut().any(|value| mutate(value, name, case)))
    }
    for name in ["readBytes", "readChunks", "writeBytes", "writeChunks"] {
        let source = source(name);
        let core = compile_core_module(CompileInput::new(
            "filesystem-bytes.ssrg",
            "native/filesystem-bytes",
            &source,
        ))
        .unwrap_or_else(|error| panic!("{name}: {error:?}"))
        .core_ir;
        emit_rust_module(&core, "main").unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(
            emit_program_wasm(&[&core], "native/filesystem-bytes", &["main".into()]).is_err(),
            "{name}"
        );
        for case in 0..7 {
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
        if name == "writeChunks" {
            for binder in [0, 1] {
                let mut changed = core.clone();
                let function = changed
                    .functions
                    .iter_mut()
                    .find(|function| function.symbol.ends_with("::operation"))
                    .unwrap();
                function.type_parameters[binder].arity = 1;
                assert!(emit_rust_module(&changed, "main").is_err());
            }
        }
    }
}

#[test]
fn buffer_capacity_checks_canonical_int_error_and_opaque_result() {
    let source = "import * as streams from \"std/stream\"\nfn capacity size: Int -> Either<streams.BufferCapacityError,streams.BufferCapacity> = streams.bufferCapacity size\npub fn main -> Int = 0\n";
    let core = compile_core_module(CompileInput::new(
        "capacity.ssrg",
        "native/capacity",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "main").unwrap();
    assert!(emit_program_wasm(&[&core], "native/capacity", &["main".into()]).is_err());
}
