use seseragi_driver::{compile_project, ProjectModuleInput};
use seseragi_project::ModuleGraph;

const MODEL: &str = r#"
pub type Box<A> = | Box A
pub alias Wrapped<A> = Box<A>
pub alias Again<A> = Wrapped<A>
pub fn boxed<A> value: A -> Again<A> = Box value
pub opaque struct Secret<A> { value: A }
pub fn secret<A> value: A -> Secret<A> = Secret { value: value }
pub type Pair<A, B> = | Pair (A, B)
pub alias Flipped<A, B> = Pair<B, A>
pub fn paired<A> value: A -> Pair<String, A> = Pair ("fixed=", value)
"#;

fn project(
    source: &str,
) -> Result<seseragi_driver::CompiledProject, seseragi_driver::ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    graph
        .add_module("fixture/model@1.0.0::lib".to_owned(), [])
        .unwrap();
    graph
        .add_module(
            "fixture/app@1.0.0::facade".to_owned(),
            [("model".to_owned(), "fixture/model@1.0.0::lib".to_owned())],
        )
        .unwrap();
    graph
        .add_module(
            "fixture/app@1.0.0::main".to_owned(),
            [
                ("model".to_owned(), "fixture/model@1.0.0::lib".to_owned()),
                (
                    "./facade".to_owned(),
                    "fixture/app@1.0.0::facade".to_owned(),
                ),
            ]
            .into_iter()
            .filter(|(specifier, _)| source.contains(&format!("from \"{specifier}\""))),
        )
        .unwrap();
    compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "model/src/lib.ssrg",
                "fixture/model@1.0.0::lib",
                MODEL,
                "dist/model/lib.js",
            )
            .with_package_scope("fixture/model@1.0.0"),
            ProjectModuleInput::new(
                "app/src/facade.ssrg",
                "fixture/app@1.0.0::facade",
                "pub import { Box, boxed, Secret, secret } from \"model\"\n",
                "dist/app/facade.js",
            )
            .with_package_scope("fixture/app@1.0.0"),
            ProjectModuleInput::new(
                "app/src/main.ssrg",
                "fixture/app@1.0.0::main",
                source,
                "dist/app/main.js",
            )
            .with_package_scope("fixture/app@1.0.0"),
        ],
    )
}

#[test]
fn infers_renamed_generic_arguments_from_an_imported_alias_return() {
    for local in ["", "type Box<A> = | Local A\n"] {
        let source = format!("import {{ Box as Foreign }} from \"model\"\nimport {{ boxed }} from \"./facade\"\n{local}fn keep<A> value: Foreign<A> -> Foreign<A> = value\npub fn main -> Int = match keep (boxed 42) {{ Foreign value -> value }}\n");
        project(&source).unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
    }
}

#[test]
fn keeps_canonical_arguments_across_direct_namespace_and_facade_imports() {
    for (import, name, call) in [
        ("import { Box, boxed } from \"model\"", "Box", "boxed"),
        (
            "import { Box as Foreign, boxed } from \"model\"",
            "Foreign",
            "boxed",
        ),
        (
            "import * as model from \"model\"",
            "model.Box",
            "model.boxed",
        ),
        (
            "import { Box as Foreign, boxed } from \"./facade\"",
            "Foreign",
            "boxed",
        ),
        (
            "import * as model from \"./facade\"",
            "model.Box",
            "model.boxed",
        ),
    ] {
        let source = format!("{import}\nfn keep<A> value: {name}<A> -> {name}<A> = value\nfn nested<A> value: {name}<{name}<A>> -> {name}<{name}<A>> = value\npub fn main -> Int = {{ let first = keep ({call} 42); let second = nested ({call} first); match second {{ {name} ({name} value) -> value }} }}\n");
        project(&source).unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
    }
}

#[test]
fn keeps_generic_scope_and_child_evidence_when_return_spelling_differs() {
    let source = r#"
import { Box as Foreign } from "model"
import { boxed } from "./facade"
type Int = | Nominal Bool
type Box<A> = | Local A
fn describe<A> value: Foreign<A> -> String where Show<A> =
  match value { Foreign inner -> show inner }
fn forward<A> value: A -> String where Show<A> = describe (boxed value)
fn wrapped<A> value: A -> Foreign<A> = boxed value
fn forwarded<A> value: A -> String where Show<A> = describe (wrapped value)
pub fn main -> String = forward 42
"#;
    project(source).unwrap_or_else(|error| panic!("{error:#?}"));
}

#[test]
fn rejects_other_owners_and_conflicting_arguments_after_generic_inference() {
    for body in [
        "fn keep<A> value: Foreign<A> -> Foreign<A> = value\npub fn main -> Box<Int> = keep (Local 42)",
        "fn same<A> first: Foreign<A> -> second: Foreign<A> -> Foreign<A> = first\npub fn main -> Foreign<Int> = same (boxed 42) (boxed True)",
        "fn same<A> first: Foreign<A> -> second: A -> Foreign<A> = first\npub fn main -> Foreign<Int> = same (boxed 42) True",
        "fn keep<A> value: Foreign<A> -> Foreign<A> = value\nfn wrong<A, B> first: A -> second: B -> Foreign<A> = keep (boxed second)",
    ] {
        let source = format!("import {{ Box as Foreign }} from \"model\"\nimport {{ boxed }} from \"./facade\"\ntype Box<A> = | Local A\n{body}\n");
        let error = project(&source).unwrap_err();
        assert!(matches!(error, seseragi_driver::ProjectCompileError::Diagnostics { modules } if modules.iter().any(|module| module.diagnostics.diagnostics.iter().any(|diagnostic| diagnostic.code == "SES-T0101"))), "{source}");
    }
}

#[test]
fn opaque_returns_keep_their_owner_without_exposing_the_representation() {
    let prefix = "import { Secret as Hidden, secret } from \"./facade\"\nfn keep<A> value: Hidden<A> -> Hidden<A> = value\n";
    project(&format!(
        "{prefix}pub fn main -> Hidden<Int> = keep (secret 42)\n"
    ))
    .unwrap();
    for body in [
        "pub fn main -> Int = (keep (secret 42)).value",
        "struct Secret<A> { value: A }\npub fn main -> Hidden<Int> = keep (Secret { value: 42 })",
    ] {
        project(&format!("{prefix}{body}\n")).unwrap_err();
    }
}

#[test]
fn generalizes_local_lambdas_through_imported_nominal_alias_returns() {
    for local in ["", "type Int = | Nominal Bool\ntype Box<A> = | Local A\n"] {
        let source = format!(
            r#"
import {{ Box as Foreign }} from "model"
import {{ boxed }} from "./facade"
{local}fn describe<A> value: Foreign<A> -> String where Show<A> =
  match value {{ Foreign inner -> show inner }}
pub effect fn main = do {{
  let forward = \value -> describe (boxed value)
  println (forward 42)
  println (forward True)
}}
"#
        );
        project(&source).unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
    }
}

#[test]
fn rank1_nominal_unification_preserves_alias_arguments_and_rejects_other_owners() {
    let source = r#"
import { Pair as Foreign, Flipped, paired } from "model"
fn describe<A> value: Flipped<A, String> -> String where Show<A> =
  match value { Foreign (prefix, inner) -> prefix + show inner }
pub effect fn main = do {
  let forward = \value -> describe (paired value)
  println (forward 42)
}
"#;
    project(source).unwrap_or_else(|error| panic!("{error:#?}"));
    project(
        r#"
import { Box as Foreign } from "model"
type Box<A> = | Local A
fn keep<A> value: Foreign<A> -> Foreign<A> = value
pub effect fn main = do {
  let wrong = \value -> keep (Local value)
  wrong 42
}
"#,
    )
    .unwrap_err();
}
