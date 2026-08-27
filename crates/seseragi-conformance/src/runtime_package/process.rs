use std::path::Path;
use std::process::Command;

pub(super) fn check_process(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { run } from \"./src/effect.ts\";\n\
             import { render } from \"./src/path.ts\";\n\
             import { currentDirectory, liveProcess, processArguments, processEnvironment } from \"./src/process.ts\";\n\
             const environment = { process: liveProcess };\n\
             const args = await run(processArguments(), environment);\n\
             const present = await run(processEnvironment(\"SESERAGI_PROCESS_PROBE\"), environment);\n\
             const missing = await run(processEnvironment(\"SESERAGI_PROCESS_PROBE_MISSING\"), environment);\n\
             const invalid = await run(processEnvironment(\"bad\\0name\"), environment);\n\
             const cwd = await run(currentDirectory(), environment);\n\
             process.stdout.write(JSON.stringify({ args: args.kind === \"success\" && Array.isArray(args.value), present: present.kind === \"success\" && present.value.tag === \"Just\" && present.value.value === \"available\", missing: missing.kind === \"success\" && missing.value.tag === \"Nothing\", invalid: invalid.kind === \"failure\" && invalid.error.tag === \"InvalidEnvironmentName\", cwd: cwd.kind === \"success\" && render(cwd.value).replaceAll(\"\\\\\", \"/\").endsWith(\"/runtime/ts\") }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .env("SESERAGI_PROCESS_PROBE", "available")
        .env_remove("SESERAGI_PROCESS_PROBE_MISSING")
        .output()
        .map_err(|error| format!("failed to run TypeScript Process runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Process runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected =
        b"{\"args\":true,\"present\":true,\"missing\":true,\"invalid\":true,\"cwd\":true}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript Process runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
