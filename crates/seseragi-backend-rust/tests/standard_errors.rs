use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn every_registered_standard_error_instance_is_available_to_native() {
    let surface = serde_json::to_value(seseragi_semantics::standard_prelude_surface()).unwrap();
    let mut checked = 0;
    for instance in surface["instances"].as_array().unwrap() {
        let owner = instance["typeConstructorCanonical"].as_str().unwrap();
        if !owner.contains("Error")
            && !owner.ends_with("::QueueClosed")
            && !owner.ends_with("::BenchmarkFailure")
        {
            continue;
        }
        let (module, name) = owner.rsplit_once("::").unwrap();
        let ty = if module == "std/prelude" {
            name.to_owned()
        } else {
            format!("errors.{name}")
        };
        let ty = if owner == "std/web/dom::DomRuntimeError" {
            format!("{ty}<String>")
        } else {
            ty
        };
        let (method, result, argument) = match instance["trait"].as_str().unwrap() {
            "Eq" => ("eq", "Bool", "value value"),
            "Show" => ("show", "String", "value"),
            "Debug" => ("debug", "String", "value"),
            other => panic!("untested standard error instance: {owner} {other}"),
        };
        let import = if module == "std/prelude" {
            String::new()
        } else {
            format!("import * as errors from \"{module}\"\n")
        };
        let source = format!(
            "{import}pub fn exercise value: {ty} -> {result} = {method} {argument}\npub fn probe -> String = \"ok\""
        );
        let core = compile_core_module(CompileInput::new("errors.ssrg", "native/errors", &source))
            .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
            .core_ir;
        emit_rust_module(&core, "probe").unwrap_or_else(|error| panic!("{source}\n{error:?}"));
        if owner == "std/benchmark::BenchmarkFailure" {
            assert!(
                emit_program_wasm(&[&core], "native/errors", &["exercise".into()]).is_err(),
                "{source}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 119);
}

#[test]
fn standard_errors_do_not_gain_instances_absent_from_ts() {
    for (module, name, methods) in [
        ("std/int", "IntParseError", &["eq", "debug"][..]),
        ("std/float", "FloatParseError", &["eq", "show", "debug"][..]),
        (
            "std/float",
            "FloatConversionError",
            &["eq", "show", "debug"][..],
        ),
        ("std/benchmark", "BenchmarkFailure", &["debug"][..]),
        ("std/path", "PathError", &["eq"][..]),
    ] {
        for method in methods {
            let (result, argument) = if *method == "eq" {
                ("Bool", "value value")
            } else {
                ("String", "value")
            };
            let source = format!(
                "import * as errors from \"{module}\"\npub fn probe value: errors.{name} -> {result} = {method} {argument}"
            );
            assert!(
                compile_core_module(CompileInput::new("errors.ssrg", "native/errors", &source))
                    .is_err(),
                "{source}"
            );
        }
    }
}

#[test]
fn standard_error_instances_reject_forged_evidence_and_method_types() {
    for (module, method, argument, result) in [
        (
            "benchmark",
            "show",
            "(errors.ExplicitBenchmarkFailure \"雪🙂\")",
            "String",
        ),
        (
            "benchmark",
            "eq",
            "(errors.ExplicitBenchmarkFailure \"雪🙂\") (errors.ExplicitBenchmarkFailure \"雪\")",
            "Bool",
        ),
        (
            "collection",
            "eq",
            "(errors.NonPositiveSize 0) (errors.NonPositiveSize (-1))",
            "Bool",
        ),
    ] {
        let source = format!(
            "import * as errors from \"std/{module}\"\npub fn probe -> {result} = {method} {argument}"
        );
        let core = compile_core_module(CompileInput::new(
            "errors-雪🙂.ssrg",
            "native/errors",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..12 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                evidence,
                trait_dispatch,
                arguments,
                type_ref,
                ..
            } = &mut changed.functions[0].body
            else {
                panic!("expected a method call")
            };
            let child = evidence[0].clone();
            let selected = &mut evidence[0];
            let CoreInstanceEvidence::Standard {
                identity,
                type_arguments,
                evidence_arguments,
            } = &mut selected.evidence
            else {
                panic!("expected a standard dictionary")
            };
            match mutation {
                0 => *identity = "counterfeit::ErrorInstance".into(),
                1 => selected.constraint.trait_identity = Some("counterfeit::Eq".into()),
                2 => selected.constraint.name = "Debug".into(),
                3 => selected.constraint.arguments.clear(),
                4 => selected.constraint.arguments[0] = CoreType::Hole,
                5 => {
                    selected.constraint.arguments[0] = CoreType::ExternalNamed {
                        name: "BenchmarkFailure".into(),
                        canonical: "counterfeit::BenchmarkFailure".into(),
                        arguments: vec![],
                    }
                }
                6 => type_arguments.push(CoreType::Hole),
                7 => evidence_arguments.push(child),
                8 => trait_dispatch.as_mut().unwrap().trait_identity = "counterfeit::Eq".into(),
                9 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
                10 => arguments.clear(),
                _ => *type_ref = CoreType::Hole,
            }
            let error = emit_rust_module(&changed, "probe")
                .expect_err("forged standard error method must be rejected");
            assert_eq!(error.code, "SES-RUST-0001", "{source}: mutation {mutation}");
            assert_eq!(
                error.origin.as_ref().unwrap().source,
                "errors-雪🙂.ssrg",
                "{source}: mutation {mutation}"
            );
        }
    }
}
