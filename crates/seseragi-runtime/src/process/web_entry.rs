use crate::{DisplayDictionary, FailureRenderer, MainContract, ProcessRunOptions, RandomSeed};
use seseragi_driver::ProviderResolution;

pub(super) fn web_entry_source(
    contract: &MainContract,
    entry_module: &str,
    providers: Option<&ProviderResolution>,
    options: ProcessRunOptions,
) -> String {
    let environment = serde_json::to_string(&contract.environment)
        .expect("the validated browser environment contract must serialize");
    let mut imports = vec![
        "import { createBrowserDom } from \"@seseragi/runtime/browser/dom\";".to_owned(),
        "import { createBrowserEnvironment } from \"@seseragi/runtime/browser/host\";".to_owned(),
        "import { startBrowserProviders } from \"@seseragi/runtime/browser/providers\";".to_owned(),
        "import { createEffectExecution, isEffectCancellation, run } from \"@seseragi/runtime/effect\";".to_owned(),
        "import { processHashSeed } from \"@seseragi/runtime/hash\";".to_owned(),
    ];
    let mut application_imports = vec![format!(
        "const {{ main }} = await import(\"{entry_module}\");"
    )];
    let selections = crate::browser_provider_selections(providers);
    let provider_selections = serde_json::to_string(&selections)
        .expect("validated browser provider selections must serialize");
    let mut provider_modules = Vec::new();
    for (index, selection) in selections.iter().enumerate() {
        let local = format!("browserProviderEntry{index}");
        imports.push(format!(
            "import {{ {} as {local} }} from {:?};",
            selection.entry_export, selection.entry_module
        ));
        provider_modules.push(format!(
            "[{:?}, {{ {:?}: {local} }}]",
            selection.entry_module, selection.entry_export
        ));
    }
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
                &mut application_imports,
                &mut dictionary_index,
            );
            format!(
                "const failureShow = {expression};\n    console.error(failureRenderShow(failureShow, result.error, {{ layout: \"compact\" }}));"
            )
        }
    };
    format!(
        r#"{}

{}
processHashSeed();
{}
const execution = createEffectExecution();
const browserDom = createBrowserDom(document, () => {{
  document.documentElement.dataset.seseragiStatus = "mounted";
}});
const browserProviderModules = new Map([{}]);
const browserProviders = await startBrowserProviders({}, async (specifier) => {{
  const module = browserProviderModules.get(specifier);
  if (module === undefined) throw new Error(`unsupported browser provider module: ${{specifier}}`);
  return module;
}});
const environment = createBrowserEnvironment(
  {},
  "",
  (value) => console.log(value.endsWith("\n") ? value.slice(0, -1) : value),
  browserDom.service,
  execution.context,
  browserProviders.services,
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
  await execution.close();
  await browserDom.dispose();
  await browserProviders.shutdown();
}}
"#,
        imports.join("\n"),
        format!(
            "{}\n{}",
            match options.hash_seed {
                RandomSeed::Entropy => "delete globalThis.__SESERAGI_HASH_SEED__;".to_owned(),
                RandomSeed::Fixed(seed) => format!("globalThis.__SESERAGI_HASH_SEED__ = {seed}n;"),
            },
            match options.random_seed {
                RandomSeed::Entropy => "delete globalThis.__SESERAGI_RANDOM_SEED__;".to_owned(),
                RandomSeed::Fixed(seed) => format!(
                    "globalThis.__SESERAGI_RANDOM_SEED__ = {:?};",
                    seed.to_string()
                ),
            }
        ),
        application_imports.join("\n"),
        provider_modules.join(", "),
        provider_selections,
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
        "const {{ {}: {local} }} = await import(\"{module}\");",
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
    use crate::{
        EnvironmentBinding, FailureRenderer, HostService, MainContract, ProcessRunOptions,
        RandomSeed,
    };

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
            None,
            ProcessRunOptions::default(),
        );

        assert!(source.contains("createBrowserDom(document"));
        assert!(source.contains(r#"[{"field":"dom","service":"dom"}]"#));
        assert!(source.contains("await run(main(undefined), environment, execution.context)"));
        assert!(source.contains("console.error(\"seseragi: runtime defect\", error)"));
        assert!(source.contains("await execution.close()"));
        assert!(source.contains("await browserDom.dispose()"));
        assert!(!source.contains("apps/playground"));
    }

    #[test]
    fn stages_browser_hash_and_random_seed_overrides() {
        let source = web_entry_source(
            &MainContract {
                environment: Vec::new(),
                failure_renderer: FailureRenderer::Never,
            },
            "./main.ts",
            None,
            ProcessRunOptions {
                hash_seed: RandomSeed::Fixed(-11),
                random_seed: RandomSeed::Fixed(17),
                ..ProcessRunOptions::default()
            },
        );

        assert!(source.contains("globalThis.__SESERAGI_HASH_SEED__ = -11n"));
        assert!(source.contains("globalThis.__SESERAGI_RANDOM_SEED__ = \"17\""));
        assert!(
            source.find("processHashSeed();").unwrap()
                < source.find("await import(\"./main.ts\")").unwrap()
        );
    }
}
