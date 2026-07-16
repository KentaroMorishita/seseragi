use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_list(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { fromArray } from \"./src/list.ts\";\n\
             const values = fromArray([1n, 2n, 3n]);\n\
             const empty = fromArray([]);\n\
             const collected = [];\n\
             let cursor = values;\n\
             while (cursor.tag === \"Cons\") { collected.push(String(cursor.head)); cursor = cursor.tail; }\n\
             process.stdout.write(JSON.stringify({ collected, empty: empty.tag, frozen: Object.isFrozen(values) && values.tag === \"Cons\" && Object.isFrozen(values.tail) }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript List runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript List runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"collected\":[\"1\",\"2\",\"3\"],\"empty\":\"Empty\",\"frozen\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript List runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
