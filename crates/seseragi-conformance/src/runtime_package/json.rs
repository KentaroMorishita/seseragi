use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_json_surface(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/json.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript JSON runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript JSON runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"json runtime probe passed\n" {
        return Err(format!(
            "TypeScript JSON runtime probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
