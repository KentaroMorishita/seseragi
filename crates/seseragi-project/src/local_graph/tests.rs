use super::*;
use crate::ModuleGraphError;
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
fn discovers_the_canonical_path_dependency_graph() {
    let graph = discover_local_package_graph(
        repository_root().join("examples/spec/fixtures/projects/package-path-dependency"),
    )
    .unwrap();

    assert_eq!(
        graph.root().name().as_str(),
        "fixture/package-path-dependency"
    );
    assert_eq!(graph.packages().count(), 2);
    let order = graph.graph().topological_order().unwrap();
    assert_eq!(order[0].name().as_str(), "fixture/math");
    assert_eq!(&order[1], graph.root());
    let dependencies = graph.graph().dependencies_for(graph.root()).unwrap();
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].0, "math");
    assert_eq!(dependencies[0].1.name().as_str(), "fixture/math");
}

#[test]
fn resolves_root_and_subpath_exports_through_the_declared_key() {
    let graph = discover_local_package_graph(
        repository_root().join("examples/spec/fixtures/projects/package-path-dependency"),
    )
    .unwrap();

    let root = graph.root();
    let root_export = graph.resolve_package_import(root, "math").unwrap();
    assert_eq!(root_export.dependency_key(), "math");
    assert_eq!(root_export.export_key(), ".");
    assert_eq!(root_export.package().name().as_str(), "fixture/math");
    assert_eq!(root_export.module().as_str(), "lib");

    let stats = graph.resolve_package_import(root, "math/stats").unwrap();
    assert_eq!(stats.export_key(), "stats");
    assert_eq!(stats.module().as_str(), "stats");

    let undeclared = graph
        .resolve_package_import(root, "other/stats")
        .unwrap_err();
    assert_eq!(undeclared.code(), "SES-K0103");
    let private = graph
        .resolve_package_import(root, "math/private")
        .unwrap_err();
    assert_eq!(private.code(), "SES-N0104");
}

#[test]
fn chooses_the_longest_declared_dependency_key() {
    let root = TempGraph::new();
    root.write(
        "seseragi.toml",
        &manifest(
            "fixture/app",
            concat!(
                "acme = { package = \"fixture/base\", path = \"base\" }\n",
                "\"acme/http\" = { package = \"fixture/http\", path = \"http\" }",
            ),
        ),
    );
    root.write(
        "base/seseragi.toml",
        &library_manifest("fixture/base", &[("http/client", "wrong")]),
    );
    root.write(
        "http/seseragi.toml",
        &library_manifest("fixture/http", &[("client", "http/client")]),
    );
    let graph = discover_local_package_graph(root.path()).unwrap();

    let resolved = graph
        .resolve_package_import(graph.root(), "acme/http/client")
        .unwrap();
    assert_eq!(resolved.dependency_key(), "acme/http");
    assert_eq!(resolved.package().name().as_str(), "fixture/http");
    assert_eq!(resolved.module().as_str(), "http/client");
}

#[test]
fn rejects_a_path_whose_manifest_declares_another_package() {
    let root = TempGraph::new();
    root.write(
        "seseragi.toml",
        &manifest(
            "fixture/app",
            "dep = { package = \"fixture/expected\", path = \"dep\" }",
        ),
    );
    root.write("dep/seseragi.toml", &manifest("fixture/actual", ""));

    let error = discover_local_package_graph(root.path()).unwrap_err();
    assert!(matches!(
        error,
        LocalPackageGraphError::DependencyNameMismatch { expected, actual, .. }
            if expected.as_str() == "fixture/expected" && actual.as_str() == "fixture/actual"
    ));
}

#[test]
fn rejects_a_cycle_between_local_packages() {
    let root = TempGraph::new();
    root.write(
        "seseragi.toml",
        &manifest(
            "fixture/app",
            "dep = { package = \"fixture/dep\", path = \"dep\" }",
        ),
    );
    root.write(
        "dep/seseragi.toml",
        &manifest(
            "fixture/dep",
            "app = { package = \"fixture/app\", path = \"..\" }",
        ),
    );

    let error = discover_local_package_graph(root.path()).unwrap_err();
    assert!(matches!(
        error,
        LocalPackageGraphError::Graph(error)
            if matches!(*error, ModuleGraphError::Cycle { ref modules } if modules.len() == 2)
    ));
}

#[test]
fn rejects_the_same_package_version_from_different_paths() {
    let root = TempGraph::new();
    root.write(
        "seseragi.toml",
        &manifest(
            "fixture/app",
            concat!(
                "first = { package = \"fixture/shared\", path = \"first\" }\n",
                "second = { package = \"fixture/shared\", path = \"second\" }",
            ),
        ),
    );
    root.write("first/seseragi.toml", &manifest("fixture/shared", ""));
    root.write("second/seseragi.toml", &manifest("fixture/shared", ""));

    let error = discover_local_package_graph(root.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0104");
    assert!(matches!(
        error,
        LocalPackageGraphError::DependencyConfusion { first, second }
            if first.name() == second.name()
                && first.version() == second.version()
                && first.source() != second.source()
    ));
}

#[test]
fn keeps_registry_resolution_out_of_the_local_path_graph() {
    let root = TempGraph::new();
    root.write(
        "seseragi.toml",
        &manifest("fixture/app", "http = \"^1.0.0\""),
    );

    let error = discover_local_package_graph(root.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0102");
    assert!(matches!(
        error,
        LocalPackageGraphError::RegistryDependencyUnsupported { key, .. } if key == "http"
    ));
}

fn manifest(name: &str, dependencies: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[dependencies]\n{dependencies}\n"
    )
}

fn library_manifest(name: &str, exports: &[(&str, &str)]) -> String {
    let exports = exports
        .iter()
        .map(|(key, module)| format!("\"{key}\" = \"{module}\""))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[exports]\n{exports}\n"
    )
}

struct TempGraph {
    path: PathBuf,
}

impl TempGraph {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "seseragi-local-package-graph-{}-{nonce}",
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

impl Drop for TempGraph {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
