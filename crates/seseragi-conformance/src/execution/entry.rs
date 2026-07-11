use std::fs;
use std::path::Path;

use crate::execution_case::environment::EnvironmentPlan;

mod environment;
mod observations;

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
    environment: &EnvironmentPlan,
) -> Result<(), String> {
    fs::write(
        execution_dir.join("entry.ts"),
        entry_source(entry_export, invocation, environment),
    )
    .map_err(|error| format!("failed to write execution entry.ts: {error}"))
}

fn entry_source(
    entry_export: &str,
    invocation: Invocation,
    environment_plan: &EnvironmentPlan,
) -> String {
    match invocation {
        Invocation::Effect { arguments } => {
            let call = render_entry_call(entry_export, &arguments);
            let environment = environment::render(environment_plan);
            let imports = format!(
                "import {{ writeFileSync }} from \"node:fs\";\n\
                 import {{ run }} from \"@seseragi/runtime/effect\";\n\
                 {}\
                 import {{ {entry_export} }} from \"./main.ts\";\n",
                environment.imports
            );
            let execution = format!(
                "let result: {{ kind: \"success\"; value: unknown }} | {{ kind: \"failure\"; error: unknown }} | undefined = undefined;\n\
                 let hasRuntimeDefect = false;\n\
                 try {{\n\
                   result = await run({call}, {});\n\
                 }} catch (_runtimeDefect) {{\n\
                   hasRuntimeDefect = true;\n\
                 }}\n",
                environment.expression
            );
            let cleanup = if environment.requires_cleanup() {
                format!(
                    "try {{\n{}}} catch (_cleanupDefect) {{\n  hasRuntimeDefect = true;\n}}\n",
                    environment.cleanup
                )
            } else {
                String::new()
            };
            let completion = observations::render(&environment);
            let guarded_host = format!("{}{execution}{cleanup}{completion}", environment.setup);
            format!(
                "{imports}try {{\n{}}} catch (_hostDefect) {{\n  process.stderr.write(\"seseragi: runtime defect\\n\");\n  process.exitCode = 70;\n}}\n",
                indent(&guarded_host)
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

fn indent(source: &str) -> String {
    source.lines().map(|line| format!("  {line}\n")).collect()
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
    use crate::execution_case::environment::parse_environment_plan;
    use serde_json::json;

    fn empty_environment() -> crate::execution_case::environment::EnvironmentPlan {
        parse_environment_plan(&json!({}), false).unwrap()
    }

    #[test]
    fn keeps_effect_execution_at_the_runner_boundary() {
        let source = entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
            &empty_environment(),
        );

        assert!(source.contains("await run(main(undefined), {})"));
        assert!(source.contains(".seseragi-effect-exit.json"));
        assert!(source.contains("result.value === undefined ? \"Unit\""));
        assert!(source.contains("error: result.error"));
        assert!(!source.contains("throw result.error"));
        assert!(source.contains("process.exitCode = 1"));
        assert!(source.contains("process.exitCode = 70"));
        assert!(source.contains("seseragi: runtime defect\\n"));
        assert!(source.contains("catch (_hostDefect)"));
    }

    #[test]
    fn invokes_pure_unit_entry_without_the_effect_runner() {
        let source = entry_source(
            "values",
            Invocation::PureJson {
                arguments: vec![InvocationArgument::Unit],
            },
            &empty_environment(),
        );

        assert!(source.contains("JSON.stringify(values(undefined))"));
        assert!(!source.contains("@seseragi/runtime/effect"));
        assert!(!source.contains("seseragi-effect-exit"));
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
            &empty_environment(),
        );

        assert!(source.contains("parse(\"rock\\n\\\"quoted\\\"\")(undefined)"));
    }

    #[test]
    fn runs_with_host_services_and_closes_stdin_before_observing_exit() {
        let environment = parse_environment_plan(
            &json!({
                "requiredEnvironment": {
                    "kind": "record",
                    "closed": true,
                    "fields": [
                        { "name": "console", "type": "Console" },
                        { "name": "stdin", "type": "Stdin" }
                    ]
                },
                "hostEnvironment": {
                    "closed": false,
                    "services": [
                        { "field": "console", "type": "Console", "adapter": "capture-console" },
                        { "field": "stdin", "type": "Stdin", "adapter": "process-stdin" }
                    ]
                }
            }),
            true,
        )
        .unwrap();
        let source = entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
            &environment,
        );

        assert!(source.contains("@seseragi/runtime/console"));
        assert!(source.contains("@seseragi/runtime/stdin"));
        assert!(source.contains("await run(main(undefined), environment)"));
        assert!(source.contains("try {"));
        assert!(source.contains("  stdinAdapter.close();"));
        assert!(
            source.find("stdinAdapter.close").unwrap()
                < source.find("writeFileSync(new URL").unwrap()
        );
        assert!(
            source.find("stdinAdapter.close").unwrap()
                < source.find(".seseragi-operation-trace.json").unwrap()
        );
        assert!(source.contains("JSON.stringify(operationTrace)"));
        assert!(source.contains("catch (_cleanupDefect)"));
    }

    #[test]
    fn suppresses_exit_observation_when_run_or_cleanup_has_a_defect() {
        let source = entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
            &empty_environment(),
        );

        let defect_guard = source.find("if (!hasRuntimeDefect").unwrap();
        let observation = source.find("const observation").unwrap();
        assert!(defect_guard < observation);
        assert!(source.contains("catch (_runtimeDefect)"));
        assert!(source.contains("catch (_observationDefect)"));
        assert!(!source.contains("String(result.error)"));
    }
}
