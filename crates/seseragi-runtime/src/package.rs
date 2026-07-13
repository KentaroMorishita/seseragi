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
        "src/effect.ts",
        include_str!("../../../runtime/ts/src/effect.ts"),
    ),
    (
        "src/int64.ts",
        include_str!("../../../runtime/ts/src/int64.ts"),
    ),
    (
        "src/service.ts",
        include_str!("../../../runtime/ts/src/service.ts"),
    ),
    (
        "src/show.ts",
        include_str!("../../../runtime/ts/src/show.ts"),
    ),
    (
        "src/stdin.ts",
        include_str!("../../../runtime/ts/src/stdin.ts"),
    ),
    ("src/sum.ts", include_str!("../../../runtime/ts/src/sum.ts")),
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
