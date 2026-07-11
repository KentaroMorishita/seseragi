use std::path::Path;
use std::process::Command;

pub(super) fn check_typed_failure_boundary(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { fail, flatMap, mapError, run, succeed } from \"./src/effect.ts\";\n\
             let continued = false;\n\
             const effect = flatMap(fail({ kind: \"expected\" }), () => { continued = true; return succeed(1); });\n\
             const cold = continued === false;\n\
             const result = await run(effect, {});\n\
             let mapped = 0;\n\
             const mappedEffect = mapError(error => { mapped += 1; return { kind: \"mapped\", source: error }; }, fail({ kind: \"source\" }));\n\
             const mapCold = mapped === 0;\n\
             const mappedResult = await run(mappedEffect, {});\n\
             const successResult = await run(mapError(() => { mapped += 100; return { kind: \"impossible\" }; }, succeed(7)), {});\n\
             let defect = false;\n\
             try { await run(mapError(() => ({ kind: \"wrong\" }), () => { throw new Error(\"defect\"); }), {}); } catch (error) { defect = error instanceof Error && error.message === \"defect\"; }\n\
             let mapperDefect = false;\n\
             try { await run(mapError(() => { throw new Error(\"mapper-defect\"); }, fail({ kind: \"source\" })), {}); } catch (error) { mapperDefect = error instanceof Error && error.message === \"mapper-defect\"; }\n\
             process.stdout.write(JSON.stringify({ cold, continued, result, mapCold, mapped, mappedResult, successResult, defect, mapperDefect }));\n",
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
    let expected = b"{\"cold\":true,\"continued\":false,\"result\":{\"kind\":\"failure\",\"error\":{\"kind\":\"expected\"}},\"mapCold\":true,\"mapped\":1,\"mappedResult\":{\"kind\":\"failure\",\"error\":{\"kind\":\"mapped\",\"source\":{\"kind\":\"source\"}}},\"successResult\":{\"kind\":\"success\",\"value\":7},\"defect\":true,\"mapperDefect\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript typed failure probe returned unexpected result: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
