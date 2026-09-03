use std::fs;
use std::path::Path;
use std::process::Command;

mod big_int;
mod bytes;
mod child_process;
mod clock_provider;
mod comprehension;
mod effect;
mod filesystem;
mod http_client;
mod http_server;
mod imports;
mod iterator;
mod json;
mod list;
mod numeric;
mod postgres;
mod process;
mod provider;
mod random_entropy;
mod range;
mod service;
mod services;
mod sqlite;
mod stream;
mod sum;
mod traversable;

pub(crate) fn check_typescript_runtime_package(
    root: &Path,
    abi: &serde_json::Value,
) -> Result<(), String> {
    if abi
        .pointer("/targetFamily")
        .and_then(|value| value.as_str())
        != Some("typescript")
    {
        return Ok(());
    }

    let package_path = root.join("runtime/ts/package.json");
    let package_raw = fs::read_to_string(&package_path)
        .map_err(|error| format!("failed to read TypeScript runtime package: {error}"))?;
    let package: serde_json::Value = serde_json::from_str(&package_raw)
        .map_err(|error| format!("failed to parse TypeScript runtime package: {error}"))?;

    if package.get("name").and_then(|value| value.as_str()) != Some("@seseragi/runtime") {
        return Err("TypeScript runtime package name must be @seseragi/runtime".to_owned());
    }
    imports::package_export_source(&package, ".")
        .ok_or_else(|| "TypeScript runtime package root export is missing".to_owned())?;

    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    for feature in features {
        match feature.get("kind").and_then(|value| value.as_str()) {
            Some("runtime-helper" | "runtime-binding") => {
                imports::check_runtime_import(root, &package, feature)?;
            }
            Some("value-representation") if feature.get("typeImport").is_some() => {
                imports::check_runtime_type_import(root, &package, feature)?;
            }
            _ => {}
        }
    }
    check_typescript_runtime_package_typecheck(root)?;
    provider::check_provider_runtime_abi(root)?;
    clock_provider::check_clock_provider(root)?;
    random_entropy::check_random_entropy(root)?;
    child_process::check_child_process(root)?;
    filesystem::check_filesystem(root)?;
    http_server::check_http_server(root)?;
    http_client::check_http_client(root)?;
    postgres::check_postgres(root)?;
    sqlite::check_sqlite(root)?;
    if runtime_helper_is_declared(abi, "process.current-directory") {
        process::check_process(root)?;
    }
    service::check_typed_service_boundary(root)?;
    sum::check_tagged_standard_sums(root)?;
    effect::check_from_either_boundary(root)?;
    effect::check_effect_concurrency_boundary(root)?;
    effect::check_effect_temporal_boundary(root)?;
    effect::check_effect_resource_boundary(root)?;
    stream::check_stream_boundary(root)?;
    if runtime_helper_is_declared(abi, "effect.stdin.readLine") {
        services::check_typescript_runtime_read_line(root)?;
    }
    if runtime_helper_is_declared(abi, "effect.console.println") {
        services::check_typescript_runtime_console_services(root)?;
    }
    if runtime_helper_is_declared(abi, "effect.core.fail")
        || runtime_helper_is_declared(abi, "effect.core.mapError")
    {
        effect::check_typed_failure_boundary(root)?;
    }
    if runtime_helper_is_declared(abi, "core.int.add") {
        check_typescript_runtime_int(root)?;
    }
    if runtime_helper_is_declared(abi, "core.big-int.api.parse") {
        big_int::check_typescript_runtime_big_int(root)?;
    }
    if runtime_helper_is_declared(abi, "core.int.api.parse")
        || runtime_helper_is_declared(abi, "core.float64.api.to-int")
        || runtime_helper_is_declared(abi, "core.int.foreign.decode")
        || runtime_helper_is_declared(abi, "core.int.foreign.encode")
        || runtime_helper_is_declared(abi, "core.int.json.decode")
        || runtime_helper_is_declared(abi, "core.int.json.encode")
    {
        numeric::check_typescript_runtime_numeric_surface(root)?;
    }
    if runtime_helper_is_declared(abi, "core.bytes.byte")
        || runtime_helper_is_declared(abi, "core.text.encode-utf8")
    {
        bytes::check_typescript_runtime_bytes_surface(root)?;
    }
    if runtime_helper_is_declared(abi, "json.parse")
        || runtime_helper_is_declared(abi, "json.encode-string")
    {
        json::check_typescript_runtime_json_surface(root)?;
    }
    if runtime_feature_is_declared(abi, "core.show.dictionary") {
        check_typescript_runtime_show(root)?;
    }
    if runtime_helper_is_declared(abi, "core.range.reduce") {
        range::check_typescript_runtime_range(root)?;
    }
    if runtime_helper_is_declared(abi, "core.iterator.unfold")
        || runtime_helper_is_declared(abi, "core.iterator.next")
    {
        iterator::check_typescript_runtime_iterator(root)?;
    }
    if runtime_helper_is_declared(abi, "core.list.from-array")
        || runtime_helper_is_declared(abi, "core.list.reduce")
        || runtime_helper_is_declared(abi, "core.list.comprehend")
        || runtime_helper_is_declared(abi, "core.non-empty-list.singleton")
    {
        list::check_typescript_runtime_list(root)?;
    }
    if runtime_helper_is_declared(abi, "core.range.comprehend") {
        comprehension::check_typescript_runtime_comprehension(root)?;
    }
    if runtime_feature_is_declared(abi, "core.array.traversable") {
        traversable::check_typescript_runtime_traversable(root)?;
    }
    Ok(())
}

fn runtime_helper_is_declared(abi: &serde_json::Value, id: &str) -> bool {
    runtime_feature_is_declared_with_kind(abi, id, Some("runtime-helper"))
}

fn runtime_feature_is_declared(abi: &serde_json::Value, id: &str) -> bool {
    runtime_feature_is_declared_with_kind(abi, id, None)
}

fn runtime_feature_is_declared_with_kind(
    abi: &serde_json::Value,
    id: &str,
    kind: Option<&str>,
) -> bool {
    abi.get("features")
        .and_then(|value| value.as_array())
        .is_some_and(|features| {
            features.iter().any(|feature| {
                kind.is_none_or(|kind| {
                    feature.get("kind").and_then(|value| value.as_str()) == Some(kind)
                }) && feature.get("id").and_then(|value| value.as_str()) == Some(id)
            })
        })
}

fn check_typescript_runtime_package_typecheck(root: &Path) -> Result<(), String> {
    let tsc = local_typescript(root)?;
    for project in [
        "runtime/ts/tsconfig.json",
        "runtime/providers/tsconfig.json",
    ] {
        let output = Command::new(&tsc)
            .arg("-p")
            .arg(project)
            .arg("--noEmit")
            .current_dir(root)
            .output()
            .map_err(|error| {
                format!(
                    "failed to type-check {project} with {}: {error}",
                    tsc.display()
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "TypeScript package type-check failed for {project}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}

fn local_typescript(root: &Path) -> Result<std::path::PathBuf, String> {
    let workspace_root = root.canonicalize().map_err(|error| {
        format!("failed to resolve repository root for TypeScript type-check: {error}")
    })?;
    let executable = if cfg!(windows) { "tsc.cmd" } else { "tsc" };
    let path = workspace_root
        .join("apps/playground/node_modules/.bin")
        .join(executable);
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "local TypeScript compiler is missing at {}; run `cd apps/playground && bun install --frozen-lockfile`",
            path.display()
        ))
    }
}

fn check_typescript_runtime_int(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { add, divide, MAX_INT, MIN_INT, multiply, power, remainder, subtract } from \"./src/int.ts\";\n\
             const defects = [];\n\
             for (const operation of [() => add(MAX_INT, 1), () => subtract(MIN_INT, 1), () => multiply(MAX_INT, 2), () => divide(1, 0), () => remainder(1, 0), () => power(2, 53), () => power(2, -1)]) {\n\
               try { operation(); defects.push(false); } catch (error) { defects.push(error instanceof RangeError); }\n\
             }\n\
             const values = [add(2, 3), subtract(2, 3), multiply(-2, 3), divide(-5, 2), remainder(-5, 2), power(0, 0), add(MAX_INT, 0), subtract(MIN_INT, 0), divide(MIN_INT, -1), remainder(MIN_INT, -1)].map(String);\n\
             const negativeZeros = [divide(0, -1), remainder(-4, 2), multiply(-1, 0)].map((value) => Object.is(value, -0));\n\
             process.stdout.write(JSON.stringify({ defects, values, negativeZeros }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Int runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Int runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout
        != b"{\"defects\":[true,true,true,true,true,true,true],\"values\":[\"5\",\"-1\",\"-6\",\"-2\",\"-1\",\"1\",\"9007199254740991\",\"-9007199254740991\",\"9007199254740991\",\"0\"],\"negativeZeros\":[false,false,false]}"
    {
        return Err(format!(
            "TypeScript Int runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

fn check_typescript_runtime_show(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("probes/show.ts")
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Show runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Show runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"show runtime probe passed\n" {
        return Err(format!(
            "TypeScript Show runtime probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
