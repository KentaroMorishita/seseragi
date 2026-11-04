use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("random-雪.ssrg", "native/random", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
        .core_ir
}

#[test]
fn random_admits_all_apis_generic_saved_functions_and_partial_ranges() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/random.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "random"));
    assert_eq!(artifact.metadata.abi, "rust-erased-sync-v2");
    for name in [
        "randomSize",
        "algorithmId",
        "nextBool",
        "nextInt",
        "intBetween",
        "unitFloat",
        "chance",
        "randomBytes",
        "choose",
        "shuffle",
    ] {
        assert!(
            artifact
                .source
                .contains(&format!("native_random_operation(\"std/random::{name}\"")),
            "{name}"
        );
    }
    let saved = checked(
        r#"
import * as random from "std/random"
import { Random, RandomRangeError } from "std/random"
import { NonEmptyList } from "std/non-empty-list"
fn choose<A> values:NonEmptyList<A> -> Effect<{random:Random},Never,A> = {
  let saved:NonEmptyList<A> -> Effect<{random:Random},Never,A> = random.choose
  saved values
}
fn shuffle<A> values:Array<A> -> Effect<{random:Random},Never,Array<A>> = {
  let saved:Array<A> -> Effect<{random:Random},Never,Array<A>> = random.shuffle
  saved values
}
fn between lower:Int -> Int -> Effect<{random:Random},RandomRangeError,Int> = random.intBetween lower
fn range -> Int -> Int -> Effect<{random:Random},RandomRangeError,Int> = random.intBetween
pub fn main -> Int = 0
"#,
    );
    emit_rust_module(&saved, "main").unwrap();
}

fn cases() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "value:Int",
            "Either<random.RandomConfigError,random.RandomSize>",
            "random.randomSize value",
        ),
        (
            "value:Unit",
            "Effect<{random:random.Random},Never,String>",
            "random.algorithmId value",
        ),
        (
            "value:Unit",
            "Effect<{random:random.Random},Never,Bool>",
            "random.nextBool value",
        ),
        (
            "value:Unit",
            "Effect<{random:random.Random},Never,Int>",
            "random.nextInt value",
        ),
        (
            "value:Unit",
            "Effect<{random:random.Random},Never,Float>",
            "random.unitFloat value",
        ),
        (
            "value:Int",
            "Effect<{random:random.Random},random.RandomRangeError,Int>",
            "random.intBetween value 10",
        ),
        (
            "value:Float",
            "Effect<{random:random.Random},random.RandomRangeError,Bool>",
            "random.chance value",
        ),
        (
            "value:random.RandomSize",
            "Effect<{random:random.Random},Never,bytes.Bytes>",
            "random.randomBytes value",
        ),
        (
            "value:nonEmpty.NonEmptyList<Int>",
            "Effect<{random:random.Random},Never,Int>",
            "random.choose value",
        ),
        (
            "value:Array<Int>",
            "Effect<{random:random.Random},Never,Array<Int>>",
            "random.shuffle value",
        ),
    ]
}

#[test]
fn random_rejects_forged_types_evidence_and_source_stages_at_the_original_span() {
    let eq = checked("pub fn main -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as random from \"std/random\"\nimport * as bytes from \"std/bytes\"\nimport * as nonEmpty from \"std/non-empty-list\"\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..9 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[probe].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => arguments.clear(),
                2 => arguments.extend(vec![arguments[0].clone(); 3]),
                3 => evidence.push(other[0].clone()),
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
                        name: "Effect".into(),
                        canonical: "fake::Effect".into(),
                        arguments: vec![],
                    }
                }
                _ => unreachable!(),
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven Random call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn random_effects_reject_forged_requirement_owners_rows_results_and_arity() {
    let core = checked(
        "import * as random from \"std/random\"\npub effect fn main = random.intBetween 0 1\n",
    );
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..9 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            type_ref,
            arguments,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        let (CoreType::Named {
            arguments: types, ..
        }
        | CoreType::ExternalNamed {
            arguments: types, ..
        }) = type_ref
        else {
            panic!()
        };
        let [requirements, failure, success] = types.as_mut_slice() else {
            panic!()
        };
        let CoreType::Record { closed, fields } = requirements else {
            panic!()
        };
        match mutation {
            0 => *closed = false,
            1 => fields[0].optional = true,
            2 => fields[0].name = "other".into(),
            3 => {
                fields[0].type_ref = CoreType::ExternalNamed {
                    name: "Random".into(),
                    canonical: "fake::Random".into(),
                    arguments: vec![],
                }
            }
            4 => fields.push(fields[0].clone()),
            5 => *failure = CoreType::Hole,
            6 => *success = CoreType::Hole,
            7 => arguments.pop().map(|_| ()).unwrap(),
            8 => {
                arguments[1] = CoreExpr::Unit {
                    origin: core_expr_origin(&arguments[1]).clone(),
                }
            }
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "main").expect_err("unproven Random effect");
        assert!(error.origin.is_some(), "mutation {mutation}: {error}");
        assert!(
            error.message.contains("Random"),
            "mutation {mutation}: {error}"
        );
    }
}

#[test]
fn unused_random_functions_do_not_expand_program_wasm_admission() {
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as random from \"std/random\"\nimport * as bytes from \"std/bytes\"\nimport * as nonEmpty from \"std/non-empty-list\"\nfn hidden {parameter} -> {result} = {expression}\npub fn main -> Int = 42\n"));
        let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert!(error.origin.is_some(), "{expression}: {error}");
        assert!(
            error.message.contains("native Random"),
            "{expression}: {error}"
        );
    }
}
