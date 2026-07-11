use super::environment::RenderedEnvironment;
use crate::execution::{exit, trace};
use crate::execution_case::FailureRenderer;

pub(super) fn render(
    environment: &RenderedEnvironment,
    failure_renderer: &FailureRenderer,
) -> String {
    let trace = environment.trace_expression.as_ref().map_or_else(
        String::new,
        |expression| {
            format!(
                "    writeFileSync(new URL(\"./{}\", import.meta.url), JSON.stringify({expression}));\n",
                trace::OBSERVATION_FILE
            )
        },
    );
    let failure = match failure_renderer {
        FailureRenderer::Never => "  process.exitCode = 1;\n".to_owned(),
        FailureRenderer::Show { .. } => {
            "  const renderedFailure = _ssrg_failureShow.show(result.error);\n\
               if (typeof renderedFailure !== \"string\") {\n\
                 throw new TypeError(\"Show dictionary returned a non-string value\");\n\
               }\n\
               process.stderr.write(renderedFailure.endsWith(\"\\n\") ? renderedFailure : renderedFailure + \"\\n\");\n\
               process.exitCode = 1;\n"
                .to_owned()
        }
    };
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
         {failure}\
         }}\n",
        exit::OBSERVATION_FILE
    )
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::execution::entry::environment::RenderedEnvironment;
    use crate::execution_case::{DictionaryImport, FailureRenderer};

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
        let traced = render(
            &environment(Some("operationTrace")),
            &FailureRenderer::Never,
        );
        assert!(traced.contains(".seseragi-operation-trace.json"));
        assert!(traced.contains("JSON.stringify(operationTrace)"));

        let untraced = render(&environment(None), &FailureRenderer::Never);
        assert!(!untraced.contains("seseragi-operation-trace"));
    }

    #[test]
    fn renders_typed_failures_with_one_conditional_trailing_newline() {
        let rendered = render(
            &environment(None),
            &FailureRenderer::Show {
                dictionary: DictionaryImport {
                    module: "./main.ts".to_owned(),
                    export: "showAppError".to_owned(),
                },
            },
        );

        assert!(rendered.contains("_ssrg_failureShow.show(result.error)"));
        assert!(rendered.contains(
            "renderedFailure.endsWith(\"\\n\") ? renderedFailure : renderedFailure + \"\\n\""
        ));
        assert!(rendered.contains("typeof renderedFailure !== \"string\""));
        assert!(rendered.contains("throw new TypeError"));
    }

    #[test]
    fn keeps_never_failure_observation_without_a_show_call() {
        let rendered = render(&environment(None), &FailureRenderer::Never);

        assert!(!rendered.contains("_ssrg_failureShow"));
        assert!(rendered.contains("process.exitCode = 1"));
    }
}
