use seseragi_driver::{
    compile_module, compile_project, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_lowering::{CoreType, TypeScriptType};
use seseragi_project::ModuleGraph;

const MODEL: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/hkt-data-projection/src/model.ssrg"
);
const FACADE: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/hkt-data-projection/src/facade.ssrg"
);
const MAIN: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/hkt-data-projection/src/main.ssrg"
);

#[test]
fn constructor_calls_retain_declared_kind_and_checked_source_arity() {
    let compiled = compile_module(CompileInput::new("model.ssrg", "fixture/model", MODEL)).unwrap();
    let typed = compiled
        .typed_hir
        .declarations
        .iter()
        .find_map(|declaration| {
            let seseragi_semantics::TypedDecl::Fn { symbol, body, .. } = declaration else {
                return None;
            };
            (symbol == "fixture/model::cold").then_some(body)
        })
        .unwrap();
    let seseragi_semantics::TypedExpr::Call {
        callee_signature: Some(typed_signature),
        ..
    } = typed
    else {
        panic!("{typed:?}");
    };
    assert_eq!(typed_signature.type_parameters[0].arity, 1);
    let cold = compiled
        .core_ir
        .functions
        .iter()
        .find(|function| function.symbol.ends_with("::cold"))
        .unwrap();
    let seseragi_lowering::CoreExpr::Call {
        callee_signature: Some(signature),
        arguments,
        ..
    } = &cold.body
    else {
        panic!("{:?}", cold.body);
    };
    assert_eq!(signature.type_parameters[0].arity, 1, "{signature:#?}");
    assert_eq!(signature.type_parameters, typed_signature.type_parameters);
    assert_eq!(signature.parameters.len(), 1, "{signature:#?}");
    assert_eq!(arguments.len(), 1);
}

#[test]
fn concrete_data_retains_the_checked_kind_application_in_both_profiles() {
    let source = "type Box<F<_>, A> = | Box F<A>\npub struct Holder<G<_>, A> { value: G<A> }\npub effect fn main = do { let boxed: Box<Maybe, Int> = Box (Just 42); println (match boxed { Box value -> show value }); let holder: Holder<Maybe, String> = Holder { value: Just \"kept\" }; println (show holder.value) }";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("data.ssrg", "fixture/data", source).with_profile(profile),
        )
        .unwrap();
        let payload = compiled.typescript_ir.adts[0].variants[0]
            .payload
            .as_ref()
            .unwrap();
        assert!(
            matches!(payload, TypeScriptType::KindApplication { constructor, arguments } if constructor == "F" && arguments.len() == 1)
        );
        let field = &compiled.typescript_ir.structs[0].fields[0].type_ref;
        assert!(
            matches!(field, TypeScriptType::KindApplication { constructor, arguments } if constructor == "G" && arguments.len() == 1)
        );
        assert!(compiled
            .generated
            .typescript
            .contains("Box<$ssrg$builtin$kind$Maybe, number>"));
        assert!(!compiled.generated.typescript.contains("value: G<A>"));
        assert!(
            matches!(&compiled.core_ir.structs[0].fields[0].type_ref, CoreType::Named { name, arguments } if name == "G" && arguments.len() == 1)
        );
    }
}

#[test]
fn renamed_facades_keep_canonical_kind_companions_and_type_only_edges() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let mut graph = ModuleGraph::new();
        for (id, edges) in [
            ("fixture/data::model", vec![]),
            (
                "fixture/data::facade",
                vec![("./model".to_owned(), "fixture/data::model".to_owned())],
            ),
            (
                "fixture/data::main",
                vec![("./facade".to_owned(), "fixture/data::facade".to_owned())],
            ),
        ] {
            graph.add_module(id.to_owned(), edges).unwrap();
        }
        let inputs =
            [("model", MODEL), ("facade", FACADE), ("main", MAIN)].map(|(name, source)| {
                ProjectModuleInput::new(
                    format!("{name}.ssrg"),
                    format!("fixture/data::{name}"),
                if name == "main" { format!("{source}\npub fn verify value: Maybe<Int> -> Pack<Maybe, Int> = Parcel value\npub fn save -> (Maybe<Int> -> Pack<Maybe, Int>) = Parcel\n") } else { source.to_owned() },
                    format!("dist/{name}.js"),
                )
                .with_profile(profile)
            });
        let compiled = compile_project(graph, inputs).unwrap();
        let model = &compiled.modules["fixture/data::model"];
        assert!(model
            .generated
            .typescript
            .contains("export type Wrapped<F extends"));
        assert!(model
            .typescript_ir
            .higher_kinds
            .as_ref()
            .unwrap()
            .constructors
            .iter()
            .any(|constructor| constructor.name == "$ssrg$kind$Box"
                && constructor.identity == "fixture/data::model::Box"));
        let facade = &compiled.modules["fixture/data::facade"].typescript_ir;
        assert!(facade
            .source_imports
            .iter()
            .flat_map(|import| &import.reexports)
            .any(|binding| binding.type_only
                && binding.imported == "$ssrg$kind$Wrapped"
                && binding.local == "$ssrg$kind$Pack"
                && binding.canonical == "fixture/data::model::Wrapped"));
        let main = &compiled.modules["fixture/data::main"].generated.typescript;
        assert!(main.contains("$ssrg$Partial$2$10<$ssrg$builtin$kind$Either, string>"));
        let core = &compiled.modules["fixture/data::main"].core_ir;
        let body = &core
            .functions
            .iter()
            .find(|function| function.symbol.ends_with("::verify"))
            .unwrap()
            .body;
        let seseragi_lowering::CoreExpr::Call {
            callee,
            callee_signature: Some(signature),
            arguments,
            ..
        } = body
        else {
            panic!("{body:?}");
        };
        assert_eq!(callee, "fixture/data::model::Box");
        assert_eq!(signature.type_parameters[0].arity, 1, "{signature:?}");
        assert_eq!(signature.parameters.len(), 1);
        assert_eq!(arguments.len(), 1);
        let body = &core
            .functions
            .iter()
            .find(|function| function.symbol.ends_with("::save"))
            .unwrap()
            .body;
        assert!(
            matches!(body, seseragi_lowering::CoreExpr::Variable { callable_signature: Some(signature), .. } if signature.type_parameters[0].arity == 1),
            "{body:?}"
        );
    }
}

#[test]
fn wrong_kind_and_distinct_nominal_owner_still_report_source_locations() {
    for source in [
        "type Box<F<_>, A> = | Box F<A>\npub fn wrong value: Box<Int, Int> -> Int = 0",
        "struct Holder<F<_>, A> { value: F<A> }\npub fn wrong value: Holder<Either, Int> -> Int = 0",
        "type First<A> = | First A\ntype Second<A> = | Second A\ntype Box<F<_>, A> = | Box F<A>\npub fn wrong value: Box<First, Int> -> Box<Second, Int> = value",
    ] {
        let error = compile_module(CompileInput::new("invalid-data.ssrg", "fixture/invalid-data", source)).expect_err("kind and canonical owner checks remain in the shared frontend");
        assert!(!error.diagnostics.is_empty(), "{error:?}");
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.primary.start < diagnostic.primary.end && diagnostic.primary.end <= source.len()), "{error:?}");
    }
}
