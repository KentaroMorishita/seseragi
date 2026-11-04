#[path = "../../test-support/native_runtime.rs"]
mod link;
#[path = "../../test-support/temporary.rs"]
mod temporary;

use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{
    core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreTraitDispatch, CoreType,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SOURCE: &str = include_str!("../../../examples/spec/fixtures/rust-backend/text-unicode.ssrg");
const EXPECTED: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/text-unicode.stdout");
const APIS: &[(&str, &str, &str, &str)] = &[
    ("version", "Unit", "String", "unicode.version value"),
    (
        "normalize",
        "String",
        "String",
        "unicode.normalize unicode.NFC value",
    ),
    (
        "isNormalized",
        "String",
        "Bool",
        "unicode.isNormalized unicode.NFC value",
    ),
    (
        "generalCategory",
        "Char",
        "unicode.UnicodeGeneralCategory",
        "unicode.generalCategory value",
    ),
    (
        "simpleCaseFold",
        "Char",
        "Char",
        "unicode.simpleCaseFold value",
    ),
    (
        "fullCaseFold",
        "String",
        "String",
        "unicode.fullCaseFold value",
    ),
    ("isAlphabetic", "Char", "Bool", "unicode.isAlphabetic value"),
    ("isWhitespace", "Char", "Bool", "unicode.isWhitespace value"),
    (
        "isDecimalDigit",
        "Char",
        "Bool",
        "unicode.isDecimalDigit value",
    ),
    ("isMark", "Char", "Bool", "unicode.isMark value"),
];

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        Self(temporary::temporary_directory("seseragi-unicode"))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn successful(command: &mut Command) -> Output {
    let output = command.output().unwrap();
    assert!(output.status.success(), "{command:?}\n{output:?}");
    output
}

fn compare(source: &str, expected: &str) {
    let scratch = Scratch::new();
    let core = checked(source);
    let artifact = emit_rust_module(&core, "probe").unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "text"));
    let rust = scratch.0.join("program.rs");
    let executable = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    link::write_program(&rust, artifact.source).unwrap();
    successful(
        Command::new("rustc")
            .args(["--edition=2021", "-O"])
            .arg(&rust)
            .arg("-o")
            .arg(&executable),
    );
    let native = successful(
        Command::new(executable)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
    );
    assert_eq!(native.stdout, expected.as_bytes());
    assert!(
        String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
        "{native:?}"
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    for profile in [
        seseragi_driver::BuildProfile::Development,
        seseragi_driver::BuildProfile::Release,
    ] {
        let compiled = seseragi_driver::compile_module(
            CompileInput::new("unicode.ssrg", "native/unicode", source).with_profile(profile),
        )
        .unwrap();
        let program = compiled.generated.typescript.replace(
            "@seseragi/runtime/",
            &format!(
                "{}/runtime/ts/src/",
                root.to_string_lossy().replace('\\', "/")
            ),
        );
        let raw = scratch.0.join("program.ts");
        fs::write(&raw, program).unwrap();
        fs::write(
            scratch.0.join("entry.ts"),
            "import { probe } from './program.ts'; console.log(probe());\n",
        )
        .unwrap();
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root.join("node_modules/@types")],"lib":["ES2023","DOM","DOM.Iterable"],"allowImportingTsExtensions":true},"files":[raw]});
        let config_path = scratch.0.join("tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(
            Command::new("node")
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&config_path),
        );
        let ts = successful(Command::new("bun").arg(scratch.0.join("entry.ts")));
        assert_eq!(ts.stdout, native.stdout, "{profile:?}");
        assert!(ts.stderr.is_empty(), "{ts:?}");
        let bundled = scratch.0.join("entry.js");
        successful(
            Command::new("bun")
                .arg("build")
                .arg(scratch.0.join("entry.ts"))
                .arg("--target=node")
                .arg("--outfile")
                .arg(&bundled),
        );
        let node = successful(Command::new("node").arg(bundled));
        assert_eq!(node.stdout, native.stdout, "{profile:?}");
        assert!(node.stderr.is_empty(), "{node:?}");
    }
}

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "unicode-雪🙂.ssrg",
        "native/unicode",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn unicode_accepts_native_calls() {
    let core = checked("import * as unicode from \"std/text/unicode\"\npub fn probe -> String = unicode.normalize unicode.NFC \"e\\u{301}\"\n");
    emit_rust_module(&core, "probe").unwrap();
}

#[test]
fn unicode_matches_raw_strict_ts_bun_node_and_native() {
    compare(SOURCE, EXPECTED);
}

#[test]
fn unicode_rejects_unproven_argument_result_evidence_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (operation, parameter, result, expression) in APIS {
        let core = checked(&format!("import * as unicode from \"std/text/unicode\"\npub fn probe value: {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..11 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[probe].body
            else {
                panic!("{operation}")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: (*result).into(),
                        canonical: "fake::Result".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/text/unicode::UnicodeGeneralCategory".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                _ => {
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven Unicode call");
            assert_eq!(error.code, "SES-RUST-0001", "{operation}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{operation}/{mutation}");
        }
    }
}

#[test]
fn unicode_keeps_unused_program_wasm_references_rejected() {
    for (operation, parameter, result, _) in APIS {
        let signature = if matches!(*operation, "normalize" | "isNormalized") {
            format!("unicode.NormalizationForm -> {parameter} -> {result}")
        } else {
            format!("{parameter} -> {result}")
        };
        let core = checked(&format!("import * as unicode from \"std/text/unicode\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = unicode.{operation}; value }}\npub fn main -> Int = probe 42\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/unicode", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific proof"),
            "{operation}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "unicode-雪🙂.ssrg");
    }
}

#[test]
fn unicode_invalid_types_forms_and_scalars_match_ts_diagnostics() {
    for expression in [
        "unicode.version 1",
        "unicode.normalize \"NFC\" \"a\"",
        "unicode.normalize unicode.NFC 'a'",
        "unicode.isNormalized unicode.NFC True",
        "unicode.isNormalized unicode.UppercaseLetter \"a\"",
        "unicode.generalCategory \"\"",
        "unicode.simpleCaseFold \"ß\"",
        "unicode.fullCaseFold 'a'",
        "unicode.isAlphabetic \"a\"",
        "unicode.isWhitespace 32",
        "unicode.isDecimalDigit True",
        "unicode.isMark ()",
        "unicode.simpleCaseFold '\\u{d800}'",
        "unicode.generalCategory '\\u{110000}'",
        "unicode.normalize unicode.NFC \"\\u{dfff}\"",
        "unicode.normalize unicode.NFC \"\\u{110000}\"",
        "unicode.isMark ''",
        "unicode.isMark 'ab'",
        "unicode.normalize NFC \"a\"",
    ] {
        let source = format!("import * as unicode from \"std/text/unicode\"\ntype NormalizationForm = | NFC\npub fn probe = {expression}\n");
        let input = CompileInput::new("unicode-type.ssrg", "native/unicode", &source);
        let ts = seseragi_driver::compile_module(input).unwrap_err();
        let native = compile_core_module(input).unwrap_err();
        assert!(!ts.diagnostics.is_empty(), "{expression}");
        assert_eq!(
            serde_json::to_value(ts).unwrap(),
            serde_json::to_value(native).unwrap(),
            "{expression}"
        );
    }
}

#[test]
fn unicode_rejects_forged_saved_functions_and_enum_constructors() {
    let other = checked("pub fn main -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &other.functions[0].body
    else {
        panic!()
    };
    let mut cases = APIS
        .iter()
        .map(|(name, parameter, result, _)| {
            let signature = if matches!(*name, "normalize" | "isNormalized") {
                format!("unicode.NormalizationForm -> {parameter} -> {result}")
            } else {
                format!("{parameter} -> {result}")
            };
            ((*name).to_owned(), signature)
        })
        .collect::<Vec<_>>();
    for (owner, constructors) in [
        ("NormalizationForm", vec!["NFC", "NFD", "NFKC", "NFKD"]),
        (
            "UnicodeGeneralCategory",
            vec![
                "UppercaseLetter",
                "LowercaseLetter",
                "TitlecaseLetter",
                "ModifierLetter",
                "OtherLetter",
                "NonspacingMark",
                "SpacingMark",
                "EnclosingMark",
                "DecimalNumber",
                "LetterNumber",
                "OtherNumber",
                "ConnectorPunctuation",
                "DashPunctuation",
                "OpenPunctuation",
                "ClosePunctuation",
                "InitialPunctuation",
                "FinalPunctuation",
                "OtherPunctuation",
                "MathSymbol",
                "CurrencySymbol",
                "ModifierSymbol",
                "OtherSymbol",
                "SpaceSeparator",
                "LineSeparator",
                "ParagraphSeparator",
                "Control",
                "Format",
                "PrivateUse",
                "Unassigned",
            ],
        ),
    ] {
        cases.extend(
            constructors
                .into_iter()
                .map(|name| (name.to_owned(), format!("unicode.{owner}"))),
        );
    }
    for (name, signature) in cases {
        let core = checked(&format!("import * as unicode from \"std/text/unicode\"\npub fn probe -> {signature} = unicode.{name}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..4 {
            let mut changed = core.clone();
            let CoreExpr::Variable {
                type_ref, evidence, ..
            } = &mut changed.functions[probe].body
            else {
                panic!("{name}")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: signature.clone(),
                        canonical: "fake::Unicode".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/text/unicode::NormalizationForm".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                _ => evidence.push(other[0].clone()),
            }
            let error = emit_rust_module(&changed, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn unicode_dictionaries_require_the_exact_owner_class_and_empty_factory() {
    for owner in ["NormalizationForm", "UnicodeGeneralCategory"] {
        for (class, expression, result) in [
            ("Eq", "value == value", "Bool"),
            ("Show", "show value", "String"),
            ("Debug", "debug value", "String"),
            ("Ord", "compare value value", "Ordering"),
        ] {
            if owner == "NormalizationForm" && class == "Ord" {
                continue;
            }
            let core = checked(&format!("import * as unicode from \"std/text/unicode\"\npub fn probe value: unicode.{owner} -> {result} = {expression}\npub fn main -> Int = 0\n"));
            emit_rust_module(&core, "main").unwrap();
            let probe = core
                .functions
                .iter()
                .position(|f| f.symbol.ends_with("::probe"))
                .unwrap();
            let origin = core_expr_origin(&core.functions[probe].body).clone();
            for mutation in 0..7 {
                let mut changed = core.clone();
                let (CoreExpr::Call { evidence, .. } | CoreExpr::Binary { evidence, .. }) =
                    &mut changed.functions[probe].body
                else {
                    panic!("{class}")
                };
                let other = evidence[0].clone();
                let selected = &mut evidence[0];
                match mutation {
                    0 => selected.constraint.name = "Hash".into(),
                    1 => selected.constraint.trait_identity = Some(format!("fake::{class}")),
                    2 => {
                        selected.constraint.arguments = vec![CoreType::ExternalNamed {
                            name: owner.into(),
                            canonical: format!("fake::{owner}"),
                            arguments: vec![],
                        }]
                    }
                    3 => selected.constraint.arguments.clear(),
                    4 => selected
                        .constraint
                        .arguments
                        .push(selected.constraint.arguments[0].clone()),
                    mutation => {
                        let CoreInstanceEvidence::Standard {
                            type_arguments,
                            evidence_arguments,
                            ..
                        } = &mut selected.evidence
                        else {
                            panic!()
                        };
                        if mutation == 5 {
                            type_arguments.push(CoreType::Hole);
                        } else {
                            evidence_arguments.push(other);
                        }
                    }
                }
                let error = emit_rust_module(&changed, "main").unwrap_err();
                assert_eq!(error.code, "SES-RUST-0001", "{owner}/{class}/{mutation}");
                assert_eq!(error.origin.unwrap(), origin, "{owner}/{class}/{mutation}");
            }
        }
    }
}
