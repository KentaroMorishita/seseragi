use crate::execution_case::environment::{EnvironmentBinding, HostAdapter};

const CONSOLE_LOCAL: &str = "failingConsoleAdapter";
const STDIN_LOCAL: &str = "failingStdinAdapter";

pub(super) struct RenderedFailingAdapters {
    pub(super) imports: String,
    pub(super) setup: String,
    has_console: bool,
    has_stdin: bool,
}

impl RenderedFailingAdapters {
    pub(super) fn adapter_for(&self, adapter: HostAdapter) -> Option<&'static str> {
        match adapter {
            HostAdapter::FailConsole if self.has_console => Some(CONSOLE_LOCAL),
            HostAdapter::FailStdin if self.has_stdin => Some(STDIN_LOCAL),
            _ => None,
        }
    }
}

pub(super) fn render(bindings: &[EnvironmentBinding]) -> RenderedFailingAdapters {
    let has_console = bindings
        .iter()
        .any(|binding| binding.adapter() == HostAdapter::FailConsole);
    let has_stdin = bindings
        .iter()
        .any(|binding| binding.adapter() == HostAdapter::FailStdin);
    if !has_console && !has_stdin {
        return RenderedFailingAdapters {
            imports: String::new(),
            setup: String::new(),
            has_console,
            has_stdin,
        };
    }

    let mut setup = String::new();
    if has_console {
        setup.push_str(&format!(
            "const {CONSOLE_LOCAL} = {{\n\
             \x20 print(_value: string) {{\n\
             \x20   return serviceFailure({{ kind: \"console-error\", message: \"injected console failure\" }} as const);\n\
             \x20 }},\n\
             \x20 println(_value: string) {{\n\
             \x20   return serviceFailure({{ kind: \"console-error\", message: \"injected console failure\" }} as const);\n\
             \x20 }}\n\
             }};\n"
        ));
    }
    if has_stdin {
        setup.push_str(&format!(
            "const {STDIN_LOCAL} = {{\n\
             \x20 readLine() {{\n\
             \x20   return serviceFailure({{ tag: \"StdinUnavailable\" }} as const);\n\
             \x20 }}\n\
             }};\n"
        ));
    }

    RenderedFailingAdapters {
        imports: "import { serviceFailure } from \"@seseragi/runtime/service\";\n".to_owned(),
        setup,
        has_console,
        has_stdin,
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::execution_case::environment::parse_environment_plan;
    use serde_json::json;

    #[test]
    fn emits_one_stateless_adapter_per_failure_service_type() {
        let plan = parse_environment_plan(
            &json!({
                "requiredEnvironment": {
                    "kind": "record",
                    "closed": true,
                    "fields": [
                        { "name": "first", "type": "Console" },
                        { "name": "second", "type": "Console" }
                    ]
                },
                "hostEnvironment": {
                    "closed": false,
                    "services": [
                        { "field": "first", "type": "Console", "adapter": "fail-console" },
                        { "field": "second", "type": "Console", "adapter": "fail-console" }
                    ]
                }
            }),
            true,
        )
        .unwrap();
        let rendered = render(plan.bindings());

        assert_eq!(
            rendered
                .setup
                .matches("const failingConsoleAdapter")
                .count(),
            1
        );
        assert_eq!(
            rendered.setup.matches("injected console failure").count(),
            2
        );
        assert!(!rendered.setup.contains("failingStdinAdapter"));
    }
}
