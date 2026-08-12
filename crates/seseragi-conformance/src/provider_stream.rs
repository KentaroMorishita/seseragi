use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_stream_case(case: &Path) -> Result<(), String> {
    let path = case.join("contract.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider stream contract: {error}"))?;
    check_provider_stream(&raw)
}

fn check_provider_stream(raw: &str) -> Result<(), String> {
    let contract: StreamContract = serde_json::from_str(raw)
        .map_err(|error| format!("failed to parse provider stream contract: {error}"))?;
    if contract.schema != 1
        || contract.kind != "provider-stream-contract"
        || contract.identity != "seseragi/provider-stream"
        || contract.version != 1
    {
        return Err("provider stream contract must identify schema 1 version 1".to_owned());
    }
    check_callbacks(&contract.callbacks)?;
    check_subscription(&contract.subscription)?;
    check_backpressure(&contract.backpressure)?;
    check_overflow(&contract.overflow)?;
    check_examples(&contract.examples)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamContract {
    schema: u32,
    kind: String,
    identity: String,
    version: u32,
    callbacks: Callbacks,
    subscription: Subscription,
    backpressure: Backpressure,
    overflow: Overflow,
    examples: Vec<StreamExample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Callbacks {
    #[serde(rename = "oneShot")]
    one_shot: String,
    #[serde(rename = "multiShot")]
    multi_shot: String,
    terminal: Vec<String>,
    #[serde(rename = "afterTerminal")]
    after_terminal: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Subscription {
    registration: String,
    #[serde(rename = "duringRegistration")]
    during_registration: String,
    #[serde(rename = "registrationFailure")]
    registration_failure: String,
    unsubscribe: String,
    #[serde(rename = "consumerCancellation")]
    consumer_cancellation: String,
    #[serde(rename = "lateEvent")]
    late_event: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Backpressure {
    demand: String,
    #[serde(rename = "pullSource")]
    pull_source: String,
    #[serde(rename = "pushSource")]
    push_source: String,
    completion: String,
    failure: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Overflow {
    suspend: String,
    dropping: String,
    failing: String,
    #[serde(rename = "protocolViolation")]
    protocol_violation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamExample {
    capability: String,
    source: String,
    mode: String,
    boundary: String,
}

fn check_callbacks(callbacks: &Callbacks) -> Result<(), String> {
    if callbacks.one_shot != "exactly-one-terminal-callback"
        || callbacks.multi_shot != "zero-or-more-next-then-one-terminal"
        || callbacks.terminal != ["complete", "typed-failure", "defect"]
        || callbacks.after_terminal != "observe-and-discard"
    {
        return Err("provider callback contract is not canonical".to_owned());
    }
    Ok(())
}

fn check_subscription(subscription: &Subscription) -> Result<(), String> {
    if subscription.registration != "bridge-armed-before-host-register"
        || subscription.during_registration != "bounded-queue-until-success"
        || subscription.registration_failure != "detach-and-discard"
        || subscription.unsubscribe != "idempotent-exactly-once-effect"
        || subscription.consumer_cancellation != "stop-demand-unsubscribe-discard"
        || subscription.late_event != "observe-and-discard"
    {
        return Err("provider subscription contract is not canonical".to_owned());
    }
    Ok(())
}

fn check_backpressure(backpressure: &Backpressure) -> Result<(), String> {
    if backpressure.demand != "positive-count-at-most-outstanding"
        || backpressure.pull_source != "emit-no-more-than-demand"
        || backpressure.push_source != "explicit-positive-bounded-buffer"
        || backpressure.completion != "drain-buffer-fifo-then-complete"
        || backpressure.failure != "discard-buffer-then-fail"
    {
        return Err("provider backpressure contract is not canonical".to_owned());
    }
    Ok(())
}

fn check_overflow(overflow: &Overflow) -> Result<(), String> {
    if overflow.suspend != "only-with-provider-pause-resume"
        || overflow.dropping != "only-when-public-api-selects-strategy"
        || overflow.failing != "typed-only-when-contract-declares-overflow"
        || overflow.protocol_violation != "defect"
    {
        return Err("provider overflow contract is not canonical".to_owned());
    }
    Ok(())
}

fn check_examples(examples: &[StreamExample]) -> Result<(), String> {
    let mut capabilities = BTreeSet::new();
    for example in examples {
        if !capabilities.insert(example.capability.as_str()) {
            return Err(format!(
                "provider stream example is duplicated: {}",
                example.capability
            ));
        }
        if example.source.is_empty()
            || !matches!(example.mode.as_str(), "pull" | "push")
            || example.boundary.is_empty()
        {
            return Err("provider stream example is invalid".to_owned());
        }
    }
    let expected = BTreeSet::from(["database-cursor", "http-body", "sse", "websocket"]);
    if capabilities != expected {
        return Err(
            "provider stream must cover HTTP body, SSE, WebSocket, and database cursor".to_owned(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_provider_stream, check_provider_stream_case};
    use std::path::PathBuf;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        Demand(usize),
        Next(i32),
        Complete,
        Failure,
        Cancel,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct State {
        demand: usize,
        delivered: Vec<i32>,
        terminal: Option<&'static str>,
        unsubscribes: usize,
        late_events: usize,
    }

    fn pull_model(events: &[Event]) -> State {
        let mut state = State {
            demand: 0,
            delivered: Vec::new(),
            terminal: None,
            unsubscribes: 0,
            late_events: 0,
        };
        for event in events {
            match event {
                Event::Demand(amount) if state.terminal.is_none() => state.demand += amount,
                Event::Next(_) if state.terminal.is_some() => state.late_events += 1,
                Event::Next(value) if state.demand > 0 => {
                    state.demand -= 1;
                    state.delivered.push(*value);
                }
                Event::Next(_) => state.terminal = Some("defect"),
                Event::Complete if state.terminal.is_none() => state.terminal = Some("complete"),
                Event::Failure if state.terminal.is_none() => state.terminal = Some("failure"),
                Event::Cancel if state.terminal.is_none() => {
                    state.terminal = Some("cancellation");
                    state.unsubscribes = 1;
                }
                Event::Demand(_) | Event::Complete | Event::Failure | Event::Cancel => {}
            }
        }
        state
    }

    fn fixture() -> &'static str {
        include_str!("../../../examples/spec/artifacts/provider-stream-schema-1/core/contract.json")
    }

    #[test]
    fn accepts_committed_provider_stream_contract() {
        let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-stream-schema-1/core");
        check_provider_stream_case(&case).unwrap();
    }

    #[test]
    fn pull_source_never_delivers_beyond_demand() {
        let state = pull_model(&[
            Event::Demand(2),
            Event::Next(10),
            Event::Next(20),
            Event::Next(30),
        ]);
        assert_eq!(state.delivered, [10, 20]);
        assert_eq!(state.terminal, Some("defect"));
    }

    #[test]
    fn cancellation_unsubscribes_once_and_discards_late_events() {
        let state = pull_model(&[
            Event::Demand(1),
            Event::Cancel,
            Event::Cancel,
            Event::Next(10),
            Event::Failure,
        ]);
        assert_eq!(state.terminal, Some("cancellation"));
        assert_eq!(state.unsubscribes, 1);
        assert_eq!(state.late_events, 1);
    }

    #[test]
    fn producer_failure_and_completion_are_distinct() {
        assert_eq!(pull_model(&[Event::Complete]).terminal, Some("complete"));
        assert_eq!(pull_model(&[Event::Failure]).terminal, Some("failure"));
    }

    #[test]
    fn rejects_unknown_fields_and_unbounded_push_buffers() {
        let raw = fixture().replace("\"callbacks\":", "\"replay\": true, \"callbacks\":");
        assert!(check_provider_stream(&raw)
            .unwrap_err()
            .contains("unknown field `replay`"));
        let raw = fixture().replace(
            "\"pushSource\": \"explicit-positive-bounded-buffer\"",
            "\"pushSource\": \"unbounded-buffer\"",
        );
        assert!(check_provider_stream(&raw)
            .unwrap_err()
            .contains("backpressure contract"));
    }

    #[test]
    fn rejects_overflow_as_undeclared_typed_failure() {
        let raw = fixture().replace(
            "\"failing\": \"typed-only-when-contract-declares-overflow\"",
            "\"failing\": \"always-typed-failure\"",
        );
        assert!(check_provider_stream(&raw)
            .unwrap_err()
            .contains("overflow contract"));
    }
}
