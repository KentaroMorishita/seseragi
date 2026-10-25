use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn analysis_program_roots_share_identity_and_overlays_without_widening_builds() {
    let root =
        repository_root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-package");
    let path = root.join("src/calculation.ssrg");
    let source = format!(
        "{}\npub fn editorOverlay -> Int = 7\n",
        fs::read_to_string(&path).unwrap()
    );
    let analysis =
        load_local_project_for_analysis(&root, [SourceOverlay::new(&path, &source)]).unwrap();
    assert!(analysis.program_errors().is_empty());
    let project = analysis.project();
    assert_eq!(project.entry().path().as_str(), "main");
    assert_eq!(project.modules().count(), 3);
    assert!(project
        .modules()
        .all(|(id, _)| id.package() == project.entry().package()));
    let calculation = project
        .modules()
        .find(|(id, _)| id.path().as_str() == "calculation")
        .unwrap();
    assert_eq!(calculation.1.source(), source);
    assert_eq!(
        project.graph().dependencies_for(calculation.0).unwrap()[0]
            .1
            .path()
            .as_str(),
        "constants"
    );
    let ordinary =
        load_local_project_with_overlays(&root, [SourceOverlay::new(&path, &source)]).unwrap();
    assert_eq!(ordinary.modules().count(), 1);
    assert_eq!(ordinary.entry().package(), project.entry().package());
}

#[test]
fn analysis_program_discovery_rolls_back_a_failed_root_and_keeps_its_error() {
    let project = TempProject::new();
    project.write("seseragi.toml", concat!(
        "[package]\nname = \"fixture/analysis-programs\"\nversion = \"0.1.0\"\nlanguage = \"^0.1.0\"\n",
        "[run]\nentry = \"main\"\ntarget = \"web\"\n",
        "[web.programs.bad]\nmodule = \"bad\"\nexports = [\"amount\"]\n",
        "[web.programs.good]\nmodule = \"good\"\nexports = [\"amount\"]\n",
    ));
    project.write(
        "src/main.ssrg",
        "import { base } from \"./shared\"\npub fn main -> Int = base\n",
    );
    project.write("src/shared.ssrg", "pub let base: Int = 42\n");
    project.write(
        "src/bad.ssrg",
        "import { value } from \"./missing\"\npub fn amount -> Int = value\n",
    );
    project.write(
        "src/good.ssrg",
        "import { base } from \"./shared\"\npub fn amount -> Int = base\n",
    );
    let analysis = load_local_project_for_analysis(project.path(), []).unwrap();
    assert_eq!(analysis.program_errors().len(), 1);
    let (identity, error) = analysis.program_errors().first_key_value().unwrap();
    assert_eq!(identity.path().as_str(), "bad");
    assert!(error.to_string().contains("missing"));
    assert_eq!(analysis.project().modules().count(), 3);
    assert!(analysis
        .project()
        .modules()
        .all(|(id, _)| id.path().as_str() != "bad"));
    assert_eq!(
        analysis
            .project()
            .graph()
            .topological_order()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        load_local_project(project.path())
            .unwrap()
            .modules()
            .count(),
        2
    );
    assert!(
        load_local_project_module(project.path(), &crate::ModulePath::parse("bad").unwrap())
            .is_err()
    );
}

#[test]
fn selecting_a_program_entry_loads_its_import_closure_not_the_ui() {
    let root =
        repository_root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-package");
    let ui = load_local_project(&root).unwrap();
    assert_eq!(ui.entry().path().as_str(), "main");
    assert_eq!(ui.modules().count(), 1);
    let program =
        load_local_project_module(&root, &crate::ModulePath::parse("calculation").unwrap())
            .unwrap();
    let order = program.graph().topological_order().unwrap();
    assert_eq!(
        order
            .iter()
            .map(|id| id.path().as_str())
            .collect::<Vec<_>>(),
        ["constants", "calculation"]
    );
    assert_eq!(ui.entry().package(), program.entry().package());
    assert!(
        load_local_project_module(&root, &crate::ModulePath::parse("missing").unwrap()).is_err()
    );
}

#[test]
fn generated_browser_import_requires_the_producer_owner_and_declared_module() {
    let root =
        repository_root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-package");
    let ui = load_local_project(&root).unwrap();
    let selected =
        load_local_project_module(&root, &crate::ModulePath::parse("calculation").unwrap())
            .unwrap();
    assert!(crate::validate_foreign_runtime_target(&ui, crate::RuntimeTarget::Browser).is_err());
    let proof = crate::GeneratedProgramRuntime::new(
        ui.packages().root().clone(),
        "calculation".into(),
        crate::logical_module_id(selected.entry()),
        "a".repeat(64),
    );
    assert!(crate::validate_foreign_runtime_target_with_programs(
        &ui,
        crate::RuntimeTarget::Browser,
        std::slice::from_ref(&proof)
    )
    .is_ok());
    let mut bad = proof.clone();
    bad.artifact.exact_identity = crate::logical_module_id(ui.entry());
    assert!(crate::validate_foreign_runtime_target_with_programs(
        &ui,
        crate::RuntimeTarget::Browser,
        &[bad]
    )
    .is_err());
    let mut bad = proof.clone();
    bad.name = "missing".into();
    assert!(crate::validate_foreign_runtime_target_with_programs(
        &ui,
        crate::RuntimeTarget::Browser,
        &[bad]
    )
    .is_err());
    let mut bad = proof.clone();
    bad.artifact.content_digest = "unmeasured".into();
    assert!(crate::validate_foreign_runtime_target_with_programs(
        &ui,
        crate::RuntimeTarget::Browser,
        &[bad]
    )
    .is_err());
    let mut bad = proof;
    bad.package = crate::load_local_project(
        repository_root().join("examples/spec/fixtures/projects/package-path-dependency"),
    )
    .unwrap()
    .entry()
    .package()
    .clone();
    assert!(crate::validate_foreign_runtime_target_with_programs(
        &ui,
        crate::RuntimeTarget::Browser,
        &[bad]
    )
    .is_err());
}

#[test]
fn discovers_source_modules_across_path_dependencies() {
    let project = load_local_project(
        repository_root().join("examples/spec/fixtures/projects/package-path-dependency"),
    )
    .unwrap();

    assert_eq!(project.modules().count(), 3);
    assert_eq!(
        project.entry().package().name().as_str(),
        "fixture/package-path-dependency"
    );
    assert_eq!(project.entry().path().as_str(), "main");
    let order = project.graph().topological_order().unwrap();
    assert_eq!(order.len(), 3);
    assert_eq!(order[0].package().name().as_str(), "fixture/math");
    assert_eq!(order[1].package().name().as_str(), "fixture/math");
    assert_eq!(&order[2], project.entry());
    let main_dependencies = project.graph().dependencies_for(project.entry()).unwrap();
    assert_eq!(main_dependencies.len(), 2);
    assert_eq!(main_dependencies[0].0, "math");
    assert_eq!(main_dependencies[0].1.path().as_str(), "lib");
    assert_eq!(main_dependencies[1].0, "math/stats");
    assert_eq!(main_dependencies[1].1.path().as_str(), "stats");
}

#[test]
fn reports_undeclared_package_import_at_the_source_edge() {
    let project = TempProject::new();
    project.write(
        "seseragi.toml",
        concat!(
            "[package]\n",
            "name = \"fixture/app\"\n",
            "version = \"1.0.0\"\n",
            "language = \"^0.1.0\"\n\n",
            "[run]\n",
            "entry = \"main\"\n",
        ),
    );
    project.write(
        "src/main.ssrg",
        "import { request } from \"acme/http\"\n\npub fn main -> Unit = ()\n",
    );

    let error = load_local_project(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0103");
    assert!(matches!(
        error,
        LocalProjectLoadError::Import {
            specifier,
            origin,
            ..
        } if specifier == "acme/http" && origin.end > origin.start
    ));
}

#[cfg(unix)]
#[test]
fn audits_unreachable_source_aliases_before_entry_discovery() {
    use std::os::unix::fs::symlink;

    let project = TempProject::new();
    project.write(
        "seseragi.toml",
        concat!(
            "[package]\n",
            "name = \"fixture/source-audit\"\n",
            "version = \"1.0.0\"\n",
            "language = \"^0.1.0\"\n\n",
            "[run]\n",
            "entry = \"main\"\n",
        ),
    );
    project.write("src/main.ssrg", "pub fn main -> Unit = ()\n");
    symlink(
        project.path().join("src/main.ssrg"),
        project.path().join("src/unreachable.ssrg"),
    )
    .unwrap();

    let error = load_local_project(project.path()).unwrap_err();
    assert!(matches!(
        error,
        LocalProjectLoadError::Filesystem { error, .. }
            if matches!(*error, PackageLoadError::DuplicatePhysicalModule { .. })
    ));
}

#[test]
fn reports_missing_generated_root_at_the_import_edge() {
    let project = TempProject::new();
    project.write("seseragi.toml", "[package]\nname = \"fixture/missing-generated\"\nversion = \"0.0.0\"\nlanguage = \"^0.1.0\"\n[run]\nentry = \"main\"\n");
    let source = "import { value } from \"gen/api\"\npub let main: Int = value\n";
    project.write("src/main.ssrg", source);
    let error = load_local_project(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-N0104");
    assert!(
        matches!(error, LocalProjectLoadError::Import { module, specifier, origin, reason, .. }
        if module.path().as_str() == "main" && specifier == "gen/api"
        && source[origin.start..origin.end].contains("gen/api")
        && reason.contains("Generated module root"))
    );

    // Generating the same dependency makes ordinary discovery succeed.
    project.write(".seseragi/generated/api.ssrg", "pub let value: Int = 42\n");
    let loaded = load_local_project(project.path()).unwrap();
    assert_eq!(loaded.modules().count(), 2);
    assert!(loaded
        .modules()
        .any(|(module, _)| module.root() == ModuleRoot::Generated));
}

#[test]
fn rejects_source_generated_logical_collisions_before_driver_compilation() {
    let project = TempProject::new();
    project.write("seseragi.toml", "[package]\nname = \"fixture/collision\"\nversion = \"0.0.0\"\nlanguage = \"^0.1.0\"\n[layout]\ngenerated = \"gen\"\n[run]\nentry = \"main\"\n");
    project.write("src/main.ssrg", "import { left } from \"./api\"\nimport { right } from \"gen/api\"\npub let main: Int = left + right\n");
    project.write("src/api.ssrg", "pub let left: Int = 1\n");
    project.write("gen/api.ssrg", "pub let right: Int = 2\n");
    let error = load_local_project(project.path()).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("src/api.ssrg"), "{message}");
    assert!(message.contains("gen/api.ssrg"), "{message}");
    assert!(
        matches!(error, LocalProjectLoadError::DuplicateLogicalModule { first, second, .. }
        if first.path() == second.path() && first.root() != second.root())
    );
}

#[test]
fn preserves_noncolliding_generated_imports_and_root_boundaries() {
    let project = TempProject::new();
    project.write("seseragi.toml", "[package]\nname = \"fixture/generated\"\nversion = \"0.0.0\"\nlanguage = \"^0.1.0\"\n[layout]\ngenerated = \"gen\"\n[run]\nentry = \"main\"\n");
    project.write(
        "src/main.ssrg",
        "import { right } from \"gen/api\"\npub let main: Int = right\n",
    );
    project.write(
        "gen/api.ssrg",
        "import { value } from \"./value\"\npub let right: Int = value\n",
    );
    project.write("gen/value.ssrg", "pub let value: Int = 2\n");
    let loaded = load_local_project(project.path()).unwrap();
    assert_eq!(loaded.modules().count(), 3);
    project.write(
        "gen/api.ssrg",
        "import { value } from \"self/value\"\npub let right: Int = value\n",
    );
    assert!(matches!(load_local_project(project.path()).unwrap_err(),
        LocalProjectLoadError::Import { reason, .. } if reason.contains("handwritten source")));
}

struct TempProject {
    path: PathBuf,
}

impl TempProject {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "seseragi-local-project-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, source).unwrap();
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
