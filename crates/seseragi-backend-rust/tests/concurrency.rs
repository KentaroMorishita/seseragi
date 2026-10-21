use serde_json::Value;
use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreStatement, CoreType};
fn node<'a>(value: &'a mut Value, callee: &str) -> Option<&'a mut Value> {
    let matches = value.get("callee").and_then(Value::as_str) == Some(callee);
    if matches {
        return Some(value);
    }
    match value {
        Value::Object(fields) => fields.values_mut().find_map(|value| node(value, callee)),
        Value::Array(values) => values.iter_mut().find_map(|value| node(value, callee)),
        _ => None,
    }
}
fn ty(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
#[test]
fn concurrent_calls_reject_counterfeit_owners_payloads_rows_kinds_and_stages() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/fiber-finalizer.ssrg");
    let core = compile_core_module(CompileInput::new("fiber-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "main").unwrap();
    let int = || ty("std/prelude::Int", vec![]);
    let never = || ty("std/prelude::Never", vec![]);
    for case in 0..11 {
        let mut changed = core.clone();
        let callee = match case {
            0..=5 => "std/deferred::make",
            6..=8 => "std/effect::fork",
            _ => "std/deferred::await",
        };
        let main = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::main"))
            .unwrap();
        let CoreExpr::Sequence { statements, .. } = &mut main.body else {
            panic!("effect sequence")
        };
        let call=statements.iter_mut().filter_map(|s|match s {CoreStatement::Bind{value,..}|CoreStatement::Effect{value}=>Some(value),_=>None}).find(|v|matches!(v,CoreExpr::EffectInvoke{callee: name,..}|CoreExpr::Call{callee:name,..} if name==callee)).unwrap();
        let mut value = serde_json::to_value(&*call).unwrap();
        let call_value = node(&mut value, callee).unwrap();
        match case {
            0 => {
                call_value["success"] =
                    serde_json::to_value(ty("counterfeit::Deferred", vec![never(), int()])).unwrap()
            }
            1 => {
                call_value["success"] =
                    serde_json::to_value(ty("std/deferred::Deferred", vec![int()])).unwrap()
            }
            2 => {
                call_value["success"] = serde_json::to_value(ty(
                    "std/deferred::Deferred",
                    vec![never(), ty("std/prelude::Maybe", vec![])],
                ))
                .unwrap()
            }
            3 => call_value["failure"] = serde_json::to_value(int()).unwrap(),
            4 => {
                call_value["requirements"] = serde_json::to_value(CoreType::Record {
                    closed: false,
                    fields: vec![],
                })
                .unwrap()
            }
            5 => {
                call_value["arguments"].as_array_mut().unwrap().clear();
            }
            6 => {
                call_value["success"] =
                    serde_json::to_value(ty("counterfeit::Fiber", vec![never(), int()])).unwrap()
            }
            7 => call_value["failure"] = serde_json::to_value(int()).unwrap(),
            8 => {
                let arg = call_value["arguments"][0].clone();
                call_value["arguments"].as_array_mut().unwrap().push(arg);
            }
            9 => call_value["failure"] = serde_json::to_value(int()).unwrap(),
            _ => call_value["success"] = serde_json::to_value(CoreType::Hole).unwrap(),
        }
        *call = serde_json::from_value(value).unwrap();
        let error = emit_rust_module(&changed, "main").expect_err(&format!("forged case {case}"));
        assert!(error.message.contains("concurrent"), "{case}: {error}");
        assert!(error.origin.is_some(), "{case}: {error}");
    }
}
#[test]
fn saved_operators_keep_canonical_payload_identity_and_scoped_value_kinds() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/fiber-results.ssrg");
    let core = compile_core_module(CompileInput::new(
        "fiber-results.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert_eq!(artifact.metadata.abi, "rust-erased-cooperative-v3");
    let mut changed = core.clone();
    let generic = changed
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::awaitValue"))
        .unwrap();
    generic.type_parameters[0].arity = 1;
    assert!(emit_rust_module(&changed, "main").is_err());
    // Source and result payloads must agree even if every displayed name is Int.
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::awaitValue"))
        .unwrap();
    let mut value = serde_json::to_value(&core.functions[index].body).unwrap();
    let call = node(&mut value, "std/effect::join").unwrap();
    call["success"] = serde_json::to_value(ty("native/main::Int", vec![])).unwrap();
    let mut changed = core.clone();
    changed.functions[index].body = serde_json::from_value(value).unwrap();
    assert!(emit_rust_module(&changed, "main").is_err());
}
