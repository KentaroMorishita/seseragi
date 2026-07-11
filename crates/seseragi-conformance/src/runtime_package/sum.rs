use std::path::Path;
use std::process::Command;

pub(super) fn check_tagged_standard_sums(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { Nothing as RootNothing } from \"./src/index.ts\";\n\
             import { Just, Left, Nothing, Right } from \"./src/sum.ts\";\n\
             const marker = { kind: \"marker\" };\n\
             const just = Just(marker);\n\
             const left = Left(marker);\n\
             const right = Right(marker);\n\
             process.stdout.write(JSON.stringify({\n\
               singleton: Nothing === RootNothing,\n\
               frozen: Object.isFrozen(Nothing),\n\
               payloads: just.value === marker && left.value === marker && right.value === marker,\n\
               shapes: [Nothing, just, left, right],\n\
             }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript standard sum probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript standard sum probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"singleton\":true,\"frozen\":true,\"payloads\":true,\"shapes\":[{\"tag\":\"Nothing\"},{\"tag\":\"Just\",\"value\":{\"kind\":\"marker\"}},{\"tag\":\"Left\",\"value\":{\"kind\":\"marker\"}},{\"tag\":\"Right\",\"value\":{\"kind\":\"marker\"}}]}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript standard sum probe returned unexpected result: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
