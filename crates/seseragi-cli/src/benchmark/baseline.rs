use serde_json::{json, Value};
use std::collections::BTreeSet;

fn object(value: &Value) -> Result<(), String> {
    value
        .as_object()
        .map(|_| ())
        .ok_or_else(|| "invalid benchmark baseline object".into())
}
fn text(value: &Value) -> Result<&str, String> {
    value
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "invalid baseline identity".into())
}
fn finite(value: &Value, minimum: f64) -> Result<f64, String> {
    value
        .as_f64()
        .filter(|value| value.is_finite() && *value >= minimum)
        .ok_or_else(|| "invalid baseline number".into())
}
fn integer(value: &Value, minimum: f64) -> Result<f64, String> {
    let value = finite(value, minimum)?;
    if value.fract() != 0.0 || value > 9_007_199_254_740_991.0 {
        return Err("invalid baseline integer".into());
    }
    Ok(value)
}
fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[middle]
    } else {
        sorted[middle - 1] + (sorted[middle] - sorted[middle - 1]) / 2.0
    }
}

pub(super) fn json_number(value: &Value) -> Value {
    let number = value.as_f64().expect("validated benchmark number");
    if number.fract() == 0.0 && number >= 0.0 && number < u64::MAX as f64 {
        json!(number as u64)
    } else {
        value.clone()
    }
}

pub(super) fn validate(report: &Value) -> Result<(), String> {
    object(report)?;
    if report["schema"].as_f64() != Some(1.0) || report["kind"] != "benchmark-report" {
        return Err("unsupported benchmark baseline schema".into());
    }
    let metadata = &report["metadata"];
    object(metadata)?;
    for key in ["languageVersion", "compilerVersion", "runtimeVersion"] {
        let version = text(&metadata[key])?;
        let mut parts = version.splitn(3, '.');
        let digits = |part: &str| !part.is_empty() && part.bytes().all(|c| c.is_ascii_digit());
        let major = parts.next().unwrap_or("");
        let minor = parts.next().unwrap_or("");
        let patch = parts.next().unwrap_or("");
        let patch_end = patch.find(['-', '+']).unwrap_or(patch.len());
        if !digits(major)
            || !digits(minor)
            || !digits(&patch[..patch_end])
            || patch.contains(['\n', '\r', '\u{2028}', '\u{2029}'])
        {
            return Err("invalid baseline toolchain version".into());
        }
    }
    if metadata["profile"] != "release" {
        return Err("benchmark baseline requires release profile".into());
    }
    let target = &metadata["target"];
    object(target)?;
    text(&target["identity"])?;
    text(&target["version"])?;
    let host = &metadata["host"];
    object(host)?;
    for key in ["os", "architecture", "cpu"] {
        text(&host[key])?;
    }
    integer(&host["logicalCores"], 1.0)?;
    if !host["gcAvailable"].is_boolean() {
        return Err("invalid GC availability".into());
    }
    if finite(&metadata["timerResolutionNs"], 0.0)? <= 0.0 {
        return Err("invalid timer resolution".into());
    }
    let config = &report["config"];
    object(config)?;
    integer(&config["warmup"], 0.0)?;
    let count = integer(&config["samples"], 3.0)?;
    integer(&config["minimumSampleMs"], 1.0)?;
    finite(&config["regressionThresholdPercent"], 0.0)?;
    integer(&config["timeoutMs"], 1.0)?;
    integer(&config["cleanupGraceMs"], 0.0)?;
    let seed = text(&config["seed"])?;
    let unsigned = seed.strip_prefix('-').unwrap_or(seed);
    if unsigned.is_empty() || !unsigned.bytes().all(|c| c.is_ascii_digit()) {
        return Err("invalid seed".into());
    }
    let normalized = unsigned.trim_start_matches('0');
    let limit = if seed.starts_with('-') {
        "9223372036854775808"
    } else {
        "9223372036854775807"
    };
    if normalized.len() > limit.len() || (normalized.len() == limit.len() && normalized > limit) {
        return Err("seed is outside Int64".into());
    }
    let cases = report["cases"]
        .as_array()
        .filter(|cases| !cases.is_empty())
        .ok_or("baseline requires cases")?;
    let mut names = BTreeSet::new();
    for entry in cases {
        object(entry)?;
        let name = text(&entry["name"])?;
        if !names.insert(name) {
            return Err(format!("duplicate baseline case {name}"));
        }
        if !entry["inputSize"].is_null() || entry.get("inputSize").is_none() {
            integer(&entry["inputSize"], 1.0)?;
        }
        if entry["status"] != "passed" {
            return Err("baseline cannot contain failed measurements".into());
        }
        integer(&entry["iterations"], 1.0)?;
        let samples = entry["samples"]
            .as_array()
            .filter(|samples| samples.len() as f64 == count)
            .ok_or("baseline sample count mismatch")?;
        let samples = samples
            .iter()
            .map(|value| finite(value, 0.0))
            .collect::<Result<Vec<_>, _>>()?;
        let center = median(&samples);
        let computed = [
            ("median", center),
            (
                "medianAbsoluteDeviation",
                median(
                    &samples
                        .iter()
                        .map(|value| (value - center).abs())
                        .collect::<Vec<_>>(),
                ),
            ),
            (
                "minimum",
                samples.iter().copied().fold(f64::INFINITY, f64::min),
            ),
            (
                "maximum",
                samples.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ),
            ("sampleCount", samples.len() as f64),
        ];
        object(&entry["summary"])?;
        for (key, expected) in computed {
            if entry["summary"][key].as_f64() != Some(expected) {
                return Err(format!("invalid baseline summary {name}:{key}"));
            }
        }
        if !entry["stdout"].is_string() || !entry["stderr"].is_string() {
            return Err("invalid capture metadata".into());
        }
    }
    Ok(())
}

pub(super) fn compare(current: &Value, baseline: &Value) -> Result<Vec<Value>, String> {
    validate(baseline)?;
    let a = &current["metadata"];
    let b = &baseline["metadata"];
    for key in ["languageVersion", "compilerVersion", "runtimeVersion"] {
        if a[key].as_str().unwrap().split('.').next() != b[key].as_str().unwrap().split('.').next()
        {
            return Err(format!("incompatible baseline {key}"));
        }
    }
    if a["profile"] != b["profile"]
        || a["target"]["identity"] != b["target"]["identity"]
        || a["target"]["version"] != b["target"]["version"]
        || ["os", "architecture", "cpu"]
            .iter()
            .any(|key| a["host"][key] != b["host"][key])
        || a["timerResolutionNs"].as_f64() != b["timerResolutionNs"].as_f64()
    {
        return Err("incompatible benchmark baseline host/profile/target/timer".into());
    }
    let mut previous = baseline["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(Some)
        .collect::<Vec<_>>();
    let mut comparison = Vec::new();
    for entry in current["cases"].as_array().unwrap() {
        let old = previous
            .iter_mut()
            .find(|old| old.is_some_and(|old| old["name"] == entry["name"]))
            .and_then(Option::take);
        let status = if entry["status"] == "failed" {
            "failure"
        } else if old.is_none() {
            "new"
        } else if entry["inputSize"].as_f64() != old.unwrap()["inputSize"].as_f64() {
            "incomparable"
        } else if entry["summary"]["median"].as_f64().unwrap()
            > old.unwrap()["summary"]["median"].as_f64().unwrap()
                * (1.0
                    + current["config"]["regressionThresholdPercent"]
                        .as_f64()
                        .unwrap()
                        / 100.0)
        {
            "regression"
        } else {
            "unchanged"
        };
        let mut result = json!({"name": entry["name"], "status": status});
        if matches!(status, "regression" | "unchanged") {
            result["baselineMedian"] = json_number(&old.unwrap()["summary"]["median"]);
            result["currentMedian"] = json_number(&entry["summary"]["median"]);
        }
        comparison.push(result);
    }
    for old in previous.into_iter().flatten() {
        comparison.push(json!({"name": old["name"], "status": "missing"}));
    }
    Ok(comparison)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path, process::Command};

    fn report() -> Value {
        json!({
            "schema":1,"kind":"benchmark-report",
            "metadata":{"languageVersion":"0.61.138","compilerVersion":"0.61.138","runtimeVersion":"0.61.138","profile":"release","target":{"identity":"fake-clock","version":"1"},"host":{"os":"test","architecture":"test","cpu":"test","logicalCores":1,"gcAvailable":false},"timerResolutionNs":1},
            "config":{"warmup":0,"samples":3,"minimumSampleMs":1,"regressionThresholdPercent":5,"seed":"42","timeoutMs":1000,"cleanupGraceMs":10},
            "cases":[{"name":"main::case","inputSize":null,"status":"passed","iterations":1,"samples":[1,2,3],"summary":{"median":2,"medianAbsoluteDeviation":1,"minimum":1,"maximum":3,"sampleCount":3},"stdout":"","stderr":""}]
        })
    }

    #[test]
    fn native_baseline_validation_and_comparison_match_the_ts_runtime() {
        let original = report();
        let mut inputs = vec![json!({"current":original,"baseline":original})];
        for (pointer, invalid) in [
            ("/schema", json!(2)),
            ("/kind", json!("other")),
            ("/metadata", Value::Null),
            ("/metadata/languageVersion", json!("1.2")),
            ("/metadata/compilerVersion", json!("1.2.3.4")),
            ("/metadata/profile", json!("development")),
            ("/metadata/target/identity", json!("")),
            ("/metadata/host/gcAvailable", json!(0)),
            ("/metadata/host/logicalCores", json!(0)),
            ("/metadata/timerResolutionNs", json!(0)),
            ("/config/samples", json!(2)),
            ("/config/samples", json!(3.5)),
            ("/config/warmup", json!(-1)),
            ("/config/cleanupGraceMs", json!(9007199254740992u64)),
            ("/config/seed", json!("+1")),
            ("/config/seed", json!("9223372036854775808")),
            ("/config/seed", json!("-9223372036854775809")),
            ("/config/seed", json!("-")),
            ("/cases", json!([])),
            ("/cases/0/inputSize", json!(0)),
            ("/cases/0/iterations", json!(0)),
            ("/cases/0/status", json!("failed")),
            ("/cases/0/samples", json!([1, 2])),
            ("/cases/0/samples", json!([-1, 2, 3])),
            ("/cases/0/summary/median", json!(3)),
            ("/cases/0/stdout", Value::Null),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = invalid;
            inputs.push(json!({"current":original,"baseline":changed}));
        }
        for seed in [
            "-9223372036854775808",
            "9223372036854775807",
            "-00000000000000000000000000000000",
            "00042",
        ] {
            let mut changed = original.clone();
            changed["config"]["seed"] = json!(seed);
            inputs.push(json!({"current":original,"baseline":changed}));
        }
        for (pointer, value) in [
            ("/metadata/languageVersion", json!("1.0.0")),
            ("/metadata/target/identity", json!("other")),
            ("/metadata/host/cpu", json!("other")),
            ("/metadata/timerResolutionNs", json!(2)),
            ("/cases/0/inputSize", json!(1)),
            ("/cases/0/name", json!("other")),
            ("/cases/0/status", json!("failed")),
            ("/config/regressionThresholdPercent", json!(0)),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            inputs.push(json!({"current":changed,"baseline":original}));
        }
        let mut extra = original.clone();
        extra["metadata"]["target"]["extra"] = json!("ignored");
        inputs.push(json!({"current":original,"baseline":extra}));
        let mut duplicate = original.clone();
        duplicate["cases"]
            .as_array_mut()
            .unwrap()
            .push(original["cases"][0].clone());
        inputs.push(json!({"current":original,"baseline":duplicate}));
        for value in [0.0, 1.0, 2.1, 2.1000000000000005, 1e30] {
            let mut changed = original.clone();
            changed["cases"][0]["summary"]["median"] = json!(value);
            inputs.push(json!({"current":changed,"baseline":original}));
        }
        let scratch = tempfile::tempdir().unwrap();
        let path = scratch.path().join("inputs.json");
        fs::write(&path, serde_json::to_vec(&inputs).unwrap()).unwrap();
        let module = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime/ts/src/benchmark-baseline.ts");
        let script = format!("import {{compareBaseline}} from {}; const inputs = await Bun.file(process.argv.at(-1)).json(); console.log(JSON.stringify(inputs.map(x => {{ try {{ return {{result:compareBaseline(x.current,x.baseline)}}; }} catch(error) {{ return {{error:error.message}}; }} }})));", serde_json::to_string(&module).unwrap());
        let output = Command::new("bun")
            .args(["--eval", &script])
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(expected.len(), inputs.len());
        for (input, expected) in inputs.iter().zip(expected) {
            let actual = match compare(&input["current"], &input["baseline"]) {
                Ok(result) => json!({"result":result}),
                Err(error) => json!({"error":error}),
            };
            assert_eq!(actual, expected, "{input}");
        }
    }
}
