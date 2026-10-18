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

#[test]
fn keeps_lexical_hkt_identity_through_a_generic_local_call() {
    let source = r#"
type F<A> = | F A
pub fn scoped<F<_>, A> value: F<A> -> F<A> = {
  fn local<G<_>, B> item: G<B> -> G<B> = item
  local value
}
pub fn concrete value: F<Int> -> F<Int> = value
"#;
    compile_module(CompileInput::new("scoped.ssrg", "fixture/scoped", source))
        .expect("local generic calls preserve the outer constructor identity");
}

#[test]
fn local_hkt_annotations_share_their_lexical_binder() {
    for body in [
        "{ let kept: G<B> = item; kept }",
        "match True { True -> item; False -> item }",
        "if True then item else item",
    ] {
        let source = format!(
            "pub fn outer<F<_>, A> value: F<A> -> F<A> where Functor<F> = {{\n\
             fn nested<G<_>, B> item: G<B> -> G<B> where Functor<G> = {body}\n\
             let kept: F<A> = nested value\nkept\n}}"
        );
        compile_module(CompileInput::new("local.ssrg", "fixture/local", &source))
            .expect("local binders and captured outer binders keep distinct identities");
    }
}

#[test]
fn local_hkt_annotations_do_not_accept_mismatches_or_shadowing() {
    for source in [
        "pub fn outer unit: Unit -> Int = { fn nested<G<_>, B> item: G<B> -> G<B> = { let kept: Int = item; item }; 1 }",
        "pub fn outer<F<_>, A> value: F<A> -> F<A> = { fn nested<F<_>, B> item: F<B> -> F<B> = item; value }",
    ] {
        assert!(compile_module(CompileInput::new("invalid.ssrg", "fixture/invalid", source)).is_err());
    }
}

#[test]
fn projects_checked_hkt_calls_without_casting_ordinary_generics() {
    use seseragi_lowering::{CoreExpr, TypeScriptExpr, TypeScriptFunction, TypeScriptType};
    let compiled = compile_module(CompileInput::new(
        "callee.ssrg",
        "fixture/callee",
        r#"
type F<A> = | Nominal A
fn keep<F<_>, A> value: F<A> -> F<A> = value
fn identity<A> value: A -> A = value
pub fn wrapped value: F<Int> -> F<Int> = keep value
pub fn ordinary value: Int -> Int = identity value
pub fn saved -> (F<Int> -> F<Int>) = keep
"#,
    ))
    .unwrap();
    for function in &compiled.core_ir.functions {
        if function.symbol.ends_with("::wrapped") {
            let CoreExpr::Call {
                callee_signature: Some(signature),
                arguments,
                ..
            } = &function.body
            else {
                panic!("{function:?}")
            };
            assert_eq!(arguments.len(), 1);
            assert_eq!(signature.type_parameters[0].arity, 1);
            assert_eq!(signature.parameters.len(), 1);
        } else if function.symbol.ends_with("::saved") {
            assert!(
                matches!(
                    &function.body,
                    CoreExpr::Variable {
                        callable_signature: Some(_),
                        evidence,
                        ..
                    } if evidence.is_empty()
                ),
                "a saved reference remains a value, without calling the function"
            );
        } else if function.symbol.ends_with("::ordinary") {
            assert!(matches!(
                &function.body,
                CoreExpr::Call {
                    callee_signature: None,
                    ..
                }
            ));
        }
    }
    for TypeScriptFunction::ConstFunction { name, body, .. } in &compiled.typescript_ir.functions {
        if name == "wrapped" {
            let TypeScriptExpr::Call {
                checked_callee_type: Some(TypeScriptType::Function { parameter, result }),
                ..
            } = body
            else {
                panic!("{body:?}")
            };
            assert_eq!(parameter, result);
            assert!(
                matches!(parameter.as_ref(), TypeScriptType::Reference { name, arguments } if name == "F" && arguments == &[TypeScriptType::Number])
            );
        } else if name == "saved" {
            assert!(matches!(
                body,
                TypeScriptExpr::CheckedResult { value, type_ref: TypeScriptType::Function { .. } }
                    if matches!(value.as_ref(), TypeScriptExpr::Identifier { .. })
            ));
        } else if name == "ordinary" {
            assert!(matches!(
                body,
                TypeScriptExpr::Call {
                    checked_callee_type: None,
                    ..
                }
            ));
        }
    }
}
