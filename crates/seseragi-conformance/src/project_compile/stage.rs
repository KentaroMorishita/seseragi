use super::compile::CompiledProjectCompileCase;
use std::fs;
use std::path::{Path, PathBuf};

/// Writes every generated module at its planned TypeScript output path below
/// one staging directory. Emitted ESM `.js` imports remain unchanged so the
/// host can resolve them against the staged TypeScript module graph.
pub(crate) fn stage_project_typescript(
    target_dir: &Path,
    compiled_case: &CompiledProjectCompileCase,
) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::with_capacity(compiled_case.compiled.order.len());
    for module_id in &compiled_case.compiled.order {
        let compiled = compiled_case
            .compiled
            .modules
            .get(module_id)
            .ok_or_else(|| format!("project compiler omitted staged module {module_id}"))?;
        let output = staged_relative_output_path(&compiled.generated.metadata.outputs.typescript)?;
        let target = target_dir.join(&output);
        let parent = target
            .parent()
            .ok_or_else(|| format!("generated output has no parent: {}", output.display()))?;
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create staged generated module directory {}: {error}",
                parent.display()
            )
        })?;
        fs::write(&target, &compiled.generated.typescript).map_err(|error| {
            format!(
                "failed to stage generated TypeScript {}: {error}",
                target.display()
            )
        })?;
        sources.push(output);
    }
    Ok(sources)
}

pub(crate) fn staged_relative_output_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
    {
        return Err(format!(
            "generated project TypeScript output must be a canonical relative path: {value}"
        ));
    }
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::staged_relative_output_path;
    use std::path::PathBuf;

    #[test]
    fn accepts_a_canonical_project_output_path() {
        assert_eq!(
            staged_relative_output_path("dist/rps/main.ts"),
            Ok(PathBuf::from("dist/rps/main.ts"))
        );
    }

    #[test]
    fn rejects_unsafe_project_output_paths() {
        for path in ["", "../main.ts", "/tmp/main.ts", "dist\\main.ts"] {
            assert!(staged_relative_output_path(path).is_err(), "{path}");
        }
    }
}
