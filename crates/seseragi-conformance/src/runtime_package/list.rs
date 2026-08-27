use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_list(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { collectMap, fromArray, fromListNonEmpty, headNonEmpty, listApplicative, listFunctor, listMonad, reduce, reduce1NonEmpty, singleton, tailNonEmpty, toArray, toListNonEmpty } from \"./src/list.ts\";\n\
             const values = fromArray([1, 2, 3]);\n\
             const empty = fromArray([]);\n\
             const collected = [];\n\
             let cursor = values;\n\
             while (cursor.tag === \"Cons\") { collected.push(String(cursor.head)); cursor = cursor.tail; }\n\
             const total = reduce(0, (sum) => (value) => sum + value, values);\n\
             const odds = collectMap(values, (value) => value % 2 === 1, (value) => String(value * value));\n\
             const toStrings = (list) => { const result = []; let cursor = list; while (cursor.tag === \"Cons\") { result.push(String(cursor.head)); cursor = cursor.tail; } return result; };\n\
             const mapped = toStrings(listFunctor.map((value) => value + 10)(values));\n\
             const applied = toStrings(listApplicative.apply(fromArray([(value) => value + 10, (value) => value * 2]))(fromArray([1, 2])));\n\
             const flattened = toStrings(listMonad.flatMap((value) => fromArray([value, value + 10]))(fromArray([1, 2])));\n\
             const pure = toStrings(listApplicative.pure(42));\n\
             const nonEmpty = fromListNonEmpty(values);\n\
             const singletonValue = singleton(9);\n\
             const nonEmptyValues = nonEmpty.tag === \"Just\" ? { head: String(headNonEmpty(nonEmpty.value)), tail: toArray(tailNonEmpty(nonEmpty.value)).map(String), list: toArray(toListNonEmpty(nonEmpty.value)).map(String), reduced: String(reduce1NonEmpty((left) => (right) => left + right, nonEmpty.value)) } : undefined;\n\
             process.stdout.write(JSON.stringify({ collected, empty: empty.tag, frozen: Object.isFrozen(values) && values.tag === \"Cons\" && Object.isFrozen(values.tail), total: String(total), odds, mapped, applied, flattened, pure, nonEmpty: nonEmptyValues, fromEmpty: fromListNonEmpty(empty).tag, singleton: String(headNonEmpty(singletonValue)) }));\n",
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
    let expected = b"{\"collected\":[\"1\",\"2\",\"3\"],\"empty\":\"Empty\",\"frozen\":true,\"total\":\"6\",\"odds\":[\"1\",\"9\"],\"mapped\":[\"11\",\"12\",\"13\"],\"applied\":[\"11\",\"12\",\"2\",\"4\"],\"flattened\":[\"1\",\"11\",\"2\",\"12\"],\"pure\":[\"42\"],\"nonEmpty\":{\"head\":\"1\",\"tail\":[\"2\",\"3\"],\"list\":[\"1\",\"2\",\"3\"],\"reduced\":\"6\"},\"fromEmpty\":\"Nothing\",\"singleton\":\"9\"}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript List runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
