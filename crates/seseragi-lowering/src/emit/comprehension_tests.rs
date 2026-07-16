use seseragi_semantics::type_module;

use crate::{emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module};

#[test]
fn filters_refutable_comprehension_patterns_before_transforming() {
    let source = r#"pub fn presentValues values: Array<Maybe<Int>> -> Array<Int> =
  [value | Just value <- values, value > 1]

pub fn matchingValues values: Array<(Int, Int)> -> Array<Int> =
  [value | (1, value) <- values]
"#;
    let typed = type_module("artifact/comprehension-pattern-filter/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains("$ssrg_match.tag === \"Just\""),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle.typescript.contains("$ssrg_match[0] === 1n"),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle
            .typescript
            .contains("(value: bigint): boolean => value > 1n"),
        "{}",
        bundle.typescript
    );
}

#[test]
fn dispatches_user_iterable_comprehension_through_its_dictionary() {
    let source = r#"pub type Bucket =
  | Bucket Int

fn advance limit: Int -> current: Int -> Maybe<(Int, Int)> =
  if current <= limit then Just (current, current + 1) else Nothing

instance Iterable<Bucket, Int> {
  fn iterate values: Bucket -> Iterator<Int> =
    match values { Bucket limit -> unfold (advance limit) 1 }
}

pub fn values bucket: Bucket -> Array<Int> =
  [value | value <- bucket]
"#;
    let typed = type_module("artifact/user-iterable/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle
            .typescript
            .contains("_ssrg_iterator_comprehend(__ssrg$instance$Iterable$0[\"iterate\"](bucket)"),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle.typescript.contains("type Iterator as Iterator"),
        "{}",
        bundle.typescript
    );
}
