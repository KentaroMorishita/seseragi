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

#[test]
fn invokes_an_unconstrained_generic_instance_dictionary_factory() {
    let source = "\
pub trait Tag<A> { fn tag value: A -> String }
instance<T> Tag<Maybe<T>> { fn tag value: Maybe<T> -> String = \"maybe\" }
pub fn label value: Maybe<Int> -> String = tag value
";
    let typed = type_module("artifact/generic-instance-dispatch/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(bundle.typescript.contains(
        "export const __ssrg$instance$Tag$0 = <T,>() => ({ \"tag\": (value: { readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: T }) => \"maybe\" }) as const;"
    ), "{}", bundle.typescript);
    assert!(bundle.typescript.contains(
        "export const label = (value: { readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: bigint }) => __ssrg$instance$Tag$0<bigint>()[\"tag\"](value)"
    ), "{}", bundle.typescript);
}

#[test]
fn passes_required_local_evidence_to_a_constrained_dictionary_factory() {
    let source = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
pub trait Render<A> { fn render value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"active\" }
instance<T> Render<Maybe<T>> where Ready<T> {
  fn render value: Maybe<T> -> String = \"ready\"
}
pub fn label value: Maybe<Badge> -> String = render value
";
    let typed = type_module("artifact/constrained-instance-dispatch/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle
            .typescript
            .contains("export const __ssrg$instance$Render$1 = <T,>(_evidence0: unknown) =>"),
        "{}",
        bundle.typescript
    );
    assert!(
        bundle.typescript.contains(
            "__ssrg$instance$Render$1<Badge>(__ssrg$instance$Ready$0)[\"render\"](value)"
        ),
        "{}",
        bundle.typescript
    );
}

#[test]
fn omits_empty_type_application_when_a_concrete_dictionary_needs_evidence() {
    let source = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
pub trait Render<A> { fn render value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"active\" }
instance Render<Badge> where Ready<Badge> {
  fn render value: Badge -> String = \"ready\"
}
pub fn label value: Badge -> String = render value
";
    let typed = type_module("artifact/concrete-constrained-dispatch/main.ssrg", source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    let bundle = emit_typescript_module(typescript, source);

    assert!(
        bundle
            .typescript
            .contains("__ssrg$instance$Render$1(__ssrg$instance$Ready$0)[\"render\"](value)"),
        "{}",
        bundle.typescript
    );
    assert!(!bundle.typescript.contains("__ssrg$instance$Render$1<>("));
}
