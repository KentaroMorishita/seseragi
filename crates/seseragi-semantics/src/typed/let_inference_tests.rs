use super::type_module;
use crate::{semantic_diagnostics, TypedDecl};

fn accepts(source: &str) {
    let diagnostics = semantic_diagnostics("artifact/let-inference/main.ssrg", source);
    assert!(
        diagnostics.diagnostics.is_empty(),
        "{source}\n{:#?}",
        diagnostics.diagnostics
    );
}
fn rejects(source: &str) {
    let diagnostics = semantic_diagnostics("artifact/let-inference/main.ssrg", source);
    assert!(
        !diagnostics.diagnostics.is_empty(),
        "unexpected acceptance: {source}"
    );
}
#[test]
fn let_inference_spec_identity() {
    let source = "let identity = \\value -> value\nlet number = identity 42\nlet text = identity \"hello\"\npub effect fn main = println text\n";
    accepts(source);
    let typed = type_module("artifact/let-inference/main.ssrg", source);
    let TypedDecl::Let { scheme, .. } = &typed.declarations[0] else {
        panic!()
    };
    assert_eq!(scheme.type_parameters.len(), 1);
}
#[test]
fn let_inference_solves_composition_and_distinct_parameters() {
    accepts("let compose = \\f -> \\g -> \\x -> f (g x)\nlet keep = \\x -> \\y -> x\nlet identity = \\x -> x\nlet number: Int = compose identity identity 42\nlet text: String = compose identity identity \"hello\"\nlet kept: String = keep \"hello\" 42\n");
}
#[test]
fn let_inference_block_and_captured_environment() {
    accepts("fn use<A> captured: A -> A = {\nlet identity = \\x -> x\nlet keep = \\x -> captured\nlet a: Int = identity 42\nlet b: String = identity \"hello\"\nkeep b\n}\n");
    rejects("fn bad<A> captured: A -> Int = {\nlet keep = \\x -> captured\nkeep \"hello\"\n}\n");
}
#[test]
fn let_inference_preserves_rank_one_and_occurs_check() {
    rejects("let invalid = \\f -> (f 42, f \"hello\")\n");
    rejects("let invalid = \\f -> f f\n");
    rejects("let invalid = \\f -> {\nlet alias = \\x -> f x\nlet number = alias 42\nalias \"hello\"\n}\n");
}
#[test]
fn let_inference_does_not_drop_traits() {
    rejects("let invalid = \\value -> show value\nlet noInstance = invalid (\\x: Int -> x)\n");
}

#[test]
fn let_inference_generalizes_trait_constraints() {
    accepts("let render = \\value -> show value\nlet number: String = render 42\nlet text: String = render \"hello\"\n");
    accepts("let double = \\value -> value + value\nlet number: Int = double 21\nlet text: String = double \"hello\"\n");
}
#[test]
fn let_inference_captures_trait_evidence_without_generalizing_environment() {
    accepts("fn renderCaptured<A> captured: A -> String where Show<A> = {\nlet render = \\value -> (show captured, show value)\nlet (first, second): (String, String) = render 42\nfirst\n}\n");
}
#[test]
fn let_inference_supports_structural_results_and_do_lets() {
    accepts("let pair = \\value -> (value, value)\nlet record = \\value -> {item: value}\nlet ints: (Int, Int) = pair 42\nlet strings: (String, String) = pair \"hello\"\nlet item: {item: Int} = record 42\npub effect fn main = do {\nlet identity = \\value -> value\nlet a: Int = identity 42\nprintln (identity \"hello\")\n}\n");
}

#[test]
fn let_inference_solves_callbacks_in_all_argument_orders() {
    accepts("fn compose<A, B, C> f: (B -> C) -> g: (A -> B) -> x: A -> C = f (g x)\nfn reversed<A, B, C> x: A -> g: (A -> B) -> f: (B -> C) -> C = f (g x)\nlet identity = \\x -> x\nlet a = compose identity identity \"hello\"\nlet b = reversed \"hello\" identity identity\npub effect fn main = do { println a; println b }\n");
    rejects("let bad = \\f -> (f \"hello\", f 42)\n");
    rejects("let bad = \\f -> { let captured = \\x -> f x; let text = captured \"hello\"; captured 42 }\n");
}

#[test]
fn let_inference_preserves_monomorphic_recursive_uses() {
    accepts("fn result -> Int = { rec { fn same<A> x: A -> A = { let again = same; again x } }; same 42 }\n");
    rejects("fn result -> Int = { rec { fn same<A> x: A -> A = { let again = same; again [x] } }; same 42 }\n");

    accepts("fn result -> Int = { rec { fn same<A> x: A -> A = { let again = \\y -> same y; again x } }; same 42 }\n");
    rejects("fn result -> Int = { rec { fn same<A> x: A -> A = { let again = \\y -> same y; again [x] } }; same 42 }\n");
}

#[test]
fn let_inference_supports_monadic_let_schemes() {
    accepts("fn result -> Maybe<String> = do { let identity = \\x -> x; let number = identity 42; Just (identity \"hello\") }\n");
    accepts("fn result -> Maybe<String> = do { let render = \\x -> show x; let number = render 42; Just (render \"hello\") }\n");
}

#[test]
fn let_inference_supports_discard_and_nested_scheme_captures() {
    accepts("let keep = \\_ -> \\value -> value\nlet a: Int = keep \"ignored\" 42\nlet b: String = keep 42 \"kept\"\n");
    accepts("fn result -> String = { let identity = \\x -> x; let alias = identity; let invoke = \\x -> alias x; let a = invoke 42; invoke \"hello\" }\n");
    accepts("let outer = \\captured -> { let render = \\x -> show x; (render captured, render 42) }\nlet pair: (String, String) = outer \"hello\"\n");
}

#[test]
fn let_inference_preserves_environment_constraints_of_unused_bindings() {
    accepts(
        "let keep = \\value -> { let rendered = show value; value }\nlet answer: Int = keep 42\n",
    );
    accepts("let keep = \\captured -> { let render = \\x -> show captured; captured }\nlet answer: Int = keep 42\n");
    rejects("let keep = \\value -> { let rendered = show value; value }\nlet noInstance = keep (\\x: Int -> x)\n");
}

#[test]
fn let_inference_preserves_constrained_callable_aliases() {
    accepts("fn render<A> x: A -> String where Show<A> = show x\nlet aliasValue = render\nlet again = aliasValue\nlet number: String = again 42\nlet text: String = aliasValue \"hello\"\n");
    accepts("let render = \\x -> show x\nlet aliasValue = render\nlet number: String = aliasValue 42\nlet text: String = aliasValue \"hello\"\n");
    accepts("fn render<A> x: A -> String where Show<A> = show x\nfn use -> String = { let aliasValue = render; let number = aliasValue 42; aliasValue \"hello\" }\n");
    accepts("fn render<A> x: A -> String where Show<A> = show x\nfn use -> Maybe<String> = do { let aliasValue = render; let number = aliasValue 42; Just (aliasValue \"hello\") }\n");
    accepts("fn use<A> captured: A -> String where Show<A> = { let render = \\x -> show captured; let aliasValue = render; aliasValue 42 }\n");
    rejects("fn render<A> x: A -> String where Show<A> = show x\nlet aliasValue = render\nlet invalid = aliasValue (\\x: Int -> x)\n");
}
