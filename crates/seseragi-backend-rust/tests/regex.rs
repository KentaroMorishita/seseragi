use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("regex.ssrg", "native/regex", source))
        .unwrap()
        .core_ir
}

fn cases() -> [(&'static str, &'static str, &'static str); 10] {
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
        (
            "value:regex.Regex",
            "Array<regex.RegexMatch>",
            "regex.findAll value \"雫🙂\"",
        ),
        (
            "value:regex.Regex",
            "Array<String>",
            "regex.split value \"雫🙂\"",
        ),
        (
            "value:regex.Regex",
            "String",
            "regex.replaceAll value \"$1\" \"雫🙂\"",
        ),
        (
            "value:regex.Regex",
            "String",
            "regex.replaceAllWith value (\\hit:regex.RegexMatch -> hit.text) \"雫🙂\"",
        ),
        ("value:String", "String", "regex.escape value"),
    ]
}

#[test]
fn regex_admits_all_ten_apis() {
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
        (
            "findAll",
            "regex.Regex -> String -> Array<regex.RegexMatch>",
        ),
        ("split", "regex.Regex -> String -> Array<String>"),
        ("replaceAll", "regex.Regex -> String -> String -> String"),
        (
            "replaceAllWith",
            "regex.Regex -> (regex.RegexMatch -> String) -> String -> String",
        ),
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

#[test]
fn regex_collection_fixture_admits_all_matches_splits_and_replacement_callbacks() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/regex-collections.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn regex_collection_calls_validate_every_argument_and_callback_signature() {
    for (parameter, result, expression) in cases().into_iter().skip(5).take(4) {
        let core = checked(&format!("import * as regex from \"std/regex\"\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        let CoreExpr::Call { arguments, .. } = &core.functions[index].body else {
            panic!()
        };
        for argument_index in 0..arguments.len() {
            let mut changed = core.clone();
            let CoreExpr::Call { arguments, .. } = &mut changed.functions[index].body else {
                panic!()
            };
            arguments[argument_index] = CoreExpr::Boolean {
                value: true,
                origin: core_expr_origin(&arguments[argument_index]).clone(),
            };
            let error = emit_rust_module(&changed, "main").expect_err("unproven Regex argument");
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(
                error.origin.unwrap(),
                origin,
                "{expression}/{argument_index}"
            );
        }
    }
    let core = checked("import * as regex from \"std/regex\"\npub fn probe pattern:regex.Regex -> render:(regex.RegexMatch -> String) -> String = regex.replaceAllWith pattern render \"雫🙂\"\npub fn main -> Int = 0\n");
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::probe"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    for mutation in 0..4 {
        let mut changed = core.clone();
        let CoreExpr::Call { arguments, .. } = &mut changed.functions[index].body else {
            panic!()
        };
        let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
            panic!()
        };
        let CoreType::Function { parameter, result } = type_ref else {
            panic!()
        };
        match mutation {
            0 => **parameter = CoreType::Hole,
            1 => {
                **parameter = CoreType::ExternalNamed {
                    name: "RegexMatch".into(),
                    canonical: "fake::RegexMatch".into(),
                    arguments: vec![],
                }
            }
            2 => {
                **result = CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                }
            }
            3 => **result = CoreType::Hole,
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "main").expect_err("unproven Regex callback");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin, "callback/{mutation}");
    }
}

#[test]
fn regex_collection_partial_applications_validate_each_remaining_stage() {
    for (result, expression) in [
        ("String -> Array<regex.RegexMatch>", "regex.findAll value"),
        ("String -> Array<String>", "regex.split value"),
        ("String -> String -> String", "regex.replaceAll value"),
        ("String -> String", "regex.replaceAll value \"$1\""),
        (
            "(regex.RegexMatch -> String) -> String -> String",
            "regex.replaceAllWith value",
        ),
        (
            "String -> String",
            "regex.replaceAllWith value (\\hit:regex.RegexMatch -> hit.text)",
        ),
    ] {
        let mut core = checked(&format!("import * as regex from \"std/regex\"\npub fn probe value:regex.Regex -> ({result}) = {expression}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        let CoreExpr::Call { type_ref, .. } = &mut core.functions[index].body else {
            panic!()
        };
        *type_ref = CoreType::Named {
            name: "std/prelude::String".into(),
            arguments: vec![],
        };
        let error = emit_rust_module(&core, "main").expect_err("unproven remaining Regex stage");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin, "{expression}");
    }
}
