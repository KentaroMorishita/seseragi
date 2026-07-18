use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_list(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { collectMap, fromArray, listApplicative, listFunctor, listMonad, reduce } from \"./src/list.ts\";\n\
             const values = fromArray([1n, 2n, 3n]);\n\
             const empty = fromArray([]);\n\
             const collected = [];\n\
             let cursor = values;\n\
             while (cursor.tag === \"Cons\") { collected.push(String(cursor.head)); cursor = cursor.tail; }\n\
             const total = reduce(0n, (sum) => (value) => sum + value, values);\n\
             const odds = collectMap(values, (value) => value % 2n === 1n, (value) => String(value * value));\n\
             const toStrings = (list) => { const result = []; let cursor = list; while (cursor.tag === \"Cons\") { result.push(String(cursor.head)); cursor = cursor.tail; } return result; };\n\
             const mapped = toStrings(listFunctor.map((value) => value + 10n)(values));\n\
             const applied = toStrings(listApplicative.apply(fromArray([(value) => value + 10n, (value) => value * 2n]))(fromArray([1n, 2n])));\n\
             const flattened = toStrings(listMonad.flatMap((value) => fromArray([value, value + 10n]))(fromArray([1n, 2n])));\n\
             const pure = toStrings(listApplicative.pure(42n));\n\
             process.stdout.write(JSON.stringify({ collected, empty: empty.tag, frozen: Object.isFrozen(values) && values.tag === \"Cons\" && Object.isFrozen(values.tail), total: String(total), odds, mapped, applied, flattened, pure }));\n",
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
    let expected = b"{\"collected\":[\"1\",\"2\",\"3\"],\"empty\":\"Empty\",\"frozen\":true,\"total\":\"6\",\"odds\":[\"1\",\"9\"],\"mapped\":[\"11\",\"12\",\"13\"],\"applied\":[\"11\",\"12\",\"2\",\"4\"],\"flattened\":[\"1\",\"11\",\"2\",\"12\"],\"pure\":[\"42\"]}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript List runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
