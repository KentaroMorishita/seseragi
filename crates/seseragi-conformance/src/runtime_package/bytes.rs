use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_bytes_surface(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/bytes.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Bytes runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Bytes runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"bytes runtime probe passed\n" {
        return Err(format!(
            "TypeScript Bytes runtime probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
