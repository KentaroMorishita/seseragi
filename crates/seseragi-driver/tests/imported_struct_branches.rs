use seseragi_driver::{compile_project, BuildProfile, ProjectModuleInput};
use seseragi_project::ModuleGraph;

fn compile(
    source: &str,
    profile: BuildProfile,
) -> Result<seseragi_driver::CompiledProject, seseragi_driver::ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    graph.add_module("fixture/model".into(), vec![]).unwrap();
    graph
        .add_module(
            "fixture/main".into(),
            vec![("./model".into(), "fixture/model".into())],
        )
        .unwrap();
    compile_project(
        graph,
        [
            (
                "model",
                "pub struct Log { text: String }\npub effect fn mark = println \"mark\"\n",
            ),
            ("main", source),
        ]
        .map(|(name, source)| {
            ProjectModuleInput::new(
                format!("{name}.ssrg"),
                format!("fixture/{name}"),
                source,
                format!("{name}.js"),
            )
            .with_profile(profile)
        }),
    )
}

#[test]
fn imported_struct_field_branches_keep_the_top_level_binding() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for expression in [
            "match custom { Just (_, output) -> output.text; Nothing -> \"missing\" }",
            "match custom { Nothing -> \"missing\"; Just (_, output) -> output.text }",
            "if True then (Log { text: \"x\" }).text else \"missing\"",
            "if False then \"missing\" else (Log { text: \"x\" }).text",
        ] {
            let source = format!(
                "import {{ Log }} from \"./model\"\n\
             let custom: Maybe<(Int, Log)> = Just (1, Log {{ text: \"x\" }})\n\
             let text = {expression}\n\
             pub effect fn main = println text\n"
            );
            let compiled = compile(&source, profile).unwrap();
            let main = &compiled.modules["fixture/main"];
            assert!(
                main.generated.typescript.contains("const text: string"),
                "{:#?}\n{}",
                main.typed_hir,
                main.generated.typescript
            );
        }
    }
}

#[test]
fn a_local_nominal_string_is_not_an_intrinsic_branch() {
    let source = "import { Log } from \"./model\"\n\
                  type String = | LocalString\n\
                  let text = if True then (Log { text: \"x\" }).text else LocalString\n\
                  pub effect fn main = println text\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        assert!(compile(source, profile).is_err());
    }
}
