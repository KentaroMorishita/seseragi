#![cfg(feature = "filesystem")]
use seseragi_dts::*;
use std::{fs, path::Path};

const DECLARATION: &str = "export type User = { readonly name: string };";

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn policy(declaration: &str, settings: &str) -> String {
    let graph = interpret_declaration(DECLARATION, declaration, settings).unwrap();
    serde_json::json!({"schema":1,"kind":"seseragi-semantic-projection-policy","producer":{"name":"explicit-publication-input","version":"1"},"graphDigest":graph.digest().unwrap(),"artifactIdentity":graph.identity,"policies":[{"selector":{"kind":"symbol","symbol":"User"},"functionName":"decodeUser","mapping":{"kind":"json-snapshot"}}]}).to_string()
}

#[test]
fn publication_preserves_each_authored_input_and_temporary_path() {
    for input in [
        "policy",
        "declaration",
        "settings",
        "host-manifest",
        "host-lockfile",
        "runtime",
        "temporary-policy",
    ] {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();
        let declaration = if input == "declaration" {
            "generated/api.ssrg"
        } else {
            "host/api.d.ts"
        };
        let bindings = if input == "settings" {
            "generated/api.report.json"
        } else {
            "bindings.toml"
        };
        let host_manifest = if input == "host-manifest" {
            "generated/api.binding.json"
        } else {
            "host/package.json"
        };
        let host_lock = if input == "host-lockfile" {
            "generated/api.report.json"
        } else {
            "host/bun.lock"
        };
        let specifier = if input == "runtime" {
            "./api.ssrg"
        } else {
            "./host-api.js"
        };
        let policy_path = if input == "temporary-policy" {
            format!("generated/api.binding.json.{}.tmp", std::process::id())
        } else {
            "generated/api.binding.json".into()
        };
        let has_policy = matches!(input, "policy" | "temporary-policy");
        let settings = format!("schema=1\n[entries.api]\ndeclaration='{declaration}'\nspecifier='{specifier}'\noutput='api'\nevaluation='pure'\n{}", if has_policy { format!("projection_policy='{policy_path}'\n") } else { String::new() });
        write(root, bindings, &settings);
        write(root, declaration, DECLARATION);
        write(root, host_manifest, "{}");
        write(root, host_lock, "{}");
        write(root, "seseragi.toml", &format!("[package]\nname='fixture/input-collision'\nversion='1.0.0'\nlanguage='^0.1.0'\n[foreign.typescript]\nresolver='node'\nmanifest='{host_manifest}'\nlockfile='{host_lock}'\nbindings='{bindings}'\n"));
        if has_policy {
            write(root, &policy_path, &policy(declaration, &settings));
        }
        if input == "runtime" {
            write(
                root,
                "generated/api.ssrg",
                "export const hostValue = 'authored JavaScript';",
            );
        }
        let protected = match input {
            "policy" | "temporary-policy" => policy_path.as_str(),
            "declaration" => declaration,
            "settings" => bindings,
            "host-manifest" => host_manifest,
            "host-lockfile" => host_lock,
            _ => "generated/api.ssrg",
        };
        let before = fs::read(root.join(protected)).unwrap();
        let error = convert_package(&ConvertRequest {
            package_root: root.into(),
            generated_root: root.join("generated"),
            bindings: bindings.into(),
            host_manifest: host_manifest.into(),
            entry: None,
        })
        .unwrap_err();
        assert!(
            error.to_string().contains("collides with authored input"),
            "{input}: {error}"
        );
        assert_eq!(fs::read(root.join(protected)).unwrap(), before, "{input}");
        for output in [
            "generated/api.ssrg",
            "generated/api.binding.json",
            "generated/api.report.json",
        ] {
            if output != protected {
                assert!(
                    !root.join(output).exists(),
                    "{input}: partial artifact {output}"
                );
            }
        }
    }
}

#[test]
fn later_entry_inputs_are_protected_before_any_entry_or_selection_is_published() {
    for selection in [None, Some("first")] {
        let root = tempfile::tempdir().unwrap();
        let package = root.path();
        let settings = "schema=1\n[entries.first]\ndeclaration='host/first.d.ts'\nspecifier='./first.js'\noutput='first'\n[entries.second]\ndeclaration='host/second.d.ts'\nspecifier='./second.js'\noutput='second'\nprojection_policy='generated/first.binding.json'\n";
        write(package, "bindings.toml", settings);
        write(package, "host/package.json", "{}");
        write(package, "host/first.d.ts", DECLARATION);
        write(package, "host/second.d.ts", DECLARATION);
        let source = policy("host/second.d.ts", settings);
        write(package, "generated/first.binding.json", &source);
        let error = convert_package(&ConvertRequest {
            package_root: package.into(),
            generated_root: package.join("generated"),
            bindings: "bindings.toml".into(),
            host_manifest: "host/package.json".into(),
            entry: selection.map(String::from),
        })
        .unwrap_err();
        assert!(error.to_string().contains("collides with authored input"));
        assert_eq!(
            fs::read_to_string(package.join("generated/first.binding.json")).unwrap(),
            source
        );
        assert!(!package.join("generated/first.ssrg").exists());
        assert!(!package.join("generated/second.ssrg").exists());
    }
}

#[test]
fn unselected_unsafe_output_does_not_block_an_unrelated_selected_entry() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    let settings = "schema=1\n[entries.safe]\ndeclaration='host/safe.d.ts'\nspecifier='./safe.js'\noutput='safe'\n[entries.unsafe]\ndeclaration='generated/unsafe.ssrg'\nspecifier='./unsafe.js'\noutput='unsafe'\n";
    write(package, "bindings.toml", settings);
    write(package, "host/package.json", "{}");
    write(package, "host/safe.d.ts", DECLARATION);
    write(package, "generated/unsafe.ssrg", DECLARATION);
    let outcome = convert_package(&ConvertRequest {
        package_root: package.into(),
        generated_root: package.join("generated"),
        bindings: "bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: Some("safe".into()),
    })
    .unwrap();
    assert!(!outcome.has_errors(), "{:?}", outcome.diagnostics);
    assert_eq!(outcome.converted.len(), 1);
    assert_eq!(outcome.converted[0].id, "safe");
    assert!(package.join("generated/safe.ssrg").is_file());
    assert_eq!(
        fs::read_to_string(package.join("generated/unsafe.ssrg")).unwrap(),
        DECLARATION
    );
    assert!(!package.join("generated/unsafe.binding.json").exists());
}

#[cfg(unix)]
#[test]
fn declaration_only_npm_manifest_is_protected_from_temporary_output_alias() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    write(package, "bindings.toml", "schema=1\n[entries.api]\ndeclaration='host/api.d.ts'\nspecifier='fixture-api'\noutput='api'\n");
    write(package, "host/package.json", "{}");
    write(package, "host/api.d.ts", DECLARATION);
    let manifest = r#"{"name":"fixture-api","version":"1.0.0","types":"index.d.ts"}"#;
    write(
        package,
        "host/node_modules/fixture-api/package.json",
        manifest,
    );
    fs::create_dir_all(package.join("generated")).unwrap();
    symlink(
        package.join("host/node_modules/fixture-api/package.json"),
        package.join(format!(
            "generated/api.binding.json.{}.tmp",
            std::process::id()
        )),
    )
    .unwrap();
    let error = convert_package(&ConvertRequest {
        package_root: package.into(),
        generated_root: package.join("generated"),
        bindings: "bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    })
    .unwrap_err();
    assert!(error.to_string().contains("collides with authored input"));
    assert_eq!(
        fs::read_to_string(package.join("host/node_modules/fixture-api/package.json")).unwrap(),
        manifest
    );
    assert!(!package.join("generated/api.ssrg").exists());
}

#[cfg(unix)]
#[test]
fn symlinked_output_parents_cannot_alias_authored_policy_or_another_output() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    let settings = "schema=1\n[entries.api]\ndeclaration='host/api.d.ts'\nspecifier='./api.js'\noutput='api'\nprojection_policy='host/api.binding.json'\n";
    write(package, "bindings.toml", settings);
    write(package, "host/package.json", "{}");
    write(package, "host/api.d.ts", DECLARATION);
    let source = policy("host/api.d.ts", settings);
    write(package, "host/api.binding.json", &source);
    symlink(package.join("host"), package.join("generated")).unwrap();
    let request = ConvertRequest {
        package_root: package.into(),
        generated_root: package.join("generated"),
        bindings: "bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    };
    assert!(convert_package(&request)
        .unwrap_err()
        .to_string()
        .contains("collides with authored input"));
    assert_eq!(
        fs::read_to_string(package.join("host/api.binding.json")).unwrap(),
        source
    );
    assert!(!package.join("host/api.ssrg").exists());
    fs::remove_file(package.join("generated")).unwrap();
    fs::create_dir_all(package.join("generated/second")).unwrap();
    symlink(
        package.join("generated/second"),
        package.join("generated/first"),
    )
    .unwrap();
    write(package, "host/second.d.ts", DECLARATION);
    write(package, "bindings.toml", "schema=1\n[entries.first]\ndeclaration='host/api.d.ts'\nspecifier='./first.js'\noutput='first/api'\n[entries.second]\ndeclaration='host/second.d.ts'\nspecifier='./second.js'\noutput='second/api'\n");
    assert!(convert_package(&request)
        .unwrap_err()
        .to_string()
        .contains("aliases another output"));
    assert!(!package.join("generated/second/api.ssrg").exists());
}
