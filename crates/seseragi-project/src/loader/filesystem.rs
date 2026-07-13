use super::PackageLoadError;
use crate::{LayoutPath, ModulePath};
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

pub(crate) fn canonical_directory(
    path: &Path,
    label: &'static str,
) -> Result<PathBuf, PackageLoadError> {
    let canonical = fs::canonicalize(path).map_err(|source| {
        PackageLoadError::io("canonicalize directory", path.to_owned(), source)
    })?;
    if !canonical.is_dir() {
        return Err(PackageLoadError::io(
            label,
            canonical,
            std::io::Error::new(std::io::ErrorKind::NotADirectory, "not a directory"),
        ));
    }
    Ok(canonical)
}

pub(crate) fn resolve_source_root(
    package_root: &Path,
    layout: &LayoutPath,
) -> Result<PathBuf, PackageLoadError> {
    let mut current = package_root.to_owned();
    for segment in layout.as_str().split('/') {
        current = exact_child(&current, segment)?;
    }
    let canonical = canonical_directory(&current, "source root")?;
    if !canonical.starts_with(package_root) {
        return Err(PackageLoadError::RootEscape {
            path: current,
            canonical_path: canonical,
        });
    }
    Ok(canonical)
}

pub(crate) fn resolve_module_file(
    source_root: &Path,
    module: &ModulePath,
) -> Result<PathBuf, PackageLoadError> {
    let mut current = source_root.to_owned();
    let segments = module.as_str().split('/').collect::<Vec<_>>();
    for segment in &segments[..segments.len() - 1] {
        current = exact_child(&current, segment)?;
    }
    exact_child(
        &current,
        &format!(
            "{}.ssrg",
            segments.last().expect("module path is non-empty")
        ),
    )
}

fn exact_child(directory: &Path, expected: &str) -> Result<PathBuf, PackageLoadError> {
    let entries = fs::read_dir(directory)
        .map_err(|source| PackageLoadError::io("read directory", directory.to_owned(), source))?;
    let mut names = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    names.sort();
    if names.iter().any(|name| name == expected) {
        return Ok(directory.join(expected));
    }

    let normalized = names
        .iter()
        .filter(|name| name.nfc().eq(expected.nfc()))
        .cloned()
        .collect::<Vec<_>>();
    match normalized.as_slice() {
        [actual] => {
            return Err(PackageLoadError::NonCanonicalSpelling {
                expected: expected.to_owned(),
                actual: actual.clone(),
                directory: directory.to_owned(),
            });
        }
        [_, _, ..] => {
            return Err(PackageLoadError::NormalizationCollision {
                expected: expected.to_owned(),
                candidates: normalized,
                directory: directory.to_owned(),
            });
        }
        [] => {}
    }

    if let Some(actual) = names
        .iter()
        .find(|name| name.to_lowercase() == expected.to_lowercase())
    {
        return Err(PackageLoadError::CaseMismatch {
            expected: expected.to_owned(),
            actual: actual.clone(),
            directory: directory.to_owned(),
        });
    }
    Err(PackageLoadError::io(
        "resolve module path",
        directory.join(expected),
        std::io::Error::new(std::io::ErrorKind::NotFound, "entry does not exist"),
    ))
}
