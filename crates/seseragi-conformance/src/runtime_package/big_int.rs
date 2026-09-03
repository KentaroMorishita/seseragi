use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_big_int(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { checkedDivide, checkedPower, format, fromInt, parse, power, remainder, toInt } from \"./src/big-int.ts\";\n\
             const parsed = parse(\"1234567890123456789012345678901234567890\");\n\
             if (parsed.tag !== \"Right\") throw new Error(\"BigInt parse failed\");\n\
             const zeroDivide = checkedDivide(fromInt(0), parsed.value);\n\
             const negativePower = checkedPower(-1, parsed.value);\n\
             const narrowed = toInt(parsed.value);\n\
             const values = [format(parsed.value), format(power(fromInt(2), 100)), format(remainder(fromInt(-17), fromInt(5)))];\n\
             process.stdout.write(JSON.stringify({ values, zeroDivide: zeroDivide.value.tag, negativePower: negativePower.value.tag, narrowed: narrowed.value.tag }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript BigInt runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript BigInt runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"values\":[\"1234567890123456789012345678901234567890\",\"1267650600228229401496703205376\",\"-2\"],\"zeroDivide\":\"BigIntDivisionByZero\",\"negativePower\":\"NegativeBigIntExponent\",\"narrowed\":\"BigIntOutsideIntRange\"}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript BigInt runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
