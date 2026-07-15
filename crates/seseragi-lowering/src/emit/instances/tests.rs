use super::render_typescript_instances;
use crate::{emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module};
use seseragi_semantics::type_module;

#[test]
fn emits_source_like_show_dictionaries_from_resolved_render_plans() {
    let source = "\
pub type Detail deriving Show =
  | Detail String

pub type AppError deriving Show =
  | Wrapped Detail
  | EndOfInput
";
    let typed = type_module("artifact/derived-show/main.ssrg", source);
    let core = lower_typed_module(typed);
    let mut typescript = lower_core_module_to_typescript_ir(core);
    let show_import = typescript
        .type_imports
        .iter_mut()
        .find(|import| import.feature == "core.show.dictionary")
        .unwrap();
    show_import.local = "ResolvedShow".to_owned();

    let mut output = String::new();
    render_typescript_instances(&mut output, &typescript.instances, &typescript.type_imports);

    assert_eq!(output.lines().count(), typescript.instances.len());
    assert_eq!(
        output,
        "\
export const __ssrg$instance$Show$0: ResolvedShow<Detail> = { show: (value: Detail): string => { switch (value.tag) { case \"Detail\": return \"Detail\" + \" \" + _ssrg_show_stringShow.show(value.value); } } };
export const __ssrg$instance$Show$1: ResolvedShow<AppError> = { show: (value: AppError): string => { switch (value.tag) { case \"Wrapped\": return \"Wrapped\" + \" \" + __ssrg$instance$Show$0.show(value.value); case \"EndOfInput\": return \"EndOfInput\"; } } };
"
    );
}

#[test]
fn emits_recursive_dictionary_references_inside_lazy_show_bodies() {
    let source = "\
pub type Chain deriving Show =
  | End
  | Link Chain
";
    let typed = type_module("artifact/recursive-show/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    let mut output = String::new();
    render_typescript_instances(&mut output, &typescript.instances, &typescript.type_imports);

    assert!(output.contains(
        "case \"Link\": return \"Link\" + \" \" + __ssrg$instance$Show$0.show(value.value);"
    ));
    assert_eq!(output.lines().count(), 1);
}

#[test]
fn emits_nothing_without_selected_instances_or_show_import() {
    let mut output = String::new();

    render_typescript_instances(&mut output, &[], &[]);

    assert!(output.is_empty());
}

#[test]
fn emits_runtime_imports_and_keeps_dictionary_exports_out_of_source_exports() {
    let source = "pub type AppError deriving Show =\n  | UnknownHand String\n";
    let typed = type_module("artifact/emitted-show/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(bundle.typescript.contains(
        "import { stringShow as _ssrg_show_stringShow, type Show as _ssrg_show_Show } from \"@seseragi/runtime/show\""
    ));
    assert!(bundle
        .typescript
        .contains("export const __ssrg$instance$Show$0: _ssrg_show_Show<AppError>"));
    assert_eq!(bundle.metadata.exports, vec!["UnknownHand"]);
    assert_eq!(bundle.metadata.instances.len(), 1);
    assert_eq!(
        bundle.metadata.instances[0].dictionary_export,
        "__ssrg$instance$Show$0"
    );
}

#[test]
fn emits_user_defined_dictionary_without_show_runtime_support() {
    let source = "\
pub type Badge = | Active
pub trait Render<A> { fn render value: A -> String }
instance Render<Badge> { fn render value: Badge -> String = \"active\" }
";
    let typed = type_module("artifact/user-instance/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);

    assert!(!typescript
        .runtime_requirements
        .iter()
        .any(|requirement| requirement.starts_with("core.show")));
    assert!(typescript.type_imports.is_empty());
    let bundle = emit_typescript_module(typescript, source);
    assert!(bundle.typescript.contains(
        "export const __ssrg$instance$Render$0 = { \"render\": (value: Badge) => \"active\" } as const;"
    ));
}

#[test]
fn dispatches_a_trait_method_call_through_selected_local_evidence() {
    let source = "\
pub type Badge = | Active
pub trait Render<A> { fn render value: A -> String }
instance Render<Badge> { fn render value: Badge -> String = \"active\" }
pub fn label value: Badge -> String = render value
";
    let typed = type_module("artifact/trait-dispatch/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(bundle.typescript.contains(
        "export const label = (value: Badge) => __ssrg$instance$Render$0[\"render\"](value)"
    ));
}
