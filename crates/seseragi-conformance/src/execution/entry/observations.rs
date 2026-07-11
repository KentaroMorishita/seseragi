use super::environment::RenderedEnvironment;
use crate::execution::{exit, trace};

pub(super) fn render(environment: &RenderedEnvironment) -> String {
    let trace = environment.trace_expression.as_ref().map_or_else(
        String::new,
        |expression| {
            format!(
                "    writeFileSync(new URL(\"./{}\", import.meta.url), JSON.stringify({expression}));\n",
                trace::OBSERVATION_FILE
            )
        },
    );
    // The conformance host cannot resolve a generated Show dictionary yet.
    // Preserve structured failures without leaking Bun's raw thrown-value text.
    format!(
        "if (!hasRuntimeDefect && result !== undefined) {{\n\
         \x20 try {{\n\
         \x20   const observation = result.kind === \"success\"\n\
         \x20     ? {{ kind: \"success\", value: result.value === undefined ? \"Unit\" : result.value }}\n\
         \x20     : {{ kind: \"failure\", error: result.error }};\n\
         \x20   writeFileSync(new URL(\"./{}\", import.meta.url), JSON.stringify(observation));\n\
         {trace}\
         \x20 }} catch (_observationDefect) {{\n\
         \x20   hasRuntimeDefect = true;\n\
         \x20 }}\n\
         }}\n\
         if (hasRuntimeDefect) {{\n\
         \x20 process.stderr.write(\"seseragi: runtime defect\\n\");\n\
         \x20 process.exitCode = 70;\n\
         }} else if (result?.kind === \"failure\") {{\n\
         \x20 process.exitCode = 1;\n\
         }}\n",
        exit::OBSERVATION_FILE
    )
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::execution::entry::environment::RenderedEnvironment;

    fn environment(trace_expression: Option<&str>) -> RenderedEnvironment {
        RenderedEnvironment {
            imports: String::new(),
            setup: String::new(),
            expression: "environment".to_owned(),
            cleanup: String::new(),
            trace_expression: trace_expression.map(str::to_owned),
        }
    }

    #[test]
    fn writes_trace_only_for_a_capturing_environment() {
        let traced = render(&environment(Some("operationTrace")));
        assert!(traced.contains(".seseragi-operation-trace.json"));
        assert!(traced.contains("JSON.stringify(operationTrace)"));

        let untraced = render(&environment(None));
        assert!(!untraced.contains("seseragi-operation-trace"));
    }
}
