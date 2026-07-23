use crate::{analyze::analyze_module_frontend, CompileInput, CompiledModule};
use seseragi_lowering::{
    emit_typescript_module, emit_typescript_module_with_output_paths,
    lower_core_module_to_typescript_ir, lower_core_module_to_typescript_ir_with_plan,
    lower_typed_module, GeneratedOutputPaths, TypeScriptLoweringError, TypeScriptOutputPlan,
};
use seseragi_semantics::analyze_linked_module;
use seseragi_syntax::{parse_diagnostics, DiagnosticArtifact};

/// Compiles one source using an explicit logical module identity. This is a
/// pure single-module pipeline. Compiler-owned standard modules are linked by
/// public interface; source-package imports still require the project driver.
pub fn compile_module(input: CompileInput<'_>) -> Result<CompiledModule, DiagnosticArtifact> {
    let analyzed = analyze_module_frontend(input)?;
    Ok(finish_compilation(
        analyzed.diagnostics,
        analyzed.typed_hir,
        analyzed.typed_interface,
        input.source(),
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkedCompileError {
    Diagnostics(DiagnosticArtifact),
    TypeScriptPlan(TypeScriptLoweringError),
}

/// Compiles a module after the project layer has fixed its dependency graph,
/// public dependency interfaces, and generated TypeScript output specifiers.
pub fn compile_linked_module(
    linked: seseragi_project::LinkedModule,
    source: &str,
    output_plan: &TypeScriptOutputPlan,
) -> Result<CompiledModule, LinkedCompileError> {
    compile_linked_module_with_output_paths(
        linked,
        source,
        output_plan,
        GeneratedOutputPaths::default(),
    )
}

/// Like [`compile_linked_module`], while preserving project-selected generated
/// artifact paths in metadata and source maps.
pub fn compile_linked_module_with_output_paths(
    linked: seseragi_project::LinkedModule,
    source: &str,
    output_plan: &TypeScriptOutputPlan,
    output_paths: GeneratedOutputPaths,
) -> Result<CompiledModule, LinkedCompileError> {
    let diagnostics = parse_diagnostics(linked.interface.source.clone(), source);
    let analyzed = analyze_linked_module(diagnostics, linked, source)
        .map_err(LinkedCompileError::Diagnostics)?;
    let core_ir = lower_typed_module(analyzed.typed_hir.clone());
    let typescript_ir = lower_core_module_to_typescript_ir_with_plan(core_ir.clone(), output_plan)
        .map_err(LinkedCompileError::TypeScriptPlan)?;
    let generated =
        emit_typescript_module_with_output_paths(typescript_ir.clone(), source, output_paths);

    Ok(CompiledModule {
        diagnostics: analyzed.diagnostics,
        typed_hir: analyzed.typed_hir,
        typed_interface: analyzed.typed_interface,
        core_ir,
        typescript_ir,
        generated,
    })
}

fn finish_compilation(
    diagnostics: DiagnosticArtifact,
    typed_hir: seseragi_semantics::TypedModule,
    typed_interface: seseragi_semantics::TypedModuleInterface,
    source: &str,
) -> CompiledModule {
    let core_ir = lower_typed_module(typed_hir.clone());
    let typescript_ir = lower_core_module_to_typescript_ir(core_ir.clone());
    let generated = emit_typescript_module(typescript_ir.clone(), source);

    CompiledModule {
        diagnostics,
        typed_hir,
        typed_interface,
        core_ir,
        typescript_ir,
        generated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_unknown_custom_operators_before_lowering() {
        let source = "fn invalid value: Int -> Int = value <^> 1\n";
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/custom-operator-unknown",
            source,
        ))
        .expect_err("unknown custom operator must reject compilation");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0101");
        assert_eq!(diagnostics.diagnostics[0].message_key, "operator.unknown");
    }

    #[test]
    fn stops_non_referenceable_operator_sections_before_lowering() {
        let source = "pub let invalid = (&&)\n";
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/operator-section-forbidden",
            source,
        ))
        .expect_err("non-referenceable operator sections must reject compilation");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "parser.expected-expression"
        );
        assert_eq!(diagnostics.diagnostics[0].primary.start, 19);
        assert_eq!(diagnostics.diagnostics[0].primary.end, 21);
    }

    #[test]
    fn lowers_local_custom_infix_calls_without_raw_typescript_operators() {
        let source = "operator infixr 4 <.> left: Int -> right: Int -> Int = left - right\n\
                      pub fn calculate unit: Unit -> Int = 10 <.> 3 <.> 2\n";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/custom-operator",
            source,
        ))
        .expect("valid custom operator should compile");

        assert!(compiled
            .generated
            .typescript
            .contains("__ssrg$operator$3c2e3e"));
        assert!(!compiled.generated.typescript.contains(" <.> "));
    }

    #[test]
    fn stops_non_binary_custom_operator_declarations_before_lowering() {
        let source = "operator infixl 4 <^> value: Int -> Int = value\n";
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/custom-operator-invalid-arity",
            source,
        ))
        .expect_err("non-binary custom operator must reject compilation");

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "operator.invalid-arity"
        );
    }

    #[test]
    fn compiles_standard_web_html_through_the_runtime_abi() {
        let source = r#"import * as html from "std/web/html"

type Msg = | Confirm

fn page -> html.Html<Msg> =
  html.div {
    id: "app",
    className: "container",
    children: [
      html.p { children: "Hello <Seseragi>" },
      html.button { onClick: Confirm, children: "OK" }
    ]
  }

pub effect fn main -> Unit
with Console
fails ConsoleError =
  println $ html.renderToString (page ())
"#;
        let compiled = compile_module(CompileInput::new("main.ssrg", "artifact/web-html", source))
            .expect("standard web HTML should compile");

        assert!(compiled
            .generated
            .typescript
            .contains("@seseragi/runtime/html"));
        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_html_renderToString"));
        assert!(!compiled.generated.typescript.contains("std/web/html"));
    }

    #[test]
    fn compiles_typed_form_event_snapshots_through_the_runtime_abi() {
        let source = r#"import * as html from "std/web/html"

type Msg =
  | DraftChanged String
  | CheckedChanged Bool
  | Submitted

fn draftMessage event: html.InputEvent -> Msg =
  DraftChanged event.value

fn checkedMessage event: html.ChangeEvent -> Msg =
  CheckedChanged event.checked

pub fn view draft: String -> checked: Bool -> html.Html<Msg> =
  html.form {
    onSubmit: Submitted,
    children: [
      html.label { htmlFor: "draft", children: "Draft" },
      html.input {
        id: "draft",
        name: "draft",
        value: draft,
        required: True,
        placeholder: "Type a task",
        inputType: "text",
        onInput: draftMessage
      },
      html.input {
        checked,
        inputType: "checkbox",
        onChange: checkedMessage
      },
      html.button {
        buttonType: "submit",
        disabled: draft == "",
        children: "Add"
      }
    ]
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-form-events",
            source,
        ))
        .expect("typed form event snapshots should compile");

        assert!(compiled.generated.typescript.contains("_ssrg_html_form"));
        assert!(compiled.generated.typescript.contains("_ssrg_html_label"));
        assert!(compiled.generated.typescript.contains("_ssrg_html_input"));
        assert!(compiled.generated.typescript.contains("type InputEvent"));
        assert!(compiled.generated.typescript.contains("type ChangeEvent"));
        assert!(compiled
            .generated
            .typescript
            .contains("(event: InputEvent)"));
        assert!(compiled
            .generated
            .typescript
            .contains("(event: ChangeEvent)"));
        assert!(!compiled.generated.typescript.contains("html_InputEvent"));
        assert!(!compiled.generated.typescript.contains("html_ChangeEvent"));
        assert!(!compiled.generated.typescript.contains("std/web/html"));
    }

    #[test]
    fn rejects_a_form_event_handler_with_the_wrong_shape_before_lowering() {
        let source = r#"import * as html from "std/web/html"

type Msg = | Submitted

pub fn invalid -> html.Html<Msg> =
  html.input { onInput: Submitted }
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-form-invalid-event",
            source,
        ))
        .expect_err("a non-mapper onInput value must stop before lowering");

        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0101"
                && diagnostic.message_key == "call.argument-type-mismatch"
        }));
    }

    #[test]
    fn compiles_the_standard_dom_app_without_manual_signal_plumbing() {
        let source = r##"import * as dom from "std/web/dom"
import * as html from "std/web/html"

type Mode = | Ready | Active
type Msg = | Activate

let initialMode: Mode = Ready

fn update message: Msg -> mode: Mode -> Mode =
  match message {
    Activate -> Active
  }

fn view mode: Mode -> html.Html<Msg> =
  match mode {
    Ready -> html.button { onClick: Activate, children: "Start" }
    Active -> html.p { children: "Active" }
  }

pub effect fn main =
  dom.app {
    target: "#app",
    initial: initialMode,
    update,
    view
  }
"##;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-dom-app",
            source,
        ))
        .expect("standard DOM app should infer the complete executable Effect type");

        assert!(compiled.generated.typescript.contains("_ssrg_dom_app"));
        assert!(!compiled.generated.typescript.contains("_ssrg_dom_query"));
        assert!(!compiled.generated.typescript.contains("_ssrg_signal_make"));
    }

    #[test]
    fn compiles_signal_read_and_assignment_sugar_through_the_runtime_abi() {
        let source = r#"import * as signals from "std/signal"

pub effect fn main -> Unit
with Console
fails ConsoleError =
  do {
    count <- signals.make 1
    count := 42
    current <- *count
    println $ `signal: ${current}`
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/signal-sugar",
            source,
        ))
        .expect("fixed Signal sugar should compile");

        assert!(compiled.generated.typescript.contains("_ssrg_signal_set"));
        assert!(compiled.generated.typescript.contains("_ssrg_signal_read"));
        assert!(!compiled.generated.typescript.contains(":="));
    }

    #[test]
    fn preserves_the_seseragi_value_type_when_creating_a_mutable_signal() {
        let source = r#"import * as signals from "std/signal"

type Mode =
  | Ready
  | Running

let initialMode: Mode = Ready

pub effect fn main -> Unit =
  do {
    mode <- signals.make initialMode
    signals.update (\current: Mode -> Running) mode
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/signal-adt",
            source,
        ))
        .expect("Signal creation should preserve the typed ADT instead of its constructor");

        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_signal_make<Mode>(initialMode)"));
    }

    #[test]
    fn rejects_signal_sugar_on_non_signal_values_before_lowering() {
        for (name, expression) in [("read", "value <- *42"), ("write", "42 := 1")] {
            let source = format!(
                "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {{\n    {expression}\n    println \"unreachable\"\n  }}\n"
            );
            let diagnostics = compile_module(CompileInput::new(
                "main.ssrg",
                &format!("artifact/signal-sugar-invalid-{name}"),
                &source,
            ))
            .expect_err("invalid Signal sugar must stop before lowering");

            assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:?}");
            assert!(diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message_key == "call.argument-type-mismatch"));
        }
    }

    #[test]
    fn infers_generic_effect_environment_from_signal_observer_lambda() {
        let source = r#"import * as signals from "std/signal"

pub effect fn main -> Unit =
  do {
    source <- signals.make 0
    mirror <- signals.make 0
    subscription <- signals.subscribe (\value: Int -> mirror := value) source
    signals.unsubscribe subscription
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/signal-subscription-lambda",
            source,
        ))
        .expect("observer lambda body should infer the unresolved effect environment");

        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_signal_subscribe"));
        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_signal_unsubscribe"));
    }

    #[test]
    fn rejects_unsupported_html_children_before_lowering() {
        let source = r#"import * as html from "std/web/html"

type Msg = | Confirm

pub fn invalid -> html.Html<Msg> =
  html.div { children: 42 }
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-invalid-child",
            source,
        ))
        .expect_err("unsupported HTML children must reject compilation");

        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
        }));
    }

    #[test]
    fn rejects_non_string_html_style_values_before_lowering() {
        let source = r#"import * as html from "std/web/html"

pub fn invalid -> html.Style =
  html.style { padding: 12 }
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-invalid-style",
            source,
        ))
        .expect_err("invalid style records must reject compilation");

        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
        }));
    }

    #[test]
    fn compiles_safe_array_and_list_observations_through_the_runtime_abi() {
        let source = r#"import * as arrays from "std/array"
import * as lists from "std/list"

pub fn arrayLength -> Int = arrays.length [10, 20]
pub fn arrayEmpty -> Bool = arrays.isEmpty [10]
pub fn arrayAt -> Maybe<Int> = arrays.get 1 [10, 20]
pub fn arrayFirst -> Maybe<Int> = arrays.head [10, 20]
pub fn arrayRest -> Maybe<Array<Int>> = arrays.tail [10, 20]

pub fn listLength -> Int = lists.length `[10, 20]
pub fn listEmpty -> Bool = lists.isEmpty `[10]
pub fn listAt -> Maybe<Int> = lists.get 1 `[10, 20]
pub fn listFirst -> Maybe<Int> = lists.head `[10, 20]
pub fn listRest -> Maybe<List<Int>> = lists.tail `[10, 20]
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/collection-access",
            source,
        ))
        .expect("standard collection observations should compile");

        for helper in [
            "_ssrg_array_length",
            "_ssrg_array_isEmpty",
            "_ssrg_array_get",
            "_ssrg_array_head",
            "_ssrg_array_tail",
            "_ssrg_list_length",
            "_ssrg_list_isEmpty",
            "_ssrg_list_get",
            "_ssrg_list_head",
            "_ssrg_list_tail",
        ] {
            assert!(compiled.generated.typescript.contains(helper), "{helper}");
        }
        assert!(!compiled.generated.typescript.contains("std/array"));
        assert!(!compiled.generated.typescript.contains("std/list"));
    }

    #[test]
    fn lowers_parameterless_pure_functions_with_an_implicit_unit() {
        let source = "fn answer -> Int = 42\npub fn run -> Int = answer ()\n";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/implicit-unit-function",
            source,
        ))
        .expect("parameterless pure function should compile");

        assert!(compiled
            .generated
            .typescript
            .contains("const answer = (_unit: undefined)"));
        assert!(compiled.generated.typescript.contains("answer(undefined)"));
        assert!(!compiled.generated.typescript.contains("answer(_)"));
    }
}
