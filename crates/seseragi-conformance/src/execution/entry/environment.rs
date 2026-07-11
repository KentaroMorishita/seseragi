use crate::execution_case::environment::{EnvironmentPlan, HostAdapter};

mod console;

pub(super) struct RenderedEnvironment {
    pub(super) imports: String,
    pub(super) setup: String,
    pub(super) expression: String,
    pub(super) cleanup: String,
    pub(super) trace_expression: Option<String>,
}

impl RenderedEnvironment {
    pub(super) fn requires_cleanup(&self) -> bool {
        !self.cleanup.is_empty()
    }
}

pub(super) fn render(plan: &EnvironmentPlan) -> RenderedEnvironment {
    if plan.bindings().is_empty() {
        return RenderedEnvironment {
            imports: String::new(),
            setup: String::new(),
            expression: "{}".to_owned(),
            cleanup: String::new(),
            trace_expression: None,
        };
    }

    let console = console::render(plan.bindings());
    let imports_stdin = plan
        .bindings()
        .iter()
        .any(|binding| binding.adapter() == HostAdapter::ProcessStdin);
    let mut imports = String::new();
    imports.push_str(&console.imports);
    if imports_stdin {
        imports.push_str("import { createProcessStdin } from \"@seseragi/runtime/stdin\";\n");
    }

    let mut setup = String::new();
    setup.push_str(&console.setup);
    let mut properties = Vec::with_capacity(plan.bindings().len());
    let mut stdin_bindings = Vec::new();
    for binding in plan.bindings() {
        let value = match binding.adapter() {
            HostAdapter::CaptureConsole => console
                .adapter_for(binding.field())
                .expect("every capture-console binding has a rendered adapter")
                .to_owned(),
            HostAdapter::ProcessStdin => {
                let ordinal = stdin_bindings.len();
                let local = if ordinal == 0 {
                    "stdinAdapter".to_owned()
                } else {
                    format!("stdinAdapter{}", ordinal + 1)
                };
                setup.push_str(&format!("const {local} = createProcessStdin();\n"));
                stdin_bindings.push(local.clone());
                local
            }
        };
        let field = serde_json::to_string(binding.field())
            .expect("a Rust string always encodes as a JSON property");
        properties.push(format!("{field}: {value}"));
    }
    setup.push_str(&format!(
        "const environment = {{ {} }};\n",
        properties.join(", ")
    ));

    let cleanup = stdin_bindings
        .iter()
        .rev()
        .map(|binding| format!("  {binding}.close();\n"))
        .collect();
    RenderedEnvironment {
        imports,
        setup,
        expression: "environment".to_owned(),
        cleanup,
        trace_expression: console.trace_expression,
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::execution_case::environment::parse_environment_plan;
    use serde_json::json;

    #[test]
    fn keeps_an_empty_environment_as_an_object_literal() {
        let environment = render(&parse_environment_plan(&json!({}), false).unwrap());

        assert_eq!(environment.expression, "{}");
        assert!(environment.imports.is_empty());
        assert!(environment.setup.is_empty());
        assert!(!environment.requires_cleanup());
    }

    #[test]
    fn renders_console_and_closable_stdin_adapters() {
        let plan = parse_environment_plan(
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
        let environment = render(&plan);

        assert!(environment.imports.contains("liveConsole"));
        assert!(environment.imports.contains("createProcessStdin"));
        assert!(environment.setup.contains("const operationTrace"));
        assert!(environment.setup.contains("liveConsole.print(value)"));
        assert!(environment.setup.contains("liveConsole.println(value)"));
        assert!(environment
            .setup
            .contains("const stdinAdapter = createProcessStdin();"));
        assert!(environment
            .setup
            .contains("\"console\": consoleAdapter, \"stdin\": stdinAdapter"));
        assert_eq!(environment.expression, "environment");
        assert_eq!(environment.cleanup, "  stdinAdapter.close();\n");
        assert_eq!(
            environment.trace_expression.as_deref(),
            Some("operationTrace")
        );
    }
}
