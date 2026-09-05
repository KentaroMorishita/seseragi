use std::{fs, path::Path};

pub(crate) fn check_standard_library_case(case: &Path) -> Result<(), String> {
    let expected = fs::read_to_string(case.join("module.json"))
        .map_err(|error| format!("failed to read standard module surface: {error}"))?;
    let expected: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse standard module surface: {error}"))?;
    let case_name = case.file_name().and_then(|name| name.to_str());
    let actual = match case_name {
        Some("prelude") => serde_json::to_value(seseragi_semantics::standard_prelude_surface()),
        Some("registry") => {
            serde_json::to_value(seseragi_project::standard_module_registry_surface())
        }
        Some("parity") => {
            let surface = seseragi_conformance::standard_module_parity_surface()
                .map_err(|error| format!("failed to build standard module parity: {error}"))?;
            let root = case.ancestors().nth(5).ok_or_else(|| {
                "standard module parity case is outside the repository".to_owned()
            })?;
            for route in &surface.routes {
                if !root.join(route.evidence).exists() {
                    return Err(format!(
                        "standard module parity evidence does not exist: {}",
                        route.evidence
                    ));
                }
            }
            if !root.join(surface.target_diagnostic.evidence).exists() {
                return Err(format!(
                    "standard module target diagnostic evidence does not exist: {}",
                    surface.target_diagnostic.evidence
                ));
            }
            serde_json::to_value(surface)
        }
        _ => return Err("unknown standard library surface case".to_owned()),
    }
    .map_err(|error| format!("failed to encode standard module surface: {error}"))?;

    if expected != actual {
        return Err("standard module surface artifact mismatch".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_prelude_artifact_preserves_instance_tracking_contract() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let case = root.join("examples/spec/artifacts/stdlib-schema-1/prelude");
        assert_eq!(check_standard_library_case(&case), Ok(()));
    }

    #[test]
    fn rejects_a_surface_that_does_not_match_the_registry() {
        let root = std::env::temp_dir().join(format!(
            "seseragi-standard-library-surface-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let case = root.join("registry");
        fs::create_dir_all(&case).unwrap();
        fs::write(case.join("module.json"), "{\"schema\":1}\n").unwrap();

        assert_eq!(
            check_standard_library_case(&case),
            Err("standard module surface artifact mismatch".to_owned())
        );

        fs::remove_dir_all(root).unwrap();
    }
}
