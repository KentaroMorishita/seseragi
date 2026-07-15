use seseragi_semantics::{ExternalTypeBinding, ExternalTypeProvider};
use seseragi_syntax::Visibility;

use crate::{
    lower_core_module_to_typescript_ir_with_plan, CoreAdt, CoreExpr, CoreFunction, CoreModule,
    CoreModuleDependency, CoreParameter, CoreType, SourceSpan, TypeScriptLoweringError,
    TypeScriptOutputPlan,
};

use super::super::lower_module_imports;

fn origin() -> SourceSpan {
    SourceSpan {
        source: "main.ssrg".to_owned(),
        start: 0,
        end: 1,
    }
}

fn external(name: &str, canonical: &str) -> CoreType {
    CoreType::ExternalNamed {
        name: name.to_owned(),
        canonical: canonical.to_owned(),
        arguments: Vec::new(),
    }
}

fn binding(name: &str, canonical: &str, module: &str, export: &str) -> ExternalTypeBinding {
    ExternalTypeBinding {
        spelling: name.to_owned(),
        canonical: canonical.to_owned(),
        provider: Some(ExternalTypeProvider {
            module: module.to_owned(),
            export: export.to_owned(),
        }),
    }
}

fn module(bindings: Vec<ExternalTypeBinding>, parameters: Vec<CoreType>) -> CoreModule {
    CoreModule {
        schema: 1,
        stage: "core-ir".to_owned(),
        module: "fixture/main".to_owned(),
        external_type_bindings: bindings,
        module_dependencies: Vec::new(),
        adts: Vec::new(),
        instances: Vec::new(),
        bindings: Vec::new(),
        functions: vec![CoreFunction {
            symbol: "fixture/main::keep".to_owned(),
            visibility: Visibility::Public,
            origin: origin(),
            type_parameters: Vec::new(),
            type_constructor_parameters: Vec::new(),
            constraints: Vec::new(),
            parameters: parameters
                .into_iter()
                .enumerate()
                .map(|(index, type_ref)| CoreParameter {
                    id: format!("value{index}"),
                    kind: "named".to_owned(),
                    type_ref,
                })
                .collect(),
            body: CoreExpr::Unit { origin: origin() },
        }],
    }
}

#[test]
fn groups_an_inferred_type_with_an_existing_direct_source_import() {
    let canonical = "fixture/domain::Hand";
    let mut module = module(
        vec![binding("Hand", canonical, "fixture/domain", "Hand")],
        vec![external("Hand", canonical)],
    );
    module.module_dependencies.push(CoreModuleDependency {
        specifier: "./domain".to_owned(),
        module: "fixture/domain".to_owned(),
        origin: origin(),
        imports: Vec::new(),
    });
    let plan = TypeScriptOutputPlan::new([("fixture/domain".to_owned(), "./domain.js".to_owned())]);

    let lowered = lower_module_imports(&module, &plan).unwrap();

    assert_eq!(lowered.imports.len(), 1);
    assert_eq!(lowered.imports[0].module, "fixture/domain");
    assert!(lowered.imports[0].runtime_edge);
    assert_eq!(lowered.imports[0].bindings.len(), 1);
    assert!(lowered.imports[0].bindings[0].type_only);
    assert_eq!(lowered.imports[0].bindings[0].imported, "Hand");
    assert!(serde_json::to_value(&lowered.imports[0])
        .unwrap()
        .get("runtimeEdge")
        .is_none());
    assert_eq!(
        lowered.type_names.get(canonical).map(String::as_str),
        Some("Hand")
    );
}

#[test]
fn creates_a_type_only_import_for_a_transitive_provider_without_a_source_binding() {
    let canonical = "fixture/domain::Hand";
    let module = module(
        vec![binding("Hand", canonical, "fixture/domain", "Hand")],
        vec![external("Hand", canonical)],
    );
    let plan =
        TypeScriptOutputPlan::new([("fixture/domain".to_owned(), "../domain.js".to_owned())]);

    let lowered = lower_module_imports(&module, &plan).unwrap();

    assert_eq!(lowered.imports.len(), 1);
    assert_eq!(lowered.imports[0].specifier, "../domain.js");
    assert!(!lowered.imports[0].runtime_edge);
    assert_eq!(lowered.imports[0].bindings[0].canonical, canonical);
    assert_eq!(
        serde_json::to_value(&lowered.imports[0]).unwrap()["runtimeEdge"],
        false,
    );
}

#[test]
fn freshens_same_spelling_owners_and_rewrites_exact_type_occurrences() {
    let left = "fixture/left::Hand";
    let right = "fixture/right::Hand";
    let mut module = module(
        vec![
            binding("Hand", left, "fixture/left", "Hand"),
            binding("Hand", right, "fixture/right", "Hand"),
        ],
        vec![external("Hand", left), external("Hand", right)],
    );
    module.adts.push(CoreAdt {
        symbol: "fixture/main::Hand".to_owned(),
        name: "Hand".to_owned(),
        visibility: Visibility::Private,
        opaque: false,
        type_parameters: Vec::new(),
        variants: Vec::new(),
        origin: origin(),
    });
    let plan = TypeScriptOutputPlan::new([
        ("fixture/left".to_owned(), "./left.js".to_owned()),
        ("fixture/right".to_owned(), "./right.js".to_owned()),
    ]);

    let lowered = lower_core_module_to_typescript_ir_with_plan(module, &plan).unwrap();
    let parameters = match &lowered.functions[0] {
        crate::TypeScriptFunction::ConstFunction { parameters, .. } => parameters,
    };

    assert_eq!(parameters[0].type_name, "Hand_1");
    assert_eq!(parameters[1].type_name, "Hand_2");
    assert_eq!(lowered.source_imports[0].bindings[0].local, "Hand_1");
    assert_eq!(lowered.source_imports[1].bindings[0].local, "Hand_2");
}

#[test]
fn rejects_missing_binding_provider_or_output_plan_without_a_spelling_fallback() {
    let canonical = "fixture/domain::Hand";
    let missing_binding = module(
        Vec::new(),
        vec![CoreType::Named {
            name: "Maybe".to_owned(),
            arguments: vec![external("Hand", canonical)],
        }],
    );
    assert_eq!(
        lower_core_module_to_typescript_ir_with_plan(
            missing_binding,
            &TypeScriptOutputPlan::default(),
        )
        .unwrap_err(),
        TypeScriptLoweringError::MissingExternalTypeBinding {
            canonical: canonical.to_owned(),
        }
    );

    let missing_provider = module(
        vec![ExternalTypeBinding {
            spelling: "Hand".to_owned(),
            canonical: canonical.to_owned(),
            provider: None,
        }],
        vec![external("Hand", canonical)],
    );
    assert_eq!(
        lower_module_imports(&missing_provider, &TypeScriptOutputPlan::default())
            .err()
            .unwrap(),
        TypeScriptLoweringError::MissingSourceTypeProvider {
            canonical: canonical.to_owned(),
        }
    );

    let missing_output = module(
        vec![binding("Hand", canonical, "fixture/domain", "Hand")],
        vec![external("Hand", canonical)],
    );
    assert_eq!(
        lower_module_imports(&missing_output, &TypeScriptOutputPlan::default())
            .err()
            .unwrap(),
        TypeScriptLoweringError::MissingTypeOutputSpecifier {
            module: "fixture/domain".to_owned(),
            canonical: canonical.to_owned(),
        }
    );
}
