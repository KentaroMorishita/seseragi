use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub(super) fn check_typescript_runtime_read_line(root: &Path) -> Result<(), String> {
    let mut child = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { run } from \"./src/effect.ts\";\n\
             import { createProcessStdin, readLine } from \"./src/stdin.ts\";\n\
             import { Just, Nothing } from \"./src/sum.ts\";\n\
             let explicitReads = 0;\n\
             const explicitService = { readLine() { explicitReads += 1; return Just(\"explicit\"); } };\n\
             const explicitEffect = readLine();\n\
             const cold = explicitReads === 0;\n\
             const explicitResult = await run(explicitEffect, { stdin: explicitService });\n\
             const stdin = createProcessStdin();\n\
             const effects = [readLine(), readLine(), readLine()];\n\
             const results = [];\n\
             for (const effect of effects) results.push(await run(effect, { stdin }));\n\
             const values = results.map((result) => result.kind === \"success\" ? result.value : null);\n\
             stdin.close();\n\
             stdin.close();\n\
             const afterClose = await run(readLine(), { stdin });\n\
             process.stdout.write(JSON.stringify({ cold, explicitReads, explicitResult, values, eofSingleton: values[2] === Nothing, afterClose, closedSingleton: afterClose.kind === \"success\" && afterClose.value === Nothing }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run TypeScript stdin runtime probe: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "failed to open TypeScript stdin runtime probe input".to_owned())?;
    stdin.write_all(b"first\nsecond\n").map_err(|error| {
        format!("failed to write TypeScript stdin runtime probe input: {error}")
    })?;
    drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for TypeScript stdin runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript stdin runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"cold\":true,\"explicitReads\":1,\"explicitResult\":{\"kind\":\"success\",\"value\":{\"tag\":\"Just\",\"value\":\"explicit\"}},\"values\":[{\"tag\":\"Just\",\"value\":\"first\"},{\"tag\":\"Just\",\"value\":\"second\"},{\"tag\":\"Nothing\"}],\"eofSingleton\":true,\"afterClose\":{\"kind\":\"success\",\"value\":{\"tag\":\"Nothing\"}},\"closedSingleton\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript stdin runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

pub(super) fn check_typescript_runtime_console_services(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { run } from \"./src/effect.ts\";\n\
             import { print, println } from \"./src/console.ts\";\n\
             const writes = [];\n\
             const console = { print(value) { writes.push([\"print\", value]); }, println(value) { writes.push([\"println\", value]); } };\n\
             const printEffect = print(12);\n\
             const printlnEffect = println(\"after\");\n\
             const cold = writes.length === 0;\n\
             const printResult = await run(printEffect, { console });\n\
             const printlnResult = await run(printlnEffect, { console });\n\
             process.stdout.write(JSON.stringify({ cold, writes, printResult, printlnResult }));\n",
        )
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
    let expected = b"{\"cold\":true,\"writes\":[[\"print\",\"12\"],[\"println\",\"after\"]],\"printResult\":{\"kind\":\"success\"},\"printlnResult\":{\"kind\":\"success\"}}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript console service probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
