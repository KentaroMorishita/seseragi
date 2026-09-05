use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn public_iterator_surface_reuses_canonical_iterator_evidence() {
    let source = r#"
import * as iterator from "std/iterator"
fn advance n: Int -> Maybe<(Int, Int)> =
  if n < 4 then Just (n, n + 1) else Nothing
pub let values: iterator.Iterator<Int> = iterator.unfold advance 1
pub let first = iterator.next values
pub let repeated = iterator.next values
pub let mapped = [n + 1 | n <- values]
fn head<C> values: C -> Maybe<(Int, iterator.Iterator<Int>)>
where Iterable<C, Int> = iterator.next (iterate values)
pub let arrayHead = head [1, 2]
pub let listHead = head `[1, 2]
pub let rangeHead = head (1..=2)
pub let iteratorHead = head values
fn headValue<C> values: C -> Maybe<Int> where Iterable<C, Int> =
  match iterator.next (iterate values) {
    Nothing -> Nothing
    Just (value, _) -> Just value
  }
pub let inferred = headValue values
"#;
    let analysis = analyze_module(CompileInput::new(
        "iterator.ssrg",
        "fixture/iterator",
        source,
    ));
    assert!(
        analysis.diagnostics.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for identity in [
        "std/prelude::Iterator",
        "std/iterator::unfold",
        "std/iterator::next",
    ] {
        assert!(
            analysis
                .standard_library
                .iter()
                .any(|item| item.identity == identity),
            "missing {identity}"
        );
    }
    let compiled = compile_module(CompileInput::new(
        "iterator.ssrg",
        "fixture/iterator",
        source,
    ))
    .expect("public Iterator uses the existing prelude identity");
    let ts = compiled.generated.typescript;
    assert!(ts.contains("@seseragi/runtime/iterator"));
    assert!(ts.contains("_ssrg_iterator_unfold"));
    assert!(ts.contains("_ssrg_iterator_next"));
    assert!(ts.contains("iteratorIterable"));
}

#[test]
fn public_iterator_has_no_reducible_instance() {
    let source = r#"
import * as iterator from "std/iterator"
fn advance n: Int -> Maybe<(Int, Int)> = Just (n, n + 1)
pub let invalid = sum (iterator.unfold advance 0)
"#;
    let diagnostics = compile_module(CompileInput::new("invalid.ssrg", "fixture/invalid", source))
        .expect_err("potentially infinite Iterator is not Reducible");
    assert!(
        diagnostics.diagnostics.iter().any(|d| d.code == "SES-T0201"
            && d.related
                .iter()
                .any(|related| related.message.contains("Reducible"))),
        "{diagnostics:?}"
    );
}

#[test]
fn iterator_identity_survives_user_module_interfaces() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let domain = r#"
import * as iterator from "std/iterator"
fn advance n: Int -> Maybe<(Int, Int)> = if n > 0 then Just (n, n - 1) else Nothing
pub fn countdown n: Int -> iterator.Iterator<Int> = iterator.unfold advance n
"#;
    let main = r#"
import { countdown } from "./domain"
import { next, Iterator as Cursor } from "std/iterator"
pub let cursor: Cursor<Int> = countdown 2
pub let first = next cursor
pub let values = [n | n <- cursor]
"#;
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/iterator::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/iterator::main".to_owned(),
            [("./domain".to_owned(), "fixture/iterator::domain".to_owned())],
        )
        .unwrap();
    compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "domain.ssrg",
                "fixture/iterator::domain",
                domain,
                "dist/domain.js",
            ),
            ProjectModuleInput::new("main.ssrg", "fixture/iterator::main", main, "dist/main.js"),
        ],
    )
    .expect("canonical Iterator survives named aliases and user module interfaces");
}
