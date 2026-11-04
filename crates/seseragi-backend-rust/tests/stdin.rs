use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

#[test]
fn stdin_apis_keep_shared_service_and_opaque_configuration_contracts() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/stdin.ssrg");
    let core = compile_core_module(CompileInput::new("stdin.ssrg", "native/stdin", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|value| value == "stdin"));
    assert!(emit_program_wasm(&[&core], "native/stdin", &["main".into()]).is_err());
}

#[test]
fn prelude_read_line_values_keep_one_unit_stage_and_the_stdin_capability() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/prelude-read-line-saved.ssrg");
    let core = compile_core_module(CompileInput::new("saved.ssrg", "native/saved", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact.metadata.capabilities.contains(&"stdin".into()));
    assert!(emit_program_wasm(&[&core], "native/saved", &["main".into()]).is_err());
}

#[test]
fn prelude_read_line_references_reject_forged_function_and_effect_types() {
    let source = "pub fn saved -> (Unit -> Effect<{stdin:Stdin},StdinError,Maybe<String>>) = readLine\npub fn main -> Int = 0\n";
    let core = compile_core_module(CompileInput::new("saved.ssrg", "native/saved", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::saved"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    for mutation in 0..8 {
        let mut changed = core.clone();
        let CoreExpr::Variable { name, type_ref, .. } = &mut changed.functions[index].body else {
            panic!("readLine must remain a callable reference");
        };
        assert_eq!(name, "std/prelude::readLine");
        let CoreType::Function { parameter, result } = type_ref else {
            panic!("readLine must take one Unit argument");
        };
        let fake = CoreType::Named {
            name: "fake::Value".into(),
            arguments: vec![],
        };
        match mutation {
            0 => **parameter = CoreType::Hole,
            1 => **parameter = fake,
            2 => *type_ref = *result.clone(),
            6 => **result = CoreType::Hole,
            _ => {
                let CoreType::Named { arguments, .. } = result.as_mut() else {
                    panic!("readLine must return an Effect");
                };
                match mutation {
                    3 => {
                        arguments[0] = CoreType::Record {
                            closed: true,
                            fields: vec![],
                        }
                    }
                    4 => arguments[1] = fake,
                    5 => arguments[2] = fake,
                    7 => {
                        let CoreType::Record { fields, .. } = &mut arguments[0] else {
                            panic!()
                        };
                        fields[0].type_ref = fake;
                    }
                    _ => unreachable!(),
                }
            }
        }
        let error = emit_rust_module(&changed, "main").expect_err("unproven readLine reference");
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.as_ref(), Some(&origin), "mutation {mutation}");
    }
}

#[test]
fn stdin_rejects_forged_owners_kinds_results_requirements_and_source_arities() {
    for (name, definition) in [
        ("readSize", "fn operation value: Int -> Either<stdin.StdinConfigError,stdin.ReadSize> = stdin.readSize value"),
        ("lineLimit", "fn operation value: Int -> Either<stdin.StdinConfigError,stdin.LineLimit> = stdin.lineLimit value"),
        ("defaultReadSize", "fn operation -> stdin.ReadSize = stdin.defaultReadSize ()"),
        ("defaultLineLimit", "fn operation -> stdin.LineLimit = stdin.defaultLineLimit ()"),
        ("readChunk", "effect fn operation size: stdin.ReadSize -> Maybe<bytes.Bytes> with Stdin fails StdinError = stdin.readChunk size"),
        ("readLine", "effect fn operation -> Maybe<String> with Stdin fails StdinError = stdin.readLine ()"),
        ("readLineWith", "effect fn operation limit: stdin.LineLimit -> Maybe<String> with Stdin fails StdinError = stdin.readLineWith limit"),
        ("lines", "fn operation limit: stdin.LineLimit -> streams.Stream<{stdin:Stdin},StdinError,String> = stdin.lines limit"),
    ] {
        let source = format!("import * as stdin from \"std/stdin\"\nimport * as streams from \"std/stream\"\nimport * as bytes from \"std/bytes\"\n{definition}\npub fn main -> Int = 0\n");
        let core = compile_core_module(CompileInput::new("stdin.ssrg", "native/stdin", &source)).unwrap().core_ir;
        emit_rust_module(&core, "main").unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(emit_program_wasm(&[&core], "native/stdin", &["main".into()]).is_err(), "{name}");
        for mutation in 0..6 {
            let mut changed = core.clone();
            let body = &mut changed.functions.iter_mut().find(|function| function.symbol.ends_with("::operation")).unwrap().body;
            let extra = CoreExpr::Unit { origin: core_expr_origin(body).clone() };
            let fake = CoreType::Named { name: "fake::Value".into(), arguments: vec![] };
            if mutation >= 4 {
                let arguments = match body {
                    CoreExpr::Call { arguments, .. }
                    | CoreExpr::EffectInvoke { arguments, .. }
                    | CoreExpr::EffectOperation { arguments, .. } => arguments,
                    _ => panic!("{name}: missing source arguments"),
                };
                if mutation == 4 {
                    let argument = arguments.first_mut().unwrap();
                    *argument = CoreExpr::Variable {
                        name: "forged".into(),
                        callable_signature: None,
                        evidence: vec![],
                        type_ref: fake,
                        origin: core_expr_origin(argument).clone(),
                    };
                } else {
                    arguments.pop().unwrap();
                }
            } else {
                match body {
                    CoreExpr::Call { arguments, type_ref, evidence_argument_index, .. } => match mutation {
                        0 => *type_ref = CoreType::Hole,
                        1 => arguments.push(extra),
                        2 => *type_ref = fake,
                        _ => *evidence_argument_index = Some(0),
                    },
                    CoreExpr::EffectInvoke { arguments, requirements, failure, success, .. }
                    | CoreExpr::EffectOperation { arguments, requirements, failure, success, .. } => match mutation {
                        0 => *success = CoreType::Hole,
                        1 => arguments.push(extra),
                        2 => *requirements = CoreType::Record { closed: true, fields: vec![] },
                        _ => *failure = fake,
                    },
                    _ => panic!("{name}: unexpected operation shape: {body:?}"),
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err(&format!("{name}/{mutation}"));
            assert!(error.origin.is_some(), "{name}/{mutation}: {error}");
        }
    }
}
