use seseragi_driver::{compile_project, BuildProfile, ProjectModuleInput};
use seseragi_project::ModuleGraph;

#[test]
fn original_array_alias_package_preserves_qualified_opaque_elements() {
    let provider=include_str!("../../../examples/spec/fixtures/rust-backend-projects/basic-array-rest/vendor/values/src/main.ssrg");
    let root = include_str!(
        "../../../examples/spec/fixtures/rust-backend-projects/basic-array-rest/src/main.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/array-alias::values".into(), [])
            .unwrap();
        graph
            .add_module(
                "fixture/array-alias::main".into(),
                [("values".into(), "fixture/array-alias::values".into())],
            )
            .unwrap();
        compile_project(
            graph,
            [("values", provider), ("main", root)].map(|(name, source)| {
                ProjectModuleInput::new(
                    format!("{name}-雪🙂.ssrg"),
                    format!("fixture/array-alias::{name}"),
                    source,
                    format!("dist/{name}.js"),
                )
                .with_profile(profile)
            }),
        )
        .unwrap_or_else(|error| panic!("{profile:?}: {error:#?}"));
    }
}

#[test]
fn equal_spelled_opaque_elements_from_different_providers_stay_distinct() {
    let left="import * as arrays from \"std/array\"\npub opaque newtype Value = Int\npub alias Items<A> = Array<A>\npub fn maker<A> -> (A -> Items<A>) = arrays.singleton\n";
    let right = "pub opaque newtype Value = Int\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for owner in ["left", "right"] {
            let root=format!("import * as left from \"left\"\nimport * as right from \"right\"\nfn operation -> (left.Value -> left.Items<{owner}.Value>) = left.maker ()\npub fn main -> Int = 0\n");
            let mut graph = ModuleGraph::new();
            graph
                .add_module("fixture/array-alias::left".into(), [])
                .unwrap();
            graph
                .add_module("fixture/array-alias::right".into(), [])
                .unwrap();
            graph
                .add_module(
                    "fixture/array-alias::main".into(),
                    [
                        ("left".into(), "fixture/array-alias::left".into()),
                        ("right".into(), "fixture/array-alias::right".into()),
                    ],
                )
                .unwrap();
            let result = compile_project(
                graph,
                [("left", left), ("right", right), ("main", root.as_str())].map(
                    |(name, source)| {
                        ProjectModuleInput::new(
                            format!("{name}.ssrg"),
                            format!("fixture/array-alias::{name}"),
                            source,
                            format!("dist/{name}.js"),
                        )
                        .with_profile(profile)
                    },
                ),
            );
            if owner == "left" {
                result.unwrap();
            } else {
                assert!(
                    result.is_err(),
                    "different canonical opaque elements were treated as equal"
                );
            }
        }
    }
}
