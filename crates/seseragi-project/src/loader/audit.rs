use super::PackageLoadError;
use crate::ModulePath;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

/// Validates the identity of every source module, including modules that are
/// not reachable from the executable entry. Parsing and compilation remain
/// demand-driven; this pass only closes filesystem identity ambiguities.
pub(crate) fn audit_source_root(root: &Path) -> Result<(), PackageLoadError> {
    let mut scanner = SourceRootScanner::new(root);
    scanner.scan_directory(Path::new(""), root)?;
    audit_records(root, scanner.files)
}

#[derive(Clone, Debug)]
struct SourceFile {
    logical_path: PathBuf,
    canonical_path: PathBuf,
}

struct SourceRootScanner<'a> {
    root: &'a Path,
    directories: BTreeMap<PathBuf, PathBuf>,
    files: Vec<SourceFile>,
}

impl<'a> SourceRootScanner<'a> {
    fn new(root: &'a Path) -> Self {
        Self {
            root,
            directories: BTreeMap::new(),
            files: Vec::new(),
        }
    }

    fn scan_directory(
        &mut self,
        logical_directory: &Path,
        physical_directory: &Path,
    ) -> Result<(), PackageLoadError> {
        let canonical_directory = fs::canonicalize(physical_directory).map_err(|source| {
            PackageLoadError::io(
                "canonicalize source directory",
                physical_directory.to_owned(),
                source,
            )
        })?;
        if !canonical_directory.starts_with(self.root) {
            return Err(PackageLoadError::RootEscape {
                path: physical_directory.to_owned(),
                canonical_path: canonical_directory,
            });
        }
        if let Some(first) = self
            .directories
            .insert(canonical_directory.clone(), logical_directory.to_owned())
        {
            if first != logical_directory {
                return Err(PackageLoadError::DuplicatePhysicalDirectory {
                    first,
                    second: logical_directory.to_owned(),
                    canonical_path: canonical_directory,
                });
            }
        }

        let entries = fs::read_dir(physical_directory).map_err(|source| {
            PackageLoadError::io(
                "read source directory",
                physical_directory.to_owned(),
                source,
            )
        })?;
        let mut entries = entries.collect::<Result<Vec<_>, _>>().map_err(|source| {
            PackageLoadError::io(
                "read source directory entry",
                physical_directory.to_owned(),
                source,
            )
        })?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let logical_path = logical_directory.join(entry.file_name());
            let canonical_path = fs::canonicalize(entry.path()).map_err(|source| {
                PackageLoadError::io("canonicalize source entry", entry.path(), source)
            })?;
            if !canonical_path.starts_with(self.root) {
                return Err(PackageLoadError::RootEscape {
                    path: entry.path(),
                    canonical_path,
                });
            }
            let metadata = fs::metadata(entry.path()).map_err(|source| {
                PackageLoadError::io("read source entry metadata", entry.path(), source)
            })?;
            if metadata.is_dir() {
                self.scan_directory(&logical_path, &entry.path())?;
            } else if metadata.is_file()
                && entry.path().extension().is_some_and(|ext| ext == "ssrg")
            {
                self.files.push(SourceFile {
                    logical_path,
                    canonical_path,
                });
            }
        }
        Ok(())
    }
}

fn audit_records(root: &Path, mut files: Vec<SourceFile>) -> Result<(), PackageLoadError> {
    files.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
    let mut normalized_owners = BTreeMap::<String, PathBuf>::new();
    let mut case_owners = BTreeMap::<String, ModulePath>::new();
    let mut physical_owners = BTreeMap::<PathBuf, ModulePath>::new();
    let mut noncanonical = None;

    for file in files {
        let raw = module_spelling(&file.logical_path)?;
        let normalized = raw.nfc().collect::<String>();
        let module = ModulePath::parse(&normalized).map_err(|error| {
            PackageLoadError::InvalidSourceModulePath {
                path: file.logical_path.clone(),
                reason: error.to_string(),
            }
        })?;
        if let Some(first) = normalized_owners.insert(normalized.clone(), file.logical_path.clone())
        {
            if first != file.logical_path {
                return Err(PackageLoadError::SourceNormalizationCollision {
                    first,
                    second: file.logical_path,
                    module,
                });
            }
        }
        let folded = normalized.to_lowercase();
        if let Some(first) = case_owners.insert(folded, module.clone()) {
            if first != module {
                return Err(PackageLoadError::SourceCaseCollision {
                    first,
                    second: module,
                });
            }
        }
        if let Some(first) = physical_owners.insert(file.canonical_path.clone(), module.clone()) {
            if first != module {
                return Err(PackageLoadError::DuplicatePhysicalModule {
                    first,
                    second: module,
                    canonical_path: file.canonical_path,
                });
            }
        }
        if raw != normalized {
            noncanonical.get_or_insert_with(|| PackageLoadError::NonCanonicalSpelling {
                expected: normalized,
                actual: raw,
                directory: root.to_owned(),
            });
        }
    }
    match noncanonical {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn module_spelling(path: &Path) -> Result<String, PackageLoadError> {
    let without_extension = path.with_extension("");
    without_extension
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .map(|segments| segments.join("/"))
        .ok_or_else(|| PackageLoadError::InvalidSourceModulePath {
            path: path.to_owned(),
            reason: "module path is not valid UTF-8".to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(logical: &str, physical: &str) -> SourceFile {
        SourceFile {
            logical_path: PathBuf::from(logical),
            canonical_path: PathBuf::from(physical),
        }
    }

    #[test]
    fn rejects_normalization_and_case_collisions_independent_of_host_filesystem() {
        let normalization = audit_records(
            Path::new("/source"),
            vec![
                record("café.ssrg", "/source/a"),
                record("cafe\u{301}.ssrg", "/source/b"),
            ],
        )
        .unwrap_err();
        assert!(matches!(
            normalization,
            PackageLoadError::SourceNormalizationCollision { .. }
        ));

        let case = audit_records(
            Path::new("/source"),
            vec![
                record("Game/Main.ssrg", "/source/a"),
                record("game/main.ssrg", "/source/b"),
            ],
        )
        .unwrap_err();
        assert!(matches!(case, PackageLoadError::SourceCaseCollision { .. }));
    }
}
