use super::*;
use crate::ModulePath;
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
fn discovers_the_split_rps_goal_from_manifest_and_imports() {
    let root = repository_root()
        .join("examples/spec/artifacts/project-schema-1/rock-paper-scissors-cli-split");
    let package = load_package(root).unwrap();

    assert_eq!(package.entry().as_str(), "main");
    assert_eq!(
        package.manifest().package.name.as_str(),
        "fixture/rps-cli-split"
    );
    assert_eq!(
        package.graph().topological_order().unwrap(),
        [
            ModulePath::parse("domain").unwrap(),
            ModulePath::parse("input").unwrap(),
            ModulePath::parse("main").unwrap(),
        ]
    );
    assert_eq!(package.modules().count(), 3);
    assert_eq!(package.identity().version().to_string(), "0.0.0");
    assert!(package
        .module(&ModulePath::parse("main").unwrap())
        .unwrap()
        .source()
        .contains("pub effect fn main"));
    assert_eq!(
        package
            .module(&ModulePath::parse("main").unwrap())
            .unwrap()
            .identity()
            .package(),
        package.identity()
    );
}

#[test]
fn rejects_case_mismatch_in_module_spelling() {
    let root = TempPackage::new();
    root.write("seseragi.toml", &manifest("main"));
    root.write("src/Main.ssrg", "pub fn value: Int = 1\n");

    let error = load_package(root.path()).unwrap_err();
    assert!(matches!(
        error,
        PackageLoadError::CaseMismatch { ref expected, ref actual, .. }
            if expected == "main.ssrg" && actual == "Main.ssrg"
    ));
}

#[test]
fn rejects_symlink_escape_from_the_source_root() {
    let root = TempPackage::new();
    root.write("seseragi.toml", &manifest("main"));
    root.write("outside.ssrg", "pub fn value: Int = 1\n");
    fs::create_dir_all(root.path().join("src")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("../outside.ssrg", root.path().join("src/main.ssrg")).unwrap();

    let error = load_package(root.path()).unwrap_err();
    assert!(matches!(error, PackageLoadError::RootEscape { .. }));
}

#[test]
fn rejects_an_incompatible_language_version_before_reading_source() {
    let root = TempPackage::new();
    root.write(
        "seseragi.toml",
        "[package]\nname = \"fixture/loader-test\"\nversion = \"0.0.0\"\nlanguage = \"^2.0.0\"\n\n[run]\nentry = \"missing\"\n",
    );

    let error = load_package(root.path()).unwrap_err();
    assert!(matches!(
        error,
        PackageLoadError::UnsupportedLanguageVersion {
            ref requirement,
            ref implemented,
        } if requirement == "^2.0.0" && implemented == IMPLEMENTED_LANGUAGE_VERSION
    ));
    assert_eq!(error.code(), "SES-K0101");
}

fn manifest(entry: &str) -> String {
    format!(
        "[package]\nname = \"fixture/loader-test\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n\n[run]\nentry = \"{entry}\"\n"
    )
}

struct TempPackage {
    path: PathBuf,
}

impl TempPackage {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "seseragi-package-loader-{}-{nonce}",
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

impl Drop for TempPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
