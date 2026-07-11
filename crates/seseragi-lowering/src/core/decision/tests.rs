use seseragi_semantics::type_module;

use crate::core::{lower_typed_module, CoreDecisionProjection, CoreDecisionTest, CoreExpr};

#[test]
fn lowers_tuple_constructor_patterns_to_ordered_decision_tests() {
    let source = r#"pub type Hand =
  | Rock
  | Paper
  | Scissors

pub type Outcome =
  | Player1Wins
  | Player2Wins
  | Draw

pub fn decide first: Hand -> second: Hand -> Outcome =
  match (first, second) {
    (Rock, Rock) -> Draw
    (Rock, Scissors) -> Player1Wins
    _ -> Player2Wins
  }
"#;
    let module = lower_typed_module(type_module("artifact/rps/main.ssrg", source));
    let CoreExpr::Decision {
        scrutinee,
        branches,
        exhaustive,
        ..
    } = &module.functions[0].body
    else {
        panic!("expected a core decision");
    };

    assert!(matches!(scrutinee.as_ref(), CoreExpr::Tuple { .. }));
    assert!(*exhaustive);
    assert_eq!(branches.len(), 3);
    assert_eq!(branches[0].tests.len(), 2);
    assert!(matches!(
        &branches[0].tests[0],
        CoreDecisionTest::Constructor {
            path,
            constructor,
            ..
        } if path == &[CoreDecisionProjection::TupleElement { index: 0 }]
            && constructor == "artifact/rps::Rock"
    ));
    assert!(matches!(
        &branches[0].tests[1],
        CoreDecisionTest::Constructor {
            path,
            constructor,
            ..
        } if path == &[CoreDecisionProjection::TupleElement { index: 1 }]
            && constructor == "artifact/rps::Rock"
    ));
    assert!(branches[2].tests.is_empty());
    assert!(branches[2].bindings.is_empty());
    assert!(branches[2].guard.is_none());
}

#[test]
fn lowers_payload_bindings_to_adt_payload_projections() {
    let source = r#"pub type Label =
  | Missing
  | Present String

pub fn unwrap label: Label -> String =
  match label {
    Present value -> value
    Missing -> "none"
  }
"#;
    let module = lower_typed_module(type_module("artifact/payload/main.ssrg", source));
    let CoreExpr::Decision {
        branches,
        exhaustive,
        ..
    } = &module.functions[0].body
    else {
        panic!("expected a core decision");
    };

    assert!(*exhaustive);
    assert!(matches!(
        &branches[0].tests[0],
        CoreDecisionTest::Constructor {
            path,
            constructor,
            ..
        } if path.is_empty() && constructor == "artifact/payload::Present"
    ));
    assert_eq!(branches[0].bindings.len(), 1);
    assert_eq!(branches[0].bindings[0].name, "value");
    assert_eq!(
        branches[0].bindings[0].path,
        vec![CoreDecisionProjection::AdtPayload]
    );
    assert!(matches!(
        &branches[0].value,
        CoreExpr::Variable { name, .. } if name == "value"
    ));
    assert!(branches[1].tests.is_empty());
}

#[test]
fn keeps_guards_separate_from_structural_tests() {
    let source = r#"pub type Flag =
  | Enabled
  | Disabled

pub fn render flag: Flag -> String =
  match flag {
    value when True -> "guarded"
    _ -> "fallback"
  }
"#;
    let module = lower_typed_module(type_module("artifact/guard/main.ssrg", source));
    let CoreExpr::Decision { branches, .. } = &module.functions[0].body else {
        panic!("expected a core decision");
    };

    assert!(branches[0].tests.is_empty());
    assert_eq!(branches[0].bindings.len(), 1);
    assert!(matches!(
        &branches[0].guard,
        Some(CoreExpr::Boolean { value: true, .. })
    ));
    assert!(branches[1].tests.is_empty());
    assert!(branches[1].guard.is_none());
}

#[test]
fn keeps_a_defensive_test_when_semantics_did_not_prove_exhaustiveness() {
    let source = r#"pub type Label =
  | Missing
  | Present String

pub fn unwrap label: Label -> String =
  match label {
    Present value -> value
  }
"#;
    let module = lower_typed_module(type_module("artifact/non-exhaustive/main.ssrg", source));
    let CoreExpr::Decision {
        branches,
        exhaustive,
        ..
    } = &module.functions[0].body
    else {
        panic!("expected a core decision");
    };

    assert!(!*exhaustive);
    assert!(matches!(
        &branches[0].tests[0],
        CoreDecisionTest::Constructor { constructor, .. }
            if constructor == "artifact/non-exhaustive::Present"
    ));
}

#[test]
fn keeps_the_final_discriminant_when_a_payload_binding_needs_narrowing() {
    let source = r#"pub type Label =
  | Missing
  | Present String

pub fn unwrap label: Label -> String =
  match label {
    Missing -> "none"
    Present value -> value
  }
"#;
    let module = lower_typed_module(type_module("artifact/payload-fallback/main.ssrg", source));
    let CoreExpr::Decision {
        branches,
        exhaustive,
        ..
    } = &module.functions[0].body
    else {
        panic!("expected a core decision");
    };

    assert!(*exhaustive);
    assert!(matches!(
        &branches[1].tests[0],
        CoreDecisionTest::Constructor { constructor, .. }
            if constructor == "artifact/payload-fallback::Present"
    ));
    assert!(branches[1].bindings[0]
        .path
        .contains(&CoreDecisionProjection::AdtPayload));
}
