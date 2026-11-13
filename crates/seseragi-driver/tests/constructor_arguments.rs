//! A constructor application passed as an argument, such as `Right 7`, takes
//! the type arguments its own arguments leave open from the other arguments
//! of the call.

use seseragi_driver::{compile_module, CompileInput};

fn diagnostics(source: &str) -> Vec<String> {
    match compile_module(CompileInput::new(
        "constructor-arguments.ssrg",
        "fixture/main",
        source,
    )) {
        Ok(compiled) => compiled
            .diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect(),
        Err(artifact) => artifact
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{} at {:?}", diagnostic.code, diagnostic.primary))
            .collect(),
    }
}

#[test]
fn a_bound_effect_takes_a_constructor_argument_typed_by_a_later_argument() {
    for argument in ["(Right 7)", "(Left \"late\")", "(Right (3 + 4))"] {
        let source = format!(
            "import * as deferreds from \"std/deferred\"\n\npub effect fn main = do {{\n  gate <- deferreds.make<String, Int> ()\n  completed <- deferreds.complete {argument} gate\n  println (show completed)\n}}\n"
        );
        assert_eq!(diagnostics(&source), Vec::<String>::new(), "{argument}");
    }
}

#[test]
fn a_pure_call_takes_a_constructor_argument_typed_by_a_later_argument() {
    let source = "import { Deferred } from \"std/deferred\"\nimport * as deferreds from \"std/deferred\"\n\nfn take<E, A> value: Either<E, A> -> gate: Deferred<E, A> -> Int = 1\nfn pair<E, A> value: Either<E, A> -> other: Either<E, A> -> Int = 2\nfn holds<A> value: Maybe<A> -> other: A -> Int = 3\n\npub effect fn main = do {\n  gate <- deferreds.make<String, Int> ()\n  let left: Either<String, Int> = Left \"x\"\n  println (show (take (Right 7) gate))\n  println (show (pair (Right 7) left))\n  println (show (holds Nothing \"a\"))\n}\n";
    assert_eq!(diagnostics(source), Vec::<String>::new());
}

#[test]
fn a_constructor_argument_that_conflicts_with_a_later_argument_is_still_refused() {
    let source = "import * as deferreds from \"std/deferred\"\n\npub effect fn main = do {\n  gate <- deferreds.make<String, Int> ()\n  completed <- deferreds.complete (Right \"text\") gate\n  println (show completed)\n}\n";
    assert!(!diagnostics(source).is_empty());
}

#[test]
fn an_earlier_constructor_argument_takes_its_open_type_from_a_later_argument() {
    let source = "fn same<A> expected: A -> actual: A -> Bool where Eq<A> = expected == actual\nfn pick<T> first: T -> second: T -> T = first\nfn parse text: String -> Either<String, Int> = if text == \"\" then Left \"empty\" else Right 1\nfn flip text: String -> Either<Int, String> = Right text\n\npub effect fn main = do {\n  println (show (same (Right 1) (parse \"x\")))\n  println (show (same (Left \"empty\") (parse \"\")))\n  println (show (pick (Left 1) (flip \"x\")))\n}\n";
    assert_eq!(diagnostics(source), Vec::<String>::new());
}

#[test]
fn an_earlier_constructor_argument_that_conflicts_with_a_later_argument_is_still_refused() {
    let source = "fn same<A> expected: A -> actual: A -> Bool where Eq<A> = expected == actual\nfn parse text: String -> Either<String, Int> = Right 1\n\npub effect fn main = do {\n  println (show (same (Right \"text\") (parse \"x\")))\n}\n";
    assert!(!diagnostics(source).is_empty());
}
