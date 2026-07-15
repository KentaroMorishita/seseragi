use crate::{emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module};
use seseragi_semantics::type_module;

#[test]
fn emits_and_invokes_a_local_constrained_function_with_dictionary_evidence() {
    let source = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"Badge is ready\" }
pub fn describe<T> value: T -> String
where Ready<T> =
  ready value
pub fn label value: Badge -> String = describe value
";
    let typed = type_module("artifact/constrained-function/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle.typescript.contains(
            "export const describe = <T,>(value: T) => (__ssrg$evidence$0: unknown) => __ssrg$evidence$0[\"ready\"](value)"
        ),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle
            .typescript
            .contains("describe(value)(__ssrg$instance$Ready$0)"),
        "{}",
        bundle.typescript
    );
}
