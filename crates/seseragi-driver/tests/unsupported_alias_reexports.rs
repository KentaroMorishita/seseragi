use seseragi_driver::{compile_project, BuildProfile, ProjectModuleInput};
use seseragi_project::ModuleGraph;

#[test]
fn reexported_higher_kinded_aliases_keep_typescript_declarations() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let mut graph = ModuleGraph::new();
        graph.add_module("fixture::model".into(), []).unwrap();
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
        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "model.ssrg",
                    "fixture::model",
                    "pub type Box<A> = | Box A\npub type Pair<A, B> = | Pair (A, B)\npub alias Apply<F<_>, A> = F<A>\npub alias Flip<A, B> = Pair<B, A>\n",
                    "dist/model.js",
                ),
                ProjectModuleInput::new(
                    "facade.ssrg",
                    "fixture::facade",
                    "pub import { Apply, Flip, Box, Pair } from \"./model\"\n",
                    "dist/facade.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture::main",
                    "import { Apply, Flip, Box, Pair } from \"./facade\"\npub fn keep value: Apply<Box, Int> -> Box<Int> = value\npub fn swapped value: Flip<Bool, Int> -> Pair<Int, Bool> = value\n",
                    "dist/main.js",
                ),
            ]
            .map(|input| input.with_profile(profile)),
        )
        .unwrap_or_else(|error| panic!("{profile:?}\n{error:#?}"));
        let model = &project.modules["fixture::model"].generated.typescript;
        let facade = &project.modules["fixture::facade"].generated.typescript;
        let main = &project.modules["fixture::main"].generated.typescript;
        assert!(
            model.contains(
                "export type Apply<F extends { readonly args: readonly [unknown]; readonly type: unknown }, A> = "
            ),
            "{profile:?}\n{model}"
        );
        assert!(
            model.contains("export type Flip<A, B> = Pair<B, A>"),
            "{profile:?}\n{model}"
        );
        assert!(
            facade.contains("type Apply") && facade.contains("type Flip"),
            "{profile:?}\n{facade}"
        );
        assert!(
            facade.contains("from \"./model.js\""),
            "{profile:?}\n{facade}"
        );
        assert!(!main.contains("Apply"), "{profile:?}\n{main}");
        assert!(!main.contains("Flip"), "{profile:?}\n{main}");
    }
}
