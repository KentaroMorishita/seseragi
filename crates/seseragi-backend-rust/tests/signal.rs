use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};

#[test]
fn official_signal_samples_emit_native_rust() {
    for (name, source) in [
        (
            "signal-state",
            include_str!("../../../examples/samples/signal-state/main.ssrg"),
        ),
        (
            "signal-composition",
            include_str!("../../../examples/samples/signal-composition/main.ssrg"),
        ),
        (
            "type-aliases",
            include_str!("../../../examples/samples/type-aliases/main.ssrg"),
        ),
    ] {
        let core =
            compile_core_module(CompileInput::new("main.ssrg", "native/signal", source)).unwrap();
        let result = emit_rust_module(&core.core_ir, "main");
        assert!(result.is_ok(), "{name}: {result:?}");
    }
}

#[test]
fn shared_signal_fixtures_emit_native_rust() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend");
    for name in [
        "signal-transactions",
        "signal-observer-queue",
        "signal-switch-distinct",
        "signal-many-subscribers",
        "signal-scopes",
    ] {
        let source = std::fs::read_to_string(root.join(format!("{name}.ssrg"))).unwrap();
        let core = compile_core_module(CompileInput::new("main.ssrg", "native/signal", &source))
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        emit_rust_module(&core.core_ir, "main").unwrap_or_else(|error| panic!("{name}: {error:?}"));
    }
}

#[test]
fn signal_operations_reject_forged_types_evidence_and_source_stages() {
    use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};
    for declaration in [
        "pub fn probe -> Task<signals.MutableSignal<Int>> = signals.make 1",
        "pub fn probe source:signals.Signal<Int> -> Task<Int> = signals.read source",
        "pub fn probe source:signals.MutableSignal<Int> -> Task<Unit> = signals.set 1 source",
        "pub fn probe source:signals.MutableSignal<Int> -> Task<Unit> = signals.update (\\n:Int -> n + 1) source",
        "pub fn probe source:signals.MutableSignal<Int> -> signals.SignalChange = signals.planSet 1 source",
        "pub fn probe source:signals.MutableSignal<Int> -> signals.SignalChange = signals.planUpdate (\\n:Int -> n + 1) source",
        "pub fn probe -> Task<Unit> = signals.transaction []",
        "pub fn probe -> signals.Signal<Int> = signals.constant 1",
        "pub fn probe source:signals.Signal<Int> -> signals.Signal<Int> = signals.map (\\n:Int -> n + 1) source",
        "pub fn probe source:signals.Signal<Int> -> signals.Signal<Int> = signals.combine (+) source source",
        "pub fn probe source:signals.Signal<Int> -> signals.Signal<Int> = signals.distinct source",
        "pub fn probe source:signals.Signal<Int> -> signals.Signal<Int> = signals.switchMap signals.constant source",
        "pub fn probe source:signals.Signal<Int> -> Task<signals.Subscription> = signals.subscribe (\\_n:Int -> effects.succeed ()) source",
        "pub fn probe subscription:signals.Subscription -> Task<Unit> = signals.unsubscribe subscription",
    ] {
        let source = format!("import * as signals from \"std/signal\"\nimport * as effects from \"std/effect\"\n{declaration}\npub fn main -> Int = 0\n");
        let core = compile_core_module(CompileInput::new("signal.ssrg", "native/signal", &source)).unwrap_or_else(|e| panic!("{source}: {e:?}")).core_ir;
        emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{source}: {e:?}"));
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..6 {
            let mut bad = core.clone();
            let CoreExpr::Call { type_ref, arguments, deferred_evidence_parameters, evidence_argument_index, evidence, .. } = &mut bad.functions[0].body else { panic!("{declaration}") };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => *type_ref = CoreType::ExternalNamed { name:"Signal".into(), canonical:"fake::Signal".into(), arguments:vec![] },
                2 => deferred_evidence_parameters.push(CoreType::Hole),
                3 => *evidence_argument_index = Some(0),
                4 => arguments.push(CoreExpr::Unit { origin:origin.clone() }),
                _ => {
                    if evidence.is_empty() { evidence.push(compile_core_module(CompileInput::new("eq.ssrg", "native/eq", "pub fn probe -> Bool = 1 == 1")).map(|c| match c.core_ir.functions[0].body.clone() { CoreExpr::Binary { evidence, .. } => evidence[0].clone(), _ => panic!() }).unwrap()); }
                    else { evidence[0].constraint.arguments[0] = CoreType::Hole; }
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin, "{declaration}/{mutation}");
        }
        let error = seseragi_backend_rust::emit_program_wasm(&[&core], "native/signal", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
    }
}

#[test]
fn signal_instance_dispatch_rejects_forged_traits_methods_and_result_types() {
    use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};
    for declaration in [
        "pub fn probe -> signals.Signal<Int> = map (\\n:Int -> n + 1) (signals.constant 1)",
        "pub fn probe -> signals.Signal<Int> = apply (signals.constant (\\n:Int -> n + 1)) (signals.constant 1)",
        "pub fn probe -> signals.Signal<Int> = pure 1",
    ] {
        let source = format!("import * as signals from \"std/signal\"\n{declaration}\npub fn main -> Int = 0\n");
        let core = compile_core_module(CompileInput::new("signal.ssrg", "native/signal", &source)).unwrap().core_ir;
        emit_rust_module(&core, "main").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..7 {
            let mut bad = core.clone();
            let CoreExpr::Call { trait_dispatch, type_ref, evidence, .. } = &mut bad.functions[0].body else { panic!() };
            let dispatch = trait_dispatch.as_mut().unwrap();
            match mutation {
                0 => dispatch.trait_identity = "fake::Functor".into(),
                1 => dispatch.method = "flatMap".into(),
                2 => *type_ref = CoreType::Hole,
                3 => evidence[0].constraint.trait_identity = Some("fake::Functor".into()),
                4 => evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {name:"Signal".into(), canonical:"fake::Signal".into(), arguments:vec![]},
                5 => {
                    let CoreInstanceEvidence::Standard { type_arguments, .. } = &mut evidence[0].evidence else { panic!() };
                    type_arguments.push(CoreType::Hole);
                }
                _ => {
                    let child = evidence[0].clone();
                    let CoreInstanceEvidence::Standard { evidence_arguments, .. } = &mut evidence[0].evidence else { panic!() };
                    evidence_arguments.push(child);
                }
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin);
        }
    }
}

#[test]
fn signal_generic_wrappers_preserve_scoped_types_and_equality_evidence() {
    let source = r#"
import * as signals from "std/signal"
alias View<A> = signals.Signal<A>
fn current<A> value:View<A> -> Task<A> = signals.read value
fn distinct<A> value:View<A> -> View<A> where Eq<A> = signals.distinct value
fn changed<A> mapper:(A -> A) -> value:signals.MutableSignal<A> -> Task<Unit> = signals.update mapper value
fn switched<A,B> mapper:(A -> View<B>) -> value:View<A> -> View<B> = signals.switchMap mapper value
fn watched<R,A> observer:(A -> Effect<R,Never,Unit>) -> value:View<A> -> Effect<R,Never,signals.Subscription> = signals.subscribe observer value
pub fn main -> Int = 0
"#;
    let core =
        compile_core_module(CompileInput::new("signal.ssrg", "native/signal", source)).unwrap();
    emit_rust_module(&core.core_ir, "main").unwrap();
}

#[test]
fn switch_map_accepts_existing_mutable_inner_signals() {
    let source = r#"
import * as signals from "std/signal"
pub effect fn main = do {
  left <- signals.make 1
  right <- signals.make 2
  flag <- signals.make False
  let selected = signals.switchMap (\value:Bool -> if value then right else left) flag
  current <- *selected
  println (show current)
}
"#;
    let core =
        compile_core_module(CompileInput::new("signal.ssrg", "native/signal", source)).unwrap();
    emit_rust_module(&core.core_ir, "main").unwrap();
}
