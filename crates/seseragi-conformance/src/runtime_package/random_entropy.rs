use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_random_entropy(root: &Path) -> Result<(), String> {
    let staging = std::env::temp_dir().join(format!(
        "seseragi-random-entropy-provider-{}",
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .map_err(|error| format!("failed to clean Random probe staging: {error}"))?;
    }
    fs::create_dir_all(&staging)
        .map_err(|error| format!("failed to create Random probe staging: {error}"))?;
    let result = run_probe(root, &staging);
    let cleanup = fs::remove_dir_all(&staging)
        .map_err(|error| format!("failed to clean Random probe staging: {error}"));
    result.and(cleanup)
}

fn run_probe(root: &Path, staging: &Path) -> Result<(), String> {
    seseragi_runtime::stage_typescript_package(staging)?;
    fs::copy(
        root.join("runtime/ts/probes/random-entropy-provider.ts"),
        staging.join("random-entropy-provider.ts"),
    )
    .map_err(|error| format!("failed to stage Random/Entropy probe: {error}"))?;
    let output = Command::new("bun")
        .arg("random-entropy-provider.ts")
        .current_dir(staging)
        .output()
        .map_err(|error| format!("failed to run Random/Entropy provider probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Random/Entropy provider probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"random and entropy provider probe passed\n" {
        return Err(format!(
            "Random/Entropy provider probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
