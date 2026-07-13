use crate::{FailureRenderer, HostService, MainContract};

pub(super) fn entry_source(contract: &MainContract, entry_module: &str) -> String {
    let mut imports = vec!["import { run } from \"@seseragi/runtime/effect\";".to_owned()];
    let mut setup = Vec::new();
    let mut fields = Vec::new();
    let mut cleanup = Vec::new();
    let mut imports_console = false;
    let mut imports_stdin = false;
    for (index, binding) in contract.environment.iter().enumerate() {
        let field = format!("{:?}", binding.field);
        match binding.service {
            HostService::Console => {
                if !imports_console {
                    imports.push(
                        "import { liveConsole } from \"@seseragi/runtime/console\";".to_owned(),
                    );
                    imports_console = true;
                }
                fields.push(format!("{field}: liveConsole"));
            }
            HostService::Stdin => {
                if !imports_stdin {
                    imports.push(
                        "import { createProcessStdin } from \"@seseragi/runtime/stdin\";"
                            .to_owned(),
                    );
                    imports_stdin = true;
                }
                let local = format!("stdinAdapter{index}");
                setup.push(format!("const {local} = createProcessStdin();"));
                fields.push(format!("{field}: {local}"));
                cleanup.push(format!("{local}.close();"));
            }
        }
    }
    let failure = match &contract.failure_renderer {
        FailureRenderer::Never => {
            imports.push(format!("import {{ main }} from \"{entry_module}\";"));
            "process.stderr.write(\"seseragi: unreachable typed failure\\n\");\n  process.exitCode = 1;".to_owned()
        }
        FailureRenderer::Show { module, export } => {
            if module == "./main.ts" {
                imports.push(format!(
                    "import {{ main, {export} as failureShow }} from \"{entry_module}\";"
                ));
            } else {
                imports.push(format!("import {{ main }} from \"{entry_module}\";"));
                imports.push(format!(
                    "import {{ {export} as failureShow }} from \"{module}\";"
                ));
            }
            "const message = failureShow.show(result.error);\n  if (typeof message !== \"string\") throw new TypeError(\"Show dictionary returned a non-string value\");\n  process.stderr.write(message.endsWith(\"\\n\") ? message : message + \"\\n\");\n  process.exitCode = 1;".to_owned()
        }
    };
    let cleanup_source = cleanup
        .iter()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}\n{}\nconst environment = {{ {} }};\nlet result;\nlet hasRuntimeDefect = false;\ntry {{\n  result = await run(main(undefined), environment);\n}} catch (_runDefect) {{\n  hasRuntimeDefect = true;\n}}\ntry {{\n{}\n}} catch (_cleanupDefect) {{\n  hasRuntimeDefect = true;\n}}\nif (hasRuntimeDefect) {{\n  process.stderr.write(\"seseragi: runtime defect\\n\");\n  process.exitCode = 70;\n}} else if (result?.kind === \"failure\") {{\n  {}\n}}\n",
        imports.join("\n"),
        setup.join("\n"),
        fields.join(", "),
        cleanup_source,
        failure,
    )
}

#[cfg(test)]
mod tests {
    use super::entry_source;
    use crate::{EnvironmentBinding, FailureRenderer, HostService, MainContract};

    #[test]
    fn prepares_live_process_services_and_typed_failure_rendering() {
        let source = entry_source(
            &MainContract {
                environment: vec![
                    EnvironmentBinding {
                        field: "console".to_owned(),
                        service: HostService::Console,
                    },
                    EnvironmentBinding {
                        field: "stdin".to_owned(),
                        service: HostService::Stdin,
                    },
                ],
                failure_renderer: FailureRenderer::Show {
                    module: "./main.ts".to_owned(),
                    export: "__ssrg$instance$Show$0".to_owned(),
                },
            },
            "./main.ts",
        );

        assert!(source.contains("liveConsole"));
        assert!(source.contains("createProcessStdin"));
        assert!(source.contains("await run(main(undefined), environment)"));
        assert!(source.contains("failureShow.show(result.error)"));
        assert!(source.contains("stdinAdapter1.close()"));
        assert!(source.contains("catch (_cleanupDefect)"));
        assert!(
            source.find("stdinAdapter1.close()").unwrap()
                < source.find("failureShow.show(result.error)").unwrap()
        );
    }

    #[test]
    fn imports_each_host_adapter_once_for_multiple_service_fields() {
        let source = entry_source(
            &MainContract {
                environment: vec![
                    EnvironmentBinding {
                        field: "first".to_owned(),
                        service: HostService::Console,
                    },
                    EnvironmentBinding {
                        field: "second".to_owned(),
                        service: HostService::Console,
                    },
                ],
                failure_renderer: FailureRenderer::Never,
            },
            "./main.ts",
        );

        assert_eq!(source.matches("import { liveConsole }").count(), 1);
        assert!(source.contains("\"first\": liveConsole"));
        assert!(source.contains("\"second\": liveConsole"));
    }
}
