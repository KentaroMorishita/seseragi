use std::path::Path;
use std::process::Command;

pub(super) fn check_provider_runtime_abi(root: &Path) -> Result<(), String> {
    check_probe(root, "provider.ts", b"provider runtime ABI probe passed\n")?;
    check_probe(
        root,
        "provider-package.ts",
        b"provider package boundary probe passed\n",
    )?;
    check_probe(
        root,
        "provider-conformance.ts",
        b"provider conformance profile probe passed\n",
    )
}

fn check_probe(root: &Path, probe: &str, expected: &[u8]) -> Result<(), String> {
    let output = Command::new("bun")
        .arg(format!("probes/{probe}"))
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run {probe} probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{probe} probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != expected {
        return Err(format!(
            "{probe} probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
