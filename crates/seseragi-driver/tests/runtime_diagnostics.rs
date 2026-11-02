use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn runtime_sources_keep_file_names_scalar_columns_and_expression_ranges() {
    let source = "pub fn broken value: Int -> Int = {\n  let text = \"雪🙂\"; let result = value / 0\n  result\n}\n";
    let start = source.find("value / 0").unwrap();
    let column = source[..start].rsplit('\n').next().unwrap().chars().count() + 1;
    for path in [
        "/sources/雪🙂/main.ssrg",
        "C:\\sources\\雪🙂\\main.ssrg",
        "nested/雪🙂/main.ssrg",
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(
                CompileInput::new(path, "fixture/diagnostics", source).with_profile(profile),
            )
            .unwrap();
            let origin = compiled
                .typescript_ir
                .runtime_sources
                .iter()
                .find(|origin| origin.start == start)
                .unwrap();
            assert_eq!(origin.source, "main.ssrg");
            assert_eq!(origin.end, start + "value / 0".len());
            assert!(compiled.generated.typescript.contains(&format!(
                "const $ssrg$source{start} = {{ file: \"main.ssrg\", line: 2, column: {column},"
            )));
            assert!(!compiled.generated.typescript.contains(path));
            assert!(compiled
                .generated
                .typescript
                .contains(&format!("_ssrg_int_divide(value, 0, $ssrg$source{start})")));
        }
    }
}

#[test]
fn project_runtime_paths_keep_package_directories_and_forward_slashes() {
    let mut graph = seseragi_project::ModuleGraph::new();
    graph
        .add_module("fixture/paths::main".to_owned(), [])
        .unwrap();
    let input = seseragi_driver::ProjectModuleInput::new(
        "/checkout/build-machine/src/雪/main.ssrg",
        "fixture/paths::main",
        "pub fn broken value: Int -> Int = value / 0\n",
        "dist/main.js",
    )
    .with_runtime_source_path("src\\雪\\main.ssrg");
    let compiled = seseragi_driver::compile_project(graph, [input]).unwrap();
    let module = &compiled.modules["fixture/paths::main"];
    assert_eq!(
        module.typescript_ir.runtime_sources[0].source,
        "src/雪/main.ssrg"
    );
    assert!(module
        .generated
        .typescript
        .contains("file: \"src/雪/main.ssrg\""));
    assert!(!module
        .generated
        .typescript
        .contains("/checkout/build-machine"));
}

#[test]
fn source_tracking_preserves_scalar_self_tail_loops() {
    let source = "pub fn loop remaining: Int -> total: Int -> Int = if remaining == 0 then total else loop (remaining - 1) (total + remaining)\n";
    let compiled = compile_module(
        CompileInput::new("loop.ssrg", "fixture/loop", source).with_profile(BuildProfile::Release),
    )
    .unwrap();
    assert!(compiled.generated.typescript.contains("while (true)"));
    assert!(compiled.generated.typescript.contains("continue;"));
    assert!(!compiled.generated.typescript.contains("() => loop("));
}
