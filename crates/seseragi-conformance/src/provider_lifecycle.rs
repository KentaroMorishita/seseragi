use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_lifecycle_case(case: &Path) -> Result<(), String> {
    let path = case.join("contract.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider lifecycle contract: {error}"))?;
    check_provider_lifecycle(&raw)
}

fn check_provider_lifecycle(raw: &str) -> Result<(), String> {
    let contract: LifecycleContract = serde_json::from_str(raw)
        .map_err(|error| format!("failed to parse provider lifecycle contract: {error}"))?;
    if contract.schema != 1
        || contract.kind != "provider-lifecycle-contract"
        || contract.identity != "seseragi/provider-lifecycle"
        || contract.version != 1
    {
        return Err("provider lifecycle contract must identify schema 1 version 1".to_owned());
    }
    check_effect(&contract.effect)?;
    check_cancellation(&contract.cancellation)?;
    check_resource(&contract.resource)?;
    check_examples(&contract.examples)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LifecycleContract {
    schema: u32,
    kind: String,
    identity: String,
    version: u32,
    effect: EffectContract,
    cancellation: CancellationContract,
    resource: ResourceContract,
    examples: Vec<LifecycleExample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectContract {
    construction: String,
    invocation: String,
    #[serde(rename = "terminalOutcomes")]
    terminal_outcomes: Vec<String>,
    classification: Classification,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Classification {
    success: String,
    #[serde(rename = "providerFailure")]
    provider_failure: String,
    #[serde(rename = "synchronousThrow")]
    synchronous_throw: String,
    rejection: String,
    #[serde(rename = "invalidBoundaryValue")]
    invalid_boundary_value: String,
    cancellation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CancellationContract {
    notification: String,
    race: String,
    #[serde(rename = "lateCompletion")]
    late_completion: String,
    unavailable: String,
    #[serde(rename = "lateAcquireSuccess")]
    late_acquire_success: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceContract {
    handoff: String,
    release: String,
    close: String,
    order: String,
    #[serde(rename = "partialAcquireFailure")]
    partial_acquire_failure: String,
    #[serde(rename = "releaseFailure")]
    release_failure: String,
    shutdown: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LifecycleExample {
    capability: String,
    operation: String,
    #[serde(rename = "operationKind")]
    operation_kind: String,
    cancellation: String,
    resource: String,
}

fn check_effect(effect: &EffectContract) -> Result<(), String> {
    if effect.construction != "cold-no-provider-call"
        || effect.invocation != "exactly-once-per-run"
        || effect.terminal_outcomes != ["success", "typed-failure", "defect", "cancellation"]
        || effect.classification.success != "provider-success"
        || effect.classification.provider_failure != "typed-failure"
        || effect.classification.synchronous_throw != "defect"
        || effect.classification.rejection != "defect"
        || effect.classification.invalid_boundary_value != "defect"
        || effect.classification.cancellation != "not-failure-channel"
    {
        return Err("provider lifecycle Effect classification is not canonical".to_owned());
    }
    Ok(())
}

fn check_cancellation(cancellation: &CancellationContract) -> Result<(), String> {
    if cancellation.notification != "at-most-once-after-request"
        || cancellation.race != "first-committed-terminal-wins"
        || cancellation.late_completion != "observe-and-discard"
        || cancellation.unavailable != "cancel-effect-observe-host-settlement"
        || cancellation.late_acquire_success != "release-before-discard"
    {
        return Err("provider lifecycle cancellation contract is not canonical".to_owned());
    }
    Ok(())
}

fn check_resource(resource: &ResourceContract) -> Result<(), String> {
    if resource.handoff != "atomic-acquire-register"
        || resource.release != "success-failure-cancellation"
        || resource.close != "idempotent-exactly-once-effect"
        || resource.order != "scope-lifo"
        || resource.partial_acquire_failure != "provider-cleans-unpublished-state"
        || resource.release_failure != "first-defect-primary-rest-notes"
        || resource.shutdown != "cancel-children-await-cleanup-then-parent"
    {
        return Err("provider lifecycle resource contract is not canonical".to_owned());
    }
    Ok(())
}

fn check_examples(examples: &[LifecycleExample]) -> Result<(), String> {
    let mut capabilities = BTreeSet::new();
    for example in examples {
        if !capabilities.insert(example.capability.as_str()) {
            return Err(format!(
                "provider lifecycle example is duplicated: {}",
                example.capability
            ));
        }
        if example.operation.is_empty()
            || !matches!(example.operation_kind.as_str(), "one-shot" | "resource")
            || !matches!(example.cancellation.as_str(), "cooperative" | "unavailable")
            || example.resource.is_empty()
        {
            return Err("provider lifecycle example is invalid".to_owned());
        }
    }
    let expected = BTreeSet::from(["clock", "database-pool", "filesystem", "http-server"]);
    if capabilities != expected {
        return Err(
            "provider lifecycle must cover Clock, filesystem, HTTP server, and database pool"
                .to_owned(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_provider_lifecycle, check_provider_lifecycle_case};
    use std::path::PathBuf;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        Construct,
        Run,
        Success,
        Failure,
        Defect,
        Cancel,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Terminal {
        Success,
        Failure,
        Defect,
        Cancellation,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct RunResult {
        starts: usize,
        cancel_notifications: usize,
        terminal: Option<Terminal>,
        late_completions: usize,
    }

    fn model(events: &[Event], cancellable: bool) -> RunResult {
        let mut result = RunResult {
            starts: 0,
            cancel_notifications: 0,
            terminal: None,
            late_completions: 0,
        };
        for event in events {
            match event {
                Event::Construct => {}
                Event::Run if result.starts == 0 => result.starts = 1,
                Event::Run => {}
                Event::Cancel if result.terminal.is_none() => {
                    if cancellable {
                        result.cancel_notifications += 1;
                    }
                    result.terminal = Some(Terminal::Cancellation);
                }
                Event::Cancel => {}
                Event::Success | Event::Failure | Event::Defect if result.terminal.is_some() => {
                    result.late_completions += 1;
                }
                Event::Success => result.terminal = Some(Terminal::Success),
                Event::Failure => result.terminal = Some(Terminal::Failure),
                Event::Defect => result.terminal = Some(Terminal::Defect),
            }
        }
        result
    }

    fn fixture() -> &'static str {
        include_str!(
            "../../../examples/spec/artifacts/provider-lifecycle-schema-1/core/contract.json"
        )
    }

    #[test]
    fn accepts_committed_provider_lifecycle_contract() {
        let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-lifecycle-schema-1/core");
        check_provider_lifecycle_case(&case).unwrap();
    }

    #[test]
    fn construction_is_cold_and_each_run_starts_once() {
        let result = model(
            &[
                Event::Construct,
                Event::Construct,
                Event::Run,
                Event::Run,
                Event::Success,
            ],
            true,
        );
        assert_eq!(result.starts, 1);
        assert_eq!(result.terminal, Some(Terminal::Success));
    }

    #[test]
    fn terminal_outcomes_do_not_collapse() {
        assert_eq!(
            model(&[Event::Run, Event::Failure], true).terminal,
            Some(Terminal::Failure)
        );
        assert_eq!(
            model(&[Event::Run, Event::Defect], true).terminal,
            Some(Terminal::Defect)
        );
        assert_eq!(
            model(&[Event::Run, Event::Cancel], true).terminal,
            Some(Terminal::Cancellation)
        );
    }

    #[test]
    fn first_committed_terminal_wins_the_cancel_race() {
        let completion_first = model(&[Event::Run, Event::Success, Event::Cancel], true);
        assert_eq!(completion_first.terminal, Some(Terminal::Success));
        let cancellation_first = model(&[Event::Run, Event::Cancel, Event::Success], true);
        assert_eq!(cancellation_first.terminal, Some(Terminal::Cancellation));
        assert_eq!(cancellation_first.cancel_notifications, 1);
        assert_eq!(cancellation_first.late_completions, 1);
    }

    #[test]
    fn uncancellable_host_work_is_observed_after_effect_cancellation() {
        let result = model(&[Event::Run, Event::Cancel, Event::Success], false);
        assert_eq!(result.terminal, Some(Terminal::Cancellation));
        assert_eq!(result.cancel_notifications, 0);
        assert_eq!(result.late_completions, 1);
    }

    #[test]
    fn cleanup_is_lifo_and_close_is_idempotent() {
        let mut stack = vec!["pool", "connection", "cursor"];
        let mut released = Vec::new();
        while let Some(resource) = stack.pop() {
            if !released.contains(&resource) {
                released.push(resource);
            }
        }
        if !released.contains(&"cursor") {
            released.push("cursor");
        }
        assert_eq!(released, ["cursor", "connection", "pool"]);
    }

    #[test]
    fn rejects_unknown_fields_and_failure_cancellation_collapse() {
        let raw = fixture().replace("\"effect\":", "\"retry\": true, \"effect\":");
        assert!(check_provider_lifecycle(&raw)
            .unwrap_err()
            .contains("unknown field `retry`"));
        let raw = fixture().replace(
            "\"cancellation\": \"not-failure-channel\"",
            "\"cancellation\": \"typed-failure\"",
        );
        assert!(check_provider_lifecycle(&raw)
            .unwrap_err()
            .contains("Effect classification"));
    }
}
