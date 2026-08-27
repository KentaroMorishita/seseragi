use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_read_line(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/stdin.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript stdin runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript stdin runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"stdin runtime probe passed\n" {
        return Err(format!(
            "TypeScript stdin runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

pub(super) fn check_typescript_runtime_console_services(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/console-logger.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript console service probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript console service probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"console and logger runtime probe passed\n";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript console service probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
