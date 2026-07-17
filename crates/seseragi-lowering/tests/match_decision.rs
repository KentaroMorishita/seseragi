use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
    TypeScriptDecisionProjection, TypeScriptDecisionTest, TypeScriptExpr, TypeScriptFunction,
};
use seseragi_semantics::type_module;

fn lower(source_name: &str, source: &str) -> seseragi_lowering::TypeScriptModule {
    lower_core_module_to_typescript_ir(lower_typed_module(type_module(source_name, source)))
}

#[test]
fn lowers_match_to_tag_tests_without_a_runtime_helper() {
    let source = r#"pub type Label =
  | Missing
  | Present String

pub fn unwrap label: Label -> String =
  match label {
    Present value -> value
    Missing -> "none"
  }
"#;
    let typescript = lower("artifact/match-label/main.ssrg", source);

    assert!(typescript.imports.is_empty());
    assert_eq!(
        typescript
            .runtime_requirements
            .iter()
            .filter(|requirement| requirement.as_str() == "core.adt")
            .count(),
        1
    );
    let TypeScriptFunction::ConstFunction { body, .. } = &typescript.functions[0];
    let TypeScriptExpr::Decision {
        scrutinee,
        branches,
        ..
    } = body
    else {
        panic!("expected TypeScript decision expression");
    };
    assert!(matches!(scrutinee.as_ref(), TypeScriptExpr::Identifier { name } if name == "label"));
    assert!(matches!(
        &branches[0].tests[0],
        TypeScriptDecisionTest::TagEquals { path, tag }
            if path.is_empty() && tag == "Present"
    ));
    assert_eq!(branches[0].bindings.len(), 1);
    assert_eq!(branches[0].bindings[0].name, "value");
    assert_eq!(
        branches[0].bindings[0].path,
        vec![TypeScriptDecisionProjection::AdtPayload]
    );
    assert!(branches[1].tests.is_empty());

    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle.typescript.contains(
        "(($ssrg_match: Label): string => $ssrg_match.tag === \"Present\" ? ((value: string): string => value)($ssrg_match.value) : \"none\")(label)"
    ));
    assert!(!bundle.typescript.contains("match("));
}

#[test]
fn evaluates_the_match_scrutinee_once() {
    let source = r#"type Hand =
  | Rock
  | Paper

fn identity hand: Hand -> Hand = hand

pub fn render hand: Hand -> String =
  match identity hand {
    Rock -> "rock"
    Paper -> "paper"
  }
"#;
    let typescript = lower("artifact/match-on-call/main.ssrg", source);
    let bundle = emit_typescript_module(typescript, source);

    assert_eq!(bundle.typescript.matches("identity(hand)").count(), 1);
}

#[test]
fn traverses_match_branches_for_runtime_requirements_and_imports() {
    let source = r#"type Hand =
  | Rock
  | Paper

pub fn score hand: Hand -> Int =
  match hand {
    Rock -> 1 + 2
    Paper -> 0
  }
"#;
    let typescript = lower("artifact/match-runtime-walk/main.ssrg", source);

    assert!(typescript
        .runtime_requirements
        .iter()
        .any(|requirement| requirement == "core.int64.add"));
    assert_eq!(typescript.imports.len(), 1);
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle.typescript.contains("_ssrg_int64_add(1n, 2n)"));
}

#[test]
fn preserves_a_final_payload_discriminant_for_typescript_narrowing() {
    let source = r#"pub type Label =
  | Missing
  | Present String

pub fn unwrap label: Label -> String =
  match label {
    Missing -> "none"
    Present value -> value
  }
"#;
    let typescript = lower("artifact/match-final-payload/main.ssrg", source);
    let bundle = emit_typescript_module(typescript, source);

    assert!(bundle.typescript.contains(
        "$ssrg_match.tag === \"Present\" ? ((value: string): string => value)($ssrg_match.value) : (()"
    ));
}

#[test]
fn scopes_payload_bindings_in_guards_and_branch_values() {
    let source = r#"type Choice =
  | Missing
  | Value Bool

pub fn render choice: Choice -> String =
  match choice {
    Value enabled when enabled -> "enabled"
    Value _ -> "disabled"
    Missing -> "missing"
  }
"#;
    let typescript = lower("artifact/match-guard-binding/main.ssrg", source);
    let bundle = emit_typescript_module(typescript, source);

    assert!(bundle.typescript.contains(
        "$ssrg_match.tag === \"Value\" && ((enabled: boolean): boolean => enabled)($ssrg_match.value)"
    ));
}

#[test]
fn lowers_literal_patterns_to_strict_equality_tests() {
    let source = r#"pub fn classify value: (Int, String, Bool) -> String =
  match value {
    (42, "ready", True) -> "matched"
    _ -> "other"
  }
"#;
    let typescript = lower("artifact/match-literals/main.ssrg", source);
    let TypeScriptFunction::ConstFunction { body, .. } = &typescript.functions[0];
    let TypeScriptExpr::Decision { branches, .. } = body else {
        panic!("expected TypeScript decision expression");
    };

    assert!(matches!(
        &branches[0].tests[0],
        TypeScriptDecisionTest::BigintEquals { path, value }
            if path == &[TypeScriptDecisionProjection::TupleElement { index: 0 }]
                && value == "42"
    ));
    assert!(matches!(
        &branches[0].tests[1],
        TypeScriptDecisionTest::StringEquals { path, value }
            if path == &[TypeScriptDecisionProjection::TupleElement { index: 1 }]
                && value == "ready"
    ));
    assert!(matches!(
        &branches[0].tests[2],
        TypeScriptDecisionTest::BooleanEquals { path, value: true }
            if path == &[TypeScriptDecisionProjection::TupleElement { index: 2 }]
    ));
    assert!(branches[1].tests.is_empty());

    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle.typescript.contains(
        "$ssrg_match[0] === 42n && $ssrg_match[1] === \"ready\" && $ssrg_match[2] === true"
    ));
}

#[test]
fn emits_structural_record_pattern_projections() {
    let source = r#"pub fn render value: { label: String, name: String } -> String =
  match value {
    { label: "Player", name } -> name
    { name } -> name
  }
"#;
    let typescript = lower("artifact/record-match/main.ssrg", source);
    let bundle = emit_typescript_module(typescript, source);

    assert!(bundle
        .typescript
        .contains("$ssrg_match[\"label\"] === \"Player\""));
    assert!(bundle
        .typescript
        .contains("((name: string): string => name)($ssrg_match[\"name\"])"));
}
