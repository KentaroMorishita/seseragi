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
const ISSUE894: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/hkt-adt-pattern-issue894.ssrg.txt");
const ISSUE908: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/hkt-holder-issue908.ssrg.txt");
const OPAQUE_MODEL: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/hkt-opaque-alias-factory/src/model.ssrg"
);
const OPAQUE_FACADE: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/hkt-opaque-alias-factory/src/facade.ssrg"
);
const OPAQUE_MAIN: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/hkt-opaque-alias-factory/src/main.ssrg"
);

#[test]
fn issue894_original_pattern_source_keeps_the_checked_payload() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("issue894.ssrg", "fixture/issue894", ISSUE894).with_profile(profile),
        )
        .unwrap();
        assert!(matches!(
            compiled.typescript_ir.adts[0].variants[0].payload.as_ref(),
            Some(TypeScriptType::KindApplication { constructor, arguments })
                if constructor == "F" && arguments.len() == 1
        ));
        assert!(compiled
            .generated
            .typescript
            .contains("Box<$ssrg$builtin$kind$Maybe, number>"));
    }
}

#[test]
fn issue908_original_saved_method_keeps_the_higher_kind_field() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("issue908.ssrg", "fixture/issue908", ISSUE908).with_profile(profile),
        )
        .unwrap();
        assert!(matches!(
            &compiled.typescript_ir.structs[0].fields[0].type_ref,
            TypeScriptType::KindApplication { constructor, arguments }
                if constructor == "G" && arguments.len() == 1
        ));
        assert!(matches!(
            &compiled.core_ir.structs[0].fields[0].type_ref,
            CoreType::Named { name, arguments } if name == "G" && arguments.len() == 1
        ));
        assert!(!compiled.generated.typescript.contains("value: G<A>"));
    }
}

fn opaque_project(
    main: &str,
    profile: BuildProfile,
) -> Result<seseragi_driver::CompiledProject, seseragi_driver::ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    for (name, imports) in [
        ("model", vec![]),
        (
            "facade",
            vec![("./model".to_owned(), "fixture/opaque-hkt::model".to_owned())],
        ),
        (
            "main",
            vec![(
                "./facade".to_owned(),
                "fixture/opaque-hkt::facade".to_owned(),
            )],
        ),
    ] {
        graph
            .add_module(format!("fixture/opaque-hkt::{name}"), imports)
            .unwrap();
    }
    compile_project(
        graph,
        [
            ("model", OPAQUE_MODEL),
            ("facade", OPAQUE_FACADE),
            ("main", main),
        ]
        .map(|(name, source)| {
            ProjectModuleInput::new(
                format!("{name}.ssrg"),
                format!("fixture/opaque-hkt::{name}"),
                source,
                format!("dist/{name}.js"),
            )
            .with_profile(profile)
        }),
    )
}

#[test]
fn opaque_hkt_alias_factories_keep_canonical_owner_and_private_representation() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = opaque_project(OPAQUE_MAIN, profile).unwrap();
        let model = &compiled.modules["fixture/opaque-hkt::model"];
        let opaque = model
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "Sealed")
            .unwrap();
        assert_eq!(opaque.declaration_kind.as_deref(), Some("opaque-struct"));
        assert!(opaque.representation.is_none());
        assert!(model
            .typescript_ir
            .higher_kinds
            .as_ref()
            .unwrap()
            .constructors
            .iter()
            .any(|constructor| constructor.name == "$ssrg$kind$Sealed"
                && constructor.identity == "fixture/opaque-hkt::model::Sealed"));
        let public = model
            .generated
            .typescript
            .split("export type Sealed<")
            .nth(1)
            .unwrap()
            .split("};")
            .next()
            .unwrap();
        assert!(!public.contains("\"value\""), "{public}");
        assert!(model.generated.typescript.contains("export type Wrapped<"));
        assert!(model.generated.typescript.contains("export type Chain<"));
        for name in ["seal", "scoped", "constrained"] {
            let parameter = model
                .typescript_ir
                .functions
                .iter()
                .find_map(|function| {
                    let seseragi_lowering::TypeScriptFunction::ConstFunction {
                        name: actual,
                        parameters,
                        ..
                    } = function;
                    (actual == name).then_some(&parameters[0])
                })
                .unwrap();
            assert_eq!(parameter.type_name, "$ssrg$Apply<F, readonly [A]>");
        }
        let returned = model
            .typescript_ir
            .functions
            .iter()
            .find_map(|function| {
                let seseragi_lowering::TypeScriptFunction::ConstFunction { name, body, .. } =
                    function;
                (name == "unseal").then_some(body)
            })
            .unwrap();
        assert!(matches!(
            returned,
            seseragi_lowering::TypeScriptExpr::CheckedResult {
                type_ref: TypeScriptType::KindApplication { constructor, arguments },
                ..
            } if constructor == "F" && arguments.len() == 1
        ));
        let constrained = model
            .core_ir
            .functions
            .iter()
            .find(|function| function.symbol.ends_with("::constrained"))
            .unwrap();
        assert_eq!(constrained.parameters.len(), 1);
        assert_eq!(constrained.constraints.len(), 1);
        let facade = &compiled.modules["fixture/opaque-hkt::facade"].typescript_ir;
        for (original, renamed) in [("Wrapped", "PublicWrapped"), ("Chain", "PublicChain")] {
            assert!(facade
                .source_imports
                .iter()
                .flat_map(|import| &import.reexports)
                .any(|binding| binding.type_only
                    && binding.imported == format!("$ssrg$kind${original}")
                    && binding.local == format!("$ssrg$kind${renamed}")
                    && binding.canonical == format!("fixture/opaque-hkt::model::{original}")));
        }
    }
}

#[test]
fn opaque_hkt_aliases_reject_external_fields_forgery_and_changed_payload_or_owner() {
    let prefix =
        "import { Kept, PublicChain } from \"./facade\"\nstruct Sealed<F<_>, A> { value: F<A> }\n";
    for (body, key) in [
        (
            "pub fn leak value: PublicChain<Maybe, Int> -> Maybe<Int> = value.value\n",
            "record.field-unresolved",
        ),
        (
            "pub fn forge value: Maybe<Int> -> PublicChain<Maybe, Int> = Kept { value }\n",
            "struct.representation-private",
        ),
        (
            "pub fn wrong value: PublicChain<Maybe, Int> -> PublicChain<Maybe, String> = value\n",
            "function.return-type-mismatch",
        ),
        (
            "pub fn wrong value: Sealed<Maybe, Int> -> PublicChain<Maybe, Int> = value\n",
            "function.return-type-mismatch",
        ),
    ] {
        let source = format!("{prefix}{body}");
        let error = opaque_project(&source, BuildProfile::Development).unwrap_err();
        let seseragi_driver::ProjectCompileError::Diagnostics { modules } = &error else {
            panic!("{source}\n{error:#?}");
        };
        let diagnostic = modules
            .iter()
            .filter(|module| module.module == "fixture/opaque-hkt::main")
            .flat_map(|module| &module.diagnostics.diagnostics)
            .find(|diagnostic| diagnostic.message_key == key)
            .unwrap_or_else(|| panic!("{source}\n{error:#?}"));
        assert!(diagnostic.primary.start < diagnostic.primary.end);
        assert!(diagnostic.primary.end <= source.len());
    }
}

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
