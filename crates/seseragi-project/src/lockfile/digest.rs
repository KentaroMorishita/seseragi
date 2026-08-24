use super::LockError;
use crate::ManifestLayout;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub(super) struct PackageDigests {
    pub manifest: String,
    pub content: String,
}

pub(super) fn package_digests(
    root: &Path,
    layout: &ManifestLayout,
) -> Result<PackageDigests, LockError> {
    let manifest_path = root.join("seseragi.toml");
    let manifest = fs::read(&manifest_path)
        .map_err(|source| LockError::io("read manifest", &manifest_path, source))?;
    let mut files = vec![("seseragi.toml".to_owned(), manifest.clone())];
    collect_root(root, &root.join(layout.source.as_str()), true, &mut files)?;
    collect_root(
        root,
        &root.join(layout.generated.as_str()),
        false,
        &mut files,
    )?;
    files.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut content = Sha256::new();
    for (path, bytes) in files {
        feed(&mut content, path.as_bytes());
        feed(&mut content, &bytes);
    }
    Ok(PackageDigests {
        manifest: digest(&manifest),
        content: format!("sha256:{:x}", content.finalize()),
    })
}

fn collect_root(
    package_root: &Path,
    directory: &Path,
    source_only: bool,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), LockError> {
    if !directory.exists() {
        return Ok(());
    }
    let mut pending = vec![directory.to_owned()];
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|source| LockError::io("read package content", &directory, source))?;
        for entry in entries {
            let entry = entry
                .map_err(|source| LockError::io("read package content", &directory, source))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|source| LockError::io("inspect package content", &path, source))?;
            if metadata.file_type().is_symlink() {
                return Err(LockError::PackageGraph(format!(
                    "package content `{}` is a symlink",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file()
                && (!source_only
                    || path.extension().and_then(|value| value.to_str()) == Some("ssrg"))
            {
                let relative = portable_relative(package_root, &path)?;
                let bytes = fs::read(&path)
                    .map_err(|source| LockError::io("read package content", &path, source))?;
                files.push((relative, bytes));
            }
        }
    }
    Ok(())
}

pub(super) fn relative_source(root: &Path, package: &Path) -> Result<String, LockError> {
    portable_relative(root, package)
}

fn portable_relative(base: &Path, target: &Path) -> Result<String, LockError> {
    let base = base.components().collect::<Vec<_>>();
    let target = target.components().collect::<Vec<_>>();
    let common = base
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return Err(LockError::PackageGraph(format!(
            "cannot create a portable path from `{}` to `{}`",
            components(&base),
            components(&target)
        )));
    }
    let mut output = Vec::<String>::new();
    output.extend((common..base.len()).map(|_| "..".to_owned()));
    for component in &target[common..] {
        let value = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| LockError::PackageGraph("package path is not valid UTF-8".to_owned()))?;
        output.push(value.to_owned());
    }
    Ok(if output.is_empty() {
        ".".to_owned()
    } else {
        output.join("/")
    })
}

fn components(components: &[std::path::Component<'_>]) -> String {
    components
        .iter()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn feed(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}
