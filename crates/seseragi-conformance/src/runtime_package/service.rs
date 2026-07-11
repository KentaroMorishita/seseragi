use std::path::Path;
use std::process::Command;

pub(super) fn check_typed_service_boundary(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { run } from \"./src/effect.ts\";\n\
             import { serviceEffect, serviceFailure, serviceSuccess } from \"./src/service.ts\";\n\
             const success = await run(serviceEffect(() => serviceSuccess(\"ok\")), {});\n\
             const failure = await run(serviceEffect(() => serviceFailure({ tag: \"ExpectedFailure\" })), {});\n\
             let rejectedDefect = false;\n\
             try { await run(serviceEffect(() => Promise.reject(new Error(\"defect\"))), {}); } catch (error) { rejectedDefect = error instanceof Error && error.message === \"defect\"; }\n\
             let invalidDefect = false;\n\
             try { await run(serviceEffect(() => undefined), {}); } catch (error) { invalidDefect = error instanceof TypeError; }\n\
             process.stdout.write(JSON.stringify({ success, failure, rejectedDefect, invalidDefect }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript service boundary probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript service boundary probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"success\":{\"kind\":\"success\",\"value\":\"ok\"},\"failure\":{\"kind\":\"failure\",\"error\":{\"tag\":\"ExpectedFailure\"}},\"rejectedDefect\":true,\"invalidDefect\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript service boundary probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
