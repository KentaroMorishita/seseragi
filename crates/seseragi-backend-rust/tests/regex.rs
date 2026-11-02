use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("regex.ssrg", "native/regex", source))
        .unwrap()
        .core_ir
}

fn cases() -> [(&'static str, &'static str, &'static str); 6] {
    [
        ("", "regex.RegexOptions", "regex.defaultOptions ()"),
        (
            "value:String",
            "Either<regex.RegexCompileError,regex.Regex>",
            "regex.compile value",
        ),
        (
            "value:String",
            "Either<regex.RegexCompileError,regex.Regex>",
            "regex.compileWith (regex.defaultOptions ()) value",
        ),
        ("value:regex.Regex", "Bool", "regex.isMatch value \"雫🙂\""),
        (
            "value:regex.Regex",
            "Maybe<regex.RegexMatch>",
            "regex.find value \"雫🙂\"",
        ),
        ("value:String", "String", "regex.escape value"),
    ]
}

#[test]
fn regex_admits_all_six_apis() {
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as regex from \"std/regex\"\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|name| name == "regex"));
    }
}

#[test]
fn regex_rejects_forged_types_evidence_arity_and_source_stages_at_the_call_span() {
    let equality = checked("pub fn main -> Bool = 1 == 1");
    let CoreExpr::Binary { evidence, .. } = &equality.functions[0].body else {
        panic!()
    };
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as regex from \"std/regex\"\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..9 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence: selected,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[index].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => arguments.clear(),
                2 => arguments.push(arguments[0].clone()),
                3 => selected.push(evidence[0].clone()),
                4 => deferred_evidence_parameters.push(CoreType::Hole),
                5 => deferred_evidence_type_constructor_parameters.push("F".into()),
                6 => *evidence_argument_index = Some(0),
                7 => {
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                8 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "Result".into(),
                        canonical: "fake::Result".into(),
                        arguments: vec![],
                    }
                }
                _ => unreachable!(),
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven Regex call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn regex_saved_references_require_the_complete_canonical_signature() {
    for (name, ty) in [
        ("defaultOptions", "Unit -> regex.RegexOptions"),
        (
            "compile",
            "String -> Either<regex.RegexCompileError,regex.Regex>",
        ),
        (
            "compileWith",
            "regex.RegexOptions -> String -> Either<regex.RegexCompileError,regex.Regex>",
        ),
        ("isMatch", "regex.Regex -> String -> Bool"),
        ("find", "regex.Regex -> String -> Maybe<regex.RegexMatch>"),
        ("escape", "String -> String"),
    ] {
        let core = checked(&format!("import * as regex from \"std/regex\"\npub fn saved -> {ty} = regex.{name}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::saved"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..3 {
            let mut changed = core.clone();
            let CoreExpr::Variable { type_ref, .. } = &mut changed.functions[index].body else {
                panic!()
            };
            *type_ref = match mutation {
                0 => CoreType::Hole,
                1 => CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                },
                _ => CoreType::Function {
                    parameter: Box::new(CoreType::ExternalNamed {
                        name: "Regex".into(),
                        canonical: "fake::Regex".into(),
                        arguments: vec![],
                    }),
                    result: Box::new(type_ref.clone()),
                },
            };
            let error = emit_rust_module(&changed, "main").expect_err("unproven Regex reference");
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
        let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("native Regex"), "{error}");
    }
}

#[test]
fn unused_regex_calls_keep_program_wasm_admission_unchanged() {
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as regex from \"std/regex\"\nfn hidden {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
        assert!(error.message.contains("native Regex"), "{error}");
    }
}

#[test]
fn regex_execution_fixture_admits_records_captures_and_partial_applications() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/regex-basic.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
}
