use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_iterator(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { next, unfold } from \"./src/iterator.ts\";\n\
             import { Just, Nothing } from \"./src/sum.ts\";\n\
             let steps = 0;\n\
             const source = unfold((value) => { steps += 1; return value <= 2n ? Just([value, value + 1n]) : Nothing; }, 1n);\n\
             const before = steps;\n\
             const first = next(source);\n\
             const afterFirst = steps;\n\
             const repeated = next(source);\n\
             const afterRepeated = steps;\n\
             if (first.tag !== \"Just\" || repeated.tag !== \"Just\") throw new Error(\"missing first step\");\n\
             const second = next(first.value[1]);\n\
             if (second.tag !== \"Just\") throw new Error(\"missing second step\");\n\
             process.stdout.write(JSON.stringify({ before, afterFirst, afterRepeated, first: String(first.value[0]), repeated: String(repeated.value[0]), second: String(second.value[0]), steps }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Iterator runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Iterator runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"before\":0,\"afterFirst\":1,\"afterRepeated\":2,\"first\":\"1\",\"repeated\":\"1\",\"second\":\"2\",\"steps\":3}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript Iterator runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
