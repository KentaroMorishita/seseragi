use super::*;
use seseragi_project::LinkError;

#[test]
fn external_package_edges_do_not_expose_private_headers() {
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/app@1.0.0::main".to_owned(),
            [("math".to_owned(), "fixture/math@1.0.0::lib".to_owned())],
        )
        .unwrap();
    graph
        .add_module("fixture/math@1.0.0::lib".to_owned(), [])
        .unwrap();

    let error = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "math/src/lib.ssrg",
                "fixture/math@1.0.0::lib",
                "fn hidden value: Int -> Int = value\npub fn double value: Int -> Int = value * 2\n",
                "dist/packages/fixture/math/1.0.0/lib.js",
            )
            .with_package_scope("fixture/math@1.0.0"),
            ProjectModuleInput::new(
                "app/src/main.ssrg",
                "fixture/app@1.0.0::main",
                "import { hidden } from \"math\"\npub fn answer -> Int = hidden 42\n",
                "dist/packages/fixture/app/1.0.0/main.js",
            )
            .with_package_scope("fixture/app@1.0.0"),
        ],
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectCompileError::Link { errors, .. }
            if matches!(errors.as_slice(), [LinkError::MissingExport { name, .. }] if name == "hidden")
    ));
}
