use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
    TypeScriptExpr,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_user_equality_and_inequality_through_eq_evidence() {
    let source = "\
pub type Status =
  | Ready
  | Waiting

instance Eq<Status> {
  fn eq left: Status -> right: Status -> Bool =
    match (left, right) {
      (Ready, Ready) -> True
      (Waiting, Waiting) -> True
      _ -> False
    }
}

pub fn same left: Status -> right: Status -> Bool =
  left == right

pub fn different<T> left: T -> right: T -> Bool
where Eq<T> =
  left != right

pub fn answer unit: Unit -> (Bool, Bool) =
  (same Ready Ready, different Ready Waiting)
";
    let typed = type_module("artifact/user-eq-operator/main.ssrg", source);
    let core = lower_typed_module(typed);
    assert!(matches!(
        core.functions[0].body,
        CoreExpr::Binary { ref evidence, .. }
            if matches!(evidence.as_slice(), [selected]
                if matches!(selected.evidence, seseragi_lowering::CoreInstanceEvidence::Local { .. }))
    ));
    assert!(matches!(
        core.functions[1].body,
        CoreExpr::Binary { ref evidence, .. }
            if matches!(evidence.as_slice(), [selected]
                if matches!(selected.evidence, seseragi_lowering::CoreInstanceEvidence::Parameter { index: 0 }))
    ));

    let typescript = lower_core_module_to_typescript_ir(core);
    assert!(matches!(
        &typescript.functions[0],
        seseragi_lowering::TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::DictionaryCall { method, .. },
            ..
        } if method == "eq"
    ));
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle
        .typescript
        .contains("__ssrg$instance$Eq$0[\"eq\"](left)(right)"));
    assert!(bundle
        .typescript
        .contains("__ssrg$evidence$0[\"eq\"](left)(right) === false"));
    assert!(bundle
        .typescript
        .contains("different(Ready)(Waiting)(__ssrg$instance$Eq$0)"));
}
