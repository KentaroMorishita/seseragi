use serde_json::Value;
use seseragi_lowering::{lower_core_module_to_typescript_ir, lower_typed_module, CoreStatement};
use seseragi_semantics::type_module;

fn local_bindings(value: &Value, bindings: &mut Vec<(String, bool)>) {
    match value {
        Value::Object(fields) => {
            if fields.get("kind").and_then(Value::as_str) == Some("local-function") {
                bindings.push((
                    fields["name"].as_str().unwrap().to_owned(),
                    fields
                        .get("initializerInOuterScope")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                ));
            }
            for child in fields.values() {
                local_bindings(child, bindings);
            }
        }
        Value::Array(children) => {
            for child in children {
                local_bindings(child, bindings);
            }
        }
        _ => {}
    }
}

#[test]
fn generalized_let_and_named_functions_keep_distinct_binding_scopes() {
    let source = r#"
pub fn probe -> String = {
  let render = \value -> show value
  {
    let render = \value -> render value + "!"
    fn loop remaining: Int -> Int =
      if remaining == 0 then 0 else loop (remaining - 1)
    render (loop 42)
  }
}
"#;
    let core = lower_typed_module(type_module("local-binding-scope.ssrg", source));
    let json = serde_json::to_value(&core).unwrap();
    let mut bindings = vec![];
    local_bindings(&json, &mut bindings);
    bindings.sort();
    assert_eq!(
        bindings,
        vec![
            ("loop".to_owned(), false),
            ("render".to_owned(), true),
            ("render".to_owned(), true),
        ]
    );

    let typescript = lower_core_module_to_typescript_ir(core);
    let mut projected = vec![];
    local_bindings(&serde_json::to_value(typescript).unwrap(), &mut projected);
    projected.sort();
    assert_eq!(projected, bindings);
}

#[test]
fn omitted_binding_scope_metadata_keeps_named_declaration_compatibility() {
    let source = "pub fn probe -> Int = { fn identity value: Int -> Int = value; identity 42 }\n";
    let core = lower_typed_module(type_module("named-binding-scope.ssrg", source));
    let json = serde_json::to_value(core).unwrap();
    fn find(value: &Value) -> Option<&Value> {
        match value {
            Value::Object(fields) => {
                if fields.get("kind").and_then(Value::as_str) == Some("local-function") {
                    Some(value)
                } else {
                    fields.values().find_map(find)
                }
            }
            Value::Array(children) => children.iter().find_map(find),
            _ => None,
        }
    }
    let declaration = find(&json).expect("named local declaration");
    assert!(declaration.get("initializerInOuterScope").is_none());
    assert!(matches!(
        serde_json::from_value::<CoreStatement>(declaration.clone()).unwrap(),
        CoreStatement::LocalFunction {
            initializer_in_outer_scope: false,
            ..
        }
    ));
}

#[test]
fn parameters_locals_and_patterns_do_not_become_self_tail_calls() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/binding-scope.ssrg");
    let core = lower_typed_module(type_module("binding-scope.ssrg", source));
    let typescript = lower_core_module_to_typescript_ir(core);
    let emitted = seseragi_lowering::emit_typescript_module(typescript, source).typescript;
    for name in ["parameter", "shadow", "branch"] {
        let line = emitted
            .lines()
            .find(|line| line.starts_with(&format!("const {name} =")))
            .expect("ordinary function declaration");
        assert!(!line.contains("$ssrg$tail"), "{name}: {line}");
    }
    let count = emitted
        .lines()
        .find(|line| line.starts_with("const count ="))
        .expect("recursive control");
    assert!(count.contains("while (true)"), "{count}");
    for name in ["scopedCount", "patternCount"] {
        let line = emitted
            .lines()
            .find(|line| line.starts_with(&format!("const {name} =")))
            .expect("recursive function with a shadowed base branch");
        assert!(line.contains("while (true)"), "{line}");
        assert_eq!(line.matches("({ [$ssrg$tail]:").count(), 1, "{line}");
    }
}
