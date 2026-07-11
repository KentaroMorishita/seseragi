use std::collections::BTreeMap;

use crate::execution_case::environment::{EnvironmentBinding, HostAdapter};

const TRACE_LOCAL: &str = "operationTrace";

pub(super) struct RenderedConsole {
    pub(super) imports: String,
    pub(super) setup: String,
    pub(super) trace_expression: Option<String>,
    adapters: BTreeMap<String, String>,
}

impl RenderedConsole {
    pub(super) fn adapter_for(&self, field: &str) -> Option<&str> {
        self.adapters.get(field).map(String::as_str)
    }
}

pub(super) fn render(bindings: &[EnvironmentBinding]) -> RenderedConsole {
    let console_bindings: Vec<_> = bindings
        .iter()
        .filter(|binding| binding.adapter() == HostAdapter::CaptureConsole)
        .collect();
    if console_bindings.is_empty() {
        return RenderedConsole {
            imports: String::new(),
            setup: String::new(),
            trace_expression: None,
            adapters: BTreeMap::new(),
        };
    }

    let mut setup = format!("const {TRACE_LOCAL}: Array<Record<string, unknown>> = [];\n");
    let mut adapters = BTreeMap::new();
    for (index, binding) in console_bindings.into_iter().enumerate() {
        let local = if index == 0 {
            "consoleAdapter".to_owned()
        } else {
            format!("consoleAdapter{}", index + 1)
        };
        setup.push_str(&render_adapter(&local, binding.field()));
        adapters.insert(binding.field().to_owned(), local);
    }
    RenderedConsole {
        imports: "import { liveConsole } from \"@seseragi/runtime/console\";\n".to_owned(),
        setup,
        trace_expression: Some(TRACE_LOCAL.to_owned()),
        adapters,
    }
}

fn render_adapter(local: &str, service: &str) -> String {
    let service = serde_json::to_string(service)
        .expect("a Rust string always encodes as a JSON string literal");
    format!(
        "const {local} = {{\n\
         \x20 async print(value: string) {{\n\
         \x20   const result = await liveConsole.print(value);\n\
         \x20   if (result.kind === \"success\") {{\n\
         \x20     {TRACE_LOCAL}.push({{ service: {service}, operation: \"print\", arguments: [value], stdout: value }});\n\
         \x20   }}\n\
         \x20   return result;\n\
         \x20 }},\n\
         \x20 async println(value: string) {{\n\
         \x20   const result = await liveConsole.println(value);\n\
         \x20   if (result.kind === \"success\") {{\n\
         \x20     {TRACE_LOCAL}.push({{ service: {service}, operation: \"println\", arguments: [value], stdout: `${{value}}\\n` }});\n\
         \x20   }}\n\
         \x20   return result;\n\
         \x20 }}\n\
         }};\n"
    )
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::execution_case::environment::parse_environment_plan;
    use serde_json::json;

    #[test]
    fn delegates_before_recording_exact_console_events() {
        let plan = parse_environment_plan(
            &json!({
                "requiredEnvironment": {
                    "kind": "record",
                    "closed": true,
                    "fields": [{ "name": "terminal", "type": "Console" }]
                },
                "hostEnvironment": {
                    "closed": false,
                    "services": [{
                        "field": "terminal",
                        "type": "Console",
                        "adapter": "capture-console"
                    }]
                }
            }),
            true,
        )
        .unwrap();
        let rendered = render(plan.bindings());

        assert_eq!(rendered.adapter_for("terminal"), Some("consoleAdapter"));
        assert!(rendered.setup.contains("service: \"terminal\""));
        assert!(rendered.setup.contains("arguments: [value]"));
        assert!(rendered.setup.contains("stdout: `${value}\\n`"));
        assert!(rendered.setup.contains("if (result.kind === \"success\")"));
        assert!(rendered.setup.contains("return result"));
        assert!(
            rendered
                .setup
                .find("await liveConsole.print(value)")
                .unwrap()
                < rendered.setup.find("operation: \"print\"").unwrap()
        );
    }
}
