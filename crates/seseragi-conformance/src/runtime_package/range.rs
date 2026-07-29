use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_range(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { exclusive, inclusive, reduce } from \"./src/range.ts\";\n\
             const sum = (range) => reduce(0, (total) => (value) => total + value, range);\n\
             const count = (range) => reduce(0, (total) => (_value) => total + 1, range);\n\
             process.stdout.write(JSON.stringify({\n\
               exclusive: String(sum(exclusive(1, 10))),\n\
               inclusive: String(sum(inclusive(1, 10))),\n\
               descending: String(sum(inclusive(10, 1))),\n\
               empty: String(count(exclusive(5, 5))),\n\
               max: String(count(inclusive(9007199254740991, 9007199254740991))),\n\
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
