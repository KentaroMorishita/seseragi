use std::path::Path;

pub(crate) fn stage_runtime(_root: &Path, target_dir: &Path) -> Result<(), String> {
    seseragi_runtime::stage_typescript_package(target_dir)
}
