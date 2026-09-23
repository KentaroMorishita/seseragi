use seseragi_driver::{compile_module, CompileInput};

#[test]
fn compiles_svg_pointer_wheel_and_dom_measurement_from_one_standard_contract() {
    let source = include_str!(
        "../../../examples/spec/fixtures/projects/web-scene-interaction/src/main.ssrg"
    );
    let compiled = compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/web-scene-interaction",
        source,
    ))
    .expect("web scene fixture must compile");
    let typescript = &compiled.generated.typescript;

    for expected in [
        "@seseragi/runtime/svg",
        "_ssrg_svg_toHtml",
        "_ssrg_dom_capturePointer",
        "_ssrg_dom_releasePointer",
        "_ssrg_dom_measure",
        "_ssrg_dom_observeResize",
        "_ssrg_dom_disconnect",
    ] {
        assert!(
            typescript.contains(expected),
            "missing {expected}\n{typescript}"
        );
    }
}

#[test]
fn compiles_typed_logical_dom_bindings_across_html_and_svg() {
    let source = r#"
import * as dom from "std/web/dom"
import * as html from "std/web/html"
import * as signals from "std/signal"
import * as svg from "std/web/svg"

type Action =
  | Changed

pub fn content
  label: signals.Signal<String>
  -> hidden: signals.Signal<Bool>
  -> viewBox: signals.Signal<String>
  -> x: signals.Signal<Float>
  -> dom.DomContent<Action> = {
  let labelRef = html.elementRef "label"
  let sceneRef = html.elementRef "scene"
  let nodeRef = html.elementRef "node"
  let result = dom.content (html.div {
      children: [
        html.span { elementRef: labelRef, children: "ready" },
        (svg.svg {
          elementRef: sceneRef,
          viewBox: "0 0 100 100",
          children: [svg.rect {
            elementRef: nodeRef,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0
          }]
        } |> svg.toHtml)
      ]
    }) [
      dom.bind (dom.textTarget labelRef) label,
      dom.bind (dom.hiddenTarget labelRef) hidden,
      dom.bind (svg.viewBoxTarget sceneRef) viewBox,
      dom.bind (svg.xTarget nodeRef) x
    ]
  result
}
"#;
    let compiled = compile_module(CompileInput::new(
        "typed-bindings.ssrg",
        "fixture/typed-bindings",
        source,
    ))
    .expect("typed logical bindings must compile");
    let typescript = &compiled.generated.typescript;

    for expected in [
        "_ssrg_html_elementRef",
        "_ssrg_dom_bind",
        "_ssrg_dom_textTarget",
        "_ssrg_dom_hiddenTarget",
        "_ssrg_svg_viewBoxTarget",
        "_ssrg_svg_xTarget",
    ] {
        assert!(
            typescript.contains(expected),
            "missing {expected}\n{typescript}"
        );
    }
}

#[test]
fn rejects_a_signal_whose_value_does_not_match_the_binding_target() {
    let source = r#"
import * as dom from "std/web/dom"
import * as html from "std/web/html"
import * as signals from "std/signal"

type Action =
  | Changed

pub fn invalid source: signals.Signal<Bool> -> dom.DomBinding<Action> = {
  let target = html.elementRef "target"
  dom.bind (dom.textTarget target) source
}
"#;
    let diagnostics = compile_module(CompileInput::new(
        "invalid-typed-binding.ssrg",
        "fixture/invalid-typed-binding",
        source,
    ))
    .expect_err("text targets must reject Bool signals");

    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-T0101"),
        "{diagnostics:?}"
    );
    assert!(diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code != "SES-P0001"));
}
