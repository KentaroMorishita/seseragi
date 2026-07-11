use std::fs;
use std::path::Path;

use crate::execution_case::{environment::EnvironmentPlan, FailureRenderer};

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
    failure_renderer: Option<&FailureRenderer>,
    environment: &EnvironmentPlan,
) -> Result<(), String> {
    let source = entry_source(entry_export, invocation, failure_renderer, environment)?;
    fs::write(execution_dir.join("entry.ts"), source)
        .map_err(|error| format!("failed to write execution entry.ts: {error}"))
}

fn entry_source(
    entry_export: &str,
    invocation: Invocation,
    failure_renderer: Option<&FailureRenderer>,
    environment_plan: &EnvironmentPlan,
) -> Result<String, String> {
    match (invocation, failure_renderer) {
        (Invocation::Effect { arguments }, Some(failure_renderer)) => {
            let call = render_entry_call(entry_export, &arguments);
            let environment = environment::render(environment_plan);
            let (entry_import, dictionary_import) = render_imports(entry_export, failure_renderer);
            let imports = format!(
                "import {{ writeFileSync }} from \"node:fs\";\n\
                 import {{ run }} from \"@seseragi/runtime/effect\";\n\
                 {}\
                 {dictionary_import}\
                 {entry_import}",
                environment.imports,
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
            let completion = observations::render(&environment, failure_renderer);
            let guarded_host = format!("{}{execution}{cleanup}{completion}", environment.setup);
            Ok(format!(
                "{imports}try {{\n{}}} catch (_hostDefect) {{\n  process.stderr.write(\"seseragi: runtime defect\\n\");\n  process.exitCode = 70;\n}}\n",
                indent(&guarded_host)
            ))
        }
        (Invocation::Effect { .. }, None) => {
            Err("Effect execution is missing its validated failure renderer".to_owned())
        }
        (Invocation::PureJson { arguments }, None) => {
            let call = render_entry_call(entry_export, &arguments);
            Ok(format!(
                "import {{ {entry_export} }} from \"./main.ts\";\n\
                 process.stdout.write(JSON.stringify({call}) + \"\\n\");\n"
            ))
        }
        (Invocation::PureJson { .. }, Some(_)) => {
            Err("pure execution must not receive an Effect failure renderer".to_owned())
        }
    }
}

fn render_imports(entry_export: &str, failure_renderer: &FailureRenderer) -> (String, String) {
    match failure_renderer {
        FailureRenderer::Never => (
            format!("import {{ {entry_export} }} from \"./main.ts\";\n"),
            String::new(),
        ),
        FailureRenderer::Show { dictionary } if dictionary.module == "./main.ts" => (
            format!(
                "import {{ {entry_export}, {} as _ssrg_failureShow }} from \"./main.ts\";\n",
                dictionary.export
            ),
            String::new(),
        ),
        FailureRenderer::Show { dictionary } => (
            format!("import {{ {entry_export} }} from \"./main.ts\";\n"),
            format!(
                "import {{ {} as _ssrg_failureShow }} from \"{}\";\n",
                dictionary.export, dictionary.module
            ),
        ),
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
    use super::{entry_source as render_entry_source, Invocation, InvocationArgument};
    use crate::execution_case::{
        environment::parse_environment_plan, DictionaryImport, FailureRenderer,
    };
    use serde_json::json;

    fn empty_environment() -> crate::execution_case::environment::EnvironmentPlan {
        parse_environment_plan(&json!({}), false).unwrap()
    }

    fn entry_source(
        entry_export: &str,
        invocation: Invocation,
        environment: &crate::execution_case::environment::EnvironmentPlan,
    ) -> String {
        let never = FailureRenderer::Never;
        let failure_renderer = matches!(&invocation, Invocation::Effect { .. }).then_some(&never);
        render_entry_source(entry_export, invocation, failure_renderer, environment).unwrap()
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

    #[test]
    fn imports_and_uses_only_the_validated_show_dictionary_for_typed_failures() {
        let renderer = FailureRenderer::Show {
            dictionary: DictionaryImport {
                module: "./main.ts".to_owned(),
                export: "__ssrg$instance$Show$0".to_owned(),
            },
        };
        let source = render_entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
            Some(&renderer),
            &empty_environment(),
        )
        .unwrap();

        assert!(source.contains(
            "import { main, __ssrg$instance$Show$0 as _ssrg_failureShow } from \"./main.ts\";"
        ));
        assert!(source.contains("_ssrg_failureShow.show(result.error)"));
        assert!(source.contains("typeof renderedFailure !== \"string\""));
        assert!(source.contains("renderedFailure.endsWith(\"\\n\")"));
        assert!(source.contains("catch (_hostDefect)"));
        assert!(source.contains("process.exitCode = 70"));
        assert!(
            source.find("JSON.stringify(observation)").unwrap()
                < source.find("_ssrg_failureShow.show").unwrap()
        );
        assert!(
            source.find("_ssrg_failureShow.show").unwrap()
                < source.find("catch (_hostDefect)").unwrap()
        );
    }

    #[test]
    fn imports_standard_show_dictionaries_from_the_runtime_module() {
        let renderer = FailureRenderer::Show {
            dictionary: DictionaryImport {
                module: "@seseragi/runtime/show".to_owned(),
                export: "stdinErrorShow".to_owned(),
            },
        };
        let source = render_entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
            Some(&renderer),
            &empty_environment(),
        )
        .unwrap();

        assert!(source.contains(
            "import { stdinErrorShow as _ssrg_failureShow } from \"@seseragi/runtime/show\";"
        ));
        assert!(source.contains("import { main } from \"./main.ts\";"));
        assert!(!source.contains("stdinErrorShow as _ssrg_failureShow } from \"./main.ts\""));
    }

    #[test]
    fn rejects_invocation_and_failure_renderer_mode_mismatches() {
        let effect_error = render_entry_source(
            "main",
            Invocation::Effect {
                arguments: vec![InvocationArgument::Unit],
            },
            None,
            &empty_environment(),
        )
        .unwrap_err();
        assert!(effect_error.contains("missing its validated failure renderer"));

        let never = FailureRenderer::Never;
        let pure_error = render_entry_source(
            "main",
            Invocation::PureJson {
                arguments: vec![InvocationArgument::Unit],
            },
            Some(&never),
            &empty_environment(),
        )
        .unwrap_err();
        assert!(pure_error.contains("pure execution"));
    }
}
