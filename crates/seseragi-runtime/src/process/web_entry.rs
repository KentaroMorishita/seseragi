use crate::{DisplayDictionary, FailureRenderer, MainContract};

pub(super) fn web_entry_source(contract: &MainContract, entry_module: &str) -> String {
    let environment = serde_json::to_string(&contract.environment)
        .expect("the validated browser environment contract must serialize");
    let mut imports = vec![
        "import { createBrowserDom } from \"@seseragi/runtime/browser/dom\";".to_owned(),
        "import { createBrowserEnvironment } from \"@seseragi/runtime/browser/host\";".to_owned(),
        "import { createEffectExecution, isEffectCancellation, run } from \"@seseragi/runtime/effect\";".to_owned(),
        format!("import {{ main }} from \"{entry_module}\";"),
    ];
    let failure = match &contract.failure_renderer {
        FailureRenderer::Never => {
            "console.error(\"seseragi: unreachable typed failure\");".to_owned()
        }
        FailureRenderer::Show {
            module,
            export,
            arguments,
        } => {
            imports.push(
                "import { renderShow as failureRenderShow } from \"@seseragi/runtime/show\";"
                    .to_owned(),
            );
            let mut dictionary_index = 0;
            let expression = display_dictionary_expression(
                &DisplayDictionary {
                    module: module.clone(),
                    export: export.clone(),
                    arguments: arguments.clone(),
                },
                entry_module,
                &mut imports,
                &mut dictionary_index,
            );
            format!(
                "const failureShow = {expression};\n    console.error(failureRenderShow(failureShow, result.error, {{ layout: \"compact\" }}));"
            )
        }
    };
    format!(
        r#"{}

const execution = createEffectExecution();
const browserDom = createBrowserDom(document, () => {{
  document.documentElement.dataset.seseragiStatus = "mounted";
}});
const unregisterDomDisposal = execution.context.onCancel(() => browserDom.dispose());
const environment = createBrowserEnvironment(
  {},
  "",
  (value) => console.log(value.endsWith("\n") ? value.slice(0, -1) : value),
  browserDom.service,
  execution.context,
);
globalThis.addEventListener("pagehide", () => {{
  void execution.cancel();
}}, {{ once: true }});

try {{
  const result = await run(main(undefined), environment, execution.context);
  if (result.kind === "failure") {{
    document.documentElement.dataset.seseragiStatus = "failed";
    {}
  }} else {{
    document.documentElement.dataset.seseragiStatus = "completed";
  }}
}} catch (error) {{
  if (!isEffectCancellation(error)) {{
    document.documentElement.dataset.seseragiStatus = "defect";
    console.error("seseragi: runtime defect", error);
  }}
}} finally {{
  unregisterDomDisposal();
  await browserDom.dispose();
}}
"#,
        imports.join("\n"),
        environment,
        failure,
    )
}

fn display_dictionary_expression(
    dictionary: &DisplayDictionary,
    entry_module: &str,
    imports: &mut Vec<String>,
    next_index: &mut usize,
) -> String {
    let index = *next_index;
    *next_index += 1;
    let local = format!("failureDisplay{index}");
    let module = if dictionary.module == "./main.ts" {
        entry_module
    } else {
        &dictionary.module
    };
    imports.push(format!(
        "import {{ {} as {local} }} from \"{module}\";",
        dictionary.export
    ));
    if dictionary.arguments.is_empty() {
        return local;
    }
    let arguments = dictionary
        .arguments
        .iter()
        .map(|argument| display_dictionary_expression(argument, entry_module, imports, next_index))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{local}({arguments})")
}

#[cfg(test)]
mod tests {
    use super::web_entry_source;
    use crate::{EnvironmentBinding, FailureRenderer, HostService, MainContract};

    #[test]
    fn starts_browser_services_and_reports_defect_details() {
        let source = web_entry_source(
            &MainContract {
                environment: vec![EnvironmentBinding {
                    field: "dom".to_owned(),
                    service: HostService::Dom,
                }],
                failure_renderer: FailureRenderer::Never,
            },
            "./main.ts",
        );

        assert!(source.contains("createBrowserDom(document"));
        assert!(source.contains(r#"[{"field":"dom","service":"dom"}]"#));
        assert!(source.contains("await run(main(undefined), environment, execution.context)"));
        assert!(source.contains("console.error(\"seseragi: runtime defect\", error)"));
        assert!(source.contains("await browserDom.dispose()"));
        assert!(!source.contains("apps/playground"));
    }
}
