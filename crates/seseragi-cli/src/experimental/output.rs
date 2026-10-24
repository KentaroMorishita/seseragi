use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

pub(super) const EXECUTABLE: &str = if cfg!(windows) {
    "program.exe"
} else {
    "program"
};

pub(super) enum Destination<'a> {
    Rust(&'a Path),
    Native(&'a Path),
}

pub(super) struct StagedOutputs {
    pub rust: PathBuf,
    pub metadata: PathBuf,
    pub executable: Option<PathBuf>,
}

pub(super) fn publish(
    destination: Destination<'_>,
    sources: &[PathBuf],
    stage: impl FnOnce(&StagedOutputs) -> Result<(), String>,
) -> Result<(), String> {
    let (directory, names, native): (PathBuf, Vec<OsString>, bool) = match destination {
        Destination::Rust(path) => {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let name = path.file_name().ok_or("Rust output must name a file")?;
            let metadata = path.with_extension("rust-backend.json");
            (
                fs::canonicalize(parent).map_err(|e| e.to_string())?,
                vec![name.to_owned(), metadata.file_name().unwrap().to_owned()],
                false,
            )
        }
        Destination::Native(path) => {
            // Normalize trailing separators/dot components before lstat so
            // `link/` cannot turn a symlink leaf into an accepted directory.
            let path = path.components().collect::<PathBuf>();
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
                Ok(_) => {
                    return Err("native output must be a directory, not a file or symlink".into())
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir_all(&path).map_err(|e| e.to_string())?
                }
                Err(error) => return Err(error.to_string()),
            }
            (
                fs::canonicalize(path).map_err(|e| e.to_string())?,
                vec![
                    "program.rs".into(),
                    "rust-backend.json".into(),
                    EXECUTABLE.into(),
                ],
                true,
            )
        }
    };
    let targets = names
        .iter()
        .map(|name| directory.join(name))
        .collect::<Vec<_>>();
    preflight(&targets, sources)?;
    let staging = private_directory(&directory, ".seseragi-rust-stage-")?;
    let staged = StagedOutputs {
        rust: staging.path().join(&names[0]),
        metadata: staging.path().join(&names[1]),
        executable: native.then(|| staging.path().join(&names[2])),
    };
    stage(&staged)?;
    // rustc may take time. Recheck every destination before changing the first
    // final output, then use rename rather than writes through existing inodes.
    preflight(&targets, sources)?;
    let backup = private_directory(&directory, ".seseragi-rust-backup-")?;
    let mut previous = Vec::new();
    for (index, target) in targets.iter().enumerate() {
        let staged = staging.path().join(&names[index]);
        let metadata = fs::symlink_metadata(&staged).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("staged Rust output must be a regular file".into());
        }
        match fs::symlink_metadata(target) {
            Ok(metadata) => {
                let saved = backup.path().join(&names[index]);
                backup_output(target, &saved, metadata.permissions()).map_err(|e| e.to_string())?;
                let permissions = metadata.permissions();
                #[cfg(unix)]
                let permissions = {
                    use std::os::unix::fs::PermissionsExt;
                    let mut permissions = permissions;
                    if native && index == 2 {
                        permissions.set_mode(permissions.mode() | 0o100);
                    }
                    permissions
                };
                fs::set_permissions(&staged, permissions).map_err(|e| e.to_string())?;
                previous.push(Some(saved));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => previous.push(None),
            Err(error) => return Err(error.to_string()),
        }
    }
    for (index, target) in targets.iter().enumerate() {
        if let Err(error) = fs::rename(staging.path().join(&names[index]), target) {
            let rollback = rollback(&targets[..index], &previous[..index]);
            if let Err(rollback) = rollback {
                let retained = backup.keep();
                return Err(format!("{error}; output rollback failed: {rollback}; previous output backups preserved at {}", retained.display()));
            }
            return Err(format!(
                "failed to publish Rust output {}: {error}",
                target.display()
            ));
        }
    }
    backup
        .close()
        .map_err(|error| format!("Rust output published; old output cleanup failed: {error}"))?;
    staging
        .close()
        .map_err(|error| format!("Rust output published; staging cleanup failed: {error}"))
}

fn preflight(targets: &[PathBuf], sources: &[PathBuf]) -> Result<(), String> {
    for target in targets {
        match fs::symlink_metadata(target) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err(format!(
                        "refusing non-regular or symlink Rust output {}",
                        target.display()
                    ));
                }
                if sources.contains(&fs::canonicalize(target).map_err(|e| e.to_string())?) {
                    return Err("output must differ from the source file".into());
                }
                if metadata.permissions().readonly() {
                    return Err(format!(
                        "refusing read-only Rust output {}",
                        target.display()
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn private_directory(parent: &Path, prefix: &str) -> Result<TempDir, String> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(prefix).rand_bytes(32);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(fs::Permissions::from_mode(0o700));
    }
    builder.tempdir_in(parent).map_err(|e| e.to_string())
}

fn backup_output(source: &Path, saved: &Path, permissions: fs::Permissions) -> io::Result<()> {
    // fs::copy can carry platform flags such as macOS immutable into the owned
    // backup, preventing its cleanup after a failed publication. Keep only the
    // contents and permissions needed to restore the previous regular output.
    let mut source = fs::File::open(source)?;
    let mut backup = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(saved)?;
    io::copy(&mut source, &mut backup)?;
    backup.set_permissions(permissions)
}

fn rollback(targets: &[PathBuf], previous: &[Option<PathBuf>]) -> Result<(), String> {
    let mut errors = Vec::new();
    for (target, saved) in targets.iter().zip(previous).rev() {
        let restored = match saved {
            Some(saved) => fs::rename(saved, target),
            None => fs::remove_file(target),
        };
        if let Err(error) = restored {
            errors.push(format!("{}: {error}", target.display()));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::rollback;
    use std::fs;

    #[cfg(unix)]
    #[test]
    fn staging_is_private_and_removed_with_its_owner() {
        use super::private_directory;
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let directory = private_directory(root.path(), ".seseragi-rust-stage-").unwrap();
        let path = directory.path().to_owned();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fs::write(path.join("owned"), "staged output").unwrap();
        drop(directory);
        assert!(!path.exists());
    }

    #[test]
    fn rollback_restores_existing_leaves_and_removes_new_leaves() {
        let root = tempfile::tempdir().unwrap();
        let existing = root.path().join("existing");
        let new = root.path().join("new");
        let saved = root.path().join("backup");
        let unrelated = root.path().join("unrelated");
        fs::write(&existing, "replacement").unwrap();
        fs::write(&new, "new output").unwrap();
        fs::write(&saved, "previous output").unwrap();
        fs::write(&unrelated, "unrelated output").unwrap();
        rollback(&[existing.clone(), new.clone()], &[Some(saved), None]).unwrap();
        assert_eq!(fs::read_to_string(existing).unwrap(), "previous output");
        assert!(!new.exists());
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "unrelated output");
    }
}
