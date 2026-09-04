use std::path::Path;
use std::process::Command;

pub(super) fn check_typescript_runtime_decimal(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { context, divide, divideExact, format, fromFloat, parse, quantize } from \"./src/decimal.ts\";\n\
             import { HalfEven, HalfUp } from \"./src/number.ts\";\n\
             import { decimalJsonEncode, encodeString } from \"./src/json.ts\";\n\
             const value = (text) => { const parsed = parse(text); if (parsed.tag !== \"Right\") throw new Error(\"Decimal parse failed\"); return parsed.value; };\n\
             const ctx3 = context(3, HalfEven);\n\
             const ctx17 = context(17, HalfEven);\n\
             if (ctx3.tag !== \"Right\" || ctx17.tag !== \"Right\") throw new Error(\"Decimal context failed\");\n\
             const rounded = divide(ctx3.value, value(\"3\"), value(\"2\"));\n\
             const exact = divideExact(value(\"8\"), value(\"1\"));\n\
             const nonTerminating = divideExact(value(\"3\"), value(\"1\"));\n\
             const converted = fromFloat(ctx17.value, 0.1);\n\
             if (rounded.tag !== \"Right\" || exact.tag !== \"Right\" || converted.tag !== \"Right\") throw new Error(\"Decimal operation failed\");\n\
             const huge = value(\"12345678901234567890.000000000000000001\");\n\
             process.stdout.write(JSON.stringify({ canonical: format(value(\"-0.00\")), rounded: format(rounded.value), exact: format(exact.value), nonTerminating: nonTerminating.value.tag, quantized: format(quantize(2, HalfUp, value(\"1.235\"))), converted: format(converted.value), json: encodeString(huge, decimalJsonEncode) }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Decimal runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Decimal runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = b"{\"canonical\":\"0\",\"rounded\":\"0.667\",\"exact\":\"0.125\",\"nonTerminating\":\"NonTerminatingDecimal\",\"quantized\":\"1.24\",\"converted\":\"0.10000000000000001\",\"json\":\"12345678901234567890.000000000000000001\"}";
    if output.stdout != expected {
        return Err(format!(
            "TypeScript Decimal runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
