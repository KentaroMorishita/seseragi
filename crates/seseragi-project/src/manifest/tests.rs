use super::{parse_manifest, ManifestDependency, ManifestError, RunSeed};
use semver::Version;

#[test]
fn parses_executable_manifest_with_canonical_defaults() {
    let manifest = parse_manifest(include_str!(
        "../../../../examples/spec/fixtures/projects/modules-reexport-run/seseragi.toml"
    ))
    .unwrap();

    assert_eq!(
        manifest.package.name.as_str(),
        "fixture/modules-reexport-run"
    );
    assert_eq!(manifest.package.version, Version::new(0, 0, 0));
    assert_eq!(manifest.package.language.as_str(), ">=0.1.0 <0.2.0");
    assert_eq!(manifest.layout.source.as_str(), "src");
    assert_eq!(manifest.layout.generated.as_str(), ".seseragi/generated");
    assert_eq!(manifest.exports["."].as_str(), "facade");
    let run = manifest.run.expect("run table");
    assert_eq!(run.entry.as_str(), "main");
    assert_eq!(run.target.unwrap().as_str(), "test-js");
    assert_eq!(run.shutdown_grace_ms, Some(10_000));
    assert_eq!(run.hash_seed, RunSeed::Entropy);
}

#[test]
fn rejects_unknown_core_tables_at_the_toml_range() {
    let source = include_str!(
        "../../../../examples/spec/fixtures/projects/package-invalid-manifest/seseragi.toml"
    );
    let error = parse_manifest(source).unwrap_err();

    assert_eq!(error.code(), "SES-K0101");
    let range = error.range().expect("TOML decoder provides a source range");
    assert_eq!(&source[range], "[compiler]");
}

#[test]
fn rejects_overlapping_layout_roots() {
    let error = parse_manifest(
        "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[layout]\nsource = \"src\"\ntests = \"src/tests\"\n",
    )
    .unwrap_err();

    assert_eq!(
        error,
        ManifestError::OverlappingLayoutRoots {
            left: "layout.source",
            right: "layout.tests"
        }
    );
}

#[test]
fn rejects_noncanonical_entry_target_and_run_policy() {
    for (run, expected) in [
        ("entry = \"main.ssrg\"\n", "invalid run entry `main.ssrg`"),
        (
            "entry = \"main\"\ntarget = \"Node\"\n",
            "invalid target id `Node`",
        ),
        (
            "entry = \"main\"\nsignal_mode = \"forward\"\nshutdown_grace_ms = 1\n",
            "run.shutdown_grace_ms is only valid",
        ),
    ] {
        let source = format!(
            "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"~0.1.0\"\n\n[run]\n{run}"
        );
        let error = parse_manifest(&source).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn parses_path_dependencies_without_assigning_resolved_identity() {
    let manifest = parse_manifest(include_str!(
        "../../../../examples/spec/fixtures/projects/package-path-dependency/seseragi.toml"
    ))
    .unwrap();

    assert_eq!(manifest.dependencies.len(), 1);
    let (key, dependency) = manifest.dependencies.first_key_value().unwrap();
    assert_eq!(key.as_str(), "math");
    assert!(matches!(
        dependency,
        ManifestDependency::Path { package, path }
            if package.as_str() == "fixture/math" && path.as_str() == "vendor/math"
    ));
    assert!(manifest.deferred.foreign.is_none());
    assert!(manifest.test.is_none());
    assert!(manifest.deferred.benchmark.is_none());
    assert!(manifest.deferred.tool.is_none());
}

#[test]
fn parses_test_settings_and_rejects_non_positive_limits() {
    let manifest = parse_manifest(
        "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[test]\ntarget = \"node\"\njobs = 4\ntimeout_ms = 250\ncleanup_grace_ms = 10\nseed = -7\n",
    )
    .unwrap();
    let test = manifest.test.expect("test table");
    assert_eq!(test.target.unwrap().as_str(), "node");
    assert_eq!(test.jobs, 4);
    assert_eq!(test.timeout_ms, 250);
    assert_eq!(test.cleanup_grace_ms, 10);
    assert_eq!(test.seed, -7);

    for setting in ["jobs = 0", "timeout_ms = 0"] {
        let source = format!(
            "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[test]\n{setting}\n"
        );
        assert!(matches!(
            parse_manifest(&source),
            Err(ManifestError::InvalidTestSetting(_))
        ));
    }
}

#[test]
fn parses_short_and_aliased_registry_dependencies() {
    let manifest = parse_manifest(
        "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[dependencies]\n\"acme/json\" = \"~1.4.2\"\nhttp = { package = \"acme/http\", version = \"^2.1.0\" }\n",
    )
    .unwrap();

    assert!(matches!(
        manifest.dependencies.get("acme/json").unwrap(),
        ManifestDependency::Registry { package, version }
            if package.as_str() == "acme/json" && version.as_str() == "~1.4.2"
    ));
    assert!(matches!(
        manifest.dependencies.get("http").unwrap(),
        ManifestDependency::Registry { package, version }
            if package.as_str() == "acme/http" && version.as_str() == "^2.1.0"
    ));
}

#[test]
fn rejects_ambiguous_or_invalid_dependency_contracts() {
    for (dependency, expected) in [
        ("std = \"^1.0.0\"", "invalid dependency key"),
        (
            "\"acme/app\" = \"^1.0.0\"",
            "conflicts with the current package",
        ),
        (
            "http = { package = \"Acme/http\", version = \"^1.0.0\" }",
            "invalid package name",
        ),
        (
            "http = { package = \"acme/http\", version = \"1.*\" }",
            "invalid version requirement",
        ),
        (
            "http = { package = \"acme/http\" }",
            "must specify exactly one",
        ),
        (
            "http = { package = \"acme/http\", version = \"^1.0.0\", path = \"../http\" }",
            "cannot specify both",
        ),
        (
            "http = { package = \"acme/http\", path = \"/tmp/http\" }",
            "not package-relative",
        ),
    ] {
        let source = format!(
            "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[dependencies]\n{dependency}\n"
        );
        let error = parse_manifest(&source).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn parses_provider_artifacts_and_root_selections_without_resolving_them() {
    let manifest = parse_manifest(
        "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n[provider]\nartifacts = [\"providers/http.json\"]\n\n[providers]\n\"std/http::HttpClient\" = \"acme/undici-provider#http-client\"\n",
    )
    .unwrap();
    assert_eq!(
        manifest.provider_artifacts[0].as_str(),
        "providers/http.json"
    );
    assert_eq!(
        manifest.providers["std/http::HttpClient"],
        "acme/undici-provider#http-client"
    );
}

#[test]
fn rejects_unsafe_duplicate_artifacts_and_invalid_provider_selections() {
    for tables in [
        "[provider]\nartifacts = [\"../provider.json\"]\n",
        "[provider]\nartifacts = [\"providers/a.json\", \"providers/a.json\"]\n",
        "[provider]\nartifacts = [\"C:/provider.json\"]\n",
        "[providers]\nClock = \"acme/provider#clock\"\n",
        "[providers]\n\"typescript/http::HttpClient\" = \"acme/provider#http-client\"\n",
        "[providers]\n\"std/http::Http_Client\" = \"acme/provider#http-client\"\n",
        "[providers]\n\"std/clock::Clock\" = \"invalid\"\n",
        "[providers]\n\"std/clock::Clock\" = \"acme/provider#2clock\"\n",
    ] {
        let source = format!(
            "[package]\nname = \"acme/app\"\nversion = \"1.0.0\"\nlanguage = \"^0.1.0\"\n\n{tables}"
        );
        assert!(parse_manifest(&source).is_err());
    }
}
