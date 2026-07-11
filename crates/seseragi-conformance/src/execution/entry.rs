use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Invocation {
    Effect { arguments: Vec<InvocationArgument> },
    PureJson { arguments: Vec<InvocationArgument> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InvocationArgument {
    Unit,
    String(String),
}

pub(super) fn write_entry(
    execution_dir: &Path,
    entry_export: &str,
    invocation: Invocation,
) -> Result<(), String> {
    fs::write(
        execution_dir.join("entry.ts"),
        entry_source(entry_export, invocation),
    )
    .map_err(|error| format!("failed to write execution entry.ts: {error}"))
}

fn entry_source(entry_export: &str, invocation: Invocation) -> String {
    match invocation {
        Invocation::Effect { arguments } => {
            let call = render_entry_call(entry_export, &arguments);
            format!(
                "import {{ run }} from \"@seseragi/runtime/effect\";\n\
                 import {{ {entry_export} }} from \"./main.ts\";\n\
                 const result = await run({call}, {{}});\n\
                 if (result.kind === \"failure\") throw result.error;\n"
            )
        }
        Invocation::PureJson { arguments } => {
            let call = render_entry_call(entry_export, &arguments);
            format!(
                "import {{ {entry_export} }} from \"./main.ts\";\n\
                 process.stdout.write(JSON.stringify({call}) + \"\\n\");\n"
            )
        }
    }
}

fn render_entry_call(entry_export: &str, arguments: &[InvocationArgument]) -> String {
    arguments
        .iter()
        .fold(entry_export.to_owned(), |call, argument| {
            format!("{call}({})", render_argument(argument))
        })
}

fn render_argument(argument: &InvocationArgument) -> String {
    match argument {
        InvocationArgument::Unit => "undefined".to_owned(),
        InvocationArgument::String(value) => {
            serde_json::to_string(value).expect("a Rust string always encodes as JSON")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{entry_source, Invocation, InvocationArgument};

    #[test]
    fn keeps_effect_execution_at_the_runner_boundary() {
        let source = entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
        );

        assert!(source.contains("await run(main(undefined), {})"));
    }

    #[test]
    fn invokes_pure_unit_entry_without_the_effect_runner() {
        let source = entry_source(
            "values",
            Invocation::PureJson {
                arguments: vec![InvocationArgument::Unit],
            },
        );

        assert!(source.contains("JSON.stringify(values(undefined))"));
        assert!(!source.contains("@seseragi/runtime/effect"));
    }

    #[test]
    fn escapes_string_arguments_and_preserves_curried_application() {
        let source = entry_source(
            "parse",
            Invocation::PureJson {
                arguments: vec![
                    InvocationArgument::String("rock\n\"quoted\"".to_owned()),
                    InvocationArgument::Unit,
                ],
            },
        );

        assert!(source.contains("parse(\"rock\\n\\\"quoted\\\"\")(undefined)"));
    }
}
