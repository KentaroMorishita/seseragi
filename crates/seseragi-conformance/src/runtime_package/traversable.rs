use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_traversable(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { arrayTraversable } from \"./src/array.ts\";\n\
             import { fromArray, listTraversable, nonEmptyListTraversable, toArray, toListNonEmpty } from \"./src/list.ts\";\n\
             import { eitherApplicative, Just, Left, maybeApplicative, Nothing, Right } from \"./src/sum.ts\";\n\
             const arraySuccess = arrayTraversable.traverse((value) => Just(value + 10))([1, 2, 3])(maybeApplicative);\n\
             const arrayFailure = arrayTraversable.traverse((value) => value === 2 ? Nothing : Just(value))([1, 2, 3])(maybeApplicative);\n\
             const listFailure = listTraversable.traverse((value) => value === 2 ? Left(\"stopped\") : Right(value))(fromArray([1, 2, 3]))(eitherApplicative);\n\
             const nonEmptySuccess = nonEmptyListTraversable.traverse((value) => Just(value + 1))({ tag: \"NonEmpty\", head: 1, tail: fromArray([2, 3]) })(maybeApplicative);\n\
             const auditApplicative = { pure: (value) => ({ events: [], value }), map: (f) => (wrapped) => ({ events: wrapped.events, value: f(wrapped.value) }), apply: (wrappedFunction) => (wrappedValue) => ({ events: [...wrappedFunction.events, ...wrappedValue.events], value: wrappedFunction.value(wrappedValue.value) }) };\n\
             const audit = arrayTraversable.traverse((value) => ({ events: [value], value: value + 30 }))([1, 2, 3])(auditApplicative);\n\
             const nonEmptyValues = nonEmptySuccess.tag === \"Just\" ? toArray(toListNonEmpty(nonEmptySuccess.value)) : [];\n\
             process.stdout.write(JSON.stringify({ arraySuccess, arrayFailure, listFailure, nonEmptyValues, audit }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Traversable probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Traversable probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"arraySuccess\":{\"tag\":\"Just\",\"value\":[11,12,13]},\"arrayFailure\":{\"tag\":\"Nothing\"},\"listFailure\":{\"tag\":\"Left\",\"value\":\"stopped\"},\"nonEmptyValues\":[2,3,4],\"audit\":{\"events\":[1,2,3],\"value\":[31,32,33]}}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript Traversable probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
