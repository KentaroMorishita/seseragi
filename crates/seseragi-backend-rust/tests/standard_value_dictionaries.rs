use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_module, CompileInput};
use std::{fs, path::PathBuf, process::Command};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("seseragi-standard-values-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn standard_display_and_append_dictionaries_match_typescript() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/standard-value-dictionaries.ssrg"
    );
    let compiled = compile_module(CompileInput::new("values.ssrg", "native/values", source))
        .unwrap_or_else(|e| panic!("{e:?}"));
    let artifact = emit_rust_module(&compiled.core_ir, "probe").unwrap_or_else(|e| panic!("{e:?}"));
    let scratch = Scratch::new();
    let rust = scratch.0.join("program.rs");
    let binary = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let symbol = |name: &str| {
        format!(
            "v_{}",
            format!("native/values::{name}")
                .bytes()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    };
    let extras = format!(
        r#"
        println!("{{}}", {}(vec![]).apply(V::adt("std/validation::Invalid", native_non_empty(vec![V::adt("native/values::Mark", V::Int(3)), V::adt("native/values::Mark", V::Int(4))]))).render());
        println!("{{}}", {}(vec![]).apply(V::adt("std/prelude::Sum", V::Int(7))).render());
        println!("{{}}", {}(vec![]).apply(V::adt("std/prelude::Product", V::Int(7))).render());
    "#,
        symbol("invalidProbe"),
        symbol("sumProbe"),
        symbol("productProbe")
    );
    fs::write(
        &rust,
        artifact.source.replace(
            "initialize_globals();",
            &format!("initialize_globals();{extras}"),
        ),
    )
    .unwrap();
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
    let generated = fs::read_to_string(&ts).unwrap().replace("console.log(probe());", "console.log(invalidProbe({tag: 'Invalid', value: {head: {tag: 'Mark', value: 3}, tail: {tag: 'Cons', head: {tag: 'Mark', value: 4}, tail: {tag: 'Empty'}}}})); console.log(sumProbe({tag: 'Sum', value: 7})); console.log(productProbe({tag: 'Product', value: 7})); console.log(probe());");
    fs::write(&ts, generated).unwrap();
    let output = Command::new("bun").arg(&ts).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(native.stdout, output.stdout);
    assert_eq!(
        String::from_utf8_lossy(&native.stdout),
        format!(
            "Invalid `[mark:3, mark:4]|Invalid `[debug:3, debug:4]\n14\n49\n{}",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/standard-value-dictionaries.stdout"
            )
        )
    );
}

#[test]
fn standard_values_reject_counterfeit_providers_types_and_children() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};
    for source in [
        "fn render<A> value: A -> String where Show<A> = show value\nfn value -> Either<String, Int> = Right 1\npub fn probe -> String = render (value ())",
        "import * as maps from \"std/map\"\nfn render<A> value: A -> String where Debug<A> = debug value\npub fn probe -> String = render (maps.singleton 1 2)",
        "import * as sets from \"std/set\"\nfn render<A> value: A -> String where Show<A> = show value\npub fn probe -> String = render (sets.singleton 1)",
        "fn merge<A> value: A -> A where Monoid<A> = append (empty ()) value\npub fn probe -> Array<Int> = merge [1]",
        "fn merge<A> value: A -> A where Monoid<A> = append (empty ()) value\npub fn probe -> Maybe<String> = merge (Just \"a\")",
        "fn merge<A> value: A -> A where Semigroup<A> = append value value\npub fn probe -> Sum<Int> = merge (empty ())",
    ] {
        let core = compile_module(CompileInput::new("evidence.ssrg", "native/values", source)).unwrap().core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..8 {
            let mut bad = core.clone();
            let CoreExpr::Call { evidence, .. } = &mut bad.functions.iter_mut().find(|f| f.symbol.ends_with("::probe")).unwrap().body else { panic!("call") };
            let selected = &mut evidence[0];
            let child = selected.clone();
            let CoreInstanceEvidence::Standard { identity, type_arguments, evidence_arguments } = &mut selected.evidence else { panic!("standard") };
            match mutation {
                0 => *identity = format!("fake::{}", selected.constraint.name),
                1 => selected.constraint.trait_identity = Some(format!("fake::{}", selected.constraint.name)),
                2 => selected.constraint.name = "Eq".into(),
                3 => selected.constraint.arguments = vec![CoreType::Hole],
                4 => type_arguments.push(CoreType::Hole),
                5 => evidence_arguments.push(child),
                6 if !evidence_arguments.is_empty() => evidence_arguments[0].constraint.trait_identity = Some("fake::Show".into()),
                7 if !evidence_arguments.is_empty() => evidence_arguments[0].constraint.arguments = vec![CoreType::Hole],
                _ => selected.constraint.arguments.clear(),
            }
            assert!(emit_rust_module(&bad, "probe").is_err(), "mutation {mutation}: {source}");
        }
    }
}

#[test]
fn standard_display_dispatch_rejects_mismatched_trait_and_method() {
    use seseragi_lowering::CoreExpr;
    let core = compile_module(CompileInput::new(
        "dispatch.ssrg",
        "native/values",
        "fn value -> Either<String, Int> = Right 1\npub fn probe -> String = show (value ())",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..2 {
        let mut bad = core.clone();
        let CoreExpr::Call {
            trait_dispatch: Some(dispatch),
            ..
        } = &mut bad
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::probe"))
            .unwrap()
            .body
        else {
            panic!("dispatch")
        };
        if mutation == 0 {
            dispatch.trait_identity = "fake::Show".into();
        } else {
            dispatch.method = "debug".into();
        }
        assert!(emit_rust_module(&bad, "probe").is_err());
    }
}
