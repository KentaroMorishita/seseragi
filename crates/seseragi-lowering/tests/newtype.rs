use seseragi_lowering::{emit_typescript_module, lower_typed_module, CoreExpr};
use seseragi_semantics::type_module;

#[test]
fn lowers_newtype_construction_and_pattern_unwrap() {
    let source = "\
pub newtype UserId = Int

pub fn raw id: UserId -> Int =
  match id {
    UserId value -> value
  }

pub fn answer unit: Unit -> Int =
  raw (UserId 42)
";
    let core = lower_typed_module(type_module("artifact/newtype/main.ssrg", source));

    assert_eq!(core.adts.len(), 1);
    assert_eq!(core.adts[0].name, "UserId");
    assert_eq!(core.adts[0].variants[0].name, "UserId");
    assert!(matches!(core.functions[0].body, CoreExpr::Decision { .. }));

    let bundle = emit_typescript_module(
        seseragi_lowering::lower_core_module_to_typescript_ir(core),
        source,
    );
    assert!(bundle.typescript.contains("export type UserId ="));
    assert!(bundle
        .typescript
        .contains("export const UserId = (value: bigint): UserId"));
    assert!(bundle.typescript.contains("raw(UserId(42n))"));
}

#[test]
fn infers_a_generic_newtype_constructor_from_its_payload() {
    let source = "\
pub newtype Tagged<A> = A

pub fn untag<T> value: Tagged<T> -> T =
  match value {
    Tagged item -> item
  }

pub fn answer unit: Unit -> Int =
  untag (Tagged 42)
";
    let core = lower_typed_module(type_module("artifact/generic-newtype/main.ssrg", source));
    let bundle = emit_typescript_module(
        seseragi_lowering::lower_core_module_to_typescript_ir(core),
        source,
    );

    assert!(bundle.typescript.contains("export type Tagged<A> ="));
    assert!(bundle
        .typescript
        .contains("export const Tagged = <A>(value: A): Tagged<A>"));
    assert!(bundle.typescript.contains("untag(Tagged(42n))"));
}
