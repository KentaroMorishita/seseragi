use seseragi_driver::{
    analyze_module, compile_linked_module, compile_module, plan_typescript_outputs, BuildProfile,
    CompileInput, TypeScriptInstanceOutput, TypeScriptModuleOutput,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

fn compiles(source: &str) {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = || {
            CompileInput::new(
                "collection-inference.ssrg",
                "proof/collection-inference",
                source,
            )
            .with_profile(profile)
        };
        compile_module(input()).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
        assert!(analyze_module(input()).diagnostics.diagnostics.is_empty());
    }
}

#[test]
fn html_children_infer_actions_from_every_element() {
    for elements in [
        r#"html.p { children: "x" }, html.button { onClick: noop (), children: "b" }"#,
        r#"html.button { onClick: noop (), children: "b" }, html.p { children: "x" }"#,
    ] {
        for body in [
            format!("fn view -> html.Html<Task<Unit>> = html.div {{ children: [{elements}] }}"),
            format!("fn view -> html.Html<Task<Unit>> = html.div {{ children: [html.div {{ children: [{elements}] }}] }}"),
            format!("fn view -> html.Html<Task<Unit>> = {{ let children = [{elements}]; html.div {{ children: children }} }}"),
            format!("fn view -> html.Html<Task<Unit>> = {{ let children = [html.div {{ children: [{elements}] }}]; html.div {{ children: children }} }}"),
            format!("fn view -> Array<html.Html<Task<Unit>>> = [{elements}]"),
        ] {
            compiles(&format!(
                "import * as html from \"std/web/html\"\nfn noop -> Task<Unit> = pure ()\n{body}\npub effect fn main = println \"ok\"\n"
            ));
        }
    }
}

#[test]
fn expected_actions_reach_children_without_handlers() {
    compiles(
        r#"
import * as html from "std/web/html"
fn direct -> Array<html.Html<Task<Unit>>> = [html.p { children: "x" }]
fn view -> html.Html<Task<Unit>> = html.div {
  children: [html.div { children: [html.p { children: "x" }] }]
}
pub effect fn main = println "ok"
"#,
    );
}

#[test]
fn phantom_adt_elements_share_inference_without_an_annotation() {
    compiles(include_str!("../../../examples/spec/fixtures/rust-backend-regressions/generic-collection-element-inference.ssrg"));
    for elements in ["Empty, Value 42", "Value 42, Empty"] {
        compiles(&format!(
            "type Box<A> = | Empty | Value A\nfn annotated -> Array<Box<Int>> = [{elements}]\nfn local -> Array<Box<Int>> = {{ let values = [{elements}]; values }}\npub effect fn main = println \"ok\"\n"
        ));
    }
}

#[test]
fn user_record_calls_and_constraints_share_element_inference() {
    for elements in [
        "blank { label: \"x\" }, populated { value: 42 }",
        "populated { value: 42 }, blank { label: \"x\" }",
    ] {
        compiles(&format!(
            r#"
type Box<A> = | Empty | Value A
trait IntoBoxes<C, A> {{ fn intoBoxes children: C -> Array<Box<A>> }}
instance<A> IntoBoxes<Array<Box<A>>, A> {{
  fn intoBoxes children: Array<Box<A>> -> Array<Box<A>> = children
}}
fn blank<A> props: {{label: String}} -> Box<A> = Empty
fn populated<A> props: {{value: A}} -> Box<A> = Value props.value
fn container<C, A> props: {{children: C}} -> Box<A> where IntoBoxes<C, A> = Empty
fn expected -> Array<Box<Int>> = [blank {{ label: "x" }}]
fn local -> Array<Box<Int>> = {{ let values = [{elements}]; values }}
fn nested -> Array<Box<Int>> = [container {{ children: [{elements}] }}]
pub effect fn main = println "ok"
"#
        ));
    }
}

#[test]
fn imported_constraints_relate_nested_generic_elements() {
    let domain = r#"
pub type Box<A> = | Empty | Value A
pub trait IntoBoxes<C, A> { fn intoBoxes children: C -> Array<Box<A>> }
instance<A> IntoBoxes<Array<Box<A>>, A> {
  fn intoBoxes children: Array<Box<A>> -> Array<Box<A>> = children
}
pub fn container<C, A> props: {children: C} -> Box<A> where IntoBoxes<C, A> = Empty
"#;
    let dependency =
        compile_module(CompileInput::new("boxes.ssrg", "proof/boxes", domain)).unwrap();
    let header = parse_unlinked_module_interface("boxes.ssrg", "proof/boxes", domain).header;
    let target =
        ModuleLinkTarget::same_package(header, dependency.typed_interface.into_link_interface())
            .unwrap();
    let plan = plan_typescript_outputs(
        "main.js",
        [
            TypeScriptModuleOutput::new("proof/boxes", "boxes.js").with_instance_exports(
                dependency
                    .generated
                    .metadata
                    .instances
                    .iter()
                    .map(|instance| {
                        TypeScriptInstanceOutput::new(
                            &instance.identity,
                            &instance.dictionary_export,
                        )
                    }),
            ),
        ],
    )
    .unwrap();
    for elements in ["boxes.Empty, boxes.Value 42", "boxes.Value 42, boxes.Empty"] {
        let source = format!(
            r#"
import * as boxes from "./boxes"
pub fn view -> Array<boxes.Box<Int>> = {{
  let values = [boxes.container {{ children: [{elements}] }}]
  values
}}
"#
        );
        let linked = link_module(
            parse_unlinked_module_interface("main.ssrg", "proof/main", &source),
            &BTreeMap::from([("./boxes".to_owned(), target.clone())]),
        )
        .unwrap();
        compile_linked_module(linked, &source, &plan)
            .unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    }
}

#[test]
fn incompatible_elements_still_report_the_array_element() {
    for (declarations, elements, rejected) in [
        ("", "1, True", "True"),
        ("", "True, 1", "1"),
        ("type Box<A> = | Empty | Value A\nlet number: Box<Int> = Value 42\nlet flag: Box<Bool> = Value True\n", "number, flag", "flag"),
        ("newtype One = Int\nnewtype Two = Int\n", "One 1, Two 2", "Two 2"),
    ] {
        let source = format!("{declarations}let values = [{elements}]\n");
        let input = CompileInput::new("invalid.ssrg", "proof/collection-inference", &source);
        let error = compile_module(input).expect_err(&source);
        assert_eq!(analyze_module(input).diagnostics, error);
        let diagnostic = error.diagnostics.iter().find(|diagnostic| {
            diagnostic.message_key == "array.element-type-mismatch"
        }).unwrap_or_else(|| panic!("{source}\n{error:?}"));
        assert_eq!(diagnostic.code, "SES-T0101");
        assert_eq!(&source[diagnostic.primary.start..diagnostic.primary.end], rejected);
    }
}
