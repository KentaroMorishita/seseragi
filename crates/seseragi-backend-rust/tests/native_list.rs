#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, rust_id, Scratch};

#[test]
fn persistent_list_values_patterns_display_and_operations_execute() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/list-core.ssrg"
    ));
    assert!(artifact.metadata.capabilities.iter().any(|x| x == "list"));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        include_str!("../../../examples/spec/fixtures/rust-backend/list-core.stdout")
    );
}

#[test]
fn checked_list_prefixes_reuse_cursors_and_preserve_nested_guards_and_escaped_tails() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/list-prefix-decisions.ssrg"
    ));
    assert!(artifact.source.contains("_cell1.head.clone()"));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/list-prefix-decisions.stdout")
    );
}

#[test]
fn wide_list_prefixes_do_not_repeat_root_walks() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/list-prefix-kernel.ssrg"
    ));
    assert!(artifact.source.contains("_cell31.head.clone()"));
    assert!(!artifact.source.contains(".list_element(31)"));
    assert!(!artifact.source.contains(".list_length_matches(32, true)"));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/list-prefix-kernel.stdout")
    );
}

#[test]
fn deep_list_construction_traversal_and_drop_are_separately_stack_safe() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/list-tail.ssrg"
    ));
    assert_eq!(artifact.metadata.self_tail_calls, 2);
    let main = artifact.source.rfind("fn main()").unwrap();
    let source = format!("{}\nfn main() {{ std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), \"5000050000\"); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}).unwrap().join().unwrap(); }}", &artifact.source[..main], rust_id(&artifact.metadata.entry));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn list_evidence_and_literal_reject_counterfeit_identity_type_child_and_dispatch() {
    let core = compile_core_module(CompileInput::new(
        "list.ssrg",
        "native/main",
        "pub fn probe -> String = show `[1, 2]\n",
    ))
    .unwrap()
    .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for mutation in 0..8 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            trait_dispatch,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("call")
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("user::Show".into()),
            1 => {
                evidence[0].constraint.arguments[0] = seseragi_lowering::CoreType::Named {
                    name: "user::List".into(),
                    arguments: vec![],
                }
            }
            2 | 3 | 4 | 5 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard {
                    type_arguments,
                    evidence_arguments,
                    ..
                } = &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                match mutation {
                    2 => type_arguments.clear(),
                    3 => evidence_arguments.clear(),
                    4 => evidence_arguments[0].constraint.name = "Debug".into(),
                    _ => {
                        evidence_arguments[0].constraint.arguments[0] =
                            seseragi_lowering::CoreType::Named {
                                name: "Char".into(),
                                arguments: vec![],
                            }
                    }
                }
            }
            6 => trait_dispatch.as_mut().unwrap().trait_identity = "user::Show".into(),
            _ => trait_dispatch.as_mut().unwrap().method = "debug".into(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
    for source in [
        "pub fn probe -> List<Int> = `[1]\n",
        "pub fn probe -> List<Int> = 1 : `[]\n",
    ] {
        let mut core = compile_core_module(CompileInput::new("list.ssrg", "native/main", source))
            .unwrap()
            .core_ir;
        let (CoreExpr::List { type_ref, .. } | CoreExpr::Binary { type_ref, .. }) =
            &mut core.functions[0].body
        else {
            panic!("list")
        };
        *type_ref = seseragi_lowering::CoreType::Named {
            name: "native/main::List".into(),
            arguments: vec![],
        };
        let error = emit_rust_module(&core, "probe").unwrap_err();
        assert!(error.origin.is_some());
    }
}
