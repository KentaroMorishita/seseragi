use seseragi_driver::{compile_module, CompileInput};

const TRAVERSABLE: &str = r#"
type Box<A> = | Box A
instance Functor<Box> {
  fn map<A, B> f: (A -> B) -> value: Box<A> -> Box<B> = match value {
    Box item -> Box (f item)
  }
}
instance Traversable<Box> {
  fn traverse<G<_>, A, B> f: (A -> G<B>) -> value: Box<A> -> G<Box<B>>
  where Applicative<G> = match value {
    Box item -> map Box (f item)
  }
}
pub effect fn main = println "ok"
"#;

#[test]
fn erases_higher_kinded_parameters_in_instance_match_bodies() {
    let compiled = compile_module(CompileInput::new("hkt.ssrg", "fixture/hkt", TRAVERSABLE))
        .expect("the source evidence contract is valid before backend erasure");
    let typescript = compiled.generated.typescript;
    assert!(!typescript.contains("G<"), "{typescript}");
    assert!(
        typescript.contains("Box<B>"),
        "concrete generic Box remains typed"
    );
}

#[test]
fn carries_lexical_erasure_through_local_functions_lambdas_and_bindings() {
    let source = r#"
type F<A> = | F A
pub fn scoped<F<_>, A> value: F<A> -> F<A> where Functor<F> = {
  let retained: F<A> = value
  fn local item: F<A> -> F<A> = match True {
    True -> if True then item else retained
    False -> retained
  }
  let identity: F<A> -> F<A> = \item -> item
  identity (local retained)
}
pub fn concrete value: F<Int> -> F<Int> = match value {
  F item -> F (item + 1)
}
"#;
    let compiled = compile_module(CompileInput::new("scoped.ssrg", "fixture/scoped", source))
        .expect(
            "constructor parameters may shadow module types while nested functions inherit scope",
        );
    let ts = compiled.generated.typescript;
    let scoped = ts
        .split("export const scoped")
        .nth(1)
        .unwrap()
        .split("export const concrete")
        .next()
        .unwrap();
    assert!(!scoped.contains("F<") && !scoped.contains("G<"), "{scoped}");
    assert!(
        ts.contains("F<number>"),
        "the concrete nominal outside that scope is preserved: {ts}"
    );
}

#[test]
fn imported_hkt_callers_keep_concrete_type_arguments() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/hkt::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/hkt::main".to_owned(),
            [("./domain".to_owned(), "fixture/hkt::domain".to_owned())],
        )
        .unwrap();
    let compiled = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "domain.ssrg",
                "fixture/hkt::domain",
                include_str!(
                    "../../../examples/spec/fixtures/projects/hkt-erasure/src/domain.ssrg"
                ),
                "dist/domain.js",
            ),
            ProjectModuleInput::new(
                "main.ssrg",
                "fixture/hkt::main",
                include_str!("../../../examples/spec/fixtures/projects/hkt-erasure/src/main.ssrg"),
                "dist/main.js",
            ),
        ],
    )
    .expect("user Traversable, Applicative and Monad evidence survives the module boundary");
    let domain = &compiled.modules["fixture/hkt::domain"].generated.typescript;
    for constructor in ["F<", "G<", "M<"] {
        assert!(!domain.contains(constructor), "{domain}");
    }
    assert!(domain.contains("Box<A>"));
    assert!(compiled.modules["fixture/hkt::main"]
        .generated
        .typescript
        .contains("Box<number>"));
}
