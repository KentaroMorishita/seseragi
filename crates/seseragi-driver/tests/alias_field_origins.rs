use seseragi_driver::{compile_project, BuildProfile, ProjectModuleInput};
use seseragi_project::ModuleGraph;

#[test]
fn alias_fields_retain_transitive_source_structs_without_import_edges() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let mut graph = ModuleGraph::new();
        graph.add_module("fixture::owner".into(), []).unwrap();
        graph
            .add_module(
                "fixture::model".into(),
                [("./owner".into(), "fixture::owner".into())],
            )
            .unwrap();
        graph
            .add_module(
                "fixture::facade".into(),
                [("./model".into(), "fixture::model".into())],
            )
            .unwrap();
        graph
            .add_module(
                "fixture::main".into(),
                [("./facade".into(), "fixture::facade".into())],
            )
            .unwrap();
        let compiled = compile_project(
            graph,
            [
                ("owner", "pub struct Box<A> { value: A }\n"),
                ("model", "import { Box } from \"./owner\"\npub alias Row = { box: Box<Int> }\n"),
                ("facade", "pub import { Row as PublicRow } from \"./model\"\n"),
                ("main", "import { PublicRow } from \"./facade\"\nstruct Box { value: Bool }\npub fn read row:PublicRow -> Int = row.box.value\npub fn local box:Box -> Bool = box.value\n"),
            ]
            .map(|(name, source)| {
                ProjectModuleInput::new(
                    format!("{name}.ssrg"),
                    format!("fixture::{name}"),
                    source,
                    format!("dist/{name}.js"),
                )
                .with_profile(profile)
            }),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
        let main = &compiled.modules["fixture::main"];
        assert_eq!(main.typed_hir.module_dependencies.len(), 1);
        assert_eq!(
            main.typed_hir.module_dependencies[0].module,
            "fixture::facade"
        );
        assert_eq!(main.typed_hir.module_dependencies[0].imports.len(), 1);
        assert_eq!(main.typed_interface.dependencies.len(), 1);
    }
}
