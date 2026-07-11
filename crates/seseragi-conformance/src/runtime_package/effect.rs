use std::path::Path;
use std::process::Command;

pub(super) fn check_typed_failure_boundary(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { fail, flatMap, run, succeed } from \"./src/effect.ts\";\n\
             let continued = false;\n\
             const effect = flatMap(fail({ kind: \"expected\" }), () => { continued = true; return succeed(1); });\n\
             const cold = continued === false;\n\
             const result = await run(effect, {});\n\
             let defect = false;\n\
             try { await run(() => { throw new Error(\"defect\"); }, {}); } catch (error) { defect = error instanceof Error && error.message === \"defect\"; }\n\
             process.stdout.write(JSON.stringify({ cold, continued, result, defect }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript typed failure probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript typed failure probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"cold\":true,\"continued\":false,\"result\":{\"kind\":\"failure\",\"error\":{\"kind\":\"expected\"}},\"defect\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript typed failure probe returned unexpected result: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
