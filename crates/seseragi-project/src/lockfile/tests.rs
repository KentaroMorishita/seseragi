use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn reads_unordered_toml_and_writes_canonical_order() {
    let source = format!(
        concat!(
            "root = \"acme/app@1.0.0#workspace:.\"\n",
            "timezone_database = \"2025b\"\n",
            "unicode = \"16.0.0\"\n",
            "standard_library = \"0.1.0\"\n",
            "language = \"0.1.0\"\n",
            "schema = 1\n",
            "unknown_future_key = true\n\n",
            "[[packages]]\n",
            "dependencies = []\n",
            "content_digest = \"sha256:{ZERO}\"\n",
            "manifest_digest = \"sha256:{ZERO}\"\n",
            "source = \".\"\n",
            "source_kind = \"workspace\"\n",
            "version = \"1.0.0\"\n",
            "name = \"acme/app\"\n",
            "id = \"acme/app@1.0.0#workspace:.\"\n",
        ),
        ZERO = ZERO,
    );
    let lockfile = parse_lockfile(&source).unwrap();
    let written = write_lockfile(&lockfile);

    assert!(written.starts_with("schema = 1\nlanguage = \"0.1.0\"\nstandard_library = \"0.1.0\"\n"));
    assert!(written.ends_with("dependencies = []\n"));
    assert!(!written.contains("unknown_future_key"));
    assert_eq!(parse_lockfile(&written).unwrap(), lockfile);
}

#[test]
fn rejects_unknown_schema_dangling_edges_and_identity_confusion() {
    let valid = single_package_lock();
    assert!(matches!(
        parse_lockfile(&valid.replace("schema = 1", "schema = 2")),
        Err(LockError::UnsupportedSchema(2))
    ));
    let dangling = valid.replace(
        "dependencies = []",
        "dependencies = [{ import = \"dep\", package = \"acme/missing@1.0.0#registry:default\" }]",
    );
    assert!(matches!(
        parse_lockfile(&dangling),
        Err(LockError::DanglingDependency { .. })
    ));

    let duplicate = format!(
        "{valid}\n[[packages]]\nid = \"acme/app@1.0.0#registry:default\"\nname = \"acme/app\"\nversion = \"1.0.0\"\nsource_kind = \"registry\"\nsource = \"default\"\nmanifest_digest = \"sha256:{ZERO}\"\ncontent_digest = \"sha256:{ZERO}\"\ndependencies = []\n"
    );
    assert!(matches!(
        parse_lockfile(&duplicate),
        Err(LockError::DuplicateIdentity(_))
    ));

    let exact_duplicate = format!(
        "{valid}\n[[packages]]{}",
        valid.split("[[packages]]").nth(1).unwrap()
    );
    assert!(matches!(
        parse_lockfile(&exact_duplicate),
        Err(LockError::DuplicatePackage(_))
    ));

    let second_workspace = format!(
        "{}\n[[packages]]\nid = \"acme/other@1.0.0#workspace:.\"\nname = \"acme/other\"\nversion = \"1.0.0\"\nsource_kind = \"workspace\"\nsource = \".\"\nmanifest_digest = \"sha256:{ZERO}\"\ncontent_digest = \"sha256:{ZERO}\"\ndependencies = []\n",
        valid.replace(
            "dependencies = []",
            "dependencies = [{ import = \"other\", package = \"acme/other@1.0.0#workspace:.\" }]"
        )
    );
    assert!(matches!(
        parse_lockfile(&second_workspace),
        Err(LockError::InvalidField { .. })
    ));

    let cyclic = format!(
        "{}\n[[packages]]\nid = \"acme/dep@1.0.0#registry:default\"\nname = \"acme/dep\"\nversion = \"1.0.0\"\nsource_kind = \"registry\"\nsource = \"default\"\nmanifest_digest = \"sha256:{ZERO}\"\ncontent_digest = \"sha256:{ZERO}\"\ndependencies = [{{ import = \"app\", package = \"acme/app@1.0.0#workspace:.\" }}]\n",
        valid.replace(
            "dependencies = []",
            "dependencies = [{ import = \"dep\", package = \"acme/dep@1.0.0#registry:default\" }]"
        )
    );
    let error = parse_lockfile(&cyclic).unwrap_err();
    assert!(error.to_string().contains("cycle"));
}

#[test]
fn generates_and_validates_a_path_dependency_graph() {
    let project = TempProject::new();
    project.write(
        "seseragi.toml",
        &manifest(
            "acme/app",
            "dep = { package = \"acme/dep\", path = \"vendor/dep\" }",
            true,
        ),
    );
    project.write(
        "src/main.ssrg",
        "import { value } from \"dep\"\npub let main = value\n",
    );
    project.write("vendor/dep/seseragi.toml", &manifest("acme/dep", "", false));
    project.write("vendor/dep/src/lib.ssrg", "pub let value: Int = 42\n");

    let lockfile = generate_lockfile(project.path()).unwrap();
    assert_eq!(lockfile.packages.len(), 2);
    assert_eq!(lockfile.packages[0].id, "acme/app@1.0.0#workspace:.");
    assert_eq!(lockfile.packages[1].source, "vendor/dep");
    project.write("seseragi.lock", &write_lockfile(&lockfile));
    read_and_validate_lockfile(project.path()).unwrap();

    project.write("vendor/dep/src/lib.ssrg", "pub let value: Int = 43\n");
    let error = read_and_validate_lockfile(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0102");
    assert!(error.to_string().contains("content digest"));

    project.write(
        "vendor/dep/seseragi.toml",
        &manifest("acme/dep", "", false).replace("^0.1.0", "^9.0.0"),
    );
    let error = read_and_validate_lockfile(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0102");
    assert!(error.to_string().contains("requires Seseragi"), "{error}");
}

#[test]
fn missing_lock_is_a_stale_lock_diagnostic() {
    let project = TempProject::new();
    project.write("seseragi.toml", &manifest("acme/app", "", true));
    project.write("src/main.ssrg", "pub let main = ()\n");
    let error = read_and_validate_lockfile(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0102");
    assert!(error.to_string().contains("lock update"));
}

#[test]
fn canonical_stale_lock_fixture_is_executed_by_the_project_loader() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/projects/package-stale-lock");
    let error = read_and_validate_lockfile(root).unwrap_err();
    assert_eq!(error.code(), "SES-K0102");
    assert!(error.to_string().contains("manifest digest"));
}

#[test]
fn canonical_current_packages_commit_fresh_locks() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative in [
        "examples/samples/project-flow-app",
        "examples/samples/project-greeting",
        "examples/samples/seseragi-landing-page",
        "examples/samples/web-starter",
        "examples/spec/fixtures/projects/cli-build-nested",
        "examples/spec/fixtures/projects/child-process-captured",
        "examples/spec/fixtures/projects/dom-hydration-mismatch",
        "examples/spec/fixtures/projects/dom-reactive-bindings",
        "examples/spec/fixtures/projects/dom-signal-lifecycle",
        "examples/spec/fixtures/projects/effect-resource-scope",
        "examples/spec/fixtures/projects/effect-temporal-control",
        "examples/spec/fixtures/projects/entry-rooted-runtime",
        "examples/spec/fixtures/projects/imported-derived-json-codecs",
        "examples/spec/fixtures/projects/logical-short-circuit",
        "examples/spec/fixtures/projects/module-generic-nominal-identity",
        "examples/spec/fixtures/projects/namespaced-reduce-rejection",
        "examples/spec/fixtures/projects/package-path-dependency",
        "examples/spec/fixtures/projects/package-path-dependency-basic",
        "examples/spec/fixtures/projects/prelude-reduce-lambda",
        "examples/spec/fixtures/projects/provider-http-client-e2e",
        "examples/spec/fixtures/projects/provider-http-server-e2e",
        "examples/spec/fixtures/projects/provider-websocket-e2e",
        "examples/spec/fixtures/projects/struct-field-generic-identity",
        "examples/spec/fixtures/projects/std-parity-portable",
        "examples/spec/fixtures/projects/std-parity-target",
    ] {
        read_and_validate_lockfile(repository.join(relative))
            .unwrap_or_else(|error| panic!("{relative}: {error}"));
    }
}

#[test]
fn validates_a_mixed_path_and_registry_graph_with_semver_ranges() {
    let project = TempProject::new();
    project.write(
        "seseragi.toml",
        &manifest(
            "acme/app",
            "local = { package = \"acme/local\", path = \"vendor/local\" }",
            true,
        ),
    );
    project.write("src/main.ssrg", "pub effect fn main = println \"ready\"\n");
    project.write(
        "vendor/local/seseragi.toml",
        "[package]\nname = \"acme/local\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[exports]\n\".\" = \"lib\"\n\n[dependencies]\nhttp = { package = \"acme/http\", version = \"^2.1.0\" }\n",
    );
    project.write("vendor/local/src/lib.ssrg", "pub let value: Int = 1\n");

    let root_manifest =
        crate::parse_manifest(&fs::read_to_string(project.path().join("seseragi.toml")).unwrap())
            .unwrap();
    let local_root = project.path().join("vendor/local");
    let local_manifest =
        crate::parse_manifest(&fs::read_to_string(local_root.join("seseragi.toml")).unwrap())
            .unwrap();
    let root_digests =
        super::digest::package_digests(project.path(), &root_manifest.layout).unwrap();
    let local_digests =
        super::digest::package_digests(&local_root, &local_manifest.layout).unwrap();
    let http_id = "acme/http@2.1.4#registry:default".to_owned();
    let lockfile = Lockfile {
        schema: 1,
        language: semver::Version::new(0, 1, 0),
        standard_library: semver::Version::new(0, 1, 0),
        unicode: "16.0.0".to_owned(),
        timezone_database: "2025b".to_owned(),
        root: "acme/app@1.0.0#workspace:.".to_owned(),
        packages: vec![
            LockPackage {
                id: "acme/app@1.0.0#workspace:.".to_owned(),
                name: crate::PackageName::parse("acme/app").unwrap(),
                version: semver::Version::new(1, 0, 0),
                source_kind: LockSourceKind::Workspace,
                source: ".".to_owned(),
                manifest_digest: root_digests.manifest,
                content_digest: root_digests.content,
                dependencies: vec![LockDependency {
                    import: "local".to_owned(),
                    package: "acme/local@1.0.0#path:vendor/local".to_owned(),
                }],
            },
            LockPackage {
                id: http_id.clone(),
                name: crate::PackageName::parse("acme/http").unwrap(),
                version: semver::Version::new(2, 1, 4),
                source_kind: LockSourceKind::Registry,
                source: "default".to_owned(),
                manifest_digest: format!("sha256:{ZERO}"),
                content_digest: format!("sha256:{ZERO}"),
                dependencies: Vec::new(),
            },
            LockPackage {
                id: "acme/local@1.0.0#path:vendor/local".to_owned(),
                name: crate::PackageName::parse("acme/local").unwrap(),
                version: semver::Version::new(1, 0, 0),
                source_kind: LockSourceKind::Path,
                source: "vendor/local".to_owned(),
                manifest_digest: local_digests.manifest,
                content_digest: local_digests.content,
                dependencies: vec![LockDependency {
                    import: "http".to_owned(),
                    package: http_id,
                }],
            },
        ],
        providers: Vec::new(),
        foreign_modules: Vec::new(),
    };
    project.write("seseragi.lock", &write_lockfile(&lockfile));
    read_and_validate_lockfile(project.path()).unwrap();

    let invalid = fs::read_to_string(project.path().join("seseragi.lock"))
        .unwrap()
        .replace("acme/http@2.1.4", "acme/http@3.0.0")
        .replace("version = \"2.1.4\"", "version = \"3.0.0\"");
    project.write("seseragi.lock", &invalid);
    let error = read_and_validate_lockfile(project.path()).unwrap_err();
    assert_eq!(error.code(), "SES-K0102");
    assert!(error.to_string().contains("does not match locked 3.0.0"));
}

#[test]
fn round_trips_provider_selection_metadata_in_the_same_lock_contract() {
    let source = format!(
        "{}\n[[providers]]\nfield = \"httpClient\"\nservice = \"std/http::HttpClient\"\nrequired_contract = \"1.0\"\nprovider_contract = \"1.1\"\nprovider = \"seseragi/runtime-node#http-client\"\npackage_version = \"0.17.0\"\npackage_source = \"toolchain:seseragi/runtime-node@0.17.0\"\npackage_digest = \"sha256:{ZERO}\"\nartifact_digest = \"sha256:{ZERO}\"\nbackend = \"typescript\"\nbackend_abi_major = 1\ntarget = \"node-process\"\nentry_module = \"@seseragi/providers/runtime-node/http-client\"\nentry_export = \"default\"\nruntime_features = [\"foreign.task-load\"]\nhost_packages = [\n  {{ name = \"acme/undici\", version = \"7.0.0\", source = \"registry:default\", content_digest = \"sha256:{ZERO}\" }},\n]\n",
        single_package_lock()
    );
    let lockfile = parse_lockfile(&source).unwrap();
    assert_eq!(lockfile.providers.len(), 1);
    assert_eq!(lockfile.providers[0].service, "std/http::HttpClient");
    assert_eq!(lockfile.providers[0].host_packages[0].name, "acme/undici");
    let written = write_lockfile(&lockfile);
    assert!(written.contains("[[providers]]"));
    assert_eq!(parse_lockfile(&written).unwrap(), lockfile);
    assert!(!written.contains("/Users/") && !written.contains("C:\\"));

    let mut unordered = lockfile.clone();
    let mut browser = unordered.providers[0].clone();
    browser.target = "browser-window".to_owned();
    unordered.providers.push(browser);
    let ordered = write_lockfile(&unordered);
    assert!(
        ordered.find("target = \"browser-window\"").unwrap()
            < ordered.find("target = \"node-process\"").unwrap()
    );

    let machine_specific = source.replace(
        "toolchain:seseragi/runtime-node@0.17.0",
        "cache\\\\runtime-node",
    );
    let error = parse_lockfile(&machine_specific).unwrap_err();
    assert_eq!(error.code(), "SES-K0001");
    assert!(error.to_string().contains("machine-independent"));
}

fn single_package_lock() -> String {
    format!(
        "schema = 1\nlanguage = \"0.1.0\"\nstandard_library = \"0.1.0\"\nunicode = \"16.0.0\"\ntimezone_database = \"2025b\"\nroot = \"acme/app@1.0.0#workspace:.\"\n\n[[packages]]\nid = \"acme/app@1.0.0#workspace:.\"\nname = \"acme/app\"\nversion = \"1.0.0\"\nsource_kind = \"workspace\"\nsource = \".\"\nmanifest_digest = \"sha256:{ZERO}\"\ncontent_digest = \"sha256:{ZERO}\"\ndependencies = []\n"
    )
}

fn manifest(name: &str, dependencies: &str, executable: bool) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[exports]\n\".\" = \"{}\"\n\n[dependencies]\n{dependencies}\n{}",
        if executable { "main" } else { "lib" },
        if executable {
            "\n[run]\nentry = \"main\"\ntarget = \"process\"\n"
        } else {
            ""
        }
    )
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
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "seseragi-lockfile-{}-{nonce}-{sequence}",
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
        fs::remove_dir_all(&self.path).unwrap();
    }
}
