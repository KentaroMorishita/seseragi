use crate::ManifestWeb;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebAsset {
    pub source: PathBuf,
    pub output: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WebAssets {
    pub index: Option<WebAsset>,
    pub public: Vec<WebAsset>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebAssetError(String);

impl WebAssetError {
    pub const fn code(&self) -> &'static str {
        "SES-K0001"
    }
}
impl fmt::Display for WebAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for WebAssetError {}

/// Collects only explicitly configured inputs. Paths and directory identities
/// are checked before reading bytes; symlinks are never followed.
pub fn load_web_assets(root: &Path, config: &ManifestWeb) -> Result<WebAssets, WebAssetError> {
    let mut result = WebAssets::default();
    if let Some(index) = &config.index {
        let path = checked_path(root, index.as_str())?;
        result.index = Some(read_asset(path, "index.html".to_owned())?);
    }
    if let Some(public) = &config.public {
        let directory = checked_path(root, public.as_str())?;
        let mut owners = BTreeMap::new();
        visit(&directory, &directory, &mut owners, &mut result.public)?;
        result
            .public
            .sort_by(|a, b| a.output.as_bytes().cmp(b.output.as_bytes()));
    }
    Ok(result)
}

fn checked_path(root: &Path, relative: &str) -> Result<PathBuf, WebAssetError> {
    let mut path = root.to_owned();
    for segment in relative.split('/') {
        check_name(segment)?;
        // Check spelling even on case-insensitive or normalizing filesystems.
        let entries = fs::read_dir(&path).map_err(|e| error(&path, e))?;
        let mut found = false;
        for entry in entries {
            let entry = entry.map_err(|e| error(&path, e))?;
            if entry.file_name() == segment {
                found = true;
                break;
            }
        }
        if !found {
            return Err(WebAssetError(format!(
                "Web input `{}` is missing or has noncanonical spelling",
                path.join(segment).display()
            )));
        }
        path.push(segment);
        inspect(&path)?;
    }
    Ok(path)
}

fn inspect(path: &Path) -> Result<fs::Metadata, WebAssetError> {
    let metadata = fs::symlink_metadata(path).map_err(|e| error(path, e))?;
    if metadata.file_type().is_symlink() {
        return Err(WebAssetError(format!(
            "Web input `{}` is a symlink",
            path.display()
        )));
    }
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(WebAssetError(format!(
            "Web input `{}` is not a regular file or directory",
            path.display()
        )));
    }
    Ok(metadata)
}

fn check_name(name: &str) -> Result<(), WebAssetError> {
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.contains(['/', '\\', ':'])
        || name.chars().any(char::is_control)
    {
        return Err(WebAssetError(format!(
            "invalid Web input path component `{name}`"
        )));
    }
    let normalized: String = name.nfc().collect();
    if normalized != name {
        return Err(WebAssetError(format!(
            "Web input `{name}` must use canonical NFC spelling `{normalized}`"
        )));
    }
    Ok(())
}

fn visit(
    root: &Path,
    directory: &Path,
    owners: &mut BTreeMap<String, String>,
    files: &mut Vec<WebAsset>,
) -> Result<(), WebAssetError> {
    if !inspect(directory)?.is_dir() {
        return Err(WebAssetError(format!(
            "web.public `{}` is not a directory",
            directory.display()
        )));
    }
    let mut entries = fs::read_dir(directory)
        .map_err(|e| error(directory, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| error(directory, e))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| WebAssetError("Web input path is not UTF-8".to_owned()))?;
        check_name(name)?;
        let output = path
            .strip_prefix(root)
            .expect("asset stays under public root")
            .components()
            .map(|part| part.as_os_str().to_str().expect("checked path components"))
            .collect::<Vec<_>>()
            .join("/");
        register_output(owners, &output)?;
        if inspect(&path)?.is_dir() {
            visit(root, &path, owners, files)?;
        } else {
            files.push(read_asset(path, output)?);
        }
    }
    Ok(())
}

fn register_output(
    owners: &mut BTreeMap<String, String>,
    output: &str,
) -> Result<(), WebAssetError> {
    let key = seseragi_syntax::unicode::lowercase(output);
    if matches!(
        key.split('/').next(),
        Some(
            "index.html"
                | "assets"
                | ".seseragi-build.json"
                | "artifact-manifest.json"
                | "__seseragi_dev"
        )
    ) {
        return Err(WebAssetError(format!(
            "public asset `{output}` collides with reserved Web output"
        )));
    }
    if let Some(first) = owners.insert(key, output.to_owned()) {
        return Err(WebAssetError(format!(
            "public assets `{first}` and `{output}` have a case/NFC path collision"
        )));
    }
    Ok(())
}

fn read_asset(source: PathBuf, output: String) -> Result<WebAsset, WebAssetError> {
    if !inspect(&source)?.is_file() {
        return Err(WebAssetError(format!(
            "Web input `{}` is not a regular file",
            source.display()
        )));
    }
    let bytes = fs::read(&source).map_err(|e| error(&source, e))?;
    Ok(WebAsset {
        source,
        output,
        bytes,
    })
}
fn error(path: &Path, error: std::io::Error) -> WebAssetError {
    WebAssetError(format!(
        "failed to read Web input `{}`: {error}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Workspace(PathBuf);
    impl Workspace {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "seseragi-web-assets-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(path.join("public/nested")).unwrap();
            Self(path)
        }
        fn config(&self, fields: &str) -> ManifestWeb {
            crate::parse_manifest(&format!("[package]\nname = \"fixture/assets\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[web]\n{fields}")).unwrap().web.unwrap()
        }
    }
    impl Drop for Workspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn loads_only_configured_regular_files_in_portable_order() {
        let root = Workspace::new();
        fs::write(root.0.join("public/z.txt"), "last").unwrap();
        fs::write(root.0.join("public/nested/a.txt"), "first").unwrap();
        fs::write(root.0.join("private.txt"), "excluded").unwrap();
        let assets = load_web_assets(&root.0, &root.config("public = \"public\"\n")).unwrap();
        assert_eq!(
            assets
                .public
                .iter()
                .map(|f| f.output.as_str())
                .collect::<Vec<_>>(),
            ["nested/a.txt", "z.txt"]
        );
        assert_eq!(assets.public[0].bytes, b"first");
        assert!(assets.index.is_none());
        assert!(load_web_assets(&root.0, &root.config(""))
            .unwrap()
            .public
            .is_empty());
    }
    #[test]
    fn rejects_missing_inputs_reserved_outputs_and_noncanonical_names() {
        let root = Workspace::new();
        let error =
            load_web_assets(&root.0, &root.config("index = \"missing.html\"\n")).unwrap_err();
        assert!(error.to_string().contains("missing"));
        for name in [
            "index.html",
            "ASSETS",
            ".seseragi-build.json",
            "artifact-manifest.json",
            "__seseragi_dev",
        ] {
            fs::write(root.0.join("public").join(name), "bad").unwrap();
            let error =
                load_web_assets(&root.0, &root.config("public = \"public\"\n")).unwrap_err();
            assert!(error.to_string().contains("reserved"), "{error}");
            fs::remove_file(root.0.join("public").join(name)).unwrap();
        }
        assert!(check_name("e\u{301}.svg").is_err());
        assert!(check_name("../secret").is_err());
    }
    #[test]
    fn rejects_case_collisions_independent_of_the_host_filesystem() {
        let mut owners = BTreeMap::new();
        register_output(&mut owners, "images/Logo.svg").unwrap();
        let error = register_output(&mut owners, "images/logo.svg").unwrap_err();
        assert!(error.to_string().contains("collision"));
        let mut owners = BTreeMap::new();
        register_output(&mut owners, "Icons").unwrap();
        assert!(register_output(&mut owners, "icons").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlink_inputs_and_nested_symlinks() {
        let root = Workspace::new();
        fs::write(root.0.join("secret"), "private").unwrap();
        std::os::unix::fs::symlink(root.0.join("secret"), root.0.join("public/nested/link"))
            .unwrap();
        assert!(
            load_web_assets(&root.0, &root.config("public = \"public\"\n"))
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
        std::os::unix::fs::symlink(root.0.join("public"), root.0.join("link")).unwrap();
        assert!(
            load_web_assets(&root.0, &root.config("public = \"link\"\n"))
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
    }
    #[test]
    fn rejects_manifest_escape_paths_and_unknown_web_keys() {
        for field in ["index", "public"] {
            for path in [
                "../outside",
                "/outside",
                "a/../outside",
                ".git/index",
                "dist/site",
                "C:/outside",
            ] {
                let source = format!("[package]\nname = \"fixture/assets\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[web]\n{field} = {path:?}\n");
                assert!(crate::parse_manifest(&source).is_err(), "{source}");
            }
        }
        let source = "[package]\nname = \"fixture/assets\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[web]\npostbuild = \"anything\"";
        assert!(crate::parse_manifest(source).is_err());
    }
}
