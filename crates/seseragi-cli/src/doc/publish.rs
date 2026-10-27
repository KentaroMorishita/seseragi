use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;

const MARKER: &str = ".seseragi-api-document";
const OWNER: &str = "seseragi-api-document:1\n";

pub(super) fn publish(root: &Path, artifacts: BTreeMap<String, String>) -> Result<(), String> {
    let dist = root.join("dist");
    match fs::symlink_metadata(&dist) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err("doc output parent must be a real directory".to_owned());
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir(&dist).map_err(|error| error.to_string())?
        }
        Err(error) => return Err(error.to_string()),
    }
    // A lease prevents cooperating writers from replacing each other's stage.
    let lease_path = dist.join(".seseragi-doc.lock");
    let mut lease = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lease_path)
        .map_err(|error| {
            format!(
                "cannot acquire doc output lease {}: {error}",
                lease_path.display()
            )
        })?;
    let _lease = Lease(lease_path);
    writeln!(&mut lease, "{}", std::process::id()).map_err(|error| error.to_string())?;
    drop(lease);
    let output = dist.join("doc");
    check_output(&output)?;
    let stage = tempfile::Builder::new()
        .prefix(".seseragi-doc-stage-")
        .tempdir_in(&dist)
        .map_err(|error| error.to_string())?;
    for (name, bytes) in artifacts {
        // Artifact names are compiler-owned, but keep the publisher closed to
        // relative traversal if another renderer is added later.
        if Path::new(&name)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("invalid doc artifact path".to_owned());
        }
        let path = stage.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
        fs::write(path, bytes).map_err(|error| error.to_string())?;
    }
    fs::write(stage.path().join(MARKER), OWNER).map_err(|error| error.to_string())?;
    check_output(&output)?;
    let backup = tempfile::Builder::new()
        .prefix(".seseragi-doc-previous-")
        .tempdir_in(&dist)
        .map_err(|error| error.to_string())?;
    fs::remove_dir(backup.path()).map_err(|error| error.to_string())?;
    let previous = output.exists();
    if previous {
        fs::rename(&output, backup.path()).map_err(|error| error.to_string())?;
    }
    if let Err(error) = fs::rename(stage.path(), &output) {
        if previous {
            if let Err(rollback) = fs::rename(backup.path(), &output) {
                let preserved = backup.keep();
                return Err(format!("doc publication failed: {error}; rollback failed: {rollback}; prior output retained at {}", preserved.display()));
            }
        }
        return Err(format!("doc publication failed: {error}"));
    }
    if previous {
        if let Err(error) = fs::remove_dir_all(backup.path()) {
            let retained = backup.keep();
            return Err(format!(
                "doc published; prior owned output cleanup failed at {}: {error}",
                retained.display()
            ));
        }
    }
    Ok(())
}

fn check_output(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err("refusing to replace a symlink or non-directory doc output".to_owned())
        }
        Ok(_) => {
            let empty = fs::read_dir(path)
                .map_err(|error| error.to_string())?
                .next()
                .is_none();
            if empty
                || fs::symlink_metadata(path.join(MARKER))
                    .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
                    && fs::read_to_string(path.join(MARKER)).is_ok_and(|value| value == OWNER)
            {
                Ok(())
            } else {
                Err(format!(
                    "refusing to replace unmanaged doc output {}",
                    path.display()
                ))
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

struct Lease(std::path::PathBuf);
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
