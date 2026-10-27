use seseragi_driver::{BuildProfile, CompiledModule};
use std::{fs, path::PathBuf};

pub fn record<'a>(
    label: &str,
    profile: BuildProfile,
    entry: &str,
    expected_stdout: &str,
    modules: impl IntoIterator<Item = (&'a str, &'a CompiledModule)>,
) {
    let Some(root) = std::env::var_os("SESERAGI_FRONTEND_BATCH_EVIDENCE") else {
        return;
    };
    let profile = match profile {
        BuildProfile::Development => "development",
        BuildProfile::Release => "release",
    };
    let directory = PathBuf::from(root).join(label).join(profile);
    fs::create_dir_all(&directory).unwrap();
    let mut files = Vec::new();
    let mut selected_entry = None;
    for (id, module) in modules {
        let relative = PathBuf::from(&module.generated.metadata.outputs.typescript);
        assert!(!relative.is_absolute());
        assert!(relative
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_))));
        let path = directory.join("raw").join(&relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &module.generated.typescript).unwrap();
        fs::write(
            path.with_extension("core.json"),
            serde_json::to_vec_pretty(&module.core_ir).unwrap(),
        )
        .unwrap();
        if id == entry {
            selected_entry = Some(relative.clone());
        }
        files.push(relative);
    }
    fs::write(
        directory.join("case.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "label": label,
            "profile": profile,
            "entry": selected_entry.expect("checked entry module"),
            "files": files,
            "expectedStdout": expected_stdout,
            "originalGeneratedTypescript": true
        }))
        .unwrap(),
    )
    .unwrap();
}
