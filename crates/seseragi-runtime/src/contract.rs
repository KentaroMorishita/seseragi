use serde::Serialize;
use seseragi_driver::{CompiledModule, CompiledProject};
use seseragi_semantics::{ExternalTypeBinding, TypedDecl, TypedParameter, TypedType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MainContract {
    pub environment: Vec<EnvironmentBinding>,
    pub failure_renderer: FailureRenderer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentBinding {
    pub field: String,
    pub service: HostService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HostService {
    Console,
    Logger,
    Stdin,
    Process,
    Dom,
    Clock,
    FileSystem,
    Navigation,
    Storage,
    HttpClient,
    HttpServer,
    WebSocketClient,
    WebSocketServer,
    Postgres,
    Sqlite,
}

impl HostService {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Console => "console",
            Self::Logger => "logger",
            Self::Stdin => "stdin",
            Self::Process => "process",
            Self::Dom => "dom",
            Self::Clock => "clock",
            Self::FileSystem => "fileSystem",
            Self::Navigation => "navigation",
            Self::Storage => "storage",
            Self::HttpClient => "httpClient",
            Self::HttpServer => "httpServer",
            Self::WebSocketClient => "webSocketClient",
            Self::WebSocketServer => "webSocketServer",
            Self::Postgres => "postgres",
            Self::Sqlite => "sqlite",
        }
    }
}

struct HostServiceSpec {
    spelling: &'static str,
    canonical: &'static str,
    service: HostService,
}

const HOST_SERVICES: &[HostServiceSpec] = &[
    HostServiceSpec {
        spelling: "Console",
        canonical: "std/prelude::Console",
        service: HostService::Console,
    },
    HostServiceSpec {
        spelling: "Logger",
        canonical: "std/log::Logger",
        service: HostService::Logger,
    },
    HostServiceSpec {
        spelling: "Stdin",
        canonical: "std/prelude::Stdin",
        service: HostService::Stdin,
    },
    HostServiceSpec {
        spelling: "Process",
        canonical: "std/process::Process",
        service: HostService::Process,
    },
    HostServiceSpec {
        spelling: "Dom",
        canonical: "std/web/dom::Dom",
        service: HostService::Dom,
    },
    HostServiceSpec {
        spelling: "Clock",
        canonical: "std/clock::Clock",
        service: HostService::Clock,
    },
    HostServiceSpec {
        spelling: "FileSystem",
        canonical: "std/fs::FileSystem",
        service: HostService::FileSystem,
    },
    HostServiceSpec {
        spelling: "Navigation",
        canonical: "std/web/navigation::Navigation",
        service: HostService::Navigation,
    },
    HostServiceSpec {
        spelling: "Storage",
        canonical: "std/web/storage::Storage",
        service: HostService::Storage,
    },
    HostServiceSpec {
        spelling: "HttpClient",
        canonical: "std/http::HttpClient",
        service: HostService::HttpClient,
    },
    HostServiceSpec {
        spelling: "HttpServer",
        canonical: "std/http/server::HttpServer",
        service: HostService::HttpServer,
    },
    HostServiceSpec {
        spelling: "WebSocketClient",
        canonical: "std/websocket::WebSocketClient",
        service: HostService::WebSocketClient,
    },
    HostServiceSpec {
        spelling: "WebSocketServer",
        canonical: "std/websocket/server::WebSocketServer",
        service: HostService::WebSocketServer,
    },
    HostServiceSpec {
        spelling: "Postgres",
        canonical: "seseragi/postgres::Postgres",
        service: HostService::Postgres,
    },
    HostServiceSpec {
        spelling: "Sqlite",
        canonical: "seseragi/sqlite::Sqlite",
        service: HostService::Sqlite,
    },
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FailureRenderer {
    Never,
    Show {
        module: String,
        export: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<DisplayDictionary>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayDictionary {
    pub module: String,
    pub export: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<DisplayDictionary>,
}

pub fn main_contract(compiled: &CompiledModule) -> Result<MainContract, String> {
    let (environment_type, failure_type) = main_effect(compiled)?;
    Ok(MainContract {
        environment: environment(environment_type, &compiled.typed_hir.external_type_bindings)?,
        failure_renderer: failure_renderer(compiled, failure_type)?,
    })
}

pub fn project_main_contract(
    project: &CompiledProject,
    entry_module: &str,
) -> Result<MainContract, String> {
    let compiled = project
        .modules
        .get(entry_module)
        .ok_or_else(|| "compiled project omitted its entry module".to_owned())?;
    let (environment_type, failure_type) = main_effect(compiled)?;
    Ok(MainContract {
        environment: environment(environment_type, &compiled.typed_hir.external_type_bindings)?,
        failure_renderer: project_failure_renderer(project, compiled, failure_type)?,
    })
}

fn main_effect(compiled: &CompiledModule) -> Result<(&TypedType, &TypedType), String> {
    let main = compiled
        .typed_hir
        .declarations
        .iter()
        .find(|declaration| matches!(declaration, TypedDecl::EffectFn { symbol, .. } if symbol.ends_with("::main")))
        .ok_or_else(|| "program must export `pub effect fn main`".to_owned())?;
    let TypedDecl::EffectFn {
        parameters, effect, ..
    } = main
    else {
        unreachable!()
    };
    if !matches!(parameters.as_slice(), [TypedParameter::ImplicitUnit { .. }]) {
        return Err("`main` must take the implicit Unit argument".to_owned());
    }
    if !is_named(&effect.success, "Unit") {
        return Err("`main` must succeed with Unit".to_owned());
    }
    if !compiled
        .generated
        .metadata
        .exports
        .iter()
        .any(|name| name == "main")
    {
        return Err("`main` must be public".to_owned());
    }
    Ok((&effect.environment, &effect.failure))
}

fn environment(
    type_ref: &TypedType,
    external_types: &[ExternalTypeBinding],
) -> Result<Vec<EnvironmentBinding>, String> {
    let TypedType::Record {
        closed: true,
        fields,
    } = type_ref
    else {
        return Err("`main` Effect environment must be a closed record".to_owned());
    };
    fields
        .iter()
        .map(|field| {
            if field.optional {
                return Err(format!(
                    "`main` environment field `{}` cannot be optional",
                    field.name
                ));
            }
            let service = host_service(&field.type_ref, external_types).ok_or_else(|| {
                format!(
                    "no command-line host adapter for `main` environment field `{}` with type {:?}",
                    field.name, field.type_ref
                )
            })?;
            Ok(EnvironmentBinding {
                field: field.name.clone(),
                service,
            })
        })
        .collect()
}

fn host_service(
    type_ref: &TypedType,
    external_types: &[ExternalTypeBinding],
) -> Option<HostService> {
    let (spelling, canonical) = match type_ref {
        TypedType::Named { name, arguments } if arguments.is_empty() => (
            name.as_str(),
            external_types
                .iter()
                .find(|binding| binding.spelling == *name)
                .map(|binding| binding.canonical.as_str()),
        ),
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } if arguments.is_empty() => (name.as_str(), Some(canonical.as_str())),
        _ => return None,
    };
    let stable = canonical.map(seseragi_driver::stable_external_service_identity);
    HOST_SERVICES
        .iter()
        .find(|spec| {
            spelling == spec.spelling
                || canonical == Some(spec.canonical)
                || stable.as_deref() == Some(spec.canonical)
        })
        .map(|spec| spec.service)
}

fn failure_renderer(
    compiled: &CompiledModule,
    failure: &TypedType,
) -> Result<FailureRenderer, String> {
    if is_named(failure, "Never") {
        return Ok(FailureRenderer::Never);
    }
    if let Some(dictionary) = module_show_dictionary(compiled, failure) {
        return Ok(FailureRenderer::Show {
            module: dictionary.module,
            export: dictionary.export,
            arguments: dictionary.arguments,
        });
    }

    Err("`main` failure type requires a selected Show instance".to_owned())
}

fn module_show_dictionary(
    compiled: &CompiledModule,
    type_ref: &TypedType,
) -> Option<DisplayDictionary> {
    if let Some(dictionary) = standard_show_dictionary(type_ref) {
        return Some(dictionary);
    }
    if let TypedType::Named { name, arguments } = type_ref {
        let (export, expected) = match name.as_str() {
            "Either" => ("eitherShow", 2),
            "Maybe" => ("maybeShow", 1),
            "Array" => ("arrayShow", 1),
            "List" => ("listShow", 1),
            _ => ("", 0),
        };
        if !export.is_empty() && arguments.len() == expected {
            return Some(DisplayDictionary {
                module: "@seseragi/runtime/show".to_owned(),
                export: export.to_owned(),
                arguments: arguments
                    .iter()
                    .map(|argument| module_show_dictionary(compiled, argument))
                    .collect::<Option<Vec<_>>>()?,
            });
        }
    }
    let selected = compiled.typed_hir.instances.iter().find(|instance| {
        instance.trait_name == "Show"
            && instance.arguments.as_slice() == std::slice::from_ref(type_ref)
    })?;
    let generated = compiled
        .generated
        .metadata
        .instances
        .iter()
        .find(|instance| {
            instance.trait_name == "Show" && instance.type_identity == selected.type_identity
        })?;
    Some(DisplayDictionary {
        module: "./main.ts".to_owned(),
        export: generated.dictionary_export.clone(),
        arguments: Vec::new(),
    })
}

fn project_failure_renderer(
    project: &CompiledProject,
    entry: &CompiledModule,
    failure: &TypedType,
) -> Result<FailureRenderer, String> {
    if is_named(failure, "Never") {
        return Ok(FailureRenderer::Never);
    }
    if let Some(dictionary) = standard_show_dictionary(failure) {
        return Ok(FailureRenderer::Show {
            module: dictionary.module,
            export: dictionary.export,
            arguments: dictionary.arguments,
        });
    }
    if let Ok(renderer) = failure_renderer(entry, failure) {
        return Ok(renderer);
    }

    let canonical = match failure {
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if arguments.is_empty() => canonical.as_str(),
        TypedType::Named { name, arguments } if arguments.is_empty() => entry
            .typed_hir
            .external_type_bindings
            .iter()
            .find(|binding| binding.spelling == *name)
            .map(|binding| binding.canonical.as_str())
            .ok_or_else(|| "`main` failure type requires a selected Show instance".to_owned())?,
        TypedType::ExternalNamed { .. } | TypedType::Named { .. } => {
            return Err(
                "`main` generic external failure type requires explicit Show evidence".to_owned(),
            );
        }
        _ => {
            return Err("`main` failure type requires a selected Show instance".to_owned());
        }
    };
    let matches = project
        .modules
        .values()
        .filter_map(|module| {
            module
                .generated
                .metadata
                .instances
                .iter()
                .find(|instance| {
                    instance.trait_name == "Show"
                        && instance.type_identity.as_deref() == Some(canonical)
                        && instance.type_parameters.is_empty()
                })
                .map(|instance| (module, instance))
        })
        .collect::<Vec<_>>();
    let [(provider, generated)] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            "`main` failure type requires a selected Show instance".to_owned()
        } else {
            "`main` failure type has multiple generated Show dictionaries".to_owned()
        });
    };
    let output = &provider.generated.metadata.outputs.typescript;
    let module = if output.starts_with("./") {
        output.clone()
    } else {
        format!("./{output}")
    };
    Ok(FailureRenderer::Show {
        module,
        export: generated.dictionary_export.clone(),
        arguments: Vec::new(),
    })
}

fn standard_show_dictionary(type_ref: &TypedType) -> Option<DisplayDictionary> {
    const RUNTIME_SHOW: &str = "@seseragi/runtime/show";
    let dictionary = |export: &str| DisplayDictionary {
        module: RUNTIME_SHOW.to_owned(),
        export: export.to_owned(),
        arguments: Vec::new(),
    };
    match type_ref {
        TypedType::Named { name, arguments } if arguments.is_empty() => {
            let export = match name.as_str() {
                "Never" => "neverShow",
                "Int" => "intShow",
                "Float" => "floatShow",
                "Bool" => "boolShow",
                "Char" => "charShow",
                "String" => "stringShow",
                "Unit" => "unitShow",
                "ConsoleError" => "consoleErrorShow",
                "StdinError" => "stdinErrorShow",
                _ => return None,
            };
            Some(dictionary(export))
        }
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if arguments.is_empty() => match canonical.as_str() {
            "std/stdin::StdinConfigError" => Some(dictionary("stdinConfigErrorShow")),
            "std/log::LogError" => Some(dictionary("logErrorShow")),
            "std/web/dom::DomError" => Some(dictionary("domErrorShow")),
            "std/web/html::HtmlBuildError" => Some(dictionary("htmlBuildErrorShow")),
            "std/path::PathError" => Some(dictionary("pathErrorShow")),
            "std/fs::FileType" => Some(dictionary("fileTypeShow")),
            "std/fs::FileSystemOperation" => Some(dictionary("fileSystemOperationShow")),
            "std/fs::FileSystemErrorKind" => Some(dictionary("fileSystemErrorKindShow")),
            "std/fs::FileSystemError" => Some(dictionary("fileSystemErrorShow")),
            "std/fs::FileMetadata" => Some(dictionary("fileMetadataShow")),
            "std/fs::DirectoryEntry" => Some(dictionary("directoryEntryShow")),
            "std/fs::WriteMode" => Some(dictionary("writeModeShow")),
            "std/fs::FileTextError" => Some(dictionary("fileTextErrorShow")),
            "std/process::ProcessSignal" => Some(dictionary("processSignalShow")),
            "std/process::ProcessError" => Some(dictionary("processErrorShow")),
            _ => None,
        },
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if canonical == "std/web/dom::DomRuntimeError" => {
            let [failure] = arguments.as_slice() else {
                return None;
            };
            Some(DisplayDictionary {
                module: RUNTIME_SHOW.to_owned(),
                export: "domRuntimeErrorShow".to_owned(),
                arguments: vec![standard_show_dictionary(failure)?],
            })
        }
        _ => None,
    }
}

fn is_named(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments } if name == expected && arguments.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{
        main_contract, project_main_contract, standard_show_dictionary, DisplayDictionary,
        FailureRenderer, HostService,
    };
    use seseragi_driver::{compile_module, CompileInput, CompiledProject};
    use seseragi_semantics::TypedType;
    use std::collections::BTreeMap;

    const DOM_RUNTIME_NEVER_MAIN: &str = r#"import { DomRuntimeError } from "std/web/dom"

pub effect fn main -> Unit
fails DomRuntimeError<Never> =
  succeed ()
"#;

    #[test]
    fn derives_host_services_and_failure_dictionary_from_compiler_output() {
        let source = "pub type AppError deriving Show =\n  | Failed ConsoleError\n\npub effect fn main = mapError Failed (println \"hello\")\n";
        let compiled = compile_module(CompileInput::new("main.ssrg", "test/main", source)).unwrap();
        let contract = main_contract(&compiled).unwrap();

        assert_eq!(contract.environment.len(), 1);
        assert_eq!(contract.environment[0].field, "console");
        assert_eq!(contract.environment[0].service, HostService::Console);
        assert!(matches!(
            contract.failure_renderer,
            FailureRenderer::Show { ref module, ref export, ref arguments }
                if module == "./main.ts" && export == "__ssrg$instance$Show$0"
                    && arguments.is_empty()
        ));
    }

    #[test]
    fn selects_the_standard_int_show_dictionary_for_a_failure() {
        let source = "pub effect fn main =\n  do {\n    fail 42\n    succeed ()\n  }\n";
        let compiled = compile_module(CompileInput::new("main.ssrg", "test/main", source)).unwrap();
        let contract = main_contract(&compiled).unwrap();

        assert!(matches!(
            contract.failure_renderer,
            FailureRenderer::Show { ref module, ref export, ref arguments }
                if module == "@seseragi/runtime/show" && export == "intShow"
                    && arguments.is_empty()
        ));
    }

    #[test]
    fn exposes_the_browser_dom_service_in_the_shared_main_contract() {
        let source =
            include_str!("../../../examples/spec/artifacts/schema-1/web-dom-counter/main.ssrg");
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "artifact/web-dom-counter",
            source,
        ))
        .unwrap();
        let contract = main_contract(&compiled).unwrap();

        assert_eq!(contract.environment.len(), 1);
        assert_eq!(contract.environment[0].field, "dom");
        assert_eq!(contract.environment[0].service, HostService::Dom);
        assert!(matches!(
            contract.failure_renderer,
            FailureRenderer::Show { ref module, ref export, ref arguments }
                if module == "@seseragi/runtime/show" && export == "stringShow"
                    && arguments.is_empty()
        ));
    }

    #[test]
    fn composes_the_dom_runtime_error_show_dictionary_from_payload_evidence() {
        let dictionary = standard_show_dictionary(&TypedType::ExternalNamed {
            name: "DomRuntimeError".to_owned(),
            canonical: "std/web/dom::DomRuntimeError".to_owned(),
            arguments: vec![TypedType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }],
        });

        assert_eq!(
            dictionary,
            Some(DisplayDictionary {
                module: "@seseragi/runtime/show".to_owned(),
                export: "domRuntimeErrorShow".to_owned(),
                arguments: vec![DisplayDictionary {
                    module: "@seseragi/runtime/show".to_owned(),
                    export: "stringShow".to_owned(),
                    arguments: Vec::new(),
                }],
            })
        );
    }

    #[test]
    fn composes_the_dom_runtime_error_show_dictionary_from_never_evidence() {
        let dictionary = standard_show_dictionary(&TypedType::ExternalNamed {
            name: "DomRuntimeError".to_owned(),
            canonical: "std/web/dom::DomRuntimeError".to_owned(),
            arguments: vec![TypedType::Named {
                name: "Never".to_owned(),
                arguments: Vec::new(),
            }],
        });

        assert_eq!(
            dictionary,
            Some(DisplayDictionary {
                module: "@seseragi/runtime/show".to_owned(),
                export: "domRuntimeErrorShow".to_owned(),
                arguments: vec![DisplayDictionary {
                    module: "@seseragi/runtime/show".to_owned(),
                    export: "neverShow".to_owned(),
                    arguments: Vec::new(),
                }],
            })
        );
    }

    #[test]
    fn builds_single_and_project_main_contracts_for_dom_runtime_error_never() {
        let module_id = "artifact/dom-runtime-never-main";
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            module_id,
            DOM_RUNTIME_NEVER_MAIN,
        ))
        .expect("DomRuntimeError<Never> must be a valid explicit main failure");
        let expected = FailureRenderer::Show {
            module: "@seseragi/runtime/show".to_owned(),
            export: "domRuntimeErrorShow".to_owned(),
            arguments: vec![DisplayDictionary {
                module: "@seseragi/runtime/show".to_owned(),
                export: "neverShow".to_owned(),
                arguments: Vec::new(),
            }],
        };

        assert_eq!(main_contract(&compiled).unwrap().failure_renderer, expected);

        let project = CompiledProject {
            order: vec![module_id.to_owned()],
            modules: BTreeMap::from([(module_id.to_owned(), compiled)]),
            provider_resolution: None,
        };
        assert_eq!(
            project_main_contract(&project, module_id)
                .unwrap()
                .failure_renderer,
            expected
        );
    }

    #[test]
    fn leaves_top_level_never_without_a_show_dictionary() {
        let source = "pub effect fn main = succeed ()\n";
        let compiled = compile_module(CompileInput::new("main.ssrg", "test/main", source)).unwrap();

        assert_eq!(
            main_contract(&compiled).unwrap().failure_renderer,
            FailureRenderer::Never
        );
    }

    #[test]
    fn does_not_accept_an_unknown_generic_external_failure() {
        let dictionary = standard_show_dictionary(&TypedType::ExternalNamed {
            name: "OtherError".to_owned(),
            canonical: "vendor/package::OtherError".to_owned(),
            arguments: vec![TypedType::Named {
                name: "Never".to_owned(),
                arguments: Vec::new(),
            }],
        });

        assert_eq!(dictionary, None);
    }
}
