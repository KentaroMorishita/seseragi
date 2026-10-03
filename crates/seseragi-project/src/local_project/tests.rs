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
fn diagnoses_handwritten_generated_logical_identity_collisions_before_driver_loading() {
    let project = TempProject::new();
    project.write("seseragi.toml", "[package]\nname=\"fixture/collision\"\nversion=\"0.0.0\"\nlanguage=\"^0.1.0\"\n[layout]\ngenerated=\"gen\"\n[run]\nentry=\"main\"\n");
    project.write("src/main.ssrg", "import { left } from \"./api\"\nimport { right } from \"gen/api\"\npub let main = left + right\n");
    project.write("src/api.ssrg", "pub let left: Int = 1\n");
    project.write("gen/api.ssrg", "pub let right: Int = 2\n");
    for error in [
        load_local_project(project.path()).unwrap_err(),
        load_local_documents(project.path()).unwrap_err(),
    ] {
        assert_eq!(error.code(), "SES-K0001");
        let message = error.to_string();
        assert!(message.contains("src/api.ssrg"), "{message}");
        assert!(message.contains("gen/api.ssrg"), "{message}");
        assert!(matches!(
            error,
            LocalProjectLoadError::DuplicateLogicalModule { .. }
        ));
    }
}

#[test]
fn diagnoses_generated_imports_before_a_generated_directory_exists() {
    let project = TempProject::new();
    project.write("seseragi.toml", "[package]\nname=\"fixture/missing-generated\"\nversion=\"0.0.0\"\nlanguage=\"^0.1.0\"\n[run]\nentry=\"main\"\n");
    project.write(
        "src/main.ssrg",
        "import { value } from \"gen/api\"\npub let main = value\n",
    );
    let error = load_local_project(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-N0104");
    match error {
        LocalProjectLoadError::Import {
            module,
            specifier,
            origin,
            reason,
            ..
        } => {
            assert_eq!(module.path().as_str(), "main");
            assert_eq!(specifier, "gen/api");
            assert!(origin.end > origin.start);
            assert!(reason.contains("Generated"));
            assert!(reason.contains("missing"));
        }
        other => panic!("unexpected error: {other}"),
    }
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
