use crate::analyze_linked_module;
use seseragi_project::{link_module, standard_module_target, ModuleLinkTarget};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

fn linked_with_std<const N: usize>(
    main_source: &str,
    dependencies: [(&str, &str, &str); N],
) -> seseragi_project::LinkedModule {
    let mut targets = BTreeMap::new();
    for std_module in ["std/signal", "std/web/html", "std/web/dom"] {
        targets.insert(
            std_module.to_owned(),
            standard_module_target(std_module).unwrap(),
        );
    }
    for (specifier, module, source) in dependencies {
        let unlinked = parse_unlinked_module_interface("dependency.ssrg", module, source);
        let interface = crate::analyze_module_interface(
            seseragi_syntax::parse_diagnostics("dependency.ssrg", source),
            unlinked.interface.clone(),
            source,
        )
        .unwrap()
        .typed_interface
        .into_link_interface();
        targets.insert(
            specifier.to_owned(),
            ModuleLinkTarget::same_package(unlinked.header, interface).unwrap(),
        );
    }
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/app::main", main_source);
    link_module(main, &targets).unwrap()
}

#[test]
fn coerces_mutable_signal_from_imported_constructor_when_mapping_to_html() {
    let phase = r#"pub type Phase deriving Eq, Show =
  | Waiting
  | Done
"#;
    let main = r##"import * as dom from "std/web/dom"
import * as html from "std/web/html"
import * as signals from "std/signal"
import { Signal } from "std/signal"
import { Phase, Waiting } from "./phase"

fn view phase: Phase -> html.Html<Task<Unit>> = html.p { children: show phase }

effect fn probe -> Signal<html.Html<Task<Unit>>> fails Never = do {
  state <- signals.make Waiting
  succeed (signals.map view state)
}

fn perform action: Task<Unit> -> Task<Unit> = action

pub effect fn main -> Unit with dom: dom.Dom fails String = do {
  shown <- probe ()
  target <- dom.query "#app" |> mapError (\_error: dom.DomError -> "no target")
  dom.run (dom.defaultOptions ()) target perform shown
    |> mapError (\_error: dom.DomRuntimeError<Never> -> "failed")
}
"##;
    let linked = linked_with_std(main, [("./phase", "fixture/app::phase", phase)]);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main),
        linked,
        main,
    )
    .unwrap_or_else(|diagnostics| panic!("unexpected diagnostics: {diagnostics:#?}"));
    assert!(analyzed.diagnostics.diagnostics.is_empty());
}

#[test]
fn rejects_reverse_signal_to_mutable_signal_coercion_with_imported_type() {
    let phase = r#"pub type Phase deriving Eq, Show =
  | Waiting
  | Done
"#;
    let main = r#"import * as signals from "std/signal"
import { MutableSignal, Signal } from "std/signal"
import { Phase, Waiting } from "./phase"

fn needMutable state: MutableSignal<Phase> -> Task<Unit> =
  signals.set state Waiting

fn bad readonly: Signal<Phase> -> Task<Unit> = needMutable readonly
"#;
    let linked = linked_with_std(main, [("./phase", "fixture/app::phase", phase)]);
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main),
        linked,
        main,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "call.argument-type-mismatch"),
        "{diagnostics:#?}"
    );
}

#[test]
fn rejects_mutable_signal_coercion_across_distinct_same_named_imported_types() {
    let phase = r#"pub type Phase deriving Eq, Show =
  | Waiting
"#;
    let other = r#"pub type Phase deriving Eq, Show =
  | OtherWaiting
"#;
    let main = r#"import * as html from "std/web/html"
import * as signals from "std/signal"
import { Signal } from "std/signal"
import { Waiting } from "./phase"
import { Phase } from "./other"

fn view phase: Phase -> html.Html<Task<Unit>> = html.p { children: show phase }

pub effect fn probe -> Signal<html.Html<Task<Unit>>> fails Never = do {
  state <- signals.make Waiting
  succeed (signals.map view state)
}
"#;
    let linked = linked_with_std(
        main,
        [
            ("./phase", "fixture/app::phase", phase),
            ("./other", "fixture/app::other", other),
        ],
    );
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main),
        linked,
        main,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "call.argument-type-mismatch"),
        "{diagnostics:#?}"
    );
}
