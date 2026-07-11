use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub(super) fn check_typescript_runtime_read_line(root: &Path) -> Result<(), String> {
    let mut child = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { PassThrough } from \"node:stream\";\n\
             import { run } from \"./src/effect.ts\";\n\
             import { createProcessStdin, readLine } from \"./src/stdin.ts\";\n\
             import { Just, Nothing } from \"./src/sum.ts\";\n\
             import { serviceFailure, serviceSuccess } from \"./src/service.ts\";\n\
             let explicitReads = 0;\n\
             const explicitService = { readLine() { explicitReads += 1; return serviceSuccess(Just(\"explicit\")); } };\n\
             const explicitEffect = readLine();\n\
             const cold = explicitReads === 0;\n\
             const explicitResult = await run(explicitEffect, { stdin: explicitService });\n\
             const typedFailure = await run(readLine(), { stdin: { readLine() { return serviceFailure({ tag: \"StdinUnavailable\" }); } } });\n\
             const rawDefect = new Error(\"raw stdin defect\");\n\
             let rawDefectPreserved = false;\n\
             try { await run(readLine(), { stdin: { readLine() { return Promise.reject(rawDefect); } } }); } catch (error) { rawDefectPreserved = error === rawDefect; }\n\
             const stdin = createProcessStdin();\n\
             const effects = [readLine(), readLine(), readLine()];\n\
             const results = [];\n\
             for (const effect of effects) results.push(await run(effect, { stdin }));\n\
             const values = results.map((result) => result.kind === \"success\" ? result.value : null);\n\
             const eofAgain = await run(readLine(), { stdin });\n\
             const eofSingleton = values[2] === Nothing && eofAgain.kind === \"success\" && eofAgain.value === Nothing;\n\
             stdin.close();\n\
             stdin.close();\n\
             let processClosedDefect = false;\n\
             try { await run(readLine(), { stdin }); } catch (error) { processClosedDefect = error instanceof Error && error.message.includes(\"host close\"); }\n\
             const concurrentInput = new PassThrough();\n\
             const concurrentStdin = createProcessStdin(concurrentInput);\n\
             const lazy = [\"data\", \"error\", \"end\"].every((event) => concurrentInput.listenerCount(event) === 0);\n\
             const leasePending = run(readLine(), { stdin: concurrentStdin });\n\
             const concurrentFailure = await run(readLine(), { stdin: concurrentStdin });\n\
             concurrentInput.end(\"leased\\n\");\n\
             const leaseSuccess = await leasePending;\n\
             const leaseEof = await run(readLine(), { stdin: concurrentStdin });\n\
             const stickyEof = await run(readLine(), { stdin: concurrentStdin });\n\
             const eofSticky = leaseEof.kind === \"success\" && leaseEof.value === Nothing && stickyEof.kind === \"success\" && stickyEof.value === Nothing;\n\
             const eofCleaned = [\"data\", \"error\", \"end\"].every((event) => concurrentInput.listenerCount(event) === 0);\n\
             concurrentStdin.close();\n\
             concurrentStdin.close();\n\
             const failureInput = new PassThrough();\n\
             const failureStdin = createProcessStdin(failureInput);\n\
             const failurePending = run(readLine(), { stdin: failureStdin });\n\
             failureInput.destroy(new Error(\"injected read failure\"));\n\
             const readFailure = await failurePending;\n\
             const failureAgain = await run(readLine(), { stdin: failureStdin });\n\
             const failureLeaseReleased = failureAgain.kind === \"failure\" && failureAgain.error.tag === \"StdinReadFailure\";\n\
             const failureCleaned = [\"data\", \"error\", \"end\"].every((event) => failureInput.listenerCount(event) === 0);\n\
             failureStdin.close();\n\
             failureStdin.close();\n\
             process.stdout.write(JSON.stringify({ cold, explicitReads, explicitResult, typedFailure, rawDefectPreserved, values, eofAgain, eofSingleton, processClosedDefect, lazy, concurrentFailure, leaseSuccess, leaseEof, stickyEof, eofSticky, eofCleaned, readFailure, failureAgain, failureLeaseReleased, failureCleaned }));\n",
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
    let expected = b"{\"cold\":true,\"explicitReads\":1,\"explicitResult\":{\"kind\":\"success\",\"value\":{\"tag\":\"Just\",\"value\":\"explicit\"}},\"typedFailure\":{\"kind\":\"failure\",\"error\":{\"tag\":\"StdinUnavailable\"}},\"rawDefectPreserved\":true,\"values\":[{\"tag\":\"Just\",\"value\":\"first\"},{\"tag\":\"Just\",\"value\":\"second\"},{\"tag\":\"Nothing\"}],\"eofAgain\":{\"kind\":\"success\",\"value\":{\"tag\":\"Nothing\"}},\"eofSingleton\":true,\"processClosedDefect\":true,\"lazy\":true,\"concurrentFailure\":{\"kind\":\"failure\",\"error\":{\"tag\":\"ConcurrentStdinRead\"}},\"leaseSuccess\":{\"kind\":\"success\",\"value\":{\"tag\":\"Just\",\"value\":\"leased\"}},\"leaseEof\":{\"kind\":\"success\",\"value\":{\"tag\":\"Nothing\"}},\"stickyEof\":{\"kind\":\"success\",\"value\":{\"tag\":\"Nothing\"}},\"eofSticky\":true,\"eofCleaned\":true,\"readFailure\":{\"kind\":\"failure\",\"error\":{\"tag\":\"StdinReadFailure\"}},\"failureAgain\":{\"kind\":\"failure\",\"error\":{\"tag\":\"StdinReadFailure\"}},\"failureLeaseReleased\":true,\"failureCleaned\":true}";
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
            "import { run, unit } from \"./src/effect.ts\";\n\
             import { liveConsole, print, println } from \"./src/console.ts\";\n\
             import { serviceFailure, serviceSuccess } from \"./src/service.ts\";\n\
             const writes = [];\n\
             const console = { print(value) { writes.push([\"print\", value]); return serviceSuccess(unit); }, println(value) { writes.push([\"println\", value]); return serviceSuccess(unit); } };\n\
             const printEffect = print(12);\n\
             const printlnEffect = println(\"after\");\n\
             const cold = writes.length === 0;\n\
             const printResult = await run(printEffect, { console });\n\
             const printlnResult = await run(printlnEffect, { console });\n\
             const typedConsole = { print() { return serviceFailure({ kind: \"console-error\", message: \"typed\" }); }, println() { return serviceSuccess(unit); } };\n\
             const typedFailure = await run(print(\"typed\"), { console: typedConsole });\n\
             const syncDefect = new Error(\"sync defect\");\n\
             let syncDefectPreserved = false;\n\
             try { await run(print(\"defect\"), { console: { print() { throw syncDefect; }, println() { return serviceSuccess(unit); } } }); } catch (error) { syncDefectPreserved = error === syncDefect; }\n\
             const rejectedDefect = new Error(\"rejected defect\");\n\
             let rejectedDefectPreserved = false;\n\
             try { await run(println(\"defect\"), { console: { print() { return serviceSuccess(unit); }, println() { return Promise.reject(rejectedDefect); } } }); } catch (error) { rejectedDefectPreserved = error === rejectedDefect; }\n\
             const stdout = process.stdout;\n\
             const originalWrite = stdout.write;\n\
             let liveSuccess;\n\
             let liveSyncFailure;\n\
             let liveCallbackFailure;\n\
             try {\n\
               stdout.write = (_value, callback) => { callback(); return true; };\n\
               liveSuccess = await liveConsole.print(\"ignored\");\n\
               stdout.write = () => { throw new Error(\"sync write\"); };\n\
               liveSyncFailure = await liveConsole.print(\"ignored\");\n\
               stdout.write = (_value, callback) => { callback(new Error(\"callback write\")); return true; };\n\
               liveCallbackFailure = await liveConsole.println(\"ignored\");\n\
             } finally { stdout.write = originalWrite; }\n\
             originalWrite.call(stdout, JSON.stringify({ cold, writes, printResult, printlnResult, typedFailure, syncDefectPreserved, rejectedDefectPreserved, liveSuccess, liveSyncFailure, liveCallbackFailure }));\n",
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
    let expected = b"{\"cold\":true,\"writes\":[[\"print\",\"12\"],[\"println\",\"after\"]],\"printResult\":{\"kind\":\"success\"},\"printlnResult\":{\"kind\":\"success\"},\"typedFailure\":{\"kind\":\"failure\",\"error\":{\"kind\":\"console-error\",\"message\":\"typed\"}},\"syncDefectPreserved\":true,\"rejectedDefectPreserved\":true,\"liveSuccess\":{\"kind\":\"success\"},\"liveSyncFailure\":{\"kind\":\"failure\",\"error\":{\"kind\":\"console-error\",\"message\":\"sync write\"}},\"liveCallbackFailure\":{\"kind\":\"failure\",\"error\":{\"kind\":\"console-error\",\"message\":\"callback write\"}}}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript console service probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
