#![cfg(not(target_arch = "wasm32"))]

#[test]
fn records_keep_prototype_like_keys_as_own_data_in_node_and_bun() {
    let source = include_str!("../../../examples/spec/fixtures/compile/record-prototype-key.ssrg");
    for profile in [
        seseragi_project::BuildProfile::Development,
        seseragi_project::BuildProfile::Release,
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "record-prototype-key.ssrg",
                "fixture/record-prototype-key",
                source,
            )
            .with_profile(profile),
        )
        .unwrap();
        assert!(compiled.generated.typescript.contains("[\"__proto__\"]:"));
        let directory = std::env::temp_dir().join(format!(
            "seseragi-record-prototype-{}-{}",
            std::process::id(),
            profile.as_str()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        seseragi_runtime::build_main_with_artifact_options(
            &compiled,
            &directory,
            seseragi_runtime::BuildTarget::Process,
            seseragi_runtime::ProcessRunOptions::default(),
            seseragi_runtime::artifact::ArtifactOptions {
                source_map: Some(seseragi_runtime::artifact::SourceMapPolicy::Emit),
                ..Default::default()
            },
        )
        .unwrap();
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(directory.join("artifact-manifest.json")).unwrap(),
        )
        .unwrap();
        for runtime in ["node", "bun"] {
            let output = std::process::Command::new(runtime)
                .arg(manifest["entry"].as_str().unwrap())
                .current_dir(&directory)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{runtime}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout),
                "kept\nnested\nkept\nctor\nnamed\n"
            );
        }
        let map_path = if profile == seseragi_project::BuildProfile::Release {
            "entry.js.map"
        } else {
            "main.js.map"
        };
        let map: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join(map_path)).unwrap()).unwrap();
        assert!(map["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "seseragi://fixture/record-prototype-key"));
        assert!(map["sourcesContent"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == source));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
