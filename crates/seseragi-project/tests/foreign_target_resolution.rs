use seseragi_project::*;
use std::collections::BTreeMap;

#[path = "../../test-support/temporary.rs"]
mod temporary_support;

#[test]
fn snapshot_runtime_identity_and_transitive_imports_match_shared_selector() {
    let files=BTreeMap::from([
        ("host/node_modules/api/package.json".into(),r#"{"name":"api","version":"1.2.3","exports":{"browser":"./web.js","node":"./node.js"}}"#.into()),
        ("host/node_modules/api/web.js".into(),"import {value} from './shared.js'; export const run = () => value;".into()),
        ("host/node_modules/api/shared.js".into(),"export const value = 42;".into()),
        ("host/node_modules/api/node.js".into(),"import fs from 'node:fs'; export const run = fs;".into()),
    ]);
    let imports = [VirtualRuntimeImport {
        importer: "generated/api.js".into(),
        specifier: "api".into(),
    }];
    let bundle = resolve_browser_runtime_snapshot(&files, &imports, "host").unwrap();
    assert_eq!(bundle.modules.len(), 2);
    let web = &bundle.modules["host/node_modules/api/web.js"];
    assert_eq!(web.exact_identity, "npm:api@1.2.3/web.js");
    assert_eq!(web.content_digest.len(), 71);
    assert_eq!(
        web.imports["./shared.js"],
        "host/node_modules/api/shared.js"
    );
    let mut changed = files;
    changed.insert(
        "host/node_modules/api/shared.js".into(),
        "import fs from 'node:fs'; export const value = fs;".into(),
    );
    let errors = resolve_browser_runtime_snapshot(&changed, &imports, "host").unwrap_err();
    assert_eq!(errors[0].code, "SES-F0203");
    assert!(errors[0].reason.contains("node:fs"));
}
#[test]
fn import_parser_ignores_type_dependencies_comments_and_strings() {
    let imports=runtime_import_specifiers("import type {A} from 'node:fs'; // import('fake')\n const s=\"require('fake')\"; export {x} from './real.js'; const load=()=>import('api');").unwrap();
    assert_eq!(imports, vec!["./real.js", "api"]);
    assert!(
        runtime_import_specifiers("const name = 'api'; import(name);")
            .unwrap_err()
            .reason
            .contains("computed")
    );
}
#[test]
fn snapshot_rejects_aliases_and_import_escape() {
    let files = BTreeMap::from([("host/a.js".into(), "export const value=1;".into())]);
    let imports = [VirtualRuntimeImport {
        importer: "main.js".into(),
        specifier: "../host/a.js".into(),
    }];
    assert!(resolve_browser_runtime_snapshot(&files, &imports, "").is_err());
    let aliases = BTreeMap::from([("host/../host/a.js".into(), "export const value=1;".into())]);
    assert!(resolve_browser_runtime_snapshot(&aliases, &[], "").is_err());
}
#[test]
fn native_resolver_rejects_builtins_for_browser_before_access_or_execution() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let importer = root.join("src/lib.rs");
    let browser = resolve_foreign_typescript_module_for_target(
        root,
        None,
        &importer,
        "node:fs",
        RuntimeTarget::Browser,
    )
    .unwrap_err();
    assert_eq!(browser.code, "SES-F0203");
    for target in [RuntimeTarget::Node, RuntimeTarget::Bun] {
        let native =
            resolve_foreign_typescript_module_for_target(root, None, &importer, "node:fs", target)
                .unwrap();
        assert_eq!(native.exact_identity(), "node:fs");
        assert_eq!(native.content_digest(), "sha256:builtin");
    }
}

struct TempProject(std::path::PathBuf);
impl TempProject {
    fn new() -> Self {
        let root = temporary_support::temporary_directory("seseragi-interop-host");
        Self(root)
    }
    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
}
impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn native_fixture() -> TempProject {
    let project = TempProject::new();
    project.write("seseragi.toml","[package]\nname=\"fixture/host-parity\"\nversion=\"1.0.0\"\nlanguage=\"^0.1.0\"\n[run]\nentry=\"main\"\n[foreign.typescript]\nresolver=\"node\"\nmanifest=\"host/package.json\"\nlockfile=\"host/bun.lock\"\n");
    project.write(
        "host/package.json",
        "{\"name\":\"host-parity\",\"version\":\"1.0.0\"}",
    );
    project.write("host/bun.lock", "{}");
    project.write("src/main.ssrg","foreign \"typescript\" from \"api\" {\n pure fn label value: String -> String\n}\npub fn main -> Unit = ()\n");
    project.write(
        "host/node_modules/api/package.json",
        r#"{"name":"api","version":"1.2.3","exports":{"browser":"./web.js","node":"./native.js"}}"#,
    );
    project.write(
        "host/node_modules/api/web.js",
        "export function label(value) {return value;}",
    );
    project.write(
        "host/node_modules/api/native.js",
        "export function label(value) {return value;}",
    );
    project
}

#[test]
fn nested_package_versions_and_exports_match_node_and_browser_snapshots() {
    for name in ["inner", "@scope/inner"] {
        let project = TempProject::new();
        let specifier = format!("{name}/feature");
        let nested = format!("host/node_modules/outer/node_modules/{name}");
        let top = format!("host/node_modules/{name}");
        let manifest = "[package]\nname='fixture/nested-host'\nversion='1.0.0'\nlanguage='^0.1.0'\n[run]\nentry='main'\n[foreign.typescript]\nresolver='node'\nmanifest='host/package.json'\nlockfile='host/bun.lock'\n";
        let mut files = BTreeMap::from([
            ("seseragi.toml".into(), manifest.into()),
            (
                "host/package.json".into(),
                r#"{"name":"nested-host","version":"1.0.0"}"#.into(),
            ),
            ("host/bun.lock".into(), "{}".into()),
            (
                "host/node_modules/outer/package.json".into(),
                r#"{"name":"outer","version":"1.0.0","type":"module","exports":"./index.js"}"#
                    .into(),
            ),
            (
                "host/node_modules/outer/index.js".into(),
                format!(
                    "import {{value}} from '{specifier}'; export function label(){{return value;}}"
                ),
            ),
            (
                format!("{nested}/package.json"),
                format!(
                    r#"{{"name":"{name}","version":"2.0.0","type":"module","exports":{{"./feature":{{"browser":"./web.js","node":"./native.js"}}}}}}"#
                ),
            ),
            (
                format!("{nested}/native.js"),
                "export const value = 'nested native';".into(),
            ),
            (
                format!("{nested}/web.js"),
                "export const value = 'nested browser';".into(),
            ),
            (
                format!("{top}/package.json"),
                format!(
                    r#"{{"name":"{name}","version":"1.0.0","type":"module","exports":{{"./feature":{{"browser":"./web.js","node":"./native.js"}}}}}}"#
                ),
            ),
            (
                format!("{top}/native.js"),
                "export const value = 'wrong top-level';".into(),
            ),
            (
                format!("{top}/web.js"),
                "import fs from 'node:fs'; export const value = fs;".into(),
            ),
        ]);
        for (path, source) in &files {
            project.write(path, source);
        }
        let config = parse_manifest(manifest).unwrap();
        let importer = project.0.join("host/node_modules/outer/index.js");
        for target in [
            RuntimeTarget::Node,
            RuntimeTarget::Bun,
            RuntimeTarget::Browser,
        ] {
            let resolved = resolve_foreign_typescript_module_for_target(
                &project.0,
                config.foreign_typescript.as_ref(),
                &importer,
                &specifier,
                target,
            )
            .unwrap();
            let selected = if target == RuntimeTarget::Browser {
                "web.js"
            } else {
                "native.js"
            };
            assert_eq!(
                resolved.exact_identity(),
                format!("npm:{name}@2.0.0/{selected}")
            );
        }
        let imports = [VirtualRuntimeImport {
            importer: "generated/api.js".into(),
            specifier: "outer".into(),
        }];
        let snapshot = resolve_browser_runtime_snapshot(&files, &imports, "host").unwrap();
        assert_eq!(
            snapshot.modules["host/node_modules/outer/index.js"].imports[&specifier],
            format!("{nested}/web.js")
        );
        assert_eq!(
            snapshot.modules[&format!("{nested}/web.js")].exact_identity,
            format!("npm:{name}@2.0.0/web.js")
        );
        assert!(resolve_foreign_runtime_artifact_for_target(
            &project.0,
            config.foreign_typescript.as_ref(),
            &project.0.join("generated/api.js"),
            "outer",
            RuntimeTarget::Browser
        )
        .is_ok());
        // The host engine independently establishes which installed version a
        // real nested ESM import uses; the shared resolver must select the same.
        let node = std::process::Command::new("node").args(["--input-type=module", "--eval", "import {pathToFileURL} from 'node:url'; const api = await import(pathToFileURL(process.argv[1]).href); console.log(api.label());"]).arg(&importer).output().unwrap();
        assert!(
            node.status.success(),
            "{}",
            String::from_utf8_lossy(&node.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&node.stdout), "nested native\n");
        // In the inverse layout, a safe top-level package must not hide a
        // Node-only dependency actually selected by the nested import.
        files.insert(
            format!("{nested}/web.js"),
            "import fs from 'node:fs'; export const value = fs;".into(),
        );
        files.insert(
            format!("{top}/web.js"),
            "export const value = 'safe but wrong';".into(),
        );
        for path in [format!("{nested}/web.js"), format!("{top}/web.js")] {
            project.write(&path, &files[&path]);
        }
        let native_error = resolve_foreign_runtime_artifact_for_target(
            &project.0,
            config.foreign_typescript.as_ref(),
            &project.0.join("generated/api.js"),
            "outer",
            RuntimeTarget::Browser,
        )
        .unwrap_err();
        assert_eq!(native_error.code, "SES-F0203");
        assert!(native_error.reason.contains("node:fs"));
        let virtual_error = resolve_browser_runtime_snapshot(&files, &imports, "host").unwrap_err();
        assert_eq!(virtual_error[0].code, "SES-F0203");
        assert!(virtual_error[0].reason.contains("node:fs"));
        // ESM package traversal includes this intermediate ancestor. Check it
        // against the host engine rather than assuming CommonJS lookup rules.
        std::fs::remove_dir_all(project.0.join(&nested)).unwrap();
        files.retain(|path, _| !path.starts_with(&format!("{nested}/")));
        let decoy = format!("host/node_modules/node_modules/{name}");
        files.insert(format!("{decoy}/package.json"), format!(r#"{{"name":"{name}","version":"99.0.0","type":"module","exports":{{"./feature":"./index.js"}}}}"#));
        files.insert(
            format!("{decoy}/index.js"),
            "import fs from 'node:fs'; export const value = 'decoy';".into(),
        );
        files.insert(
            format!("{top}/web.js"),
            "export const value = 'hoisted browser';".into(),
        );
        files.insert(
            format!("{top}/native.js"),
            "export const value = 'hoisted native';".into(),
        );
        for (path, source) in &files {
            project.write(path, source);
        }
        let intermediate = resolve_foreign_typescript_module_for_target(
            &project.0,
            config.foreign_typescript.as_ref(),
            &importer,
            &specifier,
            RuntimeTarget::Node,
        )
        .unwrap();
        assert_eq!(
            intermediate.exact_identity(),
            format!("npm:{name}@99.0.0/index.js")
        );
        assert!(
            resolve_browser_runtime_snapshot(&files, &imports, "host").unwrap_err()[0]
                .reason
                .contains("node:fs")
        );
        assert!(resolve_foreign_runtime_artifact_for_target(
            &project.0,
            config.foreign_typescript.as_ref(),
            &project.0.join("generated/api.js"),
            "outer",
            RuntimeTarget::Browser,
        )
        .unwrap_err()
        .reason
        .contains("node:fs"));
        let node = std::process::Command::new("node").args(["--input-type=module", "--eval", "import {pathToFileURL} from 'node:url'; const api = await import(pathToFileURL(process.argv[1]).href); console.log(api.label());"]).arg(&importer).output().unwrap();
        assert!(
            node.status.success(),
            "{}",
            String::from_utf8_lossy(&node.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&node.stdout), "decoy\n");
        std::fs::remove_dir_all(project.0.join(&decoy)).unwrap();
        files.retain(|path, _| !path.starts_with(&format!("{decoy}/")));
        let hoisted = resolve_foreign_typescript_module_for_target(
            &project.0,
            config.foreign_typescript.as_ref(),
            &importer,
            &specifier,
            RuntimeTarget::Node,
        )
        .unwrap();
        assert_eq!(
            hoisted.exact_identity(),
            format!("npm:{name}@1.0.0/native.js")
        );
        let browser = resolve_browser_runtime_snapshot(&files, &imports, "host").unwrap();
        assert_eq!(
            browser.modules["host/node_modules/outer/index.js"].imports[&specifier],
            format!("{top}/web.js")
        );
        assert!(resolve_foreign_runtime_artifact_for_target(
            &project.0,
            config.foreign_typescript.as_ref(),
            &project.0.join("generated/api.js"),
            "outer",
            RuntimeTarget::Browser
        )
        .is_ok());
        let node = std::process::Command::new("node").args(["--input-type=module", "--eval", "import {pathToFileURL} from 'node:url'; const api = await import(pathToFileURL(process.argv[1]).href); console.log(api.label());"]).arg(&importer).output().unwrap();
        assert!(
            node.status.success(),
            "{}",
            String::from_utf8_lossy(&node.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&node.stdout), "hoisted native\n");
    }
}
#[test]
fn native_node_and_bun_share_binding_identity_browser_selects_its_declared_export() {
    let fixture = native_fixture();
    let manifest =
        parse_manifest(&std::fs::read_to_string(fixture.0.join("seseragi.toml")).unwrap()).unwrap();
    let config = manifest.foreign_typescript.as_ref();
    let importer = fixture.0.join("src/main.ssrg");
    let node = resolve_foreign_typescript_module_for_target(
        &fixture.0,
        config,
        &importer,
        "api",
        RuntimeTarget::Node,
    )
    .unwrap();
    let bun = resolve_foreign_typescript_module_for_target(
        &fixture.0,
        config,
        &importer,
        "api",
        RuntimeTarget::Bun,
    )
    .unwrap();
    assert_eq!(node, bun);
    assert_eq!(node.exact_identity(), "npm:api@1.2.3/native.js");
    let browser = resolve_foreign_typescript_module_for_target(
        &fixture.0,
        config,
        &importer,
        "api",
        RuntimeTarget::Browser,
    )
    .unwrap();
    assert_eq!(browser.exact_identity(), "npm:api@1.2.3/web.js");
    assert_eq!(node.content_digest(), browser.content_digest());
}
#[test]
fn loaded_project_browser_preflight_checks_transitive_host_api_before_compile() {
    let fixture = native_fixture();
    fixture.write(
        "host/node_modules/api/web.js",
        "export {label} from './nested.js';",
    );
    fixture.write(
        "host/node_modules/api/nested.js",
        "import fs from 'node:fs'; export const label=fs.readFile;",
    );
    let project = load_local_project(&fixture.0).unwrap();
    validate_foreign_runtime_target(&project, RuntimeTarget::Node).unwrap();
    let diagnostics =
        validate_foreign_runtime_target(&project, RuntimeTarget::Browser).unwrap_err();
    assert_eq!(diagnostics[0].code, "SES-F0203");
    assert!(diagnostics[0].reason.contains("node:fs"));
    assert!(diagnostics[0]
        .symbol
        .as_ref()
        .unwrap()
        .contains("main.ssrg#"));
}

#[test]
fn closure_digest_changes_when_only_a_transitive_dependency_changes() {
    let mut files = BTreeMap::from([
        (
            "host/a.js".into(),
            "import {value} from './b.js'; export const run=()=>value;".into(),
        ),
        ("host/b.js".into(), "export const value=1;".into()),
    ]);
    let imports = [VirtualRuntimeImport {
        importer: "main.js".into(),
        specifier: "./host/a.js".into(),
    }];
    let initial = resolve_browser_runtime_snapshot(&files, &imports, "").unwrap();
    files.insert("host/b.js".into(), "export const value=2;".into());
    let changed = resolve_browser_runtime_snapshot(&files, &imports, "").unwrap();
    assert_eq!(
        initial.modules["host/a.js"].content_digest,
        changed.modules["host/a.js"].content_digest
    );
    assert_ne!(initial.snapshot_digest, changed.snapshot_digest);
}

#[test]
fn type_only_named_imports_and_commented_type_statements_do_not_resolve_host_modules() {
    let source="import {type Foo} from 'node:fs'; import /* comment */ type {Bar} from 'node:fs'; export {type Baz} from 'node:fs'; import {type Foo, actual} from 'api';";
    assert_eq!(runtime_import_specifiers(source).unwrap(), vec!["api"]);
    assert_eq!(
        runtime_import_specifiers("const name = import.meta.resolve('api');").unwrap(),
        vec!["api"]
    );
}
