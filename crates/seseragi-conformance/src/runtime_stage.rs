use std::fs;
use std::path::Path;

pub(crate) fn stage_runtime(root: &Path, target_dir: &Path) -> Result<(), String> {
    let runtime_source = root.join("runtime/ts");
    if !runtime_source.is_dir() {
        return Err("runtime/ts is missing".to_owned());
    }
    let runtime_target = target_dir.join("node_modules/@seseragi/runtime");
    let runtime_parent = runtime_target
        .parent()
        .ok_or_else(|| "runtime target has no parent directory".to_owned())?;
    fs::create_dir_all(runtime_parent)
        .map_err(|error| format!("failed to create staged node_modules: {error}"))?;
    copy_dir(&runtime_source, &runtime_target)
        .map_err(|error| format!("failed to stage @seseragi/runtime: {error}"))
}

fn copy_dir(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}
