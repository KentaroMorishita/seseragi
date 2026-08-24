use std::path::Path;
use std::process::Command;

pub(super) fn check_stream_boundary(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/stream.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run Stream runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Stream runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"stream runtime probe passed\n" {
        return Err(format!(
            "Stream runtime probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
