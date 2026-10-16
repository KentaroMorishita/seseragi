use super::*;
use seseragi_project::BuildProfile;

fn compile(sources: &[(&str, &str, &[&str])], profile: BuildProfile) -> CompiledProject {
    let mut graph = ModuleGraph::new();
    let inputs = sources
        .iter()
        .map(|(name, source, dependencies)| {
            graph
                .add_module(
                    (*name).to_owned(),
                    dependencies
                        .iter()
                        .map(|dependency| (format!("./{dependency}"), (*dependency).into())),
                )
                .unwrap();
            ProjectModuleInput::new(format!("{name}.ssrg"), *name, *source, format!("{name}.js"))
                .with_profile(profile)
        })
        .collect::<Vec<_>>();
    compile_project(graph, inputs).unwrap()
}

#[test]
fn type_use_of_task_only_bindings_stays_inert_while_value_use_keeps_the_import() {
    let api = "pub foreign \"typescript\" from \"fixture-api\" {\n  task fn fetchName id: Int -> String\n}\npub struct Config { label: String }\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for (body, runtime_value) in [
            (
                "pub fn identity config: api.Config -> api.Config = config\n",
                false,
            ),
            (
                "pub fn request id: Int -> Effect<{}, Js.Error, String> = api.fetchName id\n",
                true,
            ),
        ] {
            let source = format!("import * as api from \"./api\"\n{body}");
            let project = compile(&[("api", api, &[]), ("main", &source, &["api"])], profile);
            let main = &project.modules["main"];
            let import = &main.typescript_ir.source_imports[0];
            assert!(!import.runtime_edge);
            assert_eq!(
                import.bindings.iter().any(|binding| !binding.type_only),
                runtime_value
            );
            assert!(!main.generated.typescript.contains("import \"./api.js\""));
            assert!(project.modules["api"]
                .generated
                .typescript
                .contains("import(\"fixture-api\")"));
        }
    }
}

#[test]
fn unused_imports_retain_eager_values_and_pure_hosts_through_inert_facades() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for dependency in [
            "let bad: Int = 9007199254740991 + 1\npub fn unused -> Int = 42\n",
            "pub foreign \"typescript\" from \"fixture-api\" {\n  pure fn unused -> Int\n}\n",
        ] {
            let project = compile(
                &[
                    ("dependency", dependency, &[]),
                    (
                        "facade",
                        "import { unused } from \"./dependency\"\npub fn ignored -> Unit = ()\n",
                        &["dependency"],
                    ),
                    (
                        "main",
                        "import { ignored } from \"./facade\"\npub fn main -> Unit = ()\n",
                        &["facade"],
                    ),
                ],
                profile,
            );
            for (owner, dependency) in [("main", "facade"), ("facade", "dependency")] {
                assert!(project.modules[owner].typescript_ir.source_imports[0].runtime_edge);
                assert!(project.modules[owner]
                    .generated
                    .typescript
                    .contains(&format!("import \"./{dependency}.js\"")));
            }
            let mut modules = project
                .modules
                .into_iter()
                .map(|(name, module)| (name, module.typescript_ir))
                .collect();
            seseragi_lowering::retain_application(&mut modules, &[("main".into(), "main".into())]);
            assert_eq!(modules.len(), 3);
            assert_eq!(modules["main"].source_imports.len(), 1);
            assert_eq!(modules["facade"].source_imports.len(), 1);
            assert!(
                !modules["dependency"].bindings.is_empty()
                    || modules["dependency"]
                        .foreign_modules
                        .iter()
                        .any(|host| host.pure_load)
            );
        }
    }
}
