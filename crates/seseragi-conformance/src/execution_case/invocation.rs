use crate::execution::{Invocation, InvocationArgument};

pub(super) fn parse_invocation(run: &serde_json::Value) -> Result<Invocation, String> {
    let arguments = parse_invocation_arguments(run)?;
    let effect = run.pointer("/invocation/effect");
    let pure = run.pointer("/invocation/pure");
    match (effect, pure) {
        (Some(effect), None)
            if effect.pointer("/cold").and_then(|value| value.as_bool()) == Some(true)
                && effect
                    .pointer("/rootScope")
                    .and_then(|value| value.as_bool())
                    == Some(true) =>
        {
            Ok(Invocation::Effect { arguments })
        }
        (None, Some(pure))
            if pure.pointer("/result").and_then(|value| value.as_str()) == Some("json") =>
        {
            Ok(Invocation::PureJson { arguments })
        }
        (Some(_), Some(_)) => {
            Err("run.json invocation must select exactly one execution mode".to_owned())
        }
        (Some(_), None) => {
            Err("run.json effect invocation must be cold and use the root scope".to_owned())
        }
        (None, Some(_)) => Err("run.json pure invocation result must be json".to_owned()),
        (None, None) => Err("run.json invocation mode is missing".to_owned()),
    }
}

fn parse_invocation_arguments(run: &serde_json::Value) -> Result<Vec<InvocationArgument>, String> {
    let legacy = run.pointer("/invocation/argument");
    let typed = run.pointer("/invocation/arguments");
    match (legacy, typed) {
        (Some(_), Some(_)) => {
            return Err("run.json invocation must not mix argument and arguments".to_owned());
        }
        (_, Some(arguments)) => {
            let arguments = arguments
                .as_array()
                .ok_or_else(|| "run.json invocation.arguments must be an array".to_owned())?;
            if arguments.is_empty() {
                return Err("run.json invocation.arguments must not be empty".to_owned());
            }
            return arguments
                .iter()
                .enumerate()
                .map(|(index, argument)| parse_invocation_argument(argument, index))
                .collect();
        }
        _ => {}
    }
    match legacy.and_then(|value| value.as_str()) {
        Some("Unit") => Ok(vec![InvocationArgument::Unit]),
        _ => Err("run.json invocation must provide typed arguments".to_owned()),
    }
}

fn parse_invocation_argument(
    argument: &serde_json::Value,
    index: usize,
) -> Result<InvocationArgument, String> {
    match argument.get("type").and_then(|value| value.as_str()) {
        Some("Unit") if argument.get("value").is_none() => Ok(InvocationArgument::Unit),
        Some("String") => argument
            .get("value")
            .and_then(|value| value.as_str())
            .map(|value| InvocationArgument::String(value.to_owned()))
            .ok_or_else(|| {
                format!("run.json invocation.arguments[{index}] String value is missing")
            }),
        Some(other) => Err(format!(
            "run.json invocation.arguments[{index}] type {other} is not supported"
        )),
        None => Err(format!(
            "run.json invocation.arguments[{index}] type is missing"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_invocation;
    use crate::execution::{Invocation, InvocationArgument};
    use serde_json::json;

    #[test]
    fn accepts_explicit_effect_and_pure_invocations() {
        assert_eq!(
            parse_invocation(&json!({
                "invocation": {
                    "argument": "Unit",
                    "effect": { "cold": true, "rootScope": true }
                }
            }))
            .unwrap(),
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit]
            }
        );
        assert_eq!(
            parse_invocation(&json!({
                "invocation": {
                    "argument": "Unit",
                    "pure": { "result": "json" }
                }
            }))
            .unwrap(),
            Invocation::PureJson {
                arguments: vec![InvocationArgument::Unit]
            }
        );
    }

    #[test]
    fn accepts_typed_string_invocation_argument() {
        assert_eq!(
            parse_invocation(&json!({
                "invocation": {
                    "arguments": [{ "type": "String", "value": "rock" }],
                    "pure": { "result": "json" }
                }
            }))
            .unwrap(),
            Invocation::PureJson {
                arguments: vec![InvocationArgument::String("rock".to_owned())]
            }
        );
    }

    #[test]
    fn rejects_ambiguous_execution_invocation() {
        let error = parse_invocation(&json!({
            "invocation": {
                "argument": "Unit",
                "effect": { "cold": true, "rootScope": true },
                "pure": { "result": "json" }
            }
        }))
        .unwrap_err();

        assert!(error.contains("exactly one"));
    }

    #[test]
    fn rejects_mixed_or_malformed_argument_forms() {
        let mixed = parse_invocation(&json!({
            "invocation": {
                "argument": "Unit",
                "arguments": [{ "type": "Unit" }],
                "pure": { "result": "json" }
            }
        }))
        .unwrap_err();
        assert!(mixed.contains("must not mix"));

        let malformed = parse_invocation(&json!({
            "invocation": {
                "arguments": {},
                "pure": { "result": "json" }
            }
        }))
        .unwrap_err();
        assert!(malformed.contains("must be an array"));
    }
}
