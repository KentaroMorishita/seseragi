use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_module, CompileInput};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-monads-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn compare(source: &str, expected: &str) {
    compare_trace(source, expected, false);
}
fn compare_trace(source: &str, expected: &str, trace: bool) {
    compare_options(source, expected, trace, false);
}
fn compare_options(source: &str, expected: &str, trace: bool, extras: bool) {
    let compiled = compile_module(CompileInput::new("monads.ssrg", "native/monads", source))
        .unwrap_or_else(|e| panic!("{e:?}"));
    let artifact = emit_rust_module(&compiled.core_ir, "probe").unwrap_or_else(|e| panic!("{e:?}"));
    let scratch = Scratch::new();
    let rust = scratch.0.join("program.rs");
    let binary = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let mut generated = artifact.source;
    if source.contains("fn many<M<_>>") {
        assert!(generated.contains("(\"\\0nativeTailBind\", V::Bool(true))"));
    }
    if extras {
        let symbol = |name: &str| {
            format!(
                "v_{}",
                format!("native/monads::{name}")
                    .bytes()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )
        };
        let code = format!(
            r#"
            {{
                println!("{{}}", {}(vec![]).apply(V::Unit).field("head").render());
                println!("{{}}", {}(vec![]).apply(V::Unit).render());
                let mut context = NativeContext::default();
                match context.root({}(vec![]).apply(V::Unit)) {{ Ok(value) => println!("{{}}", value.render()), Err(_) => panic!("effect") }}
                let emit = V::fun(vec![], |_, value| V::leaf(vec![value], |captures, _| {{ println!("{{}}", captures[0].render()); Ok(V::Unit) }}));
                assert!(context.root(native_stream_operation("std/stream::runForEach", vec![emit, {}(vec![]).apply(V::Unit)])).is_ok());
                println!("{{}}", {}(vec![]).apply(V::Unit).field("run").payload().payload().render());
                println!("{{}}", {}(vec![]).apply(V::Unit).field("run").index(0).payload().render());
                println!("{{}}", {}(vec![]).apply(V::Unit).field("run").apply(V::text("env")).payload().render());
                println!("{{}}", {}(vec![]).apply(V::Unit).field("run").apply(V::Int(7)).payload().index(0).render());
                println!("{{}}", {}(vec![]).apply(V::Unit).field("run").payload().index(0).render());
                let callback = V::fun(vec![], |_, value| native_monad_pure(0, value));
                let large = native_monad_bind(0, &callback, V::Array(Shared::new((0..100000).map(V::Int).collect())));
                assert_eq!(native_monad_values(0, &large).len(), 100000);
                let bind = V::Tuple(Shared::new(vec![V::Int(2), V::fun(vec![], |_, state| native_monad_pure(2, match state {{ V::ListEmpty => V::Int(0), _ => binary("+", state.list_element(0), V::Int(1)) }}))]));
                let mut steps = vec![bind; 20000];
                steps.push(V::Tuple(Shared::new(vec![V::Int(3), V::fun(vec![], |_, state| native_monad_pure(2, state.list_element(0)))])));
                let result = native_monad_do(native_monad_dictionary(2, "Monad"), V::Array(Shared::new(steps)), 0, V::ListEmpty);
                assert!(result.tagged("std/prelude::Just")); assert_eq!(result.payload().render(), "19999");
                let dictionary = V::dict(vec![("\0nativeTailBind", V::Bool(true)), ("flatMap", V::fun(vec![], |_, callback| V::fun(vec![callback], |captures, value| captures[0].apply(value))))]);
                let bind = V::Tuple(Shared::new(vec![V::Int(2), V::fun(vec![], |_, state| match state {{ V::ListEmpty => V::Int(0), _ => binary("+", state.list_element(0), V::Int(1)) }})]));
                let mut steps = vec![bind; 20000];
                steps.push(V::Tuple(Shared::new(vec![V::Int(3), V::fun(vec![], |_, state| state.list_element(0))])));
                assert_eq!(native_monad_do(dictionary, V::Array(Shared::new(steps)), 0, V::ListEmpty).render(), "19999");
            }}
        "#,
            symbol("nonEmpty"),
            symbol("validation"),
            symbol("effectValue"),
            symbol("streamValue"),
            symbol("maybeValue"),
            symbol("eitherValue"),
            symbol("readerValue"),
            symbol("stateValue"),
            symbol("writerValue")
        );
        generated = generated.replace(
            "initialize_globals();",
            &format!("initialize_globals();{code}"),
        );
    }
    // Observe pure evaluation by instrumenting only the generated test copies.
    if trace {
        let symbol = |name: &str| {
            format!(
                "v_{}",
                name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
            )
        };
        let start = generated
            .find(&format!("fn {}_direct(", symbol("native/monads::mark")))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(
            body,
            &format!("println!(\"trace:{{}}\", {}.render());", symbol("value")),
        );
    }
    fs::write(&rust, generated).unwrap();
    let build = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(&rust)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let native = Command::new(&binary)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&native.stdout), expected);
    assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let ts = scratch.0.join("program.ts");
    fs::write(
        &ts,
        format!(
            "{}\nconsole.log(probe());\n",
            compiled.generated.typescript.replace(
                "\"@seseragi/runtime/",
                &format!("\"{}/runtime/ts/src/", root.display())
            )
        ),
    )
    .unwrap();
    if trace {
        let generated = fs::read_to_string(&ts).unwrap();
        let line = generated
            .lines()
            .find(|line| line.starts_with("const mark"))
            .unwrap();
        let generated = generated.replace(line, "const mark = (value: number): number => { console.log(`trace:${value}`); return value; };");
        fs::write(&ts, generated).unwrap();
    }
    if extras {
        let mut generated = fs::read_to_string(&ts).unwrap();
        let stream_module = format!("{}/runtime/ts/src/stream.ts", root.display());
        generated = generated.replace("console.log(probe());", &format!("console.log(nonEmpty().head); console.log(validation()); console.log(await effectValue()({{}})); const streamRuntime = await import({stream_module:?}); const streamValues = await streamRuntime.runCollect(streamValue())({{}}); for (const value of streamValues) console.log(value); console.log(maybeValue().run.value.value); console.log(eitherValue().run[0].value); console.log(readerValue().run(\"env\").value); console.log(stateValue().run(7).value[0]); console.log(writerValue().run.value[0]); console.log(probe());"));
        fs::write(&ts, generated).unwrap();
    }
    let output = Command::new("bun").arg(&ts).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(native.stdout, output.stdout);
}

#[test]
fn standard_dictionaries_and_do_match_typescript() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do.stdout"),
    );
}

#[test]
fn do_preserves_short_circuit_depth_first_order_and_user_dictionary_staging() {
    compare_trace(
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do-trace.stdout"),
        true,
    );
}

#[test]
fn long_do_chain_matches_typescript() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do-long.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do-long.stdout"),
    );
}

#[test]
fn refutable_do_binds_remain_type_errors() {
    for source in [
        "pub fn probe -> Maybe<Int> = do { Just x <- Just (Just 1); pure x }",
        "pub fn probe -> Maybe<Int> = do { [x] <- Just [1]; pure x }",
        "pub fn probe -> Maybe<Int> = do { 1 <- Just 1; pure 1 }",
        "pub fn probe -> Maybe<Int> = do { let Just x = Just 1; pure x }",
    ] {
        let error = compile_module(CompileInput::new("refutable.ssrg", "native/monads", source))
            .unwrap_err();
        assert!(
            error.diagnostics.iter().any(|diagnostic| matches!(
                diagnostic.message_key.as_str(),
                "binding.refutable-pattern" | "do.refutable-bind-pattern"
            )),
            "{error:?}"
        );
    }
}

#[test]
fn standard_dictionaries_reject_counterfeit_provider_trait_and_factory_arguments() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
    let core = compile_module(CompileInput::new("dictionary.ssrg", "native/monads", "fn lift<F<_>> value: F<Int> -> F<Int> where Functor<F> = map (\\x -> x + 1) value\npub fn probe -> Array<Int> = lift [1]")).unwrap().core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..5 {
        let mut bad = core.clone();
        let body = &mut bad
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::probe"))
            .unwrap()
            .body;
        let CoreExpr::Call { evidence, .. } = body else {
            panic!("call")
        };
        let selected = &mut evidence[0];
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &mut selected.evidence
        else {
            panic!("standard")
        };
        match mutation {
            0 => *identity = "std/maybe::Functor".into(),
            1 => selected.constraint.trait_identity = Some("fake::Functor".into()),
            2 => type_arguments.push(CoreType::Hole),
            3 => evidence_arguments.push(
                core.functions
                    .iter()
                    .find_map(|f| match &f.body {
                        CoreExpr::Call { evidence, .. } => evidence.first().cloned(),
                        _ => None,
                    })
                    .unwrap(),
            ),
            _ => selected.constraint.arguments[0] = CoreType::Hole,
        }
        assert!(
            emit_rust_module(&bad, "probe").is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn remaining_standard_dictionaries_and_large_native_binds() {
    compare_options(
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-standard-others.ssrg"),
        "42\n42\n42\n42\n42\n42\n42\n42\n42\nJust 42\n",
        false,
        true,
    );
}

#[test]
fn do_rejects_unproven_result_payload_and_dictionary_types() {
    use seseragi_lowering::{CoreExpr, CoreMonadDoStatement, CoreType};
    let core = compile_module(CompileInput::new(
        "do.ssrg",
        "native/monads",
        "pub fn probe -> Maybe<Int> = do { x <- Just 1; pure (x + 1) }",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..5 {
        let mut bad = core.clone();
        let CoreExpr::MonadDo {
            evidence,
            statements,
            type_ref,
            result,
            ..
        } = &mut bad.functions[0].body
        else {
            panic!("do")
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => evidence.constraint.trait_identity = Some("fake::Monad".into()),
            2 => evidence.constraint.arguments.clear(),
            3 => {
                let CoreMonadDoStatement::Bind { type_ref, .. } = &mut statements[0] else {
                    panic!("bind")
                };
                *type_ref = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                };
            }
            _ => {
                let CoreExpr::Call { type_ref, .. } = result.as_mut() else {
                    panic!("pure")
                };
                *type_ref = CoreType::Hole;
            }
        }
        let error = emit_rust_module(&bad, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "do.ssrg");
    }
}

#[test]
fn user_monad_supertraits_keep_their_checked_method_schemes() {
    use seseragi_lowering::{CoreInstanceImplementation, CoreType};
    let core = compile_module(CompileInput::new(
        "instance.ssrg",
        "native/monads",
        include_str!("../../../examples/spec/fixtures/rust-backend/monad-do.ssrg"),
    ))
    .unwrap()
    .core_ir;
    for mutation in 0..5 {
        let mut bad = core.clone();
        let instance = bad
            .instances
            .iter_mut()
            .find(|instance| instance.identity.starts_with("std/prelude::Monad<"))
            .unwrap();
        match mutation {
            0 => instance.supertrait_count = 0,
            1 => instance.constraints[0].trait_identity = Some("fake::Applicative".into()),
            2 => instance.constraints[0].arguments.clear(),
            3 => {
                let CoreInstanceImplementation::UserDefined { methods } =
                    &mut instance.implementation
                else {
                    panic!("methods")
                };
                methods[0].parameters[0].type_ref = CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                };
            }
            _ => {
                let CoreInstanceImplementation::UserDefined { methods } =
                    &mut instance.implementation
                else {
                    panic!("methods")
                };
                methods[0].type_parameters.clear();
            }
        }
        assert_eq!(
            emit_rust_module(&bad, "probe").unwrap_err().code,
            "SES-RUST-0001"
        );
    }
}
