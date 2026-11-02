/// A portable runtime label without a build machine's absolute directory.
pub fn runtime_source_path(source: &str) -> String {
    let normalized = source.replace('\\', "/");
    let drive = has_drive_prefix(&normalized);
    if normalized.starts_with('/')
        || drive
        || normalized.starts_with("file:")
        || normalized.split('/').any(|part| part == "..")
    {
        standalone_runtime_source_path(&normalized)
    } else {
        normalized.trim_start_matches("./").to_owned()
    }
}

/// A standalone source is displayed relative to its own directory.
pub fn standalone_runtime_source_path(source: &str) -> String {
    let source = if has_drive_prefix(source) {
        &source[2..]
    } else {
        source
    };
    source
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(source)
        .to_owned()
}

fn has_drive_prefix(source: &str) -> bool {
    source.as_bytes().get(1) == Some(&b':') && source.as_bytes()[0].is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_package_paths_and_normalizes_separators() {
        for input in [
            "src/errors/main.ssrg",
            "src\\errors\\main.ssrg",
            "./src/errors/main.ssrg",
        ] {
            assert_eq!(runtime_source_path(input), "src/errors/main.ssrg");
            assert_eq!(standalone_runtime_source_path(input), "main.ssrg");
        }
    }

    #[test]
    fn absolute_and_parent_paths_do_not_expose_directories() {
        for input in [
            "/private/tmp/work/main.ssrg",
            "C:\\Users\\name\\work\\main.ssrg",
            "C:main.ssrg",
            "\\\\server\\share\\main.ssrg",
            "file:///private/tmp/work/main.ssrg",
            "../other/main.ssrg",
        ] {
            assert_eq!(runtime_source_path(input), "main.ssrg");
        }
    }
}
