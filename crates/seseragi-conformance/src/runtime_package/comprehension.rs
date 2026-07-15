use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_comprehension(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { collectFlatMap as arrayFlatMap, collectMap as arrayMap } from \"./src/array.ts\";\n\
             import { collectFlatMap as rangeFlatMap, collectMap as rangeMap, inclusive } from \"./src/range.ts\";\n\
             const evenSquares = rangeMap(inclusive(1n, 10n), (value) => value % 2n === 0n, (value) => value * value);\n\
             const arrayPairs = arrayFlatMap([1n, 2n], () => true, (left) => arrayMap([10n, 20n], (right) => right > 10n, (right) => left + right));\n\
             const rangePairs = rangeFlatMap(inclusive(1n, 2n), () => true, (left) => rangeMap(inclusive(3n, 4n), () => true, (right) => left * right));\n\
             process.stdout.write(JSON.stringify({ evenSquares: evenSquares.map(String), arrayPairs: arrayPairs.map(String), rangePairs: rangePairs.map(String) }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript comprehension runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript comprehension runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"evenSquares\":[\"4\",\"16\",\"36\",\"64\",\"100\"],\"arrayPairs\":[\"21\",\"22\"],\"rangePairs\":[\"3\",\"4\",\"6\",\"8\"]}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript comprehension runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
