use super::{compile_project, ProjectModuleInput};
use seseragi_project::ModuleGraph;

#[test]
fn plans_a_transitive_nominal_provider_without_a_fake_source_edge() {
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/transitive::main".to_owned(),
            [(
                "./facade".to_owned(),
                "fixture/transitive::facade".to_owned(),
            )],
        )
        .unwrap();
    graph
        .add_module(
            "fixture/transitive::facade".to_owned(),
            [(
                "./provider".to_owned(),
                "fixture/transitive::provider".to_owned(),
            )],
        )
        .unwrap();
    graph
        .add_module("fixture/transitive::provider".to_owned(), [])
        .unwrap();

    let project = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "provider.ssrg",
                "fixture/transitive::provider",
                "pub type InputError deriving Show =\n  | InvalidInput String\n",
                "dist/domain/provider.js",
            ),
            ProjectModuleInput::new(
                "facade.ssrg",
                "fixture/transitive::facade",
                "import { InputError, InvalidInput } from \"./provider\"\n\npub effect fn reject input: String =\n  fail (InvalidInput input)\n",
                "dist/effects/facade.js",
            ),
            ProjectModuleInput::new(
                "main.ssrg",
                "fixture/transitive::main",
                "import { reject } from \"./facade\"\n\npub effect fn main =\n  reject \"lizard\"\n",
                "dist/app/main.js",
            ),
        ],
    )
    .unwrap();

    let main = project.modules.get("fixture/transitive::main").unwrap();
    assert_eq!(main.typed_hir.module_dependencies.len(), 1);
    assert_eq!(
        main.typed_hir.module_dependencies[0].module,
        "fixture/transitive::facade"
    );
    let binding = main
        .typed_hir
        .external_type_bindings
        .iter()
        .find(|binding| binding.spelling == "InputError")
        .expect("inferred Effect failure must retain its nominal provider");
    let provider = binding
        .provider
        .as_ref()
        .expect("source nominal binding must retain provider metadata");
    assert_eq!(provider.module, "fixture/transitive::provider");
    assert_eq!(provider.export, "InputError");
    assert!(main
        .generated
        .typescript
        .contains("import { type InputError } from \"../domain/provider.js\""));
    assert!(main
        .generated
        .typescript
        .contains("import { reject } from \"../effects/facade.js\""));
}
