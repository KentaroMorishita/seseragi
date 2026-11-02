//! Guard all lowering-owned runtime paths, including inline TypeScript types.
use std::{fs, path::Path};

const PREFIX: &str = "@seseragi/runtime/";

pub(crate) fn assert_public_runtime_modules<'a>(modules: impl IntoIterator<Item = &'a str>) {
    let package: serde_json::Value =
        serde_json::from_str(include_str!("../../../runtime/ts/package.json")).unwrap();
    let exports = package["exports"].as_object().unwrap();
    for module in modules {
        let subpath = module.strip_prefix(PREFIX).unwrap();
        let key = format!("./{subpath}");
        assert!(
            exports.get(&key).is_some_and(|target| !target.is_null()),
            "lowering references {module}, which is not a public runtime export"
        );
    }
}

#[test]
fn all_lowering_runtime_subpaths_are_public() {
    check_sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
}

fn check_sources(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_sources(&path);
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && path.file_name().unwrap() != "runtime_exports.rs"
        {
            let source = fs::read_to_string(&path).unwrap();
            for (offset, _) in source.match_indices(PREFIX) {
                let tail = &source[offset + PREFIX.len()..];
                // These two macros are checked after expansion against their
                // complete operation tables in their own unit tests.
                if tail.starts_with("\", $module)")
                    && matches!(
                        path.file_name().unwrap().to_str().unwrap(),
                        "collection_ops.rs" | "text_ops.rs"
                    )
                {
                    continue;
                }
                let length = tail
                    .bytes()
                    .take_while(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'/' | b'.')
                    })
                    .count();
                let terminator = tail[length..].trim_start_matches('\\');
                assert!(
                    length > 0 && terminator.starts_with('"'),
                    "{}: use a full runtime module literal so the public export guard can check it",
                    path.display()
                );
                let module = &source[offset..offset + PREFIX.len() + length];
                assert_public_runtime_modules([module]);
            }
        }
    }
}
