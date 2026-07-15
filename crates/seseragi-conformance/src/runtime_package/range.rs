use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_range(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { exclusive, inclusive, reduce } from \"./src/range.ts\";\n\
             const sum = (range) => reduce(0n, (total) => (value) => total + value, range);\n\
             const count = (range) => reduce(0n, (total) => (_value) => total + 1n, range);\n\
             process.stdout.write(JSON.stringify({\n\
               exclusive: String(sum(exclusive(1n, 10n))),\n\
               inclusive: String(sum(inclusive(1n, 10n))),\n\
               descending: String(sum(inclusive(10n, 1n))),\n\
               empty: String(count(exclusive(5n, 5n))),\n\
               max: String(count(inclusive(9223372036854775807n, 9223372036854775807n))),\n\
             }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Range runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Range runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"exclusive\":\"45\",\"inclusive\":\"55\",\"descending\":\"0\",\"empty\":\"0\",\"max\":\"1\"}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript Range runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
