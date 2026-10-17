use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn constrained_methods_keep_source_stages_and_saved_child_dictionaries() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/method-dictionaries.ssrg");
    run(
        source,
        "probe",
        include_str!("../../../examples/spec/fixtures/rust-backend/method-dictionaries.stdout"),
        0,
    );
}

fn run(source: &str, entry: &str, expected: &str, exit: i32) {
    let core = compile_core_module(CompileInput::new("method-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, entry).unwrap();
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "seseragi-methods-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let input = scratch.0.join("program.rs");
    let binary = scratch.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, artifact.source).unwrap();
    let output = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&binary)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.code() == Some(exit),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn constrained_methods_keep_cold_effects_refs_cycles_and_resource_finalizers() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-dictionary-effects.ssrg"),
        "main",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/method-dictionary-effects.stdout"
        ),
        0,
    );
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/method-dictionary-defect.ssrg"),
        "main",
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/method-dictionary-defect.stdout"
        ),
        70,
    );
}

#[test]
fn method_contracts_reject_counterfeit_source_and_dictionary_stages() {
    use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreInstanceImplementation, CoreType};
    let source = "// 雫🙂\ntype Token = | Token\ntrait Render<T> { fn render<A> tag: T -> value: A -> String where Show<A> }\ninstance Render<Token> { fn render<A> tag: Token -> value: A -> String where Show<A> = show value }\npub fn probe -> String = render Token 42\n";
    let core = compile_core_module(CompileInput::new(
        "counterfeit-λ.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for case in 0..26 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch: Some(dispatch),
            deferred_evidence_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("method call")
        };
        let contract = dispatch.method_contract.as_mut().unwrap();
        match case {
            0 => evidence.clear(),
            1 => {
                evidence.pop();
            }
            2 => evidence.push(evidence[1].clone()),
            3 => evidence[0].constraint.trait_identity = Some("other::Render".into()),
            4 => dispatch.method = "absent".into(),
            5 => dispatch.method_contract = None,
            6 => contract.parameters.push(contract.parameters[0].clone()),
            7 => {
                contract.parameters[1] = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            8 => contract.constraints[1].trait_identity = None,
            9 => {
                contract.constraints[1].name = "Debug".into();
                contract.constraints[1].trait_identity = Some("std/prelude::Debug".into());
            }
            10 => {
                let CoreInstanceEvidence::Local { type_arguments, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            11 => {
                let CoreInstanceEvidence::Local {
                    identity,
                    type_arguments,
                    evidence_arguments,
                } = evidence[0].evidence.clone()
                else {
                    panic!()
                };
                evidence[0].evidence = CoreInstanceEvidence::Imported {
                    identity,
                    type_arguments,
                    evidence_arguments,
                    provider_module: "other/provider".into(),
                };
            }
            12 => evidence[1].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            13 => deferred_evidence_parameters.push(CoreType::Hole),
            14 => *evidence_argument_index = Some(0),
            15 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[1].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            16..=20 | 23 => {
                let CoreInstanceImplementation::UserDefined { methods } =
                    &mut changed.instances[0].implementation
                else {
                    panic!()
                };
                let method = &mut methods[0];
                match case {
                    16 => method.constraints.clear(),
                    17 => {
                        method.parameters.pop();
                    }
                    18 => {
                        let CoreExpr::Call { evidence, .. } = &mut method.body else {
                            panic!()
                        };
                        evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 };
                    }
                    19 => method.type_parameters.clear(),
                    20 => method.constraints[0].trait_identity = Some("other::Show".into()),
                    23 => method.type_parameters[0].arity = 1,
                    _ => unreachable!(),
                }
            }
            21 => {
                evidence[1].constraint.arguments[0] = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            22 => contract.constraints[0].trait_identity = Some("std/prelude::Show".into()),
            24 => contract.parameters.clear(),
            25 => arguments.push(arguments[1].clone()),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "probe").expect_err(&format!("forged case {case}"));
        assert_eq!(error.code, "SES-RUST-0001", "case {case}: {error:?}");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "counterfeit-λ.ssrg");
        assert!(
            origin.start < origin.end && origin.end <= source.len(),
            "case {case}"
        );
        assert!(source.is_char_boundary(origin.start) && source.is_char_boundary(origin.end));
    }
}

#[test]
fn constrained_methods_keep_standard_ref_owner_under_nominal_shadowing() {
    let source = r#"
import * as refs from "std/ref"
import { Ref as Cell } from "std/ref"
type Ref<A> = | Ref A
trait Describe<T> { fn describe<A> state: T -> value: A -> String where Show<A> }
instance Describe<Cell<String>> {
 fn describe<A> state: Cell<String> -> value: A -> String where Show<A> = show value
}
pub effect fn main = do {
 state <- refs.make "kept"
 println (describe state 42)
}
"#;
    run(source, "main", "42\n", 0);
}

#[test]
fn factory_and_method_frames_cannot_exchange_equal_named_traits() {
    use seseragi_lowering::{CoreExpr, CoreInstanceImplementation, CoreType};
    let source = r#"// 雫🙂
type Holder<T> = | Holder T
trait Render<T> { fn render<A> owner: T -> value: A -> String where Show<A> }
instance<T> Render<Holder<T>> where Show<T> {
 fn render<A> owner: Holder<T> -> value: A -> String where Show<A> = match owner {
   Holder first -> show first + ":" + show value
 }
}
pub fn probe -> String = render (Holder 7) True
"#;
    let core = compile_core_module(CompileInput::new("factory-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for case in 0..5 {
        let mut changed = core.clone();
        if case < 3 {
            let CoreExpr::Call { evidence, .. } = &mut changed.functions[0].body else {
                panic!()
            };
            let seseragi_lowering::CoreInstanceEvidence::Local {
                type_arguments,
                evidence_arguments,
                ..
            } = &mut evidence[0].evidence
            else {
                panic!()
            };
            match case {
                0 => {
                    type_arguments[0] = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    }
                }
                1 => {
                    evidence_arguments[0].constraint.arguments[0] = CoreType::Named {
                        name: "Bool".into(),
                        arguments: vec![],
                    }
                }
                2 => evidence_arguments.clear(),
                _ => unreachable!(),
            }
        } else {
            let CoreInstanceImplementation::UserDefined { methods } =
                &mut changed.instances[0].implementation
            else {
                panic!()
            };
            if case == 3 {
                methods[0].constraints[0].arguments[0] = CoreType::Named {
                    name: "T".into(),
                    arguments: vec![],
                };
            } else {
                fn flip(value: &mut serde_json::Value) -> usize {
                    if value.get("kind").and_then(|v| v.as_str()) == Some("parameter")
                        && value.get("index").and_then(|v| v.as_u64()) == Some(1)
                    {
                        value["index"] = 0.into();
                        return 1;
                    }
                    match value {
                        serde_json::Value::Object(fields) => fields.values_mut().map(flip).sum(),
                        serde_json::Value::Array(values) => values.iter_mut().map(flip).sum(),
                        _ => 0,
                    }
                }
                let mut value = serde_json::to_value(&methods[0].body).unwrap();
                assert!(flip(&mut value) > 0);
                methods[0].body = serde_json::from_value(value).unwrap();
            }
        }
        let error = emit_rust_module(&changed, "probe").expect_err(&format!("factory case {case}"));
        assert_eq!(error.code, "SES-RUST-0001");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "factory-λ.ssrg");
        assert!(
            origin.start < origin.end
                && source.is_char_boundary(origin.start)
                && source.is_char_boundary(origin.end)
        );
    }
}
