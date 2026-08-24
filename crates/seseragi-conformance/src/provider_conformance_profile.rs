use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

pub(crate) fn check_provider_conformance_profile_case(
    root: &Path,
    case: &Path,
) -> Result<(), String> {
    let raw = fs::read_to_string(case.join("profile.json"))
        .map_err(|error| format!("failed to read provider conformance profile: {error}"))?;
    check_provider_conformance_profile(root, &raw)
}

fn check_provider_conformance_profile(root: &Path, raw: &str) -> Result<(), String> {
    let profile: ConformanceProfile = serde_json::from_str(raw)
        .map_err(|error| format!("failed to parse provider conformance profile: {error}"))?;
    if profile.schema != 1
        || profile.kind != "provider-conformance-profile"
        || profile.identity != "seseragi/provider-conformance"
        || profile.version != 1
    {
        return Err("provider conformance profile envelope is not canonical".to_owned());
    }
    check_cases(root, &profile.cases)?;
    check_capabilities(&profile.capabilities)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConformanceProfile {
    schema: u32,
    kind: String,
    identity: String,
    version: u32,
    cases: Vec<ConformanceCase>,
    capabilities: Vec<CapabilityProfile>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConformanceCase {
    id: String,
    detector: String,
    expected: String,
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapabilityProfile {
    name: String,
    contract: String,
    providers: Vec<String>,
    shapes: Vec<String>,
    cases: Vec<String>,
}

fn check_cases(root: &Path, cases: &[ConformanceCase]) -> Result<(), String> {
    let expected = BTreeMap::from([
        ("success", ("terminal", "success")),
        ("typed-failure", ("terminal", "typed-failure")),
        ("defect", ("terminal", "defect")),
        (
            "cancellation",
            (
                "cancel-notification-and-late-completion",
                "at-most-once-and-discarded",
            ),
        ),
        ("cleanup", ("acquire-release-active", "balanced-and-zero")),
        (
            "concurrency",
            ("overlap-and-settlement", "independent-and-complete"),
        ),
        (
            "invalid-value",
            (
                "boundary-classification",
                "defect-without-application-value",
            ),
        ),
        (
            "mismatch",
            ("entry-evaluation-count", "zero-before-resolution-failure"),
        ),
        (
            "ambiguity",
            ("entry-evaluation-count", "zero-before-resolution-failure"),
        ),
        ("leak", ("active-handles-after-cleanup", "zero")),
    ]);
    let mut actual = BTreeMap::new();
    for case in cases {
        if actual
            .insert(
                case.id.as_str(),
                (case.detector.as_str(), case.expected.as_str()),
            )
            .is_some()
        {
            return Err(format!(
                "provider conformance case is duplicated: {}",
                case.id
            ));
        }
        if case.evidence.is_empty() {
            return Err(format!(
                "provider conformance case has no executable evidence: {}",
                case.id
            ));
        }
        for evidence in &case.evidence {
            check_evidence_path(root, &case.id, evidence)?;
        }
    }
    if actual != expected {
        return Err("provider conformance case profile is incomplete or non-canonical".to_owned());
    }
    Ok(())
}

fn check_evidence_path(root: &Path, case_id: &str, evidence: &str) -> Result<(), String> {
    let path = Path::new(evidence);
    if evidence.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || !root.join(path).is_file()
    {
        return Err(format!(
            "provider conformance evidence is missing or unsafe for {case_id}: {evidence}"
        ));
    }
    Ok(())
}

fn check_capabilities(capabilities: &[CapabilityProfile]) -> Result<(), String> {
    let expected = BTreeMap::from([
        (
            "clock",
            (
                "std/clock::Clock",
                BTreeSet::from(["seseragi/runtime-bun#clock"]),
                BTreeSet::from(["one-shot", "cancellable"]),
            ),
        ),
        (
            "http-client",
            (
                "std/http::HttpClient",
                BTreeSet::from([
                    "seseragi/runtime-bun#http-client",
                    "seseragi/runtime-node#http-client",
                ]),
                BTreeSet::from(["one-shot", "subscription", "cancellable"]),
            ),
        ),
        (
            "http-server",
            (
                "std/http::HttpServer",
                BTreeSet::from([
                    "seseragi/runtime-bun#http-server",
                    "seseragi/runtime-node#http-server",
                ]),
                BTreeSet::from(["resource", "callback", "cancellable"]),
            ),
        ),
        (
            "filesystem",
            (
                "std/fs::FileSystem",
                BTreeSet::from([
                    "seseragi/runtime-bun#filesystem",
                    "seseragi/runtime-node#filesystem",
                ]),
                BTreeSet::from(["resource", "cancellable"]),
            ),
        ),
        (
            "postgresql",
            (
                "seseragi/postgres::Postgres",
                BTreeSet::from(["seseragi/runtime-postgres#pg"]),
                BTreeSet::from(["resource", "subscription", "cancellable", "external-driver"]),
            ),
        ),
        (
            "sqlite",
            (
                "seseragi/sqlite::Sqlite",
                BTreeSet::from(["seseragi/runtime-sqlite#bun"]),
                BTreeSet::from(["resource", "cancellable", "built-in-driver"]),
            ),
        ),
    ]);
    let allowed_cases = BTreeSet::from([
        "success",
        "typed-failure",
        "defect",
        "cancellation",
        "cleanup",
        "concurrency",
        "invalid-value",
        "mismatch",
        "ambiguity",
        "leak",
    ]);
    let required = BTreeSet::from(["success", "defect", "invalid-value"]);
    let mut actual = BTreeMap::new();
    for capability in capabilities {
        let cases = capability
            .cases
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let shapes = capability
            .shapes
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let providers = capability
            .providers
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if cases.len() != capability.cases.len()
            || !cases.is_subset(&allowed_cases)
            || shapes.len() != capability.shapes.len()
            || providers.len() != capability.providers.len()
            || providers.contains("")
            || !required.is_subset(&cases)
            || (shapes.contains("cancellable") && !cases.contains("cancellation"))
            || (shapes.contains("resource")
                && !(cases.contains("cleanup") && cases.contains("leak")))
        {
            return Err(format!(
                "provider capability profile misses required cases: {}",
                capability.name
            ));
        }
        if actual
            .insert(
                capability.name.as_str(),
                (capability.contract.as_str(), providers, shapes),
            )
            .is_some()
        {
            return Err(format!(
                "provider capability profile is duplicated: {}",
                capability.name
            ));
        }
    }
    if actual != expected {
        return Err("provider conformance must cover all implemented capability shapes".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_provider_conformance_profile, check_provider_conformance_profile_case};
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn fixture() -> &'static str {
        include_str!(
            "../../../examples/spec/artifacts/provider-conformance-profile-schema-1/core/profile.json"
        )
    }

    #[test]
    fn accepts_committed_provider_conformance_profile() {
        let case =
            root().join("examples/spec/artifacts/provider-conformance-profile-schema-1/core");
        check_provider_conformance_profile_case(&root(), &case).unwrap();
    }

    #[test]
    fn rejects_missing_leak_detection_and_unknown_fields() {
        let mut value = serde_json::from_str::<serde_json::Value>(fixture()).unwrap();
        value["cases"]
            .as_array_mut()
            .unwrap()
            .retain(|case| case["id"] != "leak");
        let raw = serde_json::to_string(&value).unwrap();
        assert!(check_provider_conformance_profile(&root(), &raw)
            .unwrap_err()
            .contains("incomplete"));

        let raw = fixture().replace("\"schema\": 1", "\"fallback\": true, \"schema\": 1");
        assert!(check_provider_conformance_profile(&root(), &raw)
            .unwrap_err()
            .contains("unknown field `fallback`"));
    }

    #[test]
    fn rejects_resource_profile_without_leak_case_and_missing_evidence() {
        let mut value = serde_json::from_str::<serde_json::Value>(fixture()).unwrap();
        value["capabilities"][2]["cases"]
            .as_array_mut()
            .unwrap()
            .retain(|case| case != "leak");
        let raw = serde_json::to_string(&value).unwrap();
        assert!(check_provider_conformance_profile(&root(), &raw)
            .unwrap_err()
            .contains("misses required cases"));

        let raw = fixture().replace(
            "runtime/ts/probes/clock-provider.ts",
            "runtime/ts/probes/not-present.ts",
        );
        assert!(check_provider_conformance_profile(&root(), &raw)
            .unwrap_err()
            .contains("evidence is missing"));
    }
}
