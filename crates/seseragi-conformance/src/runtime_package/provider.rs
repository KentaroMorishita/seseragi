use std::path::Path;
use std::process::Command;

pub(super) fn check_provider_runtime_abi(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/provider.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run provider runtime ABI probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "provider runtime ABI probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"provider runtime ABI probe passed\n" {
        return Err(format!(
            "provider runtime ABI probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
