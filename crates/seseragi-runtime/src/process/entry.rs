use crate::{
    DisplayDictionary, FailureRenderer, HostService, MainContract, ProcessRunOptions,
    ProcessSignalMode, RandomSeed,
};
use seseragi_driver::ProviderResolution;

pub(super) fn entry_source(
    contract: &MainContract,
    entry_module: &str,
    providers: Option<&ProviderResolution>,
    options: ProcessRunOptions,
) -> String {
    let mut imports = vec![
        "import { createEffectExecution, isEffectCancellation, run } from \"@seseragi/runtime/effect\";".to_owned(),
        "import { installProcessShutdown } from \"@seseragi/runtime/process\";".to_owned(),
    ];
    let mut setup = Vec::new();
    setup.push(match options.random_seed {
        RandomSeed::Entropy => "delete process.env.SESERAGI_RANDOM_SEED;".to_owned(),
        RandomSeed::Fixed(seed) => {
            format!("process.env.SESERAGI_RANDOM_SEED = {:?};", seed.to_string())
        }
    });
    let mut fields = Vec::new();
    let mut cleanup = Vec::new();
    let mut imports_console = false;
    let mut imports_logger = false;
    let mut imports_stdin = false;
    let mut imports_process = false;
    let mut imports_provider_child_process = false;
    let mut imports_provider_runtime = false;
    let mut imports_provider_clock = false;
    let mut imports_provider_random = false;
    let mut imports_provider_entropy = false;
    let mut imports_provider_filesystem = false;
    let mut imports_provider_http_client = false;
    let mut imports_provider_http_server = false;
    let mut imports_provider_websocket_client = false;
    let mut imports_provider_websocket_server = false;
    let mut imports_provider_postgres = false;
    let mut imports_provider_sqlite = false;
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
            HostService::Logger => {
                if !imports_logger {
                    imports.push(
                        "import { liveLogger } from \"@seseragi/runtime/logger\";".to_owned(),
                    );
                    imports_logger = true;
                }
                fields.push(format!("{field}: liveLogger"));
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
            HostService::Process => {
                if !imports_process {
                    imports.push(
                        "import { liveProcess } from \"@seseragi/runtime/process\";".to_owned(),
                    );
                    imports_process = true;
                }
                fields.push(format!("{field}: liveProcess"));
            }
            HostService::ChildProcesses => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution.selected.iter().find(|selection| {
                            selection.service == "std/child-process::ChildProcesses"
                        })
                    })
                    .expect("child process entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_child_process {
                    imports.push(
                        "import { createProviderChildProcesses } from \"@seseragi/runtime/provider-child-process\";"
                            .to_owned(),
                    );
                    imports_provider_child_process = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("childProcessesProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderChildProcesses(await {loader}.load({:?}), process.env.SESERAGI_APPLICATION_ROOT);",
                    selection.provider,
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::Dom | HostService::Navigation | HostService::Storage => {
                unreachable!("process target compatibility was validated before entry generation")
            }
            HostService::Clock => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == "std/clock::Clock")
                    })
                    .expect("Clock entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_clock {
                    imports.push(
                        "import { createProviderClock } from \"@seseragi/runtime/provider-clock\";"
                            .to_owned(),
                    );
                    imports_provider_clock = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("clockProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderClock(await {loader}.load({:?}));",
                    selection.provider
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::Random | HostService::Entropy => {
                let (service, adapter_module, adapter_export, local_prefix) = match binding.service
                {
                    HostService::Random => (
                        "std/random::Random",
                        "@seseragi/runtime/provider-random",
                        "createProviderRandom",
                        "randomProvider",
                    ),
                    HostService::Entropy => (
                        "std/entropy::Entropy",
                        "@seseragi/runtime/provider-entropy",
                        "createProviderEntropy",
                        "entropyProvider",
                    ),
                    _ => unreachable!(),
                };
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == service)
                    })
                    .expect("random/entropy entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                let needs_import = match binding.service {
                    HostService::Random => &mut imports_provider_random,
                    HostService::Entropy => &mut imports_provider_entropy,
                    _ => unreachable!(),
                };
                if !*needs_import {
                    imports.push(format!(
                        "import {{ {adapter_export} }} from \"{adapter_module}\";"
                    ));
                    *needs_import = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("{local_prefix}{index}");
                setup.push(format!(
                    "const {local} = {adapter_export}(await {loader}.load({:?}));",
                    selection.provider
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::FileSystem => {
                let selection = providers.and_then(|resolution| {
                    resolution
                        .selected
                        .iter()
                        .find(|selection| selection.service == "std/fs::FileSystem")
                });
                let (provider, service, entry_module, entry_export) = selection.map_or(
                    (
                        "seseragi/runtime-bun#filesystem",
                        "std/fs::FileSystem",
                        "seseragi/runtime-bun/filesystem",
                        "provider",
                    ),
                    |selection| {
                        (
                            selection.provider.as_str(),
                            selection.service.as_str(),
                            selection.entry_module.as_str(),
                            selection.entry_export.as_str(),
                        )
                    },
                );
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_filesystem {
                    imports.push(
                        "import { createProviderFileSystem } from \"@seseragi/runtime/provider-filesystem\";"
                            .to_owned(),
                    );
                    imports_provider_filesystem = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    provider,
                    service,
                    entry_module,
                    entry_export,
                    entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("fileSystemProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderFileSystem(await {loader}.load({:?}));",
                    provider
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::HttpClient => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == "std/http::HttpClient")
                    })
                    .expect("HTTP client entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_http_client {
                    imports.push(
                        "import { createProviderHttpClient } from \"@seseragi/runtime/provider-http-client\";"
                            .to_owned(),
                    );
                    imports_provider_http_client = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("httpClientProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderHttpClient(await {loader}.load({:?}));",
                    selection.provider
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::HttpServer => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == "std/http/server::HttpServer")
                    })
                    .expect("HTTP server entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_http_server {
                    imports.push(
                        "import { createProviderHttpServer } from \"@seseragi/runtime/provider-http-server\";"
                            .to_owned(),
                    );
                    imports_provider_http_server = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("httpServerProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderHttpServer(await {loader}.load({:?}));",
                    selection.provider,
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::WebSocketClient => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == "std/websocket::WebSocketClient")
                    })
                    .expect("WebSocket client entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_websocket_client {
                    imports.push(
                        "import { createProviderWebSocketClient } from \"@seseragi/runtime/provider-websocket\";"
                            .to_owned(),
                    );
                    imports_provider_websocket_client = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("webSocketClientProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderWebSocketClient(await {loader}.load({:?}));",
                    selection.provider,
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::WebSocketServer => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution.selected.iter().find(|selection| {
                            selection.service == "std/websocket/server::WebSocketServer"
                        })
                    })
                    .expect("WebSocket server entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_websocket_server {
                    imports.push(
                        "import { createProviderWebSocketServer } from \"@seseragi/runtime/provider-websocket-server\";"
                            .to_owned(),
                    );
                    imports_provider_websocket_server = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("webSocketServerProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderWebSocketServer(await {loader}.load({:?}));",
                    selection.provider,
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::Postgres => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == "seseragi/postgres::Postgres")
                    })
                    .expect("PostgreSQL entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_postgres {
                    imports.push(
                        "import { createProviderPostgres } from \"@seseragi/runtime/provider-postgres\";"
                            .to_owned(),
                    );
                    imports_provider_postgres = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("postgresProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderPostgres(await {loader}.load({:?}));",
                    selection.provider,
                ));
                fields.push(format!("{field}: {local}"));
            }
            HostService::Sqlite => {
                let selection = providers
                    .and_then(|resolution| {
                        resolution
                            .selected
                            .iter()
                            .find(|selection| selection.service == "seseragi/sqlite::Sqlite")
                    })
                    .expect("SQLite entry requires a resolved provider");
                if !imports_provider_runtime {
                    imports.push(
                        "import { ProviderPackageLoader } from \"@seseragi/runtime/provider-package\";"
                            .to_owned(),
                    );
                    imports_provider_runtime = true;
                }
                if !imports_provider_sqlite {
                    imports.push(
                        "import { createProviderSqlite } from \"@seseragi/runtime/provider-sqlite\";"
                            .to_owned(),
                    );
                    imports_provider_sqlite = true;
                }
                let loader = format!("providerLoader{index}");
                setup.push(format!(
                    "const {loader} = new ProviderPackageLoader(\"bun-process\", [{{ provider: {:?}, service: {:?}, target: \"bun-process\", module: {:?}, exportName: {:?}, loadMode: \"eager\", importModule: () => import({:?}) }}]);",
                    selection.provider,
                    selection.service,
                    selection.entry_module,
                    selection.entry_export,
                    selection.entry_module,
                ));
                setup.push(format!("await {loader}.start();"));
                cleanup.push(format!("await {loader}.shutdown();"));
                let local = format!("sqliteProvider{index}");
                setup.push(format!(
                    "const {local} = createProviderSqlite(await {loader}.load({:?}));",
                    selection.provider,
                ));
                fields.push(format!("{field}: {local}"));
            }
        }
    }
    let failure = match &contract.failure_renderer {
        FailureRenderer::Never => {
            imports.push(format!("import {{ main }} from \"{entry_module}\";"));
            "process.stderr.write(\"seseragi: unreachable typed failure\\n\");\n  process.exitCode = 1;".to_owned()
        }
        FailureRenderer::Show {
            module,
            export,
            arguments,
        } => {
            imports.push(format!("import {{ main }} from \"{entry_module}\";"));
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
            setup.push(format!("const failureShow = {expression};"));
            "const message = failureRenderShow(failureShow, result.error, { layout: \"compact\" });\n  if (typeof message !== \"string\") throw new TypeError(\"Show dictionary returned a non-string value\");\n  process.stderr.write(message.endsWith(\"\\n\") ? message : message + \"\\n\");\n  process.exitCode = 1;".to_owned()
        }
    };
    let cleanup_source = cleanup
        .iter()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let signal_mode = match options.signal_mode {
        ProcessSignalMode::Cancel => "cancel",
        ProcessSignalMode::Forward => "forward",
    };
    format!(
        "{}\n{}\nconst rootExecution = createEffectExecution();\nconst processShutdown = installProcessShutdown(rootExecution, {{ mode: {:?}, graceMs: {} }});\nconst environment = {{ {} }};\nlet result;\nlet hasRuntimeDefect = false;\nlet wasCancelled = false;\ntry {{\n  result = await run(main(undefined), environment, rootExecution.context);\n}} catch (runDefect) {{\n  if (isEffectCancellation(runDefect)) wasCancelled = true;\n  else hasRuntimeDefect = true;\n}}\ntry {{\n  await rootExecution.close();\n  await processShutdown.close();\n{}\n}} catch (_cleanupDefect) {{\n  hasRuntimeDefect = true;\n}}\nif (hasRuntimeDefect) {{\n  process.stderr.write(\"seseragi: runtime defect\\n\");\n  process.exitCode = 70;\n}} else if (wasCancelled) {{\n  process.exitCode = processShutdown.exitCode() ?? 130;\n}} else if (result?.kind === \"failure\") {{\n  {}\n}}\n",
        imports.join("\n"),
        setup.join("\n"),
        signal_mode,
        options.shutdown_grace_ms,
        fields.join(", "),
        cleanup_source,
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
    use super::entry_source;
    use crate::{
        DisplayDictionary, EnvironmentBinding, FailureRenderer, HostService, MainContract,
        ProcessRunOptions, ProcessSignalMode, RandomSeed,
    };
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
                    arguments: Vec::new(),
                },
            },
            "./main.ts",
            None,
            ProcessRunOptions::default(),
        );

        assert!(source.contains("liveConsole"));
        assert!(source.contains("createProcessStdin"));
        assert!(source.contains("await run(main(undefined), environment, rootExecution.context)"));
        assert!(source
            .contains("failureRenderShow(failureShow, result.error, { layout: \"compact\" })"));
        assert!(source.contains("stdinAdapter1.close()"));
        assert!(source.contains("catch (_cleanupDefect)"));
        assert!(source.contains("seseragi: runtime defect\\n"));
        assert!(source.contains("process.exitCode = 70"));
        assert!(!source.contains("target mismatch"));
        assert!(
            source.find("stdinAdapter1.close()").unwrap()
                < source
                    .find("failureRenderShow(failureShow, result.error")
                    .unwrap()
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
            None,
            ProcessRunOptions::default(),
        );

        assert_eq!(source.matches("import { liveConsole }").count(), 1);
        assert!(source.contains("\"first\": liveConsole"));
        assert!(source.contains("\"second\": liveConsole"));
    }

    #[test]
    fn installs_the_process_service_and_requested_shutdown_policy() {
        let source = entry_source(
            &MainContract {
                environment: vec![EnvironmentBinding {
                    field: "process".to_owned(),
                    service: HostService::Process,
                }],
                failure_renderer: FailureRenderer::Never,
            },
            "./main.ts",
            None,
            ProcessRunOptions {
                signal_mode: ProcessSignalMode::Forward,
                shutdown_grace_ms: 250,
                random_seed: RandomSeed::Entropy,
            },
        );

        assert!(source.contains("import { liveProcess } from \"@seseragi/runtime/process\";"));
        assert!(source.contains("\"process\": liveProcess"));
        assert!(source.contains(
            "installProcessShutdown(rootExecution, { mode: \"forward\", graceMs: 250 })"
        ));
        assert!(source.contains("process.exitCode = processShutdown.exitCode() ?? 130"));
    }

    #[test]
    fn instantiates_a_generic_failure_dictionary_from_nested_evidence() {
        let source = entry_source(
            &MainContract {
                environment: Vec::new(),
                failure_renderer: FailureRenderer::Show {
                    module: "@seseragi/runtime/show".to_owned(),
                    export: "domRuntimeErrorShow".to_owned(),
                    arguments: vec![DisplayDictionary {
                        module: "@seseragi/runtime/show".to_owned(),
                        export: "stringShow".to_owned(),
                        arguments: Vec::new(),
                    }],
                },
            },
            "./main.ts",
            None,
            ProcessRunOptions::default(),
        );

        assert!(source.contains("const failureShow = failureDisplay0(failureDisplay1);"));
        assert!(source
            .contains("failureRenderShow(failureShow, result.error, { layout: \"compact\" })"));
    }
}
