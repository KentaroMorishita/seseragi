use std::{fs, path::Path};

pub(crate) fn check_standard_library_case(case: &Path) -> Result<(), String> {
    let expected = fs::read_to_string(case.join("module.json"))
        .map_err(|error| format!("failed to read standard module surface: {error}"))?;
    let expected: serde_json::Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse standard module surface: {error}"))?;
    let actual = serde_json::to_value(seseragi_semantics::standard_prelude_surface())
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
    fn rejects_a_surface_that_does_not_match_the_registry() {
        let root = std::env::temp_dir().join(format!(
            "seseragi-standard-library-surface-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("module.json"), "{\"schema\":1}\n").unwrap();

        assert_eq!(
            check_standard_library_case(&root),
            Err("standard module surface artifact mismatch".to_owned())
        );

        fs::remove_dir_all(root).unwrap();
    }
}
