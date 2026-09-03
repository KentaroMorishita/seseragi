use crate::{analyze::analyze_module_frontend, CompileInput, CompiledModule};
use seseragi_lowering::{
    emit_typescript_module, emit_typescript_module_with_output_paths,
    lower_core_module_to_typescript_ir, lower_core_module_to_typescript_ir_with_plan,
    lower_typed_module, GeneratedOutputPaths, TypeScriptLoweringError, TypeScriptOutputPlan,
};
use seseragi_semantics::{analyze_linked_module, AnalyzedModule};
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
    compile_analyzed_module_with_output_paths(analyzed, source, output_plan, output_paths)
        .map_err(LinkedCompileError::TypeScriptPlan)
}

/// Finishes lowering and emission for a project module whose shared frontend
/// analysis has already completed without error diagnostics.
pub(crate) fn compile_analyzed_module_with_output_paths(
    analyzed: AnalyzedModule,
    source: &str,
    output_plan: &TypeScriptOutputPlan,
    output_paths: GeneratedOutputPaths,
) -> Result<CompiledModule, TypeScriptLoweringError> {
    let core_ir = lower_typed_module(analyzed.typed_hir.clone());
    let typescript_ir = lower_core_module_to_typescript_ir_with_plan(core_ir.clone(), output_plan)?;
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
    fn rejects_local_alias_arity_mismatches_before_lowering() {
        const SOURCE: &str = include_str!(
            "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/local-alias-arity/main.ssrg"
        );
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/local-alias-arity",
            SOURCE,
        ))
        .expect_err("local alias arity mismatches must stop before lowering");

        assert_eq!(
            diagnostics
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "SES-T0601")
                .count(),
            6
        );
        assert!(diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message_key == "alias.arity-mismatch"));
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
    fn compiles_local_functions_with_lexical_capture() {
        let source = "fn calculate base: Int -> Int = {\n\
                      let offset: Int = 2\n\
                      fn add value: Int -> Int = value + offset\n\
                      add base\n\
                      }\n";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/local-function",
            source,
        ))
        .expect("local function and its capture should compile");

        assert!(compiled
            .generated
            .typescript
            .contains("const offset: number = 2"));
        assert!(compiled
            .generated
            .typescript
            .contains("const add = (value: number) =>"));
        assert!(compiled.generated.typescript.matches("offset").count() >= 2);
        assert!(compiled.generated.typescript.contains("add(base)"));
    }

    #[test]
    fn compiles_generic_and_self_recursive_local_functions() {
        let source = "fn countdown start: Int -> Int = {\n\
                      fn identity<A> value: A -> A = value\n\
                      fn loop current: Int -> Int =\n\
                        if current == 0 then identity current else loop (current - 1)\n\
                      loop start\n\
                      }\n";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/local-recursion",
            source,
        ))
        .expect("generic and self-recursive local functions should compile");

        assert!(compiled
            .generated
            .typescript
            .contains("const identity = <A,>"));
        assert!(compiled
            .generated
            .typescript
            .contains("const loop = (current: number) =>"));
        assert!(
            compiled
                .generated
                .typescript
                .contains("({ [$ssrg$tail]: [_ssrg_int_subtract(current, 1)] } as never)"),
            "{}",
            compiled.generated.typescript
        );
        assert!(compiled.generated.typescript.contains("while (true)"));
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

type Action = | Confirm

fn page -> html.Html<Action> =
  html.div {
    id: "app",
    class: "container",
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
    fn compiles_document_text_list_and_void_html_tags() {
        let source = r#"import * as html from "std/web/html"

type Action = | Navigate

fn page stylesheet: html.WebUrl -> html.Html<Action> =
  html.html {
    children: [
      html.head {
        children: [
          html.title { children: "Seseragi" },
          html.meta { id: "metadata" },
          html.link { rel: "stylesheet", href: stylesheet }
        ]
      },
      html.body {
        children: [
          html.header { children: html.h1 { children: "Reference" } },
          html.nav { children: html.small { children: "Contents" } },
          html.article {
            children: [
              html.h2 { children: "Document" },
              html.h3 { children: "Section" },
              html.h4 { children: "Topic" },
              html.h5 { children: "Detail" },
              html.h6 { children: "Note" },
              html.strong { children: "Strong" },
              html.em { children: "Emphasis" },
              html.code { children: "let value = 1" },
              html.pre { children: "line 1\nline 2" },
              html.blockquote { children: "Typed HTML" },
              html.ul { children: [html.li { children: "One" }] },
              html.ol { children: [html.li { children: "First" }] },
              html.br { id: "break" },
              html.hr { id: "rule" }
            ]
          },
          html.aside { children: "Related" },
          html.footer { children: "End" }
        ]
      }
    ]
  }

fn renderedDocument parsed: Either<html.HtmlBuildError, html.WebUrl> -> String =
  match parsed {
    Left error -> show error
    Right stylesheet -> html.renderDocument (page stylesheet)
  }

pub effect fn main -> Unit
with Console
fails ConsoleError =
  println $ renderedDocument (html.parseWebUrl "/styles.css")
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-document-tags",
            source,
        ))
        .expect("document HTML tags should compile through the runtime ABI");

        for runtime_name in [
            "html",
            "head",
            "body",
            "title",
            "meta",
            "link",
            "header",
            "footer",
            "nav",
            "article",
            "aside",
            "h3",
            "h4",
            "h5",
            "h6",
            "strong",
            "em",
            "small",
            "code",
            "pre",
            "blockquote",
            "ul",
            "ol",
            "li",
            "br",
            "hr",
        ] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("_ssrg_html_{runtime_name}")),
                "missing runtime import for {runtime_name}"
            );
        }
        assert!(!compiled.generated.typescript.contains("std/web/html"));
    }

    #[test]
    fn compiles_link_image_and_media_tags_with_tag_specific_props() {
        let source = r#"import * as html from "std/web/html"

type Action = | Navigate

pub fn page destination: html.WebUrl -> media: html.WebUrl -> html.Html<Action> =
  html.article {
    children: [
      html.a {
        href: destination,
        target: "_blank",
        rel: "noopener",
        download: True,
        children: "Docs"
      },
      html.img {
        src: media,
        alt: "Seseragi",
        width: 640,
        height: 360,
        loading: "lazy"
      },
      html.picture {
        children: html.source {
          src: media,
          media: "(min-width: 48rem)",
          mimeType: "image/png"
        }
      },
      html.video {
        src: media,
        width: 640,
        height: 360,
        children: html.source { src: media, mimeType: "video/webm" }
      },
      html.audio {
        src: media,
        children: html.source { src: media, mimeType: "audio/ogg" }
      }
    ]
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-link-media",
            source,
        ))
        .expect("link, image, picture, source, video, and audio should compile");

        for runtime_name in ["a", "img", "picture", "source", "video", "audio"] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("_ssrg_html_{runtime_name}")),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn rejects_raw_strings_for_security_sensitive_url_props() {
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-raw-url",
            r#"import * as html from "std/web/html"

pub fn unsafe -> html.Html<Never> =
  html.a { href: "javascript:alert(1)", children: "unsafe" }
"#,
        ))
        .expect_err("URL props must require an opaque WebUrl");

        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0101"
                && diagnostic.message_key == "call.argument-type-mismatch"
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.message.contains("WebUrl"))
        }));
    }

    #[test]
    fn diagnoses_children_on_standard_void_html_elements() {
        for tag in ["img", "source"] {
            let props = if tag == "img" {
                r#"src: source, alt: "Hero", children: "invalid""#
            } else {
                r#"src: source, children: "invalid""#
            };
            let source = format!(
                "import * as html from \"std/web/html\"\n\
                 pub fn invalid source: html.WebUrl -> html.Html<Never> = html.{tag} {{ {props} }}\n"
            );
            let module_id = format!("artifact/web-html-{tag}-void-children");
            let diagnostics = compile_module(CompileInput::new("main.ssrg", &module_id, &source))
                .expect_err("void element children must stop compilation");

            assert!(
                diagnostics.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == "SES-T0701"
                        && diagnostic.message_key == "web.html.void-children"
                }),
                "{tag}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn diagnoses_standard_html_prop_contracts_and_spelling() {
        let warnings = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-prop-warnings",
            r#"import * as html from "std/web/html"

pub fn typo -> html.Html<Never> =
  html.div { clas: "hero", children: "Typo" }

pub fn wrongTag -> html.Html<Never> =
  html.textarea { href: "/wrong" }

pub fn unusedControl -> html.Html<Never> =
  html.button { preventClickDefault: True, children: "Save" }
"#,
        ))
        .expect("unknown standard props and unused controls are lint warnings");

        let warning_keys = warnings
            .diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message_key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            warning_keys,
            [
                "web.html.unknown-prop",
                "web.html.unknown-prop",
                "web.html.event-control-without-handler",
            ]
        );
        let typo = &warnings.diagnostics.diagnostics[0];
        assert_eq!(typo.code, "SES-L0101");
        assert_eq!(typo.fixes[0].edits[0].replacement, "class");

        let missing = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-required-prop",
            r#"import * as html from "std/web/html"

pub fn missing -> html.Html<Never> =
  html.img { alt: "missing source" }
"#,
        ))
        .expect_err("missing required standard props must stop compilation");
        assert!(missing.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0702"
                && diagnostic.message_key == "web.html.missing-required-prop"
                && diagnostic.related[0].message.contains("`src`")
        }));
        assert!(!missing
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "call.argument-type-mismatch"));

        let recovery = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-prop-recovery",
            concat!(
                "import * as html from \"std/web/html\"\n",
                "fn typo -> html.Html<Never> =\n",
                "  html.div { clas: \"hero\", children: \"Typo\" }\n",
                "pub let broken: Int =\n",
            ),
        ))
        .expect_err("parser recovery nodes must stop compilation");
        assert!(!recovery
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key.starts_with("web.html.")));
    }

    #[test]
    fn compiles_public_html_props_aliases_and_validated_custom_attributes() {
        let source = r#"import * as html from "std/web/html"

fn renderAnchor props: html.AnchorProps<Never, String> -> html.Html<Never> =
  html.a props

fn renderImage props: html.ImageProps<Never> -> html.Html<Never> =
  html.img props

fn renderForm props: html.FormProps<Never, String> -> html.Html<Never> =
  html.form props

fn renderTextarea props: html.TextareaProps<Never> -> html.Html<Never> =
  html.textarea props

fn renderCustom ariaLabel: html.Attribute -> dataState: html.Attribute -> html.Html<Never> =
  html.div {
    attributes: [ariaLabel, dataState],
    children: "custom attributes"
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-html-props-aliases",
            source,
        ))
        .expect("public props aliases and validated custom attributes should compile");

        assert!(!compiled.diagnostics.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.message_key.as_str(),
                "web.html.unknown-prop" | "web.html.missing-required-prop"
            )
        }));
    }

    #[test]
    fn compiles_typed_form_event_snapshots_through_the_runtime_abi() {
        let source = r#"import * as html from "std/web/html"

type Action =
  | DraftChanged String
  | CheckedChanged Bool
  | Submitted

fn draftAction event: html.InputEvent -> Action =
  DraftChanged event.value

fn checkedAction event: html.ChangeEvent -> Action =
  CheckedChanged event.checked

pub fn view draft: String -> checked: Bool -> html.Html<Action> =
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
        onInput: draftAction
      },
      html.input {
        checked,
        inputType: "checkbox",
        onChange: checkedAction
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
    fn compiles_focus_and_keyboard_actions_through_the_runtime_abi() {
        let source = r#"import * as html from "std/web/html"

type Action =
  | Focused
  | Blurred
  | KeyPressed String
  | ControlKey

fn keyAction event: html.KeyboardEvent -> html.EventAction<Action> =
  html.Dispatch (if event.controlKey then ControlKey else KeyPressed event.key)

fn taskKey action: Task<Unit> -> event: html.KeyboardEvent -> html.EventAction<Task<Unit>> =
  html.Dispatch action

fn ignoreKey event: html.KeyboardEvent -> html.EventAction<Action> =
  html.IgnoreEvent

pub fn view -> html.Html<Action> =
  html.button {
    onFocus: Focused,
    onBlur: Blurred,
    onKeyDown: keyAction,
    onKeyUp: ignoreKey,
    children: "Keyboard target"
  }

pub fn taskView action: Task<Unit> -> html.Html<Task<Unit>> =
  html.button {
    onFocus: action,
    onKeyDown: taskKey action,
    children: "Effect target"
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-keyboard-events",
            source,
        ))
        .expect("focus and keyboard actions should compile");

        assert!(compiled.generated.typescript.contains("type KeyboardEvent"));
        assert!(compiled.generated.typescript.contains("type EventAction"));
        assert!(compiled.generated.typescript.contains("_ssrg_html_button"));
        assert!(compiled
            .generated
            .typescript
            .contains("\"onFocus\": Focused"));
        assert!(compiled
            .generated
            .typescript
            .contains("\"onKeyDown\": keyAction"));
        assert!(compiled
            .generated
            .typescript
            .contains("(event: KeyboardEvent)"));
        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_html_Dispatch"));
        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_html_IgnoreEvent"));
        assert!(!compiled.generated.typescript.contains("std/web/html"));
    }

    #[test]
    fn compiles_pointer_scroll_and_event_controls_through_the_runtime_abi() {
        let source = r#"import * as html from "std/web/html"

type Action =
  | MouseButton Int
  | PointerKind String
  | Scrolled

fn mouseAction event: html.MouseEvent -> html.EventAction<Action> =
  html.DispatchPreventDefault (MouseButton event.button)

fn pointerAction event: html.PointerEvent -> html.EventAction<Action> =
  html.DispatchStopPropagation (PointerKind event.pointerType)

fn scrollAction event: html.ScrollEvent -> html.EventAction<Action> =
  html.DispatchPreventDefaultAndStop Scrolled

pub fn view -> html.Html<Action> =
  html.div {
    onMouseDown: mouseAction,
    onMouseUp: mouseAction,
    onPointerDown: pointerAction,
    onPointerUp: pointerAction,
    onDoubleClick: mouseAction,
    onContextMenu: mouseAction,
    onScroll: scrollAction,
    children: "Pointer target"
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-pointer-events",
            source,
        ))
        .expect("pointer, scroll, and event controls should compile");

        for type_name in [
            "type EventAction",
            "type MouseEvent",
            "type PointerEvent",
            "type ScrollEvent",
        ] {
            assert!(compiled.generated.typescript.contains(type_name));
        }
        for helper in [
            "_ssrg_html_DispatchPreventDefault",
            "_ssrg_html_DispatchStopPropagation",
            "_ssrg_html_DispatchPreventDefaultAndStop",
        ] {
            assert!(compiled.generated.typescript.contains(helper));
        }
        assert!(compiled
            .generated
            .typescript
            .contains("\"onPointerDown\": pointerAction"));
        assert!(compiled
            .generated
            .typescript
            .contains("\"onScroll\": scrollAction"));
        assert!(!compiled.generated.typescript.contains("std/web/html"));
    }

    #[test]
    fn compiles_form_table_and_interactive_tags_with_tag_specific_props() {
        let source = r#"import * as html from "std/web/html"

type Action = | Changed String

fn changed event: html.ChangeEvent -> Action =
  Changed event.value

pub fn view -> html.Html<Action> =
  html.div {
    children: [
      html.form {
        name: "profile",
        autoComplete: "on",
        children: html.fieldset {
          children: [
            html.legend { children: "Profile" },
            html.label { htmlFor: "age", children: "Age" },
            html.input {
              id: "age",
              name: "age",
              value: "18",
              readOnly: True,
              multiple: True,
              autoComplete: "off",
              autoFocus: True,
              min: "0",
              max: "120",
              step: "1",
              pattern: "[0-9]+"
            },
            html.textarea {
              name: "bio",
              value: "Typed UI",
              readOnly: True,
              autoComplete: "off",
              autoFocus: True,
              rows: 4,
              cols: 40
            },
            html.select {
              name: "theme",
              value: "dark",
              required: True,
              multiple: True,
              autoFocus: True,
              onChange: changed,
              children: [
                html.option { value: "light", disabled: True, children: "Light" },
                html.option { value: "dark", selected: True, children: "Dark" }
              ]
            },
            html.button {
              name: "save",
              value: "yes",
              autoFocus: True,
              children: "Save"
            }
          ]
        }
      },
      html.table {
        children: [
          html.caption { children: "Scores" },
          html.thead {
            children: html.tr {
              children: html.th { colSpan: 2, children: "Result" }
            }
          },
          html.tbody {
            children: html.tr {
              children: html.td { rowSpan: 2, children: "42" }
            }
          },
          html.tfoot {
            children: html.tr { children: html.td { children: "End" } }
          }
        ]
      },
      html.details {
        open: True,
        children: html.summary { children: "More details" }
      },
      html.dialog { open: True, children: "Ready" }
    ]
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-form-table-interactive",
            source,
        ))
        .expect("form, table, and interactive tags should compile");

        for runtime_name in [
            "form", "fieldset", "legend", "label", "input", "textarea", "select", "option",
            "button", "table", "caption", "thead", "tbody", "tfoot", "tr", "th", "td", "details",
            "summary", "dialog",
        ] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("_ssrg_html_{runtime_name}")),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        assert!(compiled.generated.typescript.contains("type ChangeEvent"));
    }

    #[test]
    fn compiles_global_attributes_and_validated_custom_html_values() {
        let source = r#"import * as html from "std/web/html"

type Action = | Selected

pub fn safeTag name: String -> Either<html.HtmlBuildError, html.Tag> =
  html.customTag name

pub fn safeAttribute name: String -> value: String
  -> Either<html.HtmlBuildError, html.Attribute> =
  html.attribute name value

pub fn safeUrl value: String -> Either<html.HtmlBuildError, html.WebUrl> =
  html.parseWebUrl value

pub effect fn parsedUrl value: String -> html.WebUrl
fails html.HtmlBuildError =
  fromEither (html.parseWebUrl value)

pub fn renderUrl parsed: Either<html.HtmlBuildError, html.WebUrl> -> String =
  match parsed {
    Left error -> show error
    Right _ -> "safe"
  }

pub fn invalidTag name: String -> html.HtmlBuildError =
  html.InvalidTagName name

pub fn unsafeUrl value: String -> html.HtmlBuildError =
  html.UnsafeWebUrlScheme value

pub fn card tag: html.Tag -> label: html.Attribute -> html.Html<Action> =
  html.custom tag {
    id: "profile",
    role: "article",
    tabIndex: 0,
    lang: "en",
    dir: "ltr",
    draggable: False,
    contentEditable: True,
    attributes: [label],
    onClick: Selected,
    children: "Mio"
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-global-custom-attributes",
            source,
        ))
        .expect("global and validated custom HTML values should compile");

        for runtime_name in [
            "customTag",
            "attribute",
            "parseWebUrl",
            "custom",
            "InvalidTagName",
            "UnsafeWebUrlScheme",
        ] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("_ssrg_html_{runtime_name}")),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        for type_name in ["Tag", "Attribute", "WebUrl", "HtmlBuildError"] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("type {type_name}")),
                "missing runtime type import for {type_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn compiles_bytes_and_utf8_through_runtime_imports() {
        let source = include_str!("../../../examples/spec/fixtures/compile/bytes-core.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/bytes-core",
            source,
        ))
        .expect("Bytes and UTF-8 source should compile");

        for runtime_name in [
            "_ssrg_bytes_fromInts",
            "_ssrg_bytes_length",
            "_ssrg_bytes_get",
            "_ssrg_bytes_append",
            "_ssrg_text_encodeUtf8",
            "_ssrg_text_decodeUtf8",
            "_ssrg_text_decodeUtf8Lossy",
            "_ssrg_show_bytesSliceErrorShow",
        ] {
            assert!(
                compiled.generated.typescript.contains(runtime_name),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        for type_name in [
            "Byte",
            "Bytes",
            "ByteError",
            "BytesSliceError",
            "Utf8DecodeError",
        ] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("type {type_name}")),
                "missing runtime type import for {type_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn compiles_hex_and_base64_through_runtime_imports() {
        let source = include_str!("../../../examples/spec/fixtures/compile/bytes-and-unicode.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/bytes-and-unicode",
            source,
        ))
        .expect("Hex, Base64, and Unicode source should compile");

        for runtime_name in [
            "_ssrg_base64_encodeUrl",
            "_ssrg_base64_decodeUrl",
            "_ssrg_hex_encode",
            "_ssrg_hex_decode",
        ] {
            assert!(
                compiled.generated.typescript.contains(runtime_name),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        for type_name in ["HexDecodeError", "Base64DecodeError"] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("type {type_name}")),
                "missing runtime type import for {type_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn compiles_unicode_text_lesson_and_canonical_execution_fixture() {
        for source in [
            include_str!("../../../examples/spec/fixtures/compile/unicode-text.ssrg"),
            include_str!("../../../examples/spec/lessons/28-unicode-text.ssrg"),
            include_str!("../../../examples/spec/artifacts/schema-1/unicode-text/main.ssrg"),
        ] {
            let compiled = compile_module(CompileInput::new(
                "main.ssrg",
                "artifact/unicode-text",
                source,
            ))
            .expect("canonical text and Unicode source should compile");
            assert_eq!(
                compiled.generated.metadata.runtime.unicode_version,
                seseragi_syntax::unicode::UNICODE_VERSION
            );
            assert!(compiled
                .generated
                .typescript
                .contains("@seseragi/runtime/unicode"));
            assert!(compiled
                .generated
                .typescript
                .contains("@seseragi/runtime/grapheme"));
        }
    }

    #[test]
    fn checks_unicode_version_before_source_initializers_and_preserves_source_maps() {
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/unicode-guard",
            "pub let answer: Int = 42\n",
        ))
        .unwrap();
        let lines = compiled.generated.typescript.lines().collect::<Vec<_>>();
        let guard = lines
            .iter()
            .position(|line| line.starts_with("$ssrg$assertUnicodeVersion("))
            .unwrap();
        let declaration = lines
            .iter()
            .position(|line| line.starts_with("export const answer"))
            .unwrap();
        assert!(guard < declaration);
        let mappings = compiled
            .generated
            .source_map
            .mappings
            .split(';')
            .collect::<Vec<_>>();
        assert!(mappings[..declaration]
            .iter()
            .all(|mapping| mapping.is_empty()));
        assert!(!mappings[declaration].is_empty());
    }

    #[test]
    fn unicode_abi_guard_cannot_be_shadowed_by_a_source_binding() {
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/unicode-guard-shadow",
            "fn _ssrg_assertUnicodeVersion value: String -> Int = 42\npub let answer: Int = _ssrg_assertUnicodeVersion \"local\"\n",
        ))
        .unwrap();
        let typescript = &compiled.generated.typescript;
        assert!(
            typescript.contains("import { assertUnicodeVersion as $ssrg$assertUnicodeVersion }")
        );
        assert!(typescript.contains("const _ssrg_assertUnicodeVersion = (value: string) => 42"));
        assert!(typescript
            .contains("export const answer: number = _ssrg_assertUnicodeVersion(\"local\")"));
        assert!(
            typescript
                .find("$ssrg$assertUnicodeVersion(\"17.0.0\")")
                .unwrap()
                < typescript.find("const _ssrg_assertUnicodeVersion").unwrap()
        );
    }

    #[test]
    fn compiles_random_and_entropy_through_provider_runtime_imports() {
        let source =
            include_str!("../../../examples/spec/fixtures/compile/random-and-entropy.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/random-and-entropy",
            source,
        ))
        .expect("Random and Entropy source should compile");

        for runtime_name in ["_ssrg_random_randomBytes", "_ssrg_entropy_secureBytes"] {
            assert!(
                compiled.generated.typescript.contains(runtime_name),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        for type_name in [
            "Random",
            "RandomSize",
            "Entropy",
            "EntropySize",
            "EntropyError",
        ] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("type {type_name}")),
                "missing runtime type import for {type_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn compiles_calendar_and_timezones_through_runtime_imports() {
        let source =
            include_str!("../../../examples/spec/fixtures/compile/calendar-timezones.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/calendar-timezones",
            source,
        ))
        .expect("calendar and timezone source should compile");

        for runtime_name in [
            "_ssrg_time_parseOffsetDateTime",
            "_ssrg_time_loadTimeZone",
            "_ssrg_time_resolveLocal",
        ] {
            assert!(
                compiled.generated.typescript.contains(runtime_name),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        for type_name in [
            "DateTimeError",
            "OffsetDateTime",
            "TimeZones",
            "TimeZoneError",
            "TimeZone",
            "LocalDateTime",
            "LocalResolution",
        ] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("type {type_name}")),
                "missing runtime type import for {type_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn compiles_json_core_and_codec_evidence_through_runtime_imports() {
        let source = include_str!("../../../examples/spec/fixtures/compile/json-core.ssrg");
        let compiled = compile_module(CompileInput::new("main.ssrg", "artifact/json-core", source))
            .expect("JSON core and codec source should compile");

        for runtime_name in [
            "_ssrg_json_parse",
            "_ssrg_json_stringify",
            "_ssrg_json_encodeString",
            "_ssrg_json_decodeString",
            "_ssrg_json_field",
            "_ssrg_json_optionalField",
            "_ssrg_json_index",
            "_ssrg_json_array",
            "_ssrg_json_record",
            "_ssrg_int_json_encode",
            "_ssrg_int_json_decode",
            "_ssrg_array_json_encode",
            "_ssrg_array_json_decode",
            "_ssrg_tuple_json_encode",
            "_ssrg_record_json_encode",
            "_ssrg_record_json_decode",
        ] {
            assert!(
                compiled.generated.typescript.contains(runtime_name),
                "missing runtime import for {runtime_name}: {}",
                compiled.generated.typescript
            );
        }
        for type_name in ["Json", "JsonParseError", "JsonReadError", "DecodeError"] {
            assert!(
                compiled
                    .generated
                    .typescript
                    .contains(&format!("type {type_name}")),
                "missing runtime type import for {type_name}: {}",
                compiled.generated.typescript
            );
        }
    }

    #[test]
    fn rejects_implicit_float_json_codecs() {
        let source = r#"import * as json from "std/json"

pub fn invalid value: Float -> String =
  json.encodeString value
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/json-float-codec",
            source,
        ))
        .expect_err("Float must not receive an implicit JSON codec");

        assert!(
            diagnostics.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
            }),
            "{:#?}",
            diagnostics.diagnostics
        );
    }

    #[test]
    fn rejects_a_form_event_handler_with_the_wrong_shape_before_lowering() {
        let source = r#"import * as html from "std/web/html"

type Action = | Submitted

pub fn invalid -> html.Html<Action> =
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
type Action = | Activate

let initialMode: Mode = Ready

fn update action: Action -> mode: Mode -> Mode =
  match action {
    Activate -> Active
  }

fn view mode: Mode -> html.Html<Action> =
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
    fn compiles_same_module_compact_effect_calls_with_inferred_contracts() {
        let source = r#"effect fn hello value: String =
  println value

effect fn keep<A> value: A =
  succeed value

pub effect fn main =
  do {
    hello "normal"
    hello $ "dollar"
    value <- keep 41
    hello (show value)
  }
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/compact-effect-callee",
            source,
        ))
        .expect("same-module compact effect calls should use their inferred contracts");

        assert!(compiled.generated.typescript.contains("const hello ="));
        assert!(compiled.generated.typescript.contains("const keep ="));
    }

    #[test]
    fn rejects_parameterized_external_failure_conflicts_in_compact_effects() {
        let source = include_str!(
            "../../../examples/spec/fixtures/compile/effect-compact-parameterized-failure-conflict.ssrg"
        );
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/effect-compact-parameterized-failure-conflict",
            source,
        ))
        .expect_err("distinct parameterized failures must stop before main lowering");

        assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
        let diagnostic = &diagnostics.diagnostics[0];
        assert_eq!(diagnostic.code, "SES-E0001");
        assert_eq!(diagnostic.message_key, "effect.compact-failure-conflict");
        assert_eq!(diagnostic.related.len(), 2);
        assert_eq!(
            diagnostic.related[0].message,
            "operation can fail with DomError"
        );
        assert_eq!(
            diagnostic.related[1].message,
            "operation can fail with DomRuntimeError<Never>"
        );
        let query_start = source.find("dom.query").expect("query origin");
        let run_start = source.find("dom.run").expect("run origin");
        assert_eq!(diagnostic.related[0].primary.start, query_start);
        assert_eq!(diagnostic.related[1].primary.start, run_start);
        assert_eq!(diagnostic.primary, diagnostic.related[1].primary);
    }

    #[test]
    fn rejects_explicit_success_and_environment_contract_mismatches() {
        let source = include_str!(
            "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/effect-explicit-contract-mismatch/main.ssrg"
        );
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/effect-explicit-contract-mismatch",
            source,
        ))
        .expect_err("invalid explicit contracts must stop before typed output is published");

        assert_eq!(diagnostics.diagnostics.len(), 3, "{diagnostics:#?}");
        assert_eq!(
            diagnostics
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message_key.as_str())
                .collect::<Vec<_>>(),
            [
                "effect.explicit-success-mismatch",
                "effect.explicit-environment-mismatch",
                "effect.explicit-environment-mismatch",
            ]
        );
        assert!(diagnostics.diagnostics.iter().all(|diagnostic| {
            diagnostic.code == "SES-E0001"
                && diagnostic.type_difference.is_some()
                && diagnostic.related.len() >= 2
        }));
    }

    #[test]
    fn accepts_an_external_explicit_success_contract() {
        let source = r#"import * as html from "std/web/html"

effect fn external -> html.WebUrl
fails html.HtmlBuildError =
  fromEither (html.parseWebUrl "https://example.com")
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/effect-explicit-external-success",
            source,
        ))
        .expect("external canonical success identity must satisfy the explicit contract");

        assert!(compiled.diagnostics.diagnostics.is_empty());
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
    fn lowers_signal_distinct_with_standard_and_user_eq_evidence() {
        let source = r#"import * as signals from "std/signal"

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

pub fn distinctInts source: signals.Signal<Int> -> signals.Signal<Int> =
  signals.distinct source

pub fn distinctStatuses source: signals.Signal<Status> -> signals.Signal<Status> =
  signals.distinct source

pub fn distinctGeneric<A> source: signals.Signal<A> -> signals.Signal<A>
where Eq<A> =
  signals.distinct source

pub let distinctStatusReference: signals.Signal<Status> -> signals.Signal<Status> =
  signals.distinct
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/signal-distinct-evidence",
            source,
        ))
        .expect("Signal.distinct should select and lower Eq evidence");

        assert!(compiled
            .generated
            .metadata
            .runtime
            .requirements
            .contains(&"signal.distinct".to_owned()));
        assert!(compiled
            .generated
            .typescript
            .contains("_ssrg_signal_distinct"));
        assert!(compiled.generated.typescript.contains("==="));
        assert!(compiled
            .generated
            .typescript
            .contains("__ssrg$instance$Eq$0[\"eq\"]"));
    }

    #[test]
    fn rejects_signal_distinct_without_eq_evidence() {
        let source = r#"import * as signals from "std/signal"

struct Reading {
  value: Int,
}

pub fn invalid source: signals.Signal<Reading> -> signals.Signal<Reading> =
  signals.distinct source
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/signal-distinct-missing-eq",
            source,
        ))
        .expect_err("Signal.distinct must require Eq evidence");

        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
        }));
    }

    #[test]
    fn rejects_unsupported_html_children_before_lowering() {
        let source = r#"import * as html from "std/web/html"

type Action = | Confirm

pub fn invalid -> html.Html<Action> =
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
    fn reports_missing_display_instances_for_effects_functions_and_opaque_handles() {
        let source = r#"import * as signals from "std/signal"
import * as dom from "std/web/dom"
import * as html from "std/web/html"

pub fn showEffect value: Effect<{}, Never, Unit> -> String = show value
pub fn debugTask value: Task<Unit> -> String = debug value
pub fn showSignal value: signals.Signal<Int> -> String = show value
pub fn debugMutableSignal value: signals.MutableSignal<Int> -> String = debug value
pub fn showFunction value: (Int -> Int) -> String = show value
pub fn debugDomTarget value: dom.DomTarget -> String = debug value
pub fn showHtml value: html.Html<Unit> -> String = show value
pub fn debugAttribute value: html.Attribute -> String = debug value
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/display-instance-unavailable",
            source,
        ))
        .expect_err("non-display runtime values must stop before lowering");

        assert_eq!(diagnostics.diagnostics.len(), 8);
        assert!(diagnostics.diagnostics.iter().all(|diagnostic| {
            diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
        }));
        for expected in [
            "Show<Effect<{}, Never, Unit>>",
            "Debug<Effect<{}, Never, Unit>>",
            "Show<signals.Signal<Int>>",
            "Debug<signals.MutableSignal<Int>>",
            "Show<Int -> Int>",
            "Debug<dom.DomTarget>",
            "Show<html.Html<Unit>>",
            "Debug<html.Attribute>",
        ] {
            assert!(
                diagnostics.diagnostics.iter().any(|diagnostic| diagnostic
                    .related
                    .iter()
                    .any(|related| related.message.contains(expected))),
                "missing diagnostic for {expected}: {diagnostics:#?}"
            );
        }
    }

    #[test]
    fn compiles_standard_error_display_with_canonical_provider_types() {
        let source = include_str!(
            "../../../examples/spec/artifacts/schema-1/standard-error-display/main.ssrg"
        );
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/standard-error-display",
            source,
        ))
        .expect("standard errors must select Show and Debug before lowering");

        for requirement in [
            "web.dom.error.show",
            "web.dom.error.debug",
            "web.dom.runtime-error.show",
            "web.dom.runtime-error.debug",
            "web.html.build-error.show",
            "web.html.build-error.debug",
        ] {
            assert!(
                compiled
                    .generated
                    .metadata
                    .runtime
                    .requirements
                    .contains(&requirement.to_owned()),
                "missing runtime requirement {requirement}"
            );
        }
        assert!(compiled
            .generated
            .typescript
            .contains("domRuntimeErrorShow<string>(_ssrg_show_stringShow)"));
        assert!(compiled
            .generated
            .typescript
            .contains("(value: HtmlBuildError)"));
        assert!(!compiled
            .generated
            .typescript
            .contains("html_HtmlBuildError"));
    }

    #[test]
    fn compiles_safe_integer_and_float_apis_through_the_runtime_abi() {
        let source = include_str!("../../../examples/spec/fixtures/compile/number-apis.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/number-apis",
            source,
        ))
        .expect("safe Int and Float standard APIs must compile through runtime bindings");

        for requirement in [
            "core.int.api.parse",
            "core.int.api.checked-multiply",
            "core.int.api.checked-add",
            "core.int.api.saturating-add",
            "core.int.api.saturating-multiply",
            "core.float64.api.from-int",
            "core.float64.api.to-int",
            "core.float64.api.round-integral",
            "core.float64.api.format",
            "core.float64.api.total-compare",
            "core.number.rounding.half-even",
        ] {
            assert!(
                compiled
                    .generated
                    .metadata
                    .runtime
                    .requirements
                    .contains(&requirement.to_owned()),
                "missing runtime requirement {requirement}"
            );
        }
        for module in [
            "@seseragi/runtime/int",
            "@seseragi/runtime/float",
            "@seseragi/runtime/number",
        ] {
            assert!(compiled.generated.typescript.contains(module));
        }
        for removed in ["wrappingAdd", "fromIntExact", "IntDivisionOverflow"] {
            assert!(!compiled.generated.typescript.contains(removed));
        }
        assert!(compiled.generated.typescript.contains(
            "(__ssrg$numeric$partial$0: number) => _ssrg_int_checkedAdd(1, __ssrg$numeric$partial$0)"
        ));
    }

    #[test]
    fn exposes_public_int_values_as_typescript_numbers_with_portable_metadata() {
        let source = "pub let answer: Int = 42\n\npub fn increment value: Int -> Int = value + 1\n";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/typescript-int-number",
            source,
        ))
        .expect("public Int values must lower to the safe-number ABI");

        assert!(compiled
            .generated
            .typescript
            .contains("export const answer: number = 42;"));
        assert!(compiled
            .generated
            .typescript
            .contains("export const increment = (value: number)"));
        assert!(!compiled.generated.typescript.contains("bigint"));
        assert!(!compiled.generated.typescript.contains("42n"));
        assert_eq!(
            compiled.generated.metadata.runtime.requirements,
            vec!["core.int", "core.int.add"]
        );
        assert_eq!(
            compiled.generated.source_map.sources,
            vec!["seseragi://artifact/typescript-int-number"]
        );
        assert!(compiled
            .generated
            .source_map
            .names
            .contains(&"answer".to_owned()));
        assert!(compiled
            .generated
            .source_map
            .names
            .contains(&"increment".to_owned()));
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
    fn compiles_sequence_apis_with_scoped_evidence_first_class_functions_and_callable_accumulators()
    {
        let source =
            include_str!("../../../examples/spec/artifacts/schema-1/array-list-apis/main.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/array-list-apis",
            source,
        ))
        .expect("all concrete sequence APIs must compile through the public registry");
        let output = &compiled.generated.typescript;
        assert!(!output.contains("= zip;"));
        assert!(output
            .contains("_ssrg_array_zip(__ssrg$collection$partial$0, __ssrg$collection$partial$1)"));
        assert!(output
            .contains("_ssrg_list_zip(__ssrg$collection$partial$0, __ssrg$collection$partial$1)"));
        assert!(output.contains("const folded: (argument: number) => number = _ssrg_array_reduceRight(identity, step, [1, 2, 3])"));
        assert!(output.contains("_ssrg_sizeErrorEq"));
        assert!(output.contains("_ssrg_int_ord_dictionary"));
        assert!(!output.contains("from \"std/"));
    }

    #[test]
    fn compiles_collection_reduce_until_with_generic_evidence_and_callable_accumulators() {
        let source = include_str!(
            "../../../examples/spec/artifacts/schema-1/collection-reduce-until/main.ssrg"
        );
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/collection-reduce-until",
            source,
        ))
        .expect("public generic short-circuit traversal must compile");
        let output = &compiled.generated.typescript;
        assert!(output.contains("_ssrg_collection_reduceUntil"));
        assert!(output.contains("type ReduceStep"));
        assert!(output.contains("generic as unknown as (argument: string) =>"));
        assert!(output.contains("generic as unknown as (argument: readonly [number, string]) =>"));
        assert!(!output.contains("from \"std/"));
        assert!(!output.contains("_ssrg_array_fromIterable"));
    }

    #[test]
    fn compiles_validation_with_standard_and_conditional_evidence() {
        let source =
            include_str!("../../../examples/spec/artifacts/schema-1/validation-apis/main.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/validation-apis",
            source,
        ))
        .expect("canonical Validation constructors, operations and evidence must compile");
        let output = &compiled.generated.typescript;
        for helper in [
            "validationEq",
            "validationShow",
            "validationDebug",
            "validationFunctor",
            "validationApplicative",
            "_ssrg_validation_fromEither",
            "_ssrg_validation_toEither",
        ] {
            assert!(output.contains(helper), "missing {helper}");
        }
        assert!(!output.contains("validationMonad"));
        assert!(!output.contains("from \"std/"));
    }

    #[test]
    fn rejects_validation_monad_and_invalid_constructor_payloads() {
        let source =
            include_str!("../../../examples/spec/artifacts/schema-1/validation-no-monad/main.ssrg");
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/validation-no-monad",
            source,
        ))
        .unwrap_err();
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SES-T0201"
                    && diagnostic.message_key == "instance.missing"),
            "{diagnostics:#?}"
        );
        for source in [
            "import * as v from \"std/validation\"\npub fn bad -> v.Validation<String, Int> = v.Invalid `[\"bad\"]\n",
            "import * as v from \"std/validation\"\npub fn bad -> v.Validation<String, Int> = v.Valid \"wrong\"\n",
            "import * as v from \"std/validation\"\ntype Error = | Error\npub fn bad value: v.Validation<Error, Int> -> String = show value\n",
            "import * as v from \"std/validation\"\ntype Validation<E, A> = | Local A\npub fn bad value: Validation<String, Int> -> Validation<String, Int> = map (\\n: Int -> n + 1) value\n",
            "import { Validation, Valid, Invalid } from \"std/validation\"\npub fn bad value: Validation<String, Int> -> Int = match value { Valid n -> n }\n",
        ] {
            assert!(compile_module(CompileInput::new("main.ssrg", "artifact/invalid-validation", source)).is_err(), "{source}");
        }
    }

    #[test]
    fn compiles_maybe_either_apis_with_traversable_and_conditional_monoid_evidence() {
        let source =
            include_str!("../../../examples/spec/artifacts/schema-1/maybe-either-apis/main.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/maybe-either-apis",
            source,
        ))
        .expect("Maybe/Either APIs must compile through the standard registry");
        let output = &compiled.generated.typescript;
        for helper in [
            "_ssrg_maybe_sequence",
            "_ssrg_maybe_traverse",
            "_ssrg_either_fold",
            "_ssrg_either_bimap",
            "_ssrg_either_sequence",
            "_ssrg_either_traverse",
            "_ssrg_maybe_monoid",
        ] {
            assert!(output.contains(helper), "missing {helper}");
        }
        assert!(!output.contains("from \"std/"));
    }

    #[test]
    fn rejects_maybe_either_calls_without_matching_types_and_evidence() {
        for source in [
            "import * as m from \"std/maybe\"\npub fn bad -> Int = m.withDefault 1 (Just \"wrong\")\n",
            "import * as m from \"std/maybe\"\npub fn bad values: Iterator<Maybe<Int>> -> Maybe<Iterator<Int>> = m.sequence values\n",
            "import * as e from \"std/either\"\npub fn bad value: Either<String, Int> -> Int = e.fold (\\error -> error) (\\n -> n) value\n",
            "pub fn bad values: Array<Maybe<Int>> -> Maybe<Int> = combine values\n",
        ] {
            assert!(compile_module(CompileInput::new("main.ssrg", "artifact/invalid-sum-apis", source)).is_err(), "{source}");
        }
    }

    #[test]
    fn rejects_invalid_reduce_until_step_accumulator_and_missing_iterable() {
        for source in [
            "import * as c from \"std/collection\"\npub fn bad -> Int = c.reduceUntil 0 (\\acc value -> acc + value) [1]\n",
            "import * as c from \"std/collection\"\npub fn bad -> Int = c.reduceUntil 0 (\\acc value -> c.Done \"wrong\") [1]\n",
            "import * as c from \"std/collection\"\npub fn bad -> Int = c.reduceUntil 0 (\\acc value: Int -> c.Next acc) True\n",
            "import * as c from \"std/collection\"\ntype ReduceStep<A> = | Next A\npub fn bad -> Int = c.reduceUntil 0 (\\acc value -> Next acc) [1]\n",
            "pub fn bad values: Iterator<Int> -> Int = reduce 0 (+) values\n",
        ] {
            assert!(compile_module(CompileInput::new("main.ssrg", "artifact/invalid-reduce-until", source)).is_err(), "{source}");
        }
    }

    #[test]
    fn rejects_sequence_calls_without_required_evidence_or_matching_element_types() {
        for source in [
            "import * as arrays from \"std/array\"\npub fn bad -> Array<Float> = arrays.sort [1.0, 2.0]\n",
            "import * as arrays from \"std/array\"\nimport * as maps from \"std/map\"\ntype Key deriving Eq = Key Int\npub fn bad -> maps.Map<Key, Array<Int>> = arrays.groupBy Key [1, 2]\n",
            "import * as arrays from \"std/array\"\npub fn bad -> Array<(Int, Int)> = arrays.zip [\"x\"] [1]\n",
        ] {
            assert!(compile_module(CompileInput::new("main.ssrg", "artifact/invalid-sequence", source)).is_err(), "{source}");
        }
    }

    #[test]
    fn compiles_the_current_process_directory_as_a_portable_path() {
        const SOURCE: &str =
            include_str!("../../../examples/spec/fixtures/compile/process-current-directory.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/process-current-directory",
            SOURCE,
        ))
        .expect("the canonical current-directory source must compile");

        for requirement in ["process.service-type", "process.current-directory"] {
            assert!(
                compiled
                    .generated
                    .metadata
                    .runtime
                    .requirements
                    .contains(&requirement.to_owned()),
                "missing runtime requirement {requirement}"
            );
        }
        for generated in [
            "@seseragi/runtime/path",
            "@seseragi/runtime/process",
            "_ssrg_process_currentDirectory",
            "type Path",
            "type Process",
        ] {
            assert!(
                compiled.generated.typescript.contains(generated),
                "missing generated process/path surface {generated}"
            );
        }
    }

    #[test]
    fn compiles_array_and_list_transformations_through_the_runtime_abi() {
        let source = r#"import * as arrays from "std/array"
import * as lists from "std/list"

fn even value: Int -> Bool = value % 2 == 0
fn labelEven value: Int -> Maybe<String> =
  if even value then Just `#${value}` else Nothing
fn repeatArray value: Int -> Array<Int> = [value, value]
fn repeatList value: Int -> List<Int> = `[value, value]

pub fn filteredArray -> Array<Int> = arrays.filter even [1, 2, 3]
pub fn filteredList -> List<Int> = lists.filter even `[1, 2, 3]
pub fn mappedArray -> Array<String> = arrays.filterMap labelEven [1, 2, 3]
pub fn mappedList -> List<String> = lists.filterMap labelEven `[1, 2, 3]
pub fn flattenedArray -> Array<Int> = arrays.flatMap repeatArray [1, 2]
pub fn flattenedList -> List<Int> = lists.flatMap repeatList `[1, 2]
pub fn foundArray -> Maybe<Int> = arrays.find even [1, 2, 3]
pub fn foundList -> Maybe<Int> = lists.find even `[1, 2, 3]
pub fn takenArray -> Array<Int> = arrays.take 2 [1, 2, 3]
pub fn takenList -> List<Int> = lists.take 2 `[1, 2, 3]
pub fn droppedArray -> Array<Int> = arrays.drop 2 [1, 2, 3]
pub fn droppedList -> List<Int> = lists.drop 2 `[1, 2, 3]
pub fn appendedArray -> Array<Int> = arrays.append [3, 4] [1, 2]
pub fn appendedList -> List<Int> = lists.append `[3, 4] `[1, 2]
pub fn concatenatedArray -> Array<Int> = arrays.concat [[1, 2], [3, 4]]
pub fn reversedArray -> Array<Int> = arrays.reverse [1, 2, 3]
pub fn reversedList -> List<Int> = lists.reverse `[1, 2, 3]
pub fn convertedList -> List<Int> = arrays.toList [1, 2]
pub fn convertedArray -> Array<Int> = lists.toArray `[1, 2]
"#;
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/collection-transform",
            source,
        ))
        .expect("standard collection transformations should compile");

        for helper in [
            "_ssrg_array_filter",
            "_ssrg_array_filterMap",
            "_ssrg_array_flatMap",
            "_ssrg_list_filter",
            "_ssrg_list_filterMap",
            "_ssrg_list_flatMap",
            "_ssrg_array_find",
            "_ssrg_list_find",
            "_ssrg_array_take",
            "_ssrg_list_take",
            "_ssrg_array_drop",
            "_ssrg_list_drop",
            "_ssrg_array_append",
            "_ssrg_list_append",
            "_ssrg_array_concat",
            "_ssrg_array_reverse",
            "_ssrg_list_reverse",
            "_ssrg_array_toList",
            "_ssrg_list_toArray",
        ] {
            assert!(compiled.generated.typescript.contains(helper), "{helper}");
        }
    }

    #[test]
    fn rejects_a_malformed_nested_lambda_before_lowering() {
        let source = r#"import * as arrays from "std/array"

pub fn text parts: Array<String> -> String =
  arrays.reduce "" (\current: String -> part: String -> current + part) parts
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/namespaced-reduce",
            source,
        ))
        .expect_err("a malformed nested lambda must not reach TypeScript lowering");

        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-P0001" && diagnostic.message_key == "parser.expected-expression"
        }));
    }

    #[test]
    fn rejects_unexported_namespaced_reduce_across_collection_modules() {
        for (alias, module, collection) in [
            ("arrays", "std/array", "Array"),
            ("lists", "std/list", "List"),
        ] {
            let source = format!(
                "import * as {alias} from \"{module}\"\n\nfn append current: String -> part: String -> String = current + part\n\npub fn text parts: {collection}<String> -> String =\n  {alias}.reduce \"\" append parts\n"
            );
            let diagnostics = compile_module(CompileInput::new(
                "main.ssrg",
                "artifact/namespaced-reduce",
                &source,
            ))
            .expect_err("collection modules do not export the Prelude reduce function");

            assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "SES-N0104"
                    && diagnostic.message_key == "module.export-unresolved"
            }));
        }
    }

    #[test]
    fn rejects_collection_transform_callbacks_with_the_wrong_result_type() {
        let source = r#"import * as arrays from "std/array"

pub fn invalid -> Array<Int> =
  arrays.filter (\value: Int -> value + 1) [1, 2, 3]
"#;
        let diagnostics = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/collection-transform-invalid",
            source,
        ))
        .expect_err("a filter callback must return Bool before lowering");

        assert!(
            diagnostics.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "SES-T0101"
                    && diagnostic.message_key == "lambda.body-type-mismatch"
            }),
            "{diagnostics:?}"
        );
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
