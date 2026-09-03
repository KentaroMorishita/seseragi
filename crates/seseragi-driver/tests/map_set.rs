use seseragi_driver::{compile_module, compile_project, CompileInput, ProjectModuleInput};
use seseragi_project::ModuleGraph;

#[test]
fn compiles_map_set_surface_and_materializes_scoped_and_partial_evidence() {
    let source = include_str!("../../../examples/spec/artifacts/schema-1/map-set/main.ssrg");
    let compiled = compile_module(CompileInput::new(
        "map-set.ssrg",
        "artifact/map-set",
        source,
    ))
    .expect("all standard Map / Set operations compile");
    for surface in [
        "@seseragi/runtime/map",
        "@seseragi/runtime/set",
        "mapJsonEncode",
        "mapJsonDecode",
        "mapIterable",
        "mapReducible",
        "mapFunctor",
        "setIterable",
        "setReducible",
        "mapEq",
        "setEq",
    ] {
        assert!(
            compiled.generated.typescript.contains(surface),
            "missing {surface}"
        );
    }
}

#[test]
fn rejects_set_functor_and_keys_without_hash() {
    for source in [
        "import * as sets from \"std/set\"\npub let invalid = map (\\x: Int -> x + 1) (sets.singleton 1)\n",
        "import * as maps from \"std/map\"\ntype Key =\n  | Key Int\ninstance Eq<Key> { fn eq left: Key -> right: Key -> Bool = True }\npub let invalid = maps.singleton (Key 1) 2\n",
        "import * as sets from \"std/set\"\npub let invalid: sets.Set<String> = sets.fromIterable [1, 2]\n",
    ] {
        let diagnostics = compile_module(CompileInput::new("invalid.ssrg", "artifact/invalid", source))
            .expect_err("invalid collection evidence must prevent lowering");
        assert!(!diagnostics.diagnostics.is_empty());
        assert!(diagnostics.diagnostics.iter().all(|diagnostic| diagnostic.code != "SES-P0001"), "negative test must parse: {diagnostics:?}");
    }
}

#[test]
fn infers_element_types_from_user_collection_instances() {
    let source = include_str!(
        "../../../examples/spec/artifacts/schema-1/user-iterable-comprehension/main.ssrg"
    );
    let source = format!("import * as sets from \"std/set\"\n{source}\npub let distinct = sets.fromIterable (Countdown 4)\npub let result = show (sum (Countdown 4))\n");
    let compiled = compile_module(CompileInput::new(
        "user-map-set.ssrg",
        "artifact/user-map-set",
        &source,
    ))
    .expect("functional collection dependencies infer user elements");
    assert!(compiled
        .generated
        .typescript
        .contains("_ssrg_set_fromIterable"));
}

#[test]
fn infers_imported_collection_instances_and_preserves_generic_dictionary_arguments() {
    let domain = include_str!("../../../examples/spec/artifacts/project-schema-1/imported-iterable-comprehension/src/domain.ssrg");
    let domain = format!("import * as sets from \"std/set\"\n{domain}\npub fn unique<C, A> values: C -> sets.Set<A> where Iterable<C, A>, Eq<A>, Hash<A> = sets.fromIterable values\n");
    let main = "import { Countdown, unique } from \"./domain\"\nimport * as sets from \"std/set\"\npub let direct = show (sets.fromIterable (Countdown 4))\npub let generic = show (unique (Countdown 4))\npub let total = show (sum (Countdown 4))\n";
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/map-set::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/map-set::main".to_owned(),
            [("./domain".to_owned(), "fixture/map-set::domain".to_owned())],
        )
        .unwrap();
    let compiled = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "domain.ssrg",
                "fixture/map-set::domain",
                domain,
                "dist/domain.js",
            ),
            ProjectModuleInput::new("main.ssrg", "fixture/map-set::main", main, "dist/main.js"),
        ],
    )
    .expect("imported collection evidence and functional dependencies resolve");
    assert!(compiled.modules["fixture/map-set::main"]
        .generated
        .typescript
        .contains("_ssrg_set_fromIterable"));
}
