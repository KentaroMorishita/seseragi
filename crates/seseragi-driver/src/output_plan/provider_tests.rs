use super::{plan_typescript_outputs, TypeScriptModuleOutput, TypeScriptOutputPlanError};

#[test]
fn plans_a_transitive_provider_without_a_source_dependency_edge() {
    let plan = plan_typescript_outputs(
        "dist/app/main.js",
        [
            TypeScriptModuleOutput::new("fixture/app::provider", "dist/domain/provider.js"),
            TypeScriptModuleOutput::new("fixture/app::facade", "dist/effects/facade.js"),
        ],
    )
    .unwrap();

    assert_eq!(
        plan.specifier_for("fixture/app::provider"),
        Some("../domain/provider.js")
    );
    assert_eq!(
        plan.specifier_for("fixture/app::facade"),
        Some("../effects/facade.js")
    );
}

#[test]
fn rejects_invalid_or_colliding_provider_paths() {
    assert_eq!(
        plan_typescript_outputs(
            "dist/game/main.js",
            [TypeScriptModuleOutput::new(
                "fixture/game::provider",
                "dist/game/main.js",
            )],
        )
        .unwrap_err(),
        TypeScriptOutputPlanError::DuplicateOutputPath {
            path: "dist/game/main.js".to_owned(),
        }
    );
    assert!(matches!(
        plan_typescript_outputs(
            "dist/game/main.js",
            [TypeScriptModuleOutput::new(
                "fixture/game::provider",
                "../provider.js"
            )],
        ),
        Err(TypeScriptOutputPlanError::InvalidDependencyPath { module, .. })
            if module == "fixture/game::provider"
    ));
}
