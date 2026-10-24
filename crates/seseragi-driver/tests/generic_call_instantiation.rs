use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_lowering::{CoreExpr, CoreType};

const EFFECT_FACTORY: &str = r#"import * as deferred from "std/deferred"
effect fn fresh<E, A> -> deferred.Deferred<E, A> = deferred.make<E, A> ()
pub effect fn probe -> Int = do {
  gate <- fresh<Never, Int> ()
  deferred.succeed 42 gate
  deferred.await gate
}
pub effect fn allocate -> deferred.Deferred<Never, Int> = fresh<Never, Int> ()
pub effect fn failed -> Int fails String = do {
  gate <- fresh<String, Int> ()
  deferred.fail "expected" gate
  deferred.await gate
}
"#;

#[test]
fn ordinary_generic_effect_factory_retains_checked_call_instantiation() {
    let mut outputs = Vec::new();
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let compiled = compile_module(
            CompileInput::new("factory.ssrg", "fixture/factory", EFFECT_FACTORY)
                .with_profile(profile),
        )
        .expect("the generic factory is a valid checked source program");
        assert!(compiled.diagnostics.diagnostics.is_empty());
        let allocation = compiled
            .core_ir
            .functions
            .iter()
            .find(|function| function.symbol.ends_with("::allocate"))
            .unwrap();
        let CoreExpr::Call {
            arguments,
            callee_signature: Some(signature),
            evidence,
            type_ref,
            ..
        } = &allocation.body
        else {
            panic!("the source call must retain its checked instantiation");
        };
        assert!(matches!(arguments.as_slice(), [CoreExpr::Unit { .. }]));
        assert!(evidence.is_empty());
        assert_eq!(signature.parameters.len(), 1);
        assert_eq!(signature.type_parameters.len(), 2);
        assert!(signature.type_parameters.iter().all(|p| p.arity == 0));
        let CoreType::Named { arguments, .. } = type_ref else {
            panic!("the checked call result remains an Effect");
        };
        let CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } = &arguments[2]
        else {
            panic!("the result retains the Deferred nominal owner");
        };
        assert_eq!(canonical, "std/deferred::Deferred");
        assert_eq!(&signature.instantiated_type_arguments, arguments);
        if let Some(directory) = std::env::var_os("SESERAGI_GENERIC_CALL_CAPTURE_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            for (suffix, data) in [
                (
                    "typed.json",
                    serde_json::to_string_pretty(&compiled.typed_hir).unwrap(),
                ),
                (
                    "core.json",
                    serde_json::to_string_pretty(&compiled.core_ir).unwrap(),
                ),
                (
                    "typescript.json",
                    serde_json::to_string_pretty(&compiled.typescript_ir).unwrap(),
                ),
                ("ts", compiled.generated.typescript.clone()),
            ] {
                std::fs::write(directory.join(format!("{name}.{suffix}")), data).unwrap();
            }
        }
        outputs.push((name, compiled.generated.typescript));
    }
    for (name, typescript) in outputs {
        assert!(
            typescript.contains("fresh<never, number>(undefined)")
                && typescript.contains("fresh<string, number>(undefined)"),
            "{name} must retain the checked Deferred failure/payload instantiation instead of host unknown inference:\n{typescript}",
        );
    }
}

#[test]
fn imported_generic_effect_factory_retains_checked_call_instantiation() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;

    let declaration = "effect fn fresh<E, A> -> deferred.Deferred<E, A> = deferred.make<E, A> ()\n";
    let domain = format!("import * as deferred from \"std/deferred\"\npub {declaration}");
    let caller = EFFECT_FACTORY
        .replacen(declaration, "import * as factory from \"./domain\"\n", 1)
        .replace("fresh<", "factory.fresh<");
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/factory::domain".to_owned(),
                std::iter::empty::<(String, String)>(),
            )
            .unwrap();
        graph
            .add_module(
                "fixture/factory::main".to_owned(),
                [("./domain".to_owned(), "fixture/factory::domain".to_owned())],
            )
            .unwrap();
        let compiled = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/factory::domain",
                    &domain,
                    "dist/domain.js",
                )
                .with_profile(profile),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/factory::main",
                    &caller,
                    "dist/main.js",
                )
                .with_profile(profile),
            ],
        )
        .expect("the checked factory contract survives the module boundary");
        let caller = &compiled.modules["fixture/factory::main"]
            .generated
            .typescript;
        assert!(caller.contains("<never, number>(undefined)"), "{caller}");
        assert!(caller.contains("<string, number>(undefined)"), "{caller}");
        if let Some(directory) = std::env::var_os("SESERAGI_GENERIC_CALL_CAPTURE_DIR") {
            let directory = std::path::PathBuf::from(directory).join(format!("imported-{name}"));
            std::fs::create_dir_all(&directory).unwrap();
            for (module, compiled) in &compiled.modules {
                let file = module.rsplit("::").next().unwrap();
                std::fs::write(
                    directory.join(format!("{file}.ts")),
                    &compiled.generated.typescript,
                )
                .unwrap();
            }
        }
    }
}
