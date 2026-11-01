#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, run, rust_id, Scratch};

#[test]
fn scalar_adt_roundtrip_bounds_actual_system_allocations_and_releases_every_node() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/char-roundtrip-kernel.ssrg"
    ));
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    generated.push_str(
        &include_str!("../examples/native_lifetimes/char_allocation.rs")
            .replace("KERNEL", &rust_id("native/main::kernel"))
            .replace("ALLOCATION_BOUND", "1_114_113"),
    );
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let measured: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(measured["checksum"], 620_506_872_831_i64);
    assert_eq!(measured["managedLive"], 0);
    assert_eq!(measured["allocations"], measured["deallocations"]);
    assert_eq!(
        measured["cumulativeRequestedBytes"],
        measured["cumulativeReleasedBytes"]
    );
    assert!(measured["allocations"].as_u64().unwrap() <= 1_114_113);
}

#[test]
fn unicode_character_program_preserves_scalar_operations_patterns_and_dictionaries() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/char-unicode.ssrg");
    run(
        source,
        r#"[0, 955, 128578, 1114111, 0, 55295, -1, -1, 57344, 1114111, -1, -1, -1, 955, -1, 0, 1, 2, 3, 4, 2, True, False, True, True, True, True, True, True, True, True, λ, 🙂, 🙂, '\0', '\b', '\f', '\u{1F}', '\u{7F}', '\u{9F}', '\'', '\\', '\n', '́', '🙂']"#,
    );
}

#[test]
fn char_intrinsics_preserve_the_full_domain_without_per_call_callable_nodes() {
    let artifact = emit(include_str!(
        "../../../examples/spec/fixtures/rust-backend/char-unicode.ssrg"
    ));
    let name = rust_id("native/main::pointRoundtrip");
    let direct = artifact
        .source
        .lines()
        .find(|line| line.starts_with(&format!("fn {name}_direct(")))
        .unwrap();
    assert!(direct.contains("native_char_from_code_point("));
    assert!(!direct.contains("V::fun("));
    let end = artifact.source.rfind("fn main()").unwrap();
    let harness = r#"
fn main() {
 let before=ALLOCATIONS.with(Cell::get);
 let mut checksum=0_i64;
 for point in 0..=0x110000_i64 {
  let expected=if point <= 0x10ffff && !(0xd800..=0xdfff).contains(&point) { point } else {-1};
  let value=<ROUNDTRIP>_direct(vec![],checked_int(point));
  let V::Int(value)=value else {panic!("wrong return type")};
  assert_eq!(value,expected,"point={point:X}");
  checksum+=value;
 }
 assert_eq!(checksum,<CHECKSUM>);
 // At most the result Maybe node; no transient callable node for each intrinsic.
 assert!(ALLOCATIONS.with(Cell::get)-before <= 0x110001);
 clear_dictionaries(); assert_eq!(LIVE.with(Cell::get),0);
}
"#
    .replace("<ROUNDTRIP>", &name)
    .replace("<CHECKSUM>", "620506872831");
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&(artifact.source[..end].to_owned() + &harness)))
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
fn generic_character_comparisons_use_the_selected_user_dictionary() {
    run(
        r#"
type Mark = | Mark Int
instance Eq<Mark> {
  fn eq left: Mark -> right: Mark -> Bool = True
}
fn same<A> left: A -> right: A -> Bool where Eq<A> = left == right
pub fn probe -> Bool = same (Mark 1) (Mark 2)
"#,
        "True",
    );
}

#[test]
fn malformed_char_values_and_nominal_standard_evidence_are_rejected() {
    let mut core = compile_core_module(CompileInput::new(
        "char.ssrg",
        "native/main",
        "pub fn probe -> Char = 'λ'\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Char { value, .. } = &mut core.functions[0].body else {
        panic!("expected Char")
    };
    *value = "e\u{301}".into();
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.origin.is_some());
    let mut core = compile_core_module(CompileInput::new(
        "char.ssrg",
        "native/main",
        "pub fn probe -> String = show 'λ'\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Call { evidence, .. } = &mut core.functions[0].body else {
        panic!("expected call")
    };
    evidence[0].constraint.arguments[0] = seseragi_lowering::CoreType::ExternalNamed {
        name: "Char".into(),
        canonical: "user::Char".into(),
        arguments: vec![],
    };
    let error = emit_rust_module(&core, "probe").unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("canonical Char"));
}
