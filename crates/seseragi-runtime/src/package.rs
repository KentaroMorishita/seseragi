use std::fs;
use std::path::Path;

const FILES: &[(&str, &str)] = &[
    (
        "package.json",
        include_str!("../../../runtime/ts/package.json"),
    ),
    (
        "src/index.ts",
        include_str!("../../../runtime/ts/src/index.ts"),
    ),
    (
        "src/console.ts",
        include_str!("../../../runtime/ts/src/console.ts"),
    ),
    (
        "src/console-service.ts",
        include_str!("../../../runtime/ts/src/console-service.ts"),
    ),
    (
        "src/effect.ts",
        include_str!("../../../runtime/ts/src/effect.ts"),
    ),
    ("src/int.ts", include_str!("../../../runtime/ts/src/int.ts")),
    (
        "src/number.ts",
        include_str!("../../../runtime/ts/src/number.ts"),
    ),
    (
        "src/float.ts",
        include_str!("../../../runtime/ts/src/float.ts"),
    ),
    (
        "src/string.ts",
        include_str!("../../../runtime/ts/src/string.ts"),
    ),
    (
        "src/array.ts",
        include_str!("../../../runtime/ts/src/array.ts"),
    ),
    (
        "src/collection.ts",
        include_str!("../../../runtime/ts/src/collection.ts"),
    ),
    (
        "src/range.ts",
        include_str!("../../../runtime/ts/src/range.ts"),
    ),
    (
        "src/iterator.ts",
        include_str!("../../../runtime/ts/src/iterator.ts"),
    ),
    (
        "src/list.ts",
        include_str!("../../../runtime/ts/src/list.ts"),
    ),
    (
        "src/service.ts",
        include_str!("../../../runtime/ts/src/service.ts"),
    ),
    (
        "src/provider.ts",
        include_str!("../../../runtime/ts/src/provider.ts"),
    ),
    (
        "src/provider-package.ts",
        include_str!("../../../runtime/ts/src/provider-package.ts"),
    ),
    (
        "src/show.ts",
        include_str!("../../../runtime/ts/src/show.ts"),
    ),
    (
        "src/html.ts",
        include_str!("../../../runtime/ts/src/html.ts"),
    ),
    ("src/dom.ts", include_str!("../../../runtime/ts/src/dom.ts")),
    (
        "src/signal.ts",
        include_str!("../../../runtime/ts/src/signal.ts"),
    ),
    (
        "src/stdin.ts",
        include_str!("../../../runtime/ts/src/stdin.ts"),
    ),
    (
        "src/stdin-service.ts",
        include_str!("../../../runtime/ts/src/stdin-service.ts"),
    ),
    ("src/sum.ts", include_str!("../../../runtime/ts/src/sum.ts")),
    (
        "src/browser/console.ts",
        include_str!("../../../runtime/ts/src/browser/console.ts"),
    ),
    (
        "src/browser/dom.ts",
        include_str!("../../../runtime/ts/src/browser/dom.ts"),
    ),
    (
        "src/browser/host.ts",
        include_str!("../../../runtime/ts/src/browser/host.ts"),
    ),
    (
        "src/browser/ime-input.ts",
        include_str!("../../../runtime/ts/src/browser/ime-input.ts"),
    ),
    (
        "src/browser/stdin.ts",
        include_str!("../../../runtime/ts/src/browser/stdin.ts"),
    ),
];

/// Stages the TypeScript runtime package embedded in this Rust crate.
///
/// Both product runners and conformance use this function, so the package
/// executed by a user-facing command is the package verified by fixtures.
pub fn stage_typescript_package(target: &Path) -> Result<(), String> {
    let package = target.join("node_modules/@seseragi/runtime");
    for (relative, contents) in FILES {
        let path = package.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create runtime directory: {error}"))?;
        }
        fs::write(&path, contents)
            .map_err(|error| format!("failed to stage runtime file {relative}: {error}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::stage_typescript_package;
    use std::fs;

    #[test]
    fn stages_the_provider_package_exports_and_sources() {
        let root = std::env::temp_dir().join(format!(
            "seseragi-runtime-provider-package-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }

        stage_typescript_package(&root).unwrap();

        let package = root.join("node_modules/@seseragi/runtime");
        let manifest = fs::read_to_string(package.join("package.json")).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(
            manifest.pointer("/exports/.~1provider/default"),
            Some(&serde_json::Value::String("./src/provider.ts".to_owned()))
        );
        assert_eq!(
            manifest.pointer("/exports/.~1provider-package/default"),
            Some(&serde_json::Value::String(
                "./src/provider-package.ts".to_owned()
            ))
        );
        assert!(package.join("src/provider.ts").is_file());
        assert!(package.join("src/provider-package.ts").is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
