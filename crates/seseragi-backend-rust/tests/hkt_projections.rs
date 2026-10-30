use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};

#[test]
fn canonical_native_hkt_supertraits_keep_the_same_scoped_constructor_frame() {
    let source="fn mapped<F<_>,A,B> callback:(A -> B) -> value:F<A> -> F<B> where Applicative<F> = map callback value\nfn point<F<_>,A> value:A -> F<A> where Applicative<F> = pure value\nfn inherited<M<_>,A,B> callback:(A -> B) -> value:M<A> -> M<B> where Monad<M> = map callback value\nfn traversed<F<_>,A,B> callback:(A -> B) -> value:F<A> -> F<B> where Traversable<F> = map callback value\npub fn probe -> Int = 0\n";
    let core = compile_core_module(CompileInput::new(
        "hkt-projection-雪🙂.ssrg",
        "native/hkt-projection",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let error =
        emit_program_wasm(&[&core], "native/hkt-projection", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
}

#[test]
fn projected_hkt_parameters_refuse_forged_trait_constructor_kind_and_scope() {
    for declaration in [
        "fn mapped<F<_>,A,B> callback:(A -> B) -> value:F<A> -> F<B> where Applicative<F> = map callback value",
        "fn point<F<_>,A> value:A -> F<A> where Applicative<F> = pure value",
        "fn mapped<F<_>,A,B> callback:(A -> B) -> value:F<A> -> F<B> where Monad<F> = map callback value",
        "fn mapped<F<_>,A,B> callback:(A -> B) -> value:F<A> -> F<B> where Traversable<F> = map callback value",
    ] {
        let source=format!("{declaration}\npub fn probe -> Int = 0\n");
        let core=compile_core_module(CompileInput::new("hkt-雪🙂.ssrg","native/hkt",&source)).unwrap().core_ir;
        emit_rust_module(&core,"probe").unwrap();
        let index=core.functions.iter().position(|f|!f.symbol.ends_with("::probe")).unwrap();
        for mutation in 0..7 {
            let mut bad=core.clone();
            let function=&mut bad.functions[index];
            let constructor=function.type_parameters.iter().position(|p|p.name=="F").unwrap();
            let seseragi_lowering::CoreExpr::Call {evidence,..}=&mut function.body else {panic!()};
            match mutation {
                0=>function.constraints[0].trait_identity=Some("fake::Applicative".into()),
                1=>evidence[0].constraint.trait_identity=Some("fake::Functor".into()),
                2=>function.type_parameters[constructor].arity=0,
                3=>function.type_parameters[constructor].arity=2,
                4=>evidence[0].constraint.arguments[0]=seseragi_lowering::CoreType::Named {name:"std/prelude::Array".into(),arguments:vec![]},
                5=>evidence[0].evidence=seseragi_lowering::CoreInstanceEvidence::Parameter {index:999},
                6=>evidence[0].constraint.name="Fake".into(),
                _=>unreachable!(),
            }
            assert!(emit_rust_module(&bad,"probe").is_err(),"{declaration}/{mutation}");
        }
    }
}
