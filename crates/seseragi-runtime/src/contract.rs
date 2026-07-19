use serde::Serialize;
use seseragi_driver::CompiledModule;
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
    Stdin,
    Dom,
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
        spelling: "Stdin",
        canonical: "std/prelude::Stdin",
        service: HostService::Stdin,
    },
    HostServiceSpec {
        spelling: "Dom",
        canonical: "std/web/dom::Dom",
        service: HostService::Dom,
    },
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FailureRenderer {
    Never,
    Show { module: String, export: String },
}

pub fn main_contract(compiled: &CompiledModule) -> Result<MainContract, String> {
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

    Ok(MainContract {
        environment: environment(
            &effect.environment,
            &compiled.typed_hir.external_type_bindings,
        )?,
        failure_renderer: failure_renderer(compiled, &effect.failure)?,
    })
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
    HOST_SERVICES
        .iter()
        .find(|spec| spelling == spec.spelling || canonical == Some(spec.canonical))
        .map(|spec| spec.service)
}

fn failure_renderer(
    compiled: &CompiledModule,
    failure: &TypedType,
) -> Result<FailureRenderer, String> {
    if is_named(failure, "Never") {
        return Ok(FailureRenderer::Never);
    }
    if let TypedType::Named { name, arguments } = failure {
        if arguments.is_empty() {
            let standard = match name.as_str() {
                "Int" => Some("intShow"),
                "String" => Some("stringShow"),
                "ConsoleError" => Some("consoleErrorShow"),
                "StdinError" => Some("stdinErrorShow"),
                _ => None,
            };
            if let Some(export) = standard {
                return Ok(FailureRenderer::Show {
                    module: "@seseragi/runtime/show".to_owned(),
                    export: export.to_owned(),
                });
            }
        }
    }

    let selected = compiled
        .typed_hir
        .instances
        .iter()
        .find(|instance| {
            instance.trait_name == "Show"
                && instance.arguments.as_slice() == std::slice::from_ref(failure)
        })
        .ok_or_else(|| "`main` failure type requires a selected Show instance".to_owned())?;
    let generated = compiled
        .generated
        .metadata
        .instances
        .iter()
        .find(|instance| {
            instance.trait_name == "Show" && instance.type_identity == selected.type_identity
        })
        .ok_or_else(|| "generated module is missing the selected Show dictionary".to_owned())?;
    Ok(FailureRenderer::Show {
        module: "./main.ts".to_owned(),
        export: generated.dictionary_export.clone(),
    })
}

fn is_named(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments } if name == expected && arguments.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{main_contract, FailureRenderer, HostService};
    use seseragi_driver::{compile_module, CompileInput};

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
            FailureRenderer::Show { ref module, ref export }
                if module == "./main.ts" && export == "__ssrg$instance$Show$0"
        ));
    }

    #[test]
    fn selects_the_standard_int_show_dictionary_for_a_failure() {
        let source = "pub effect fn main =\n  do {\n    fail 42\n    succeed ()\n  }\n";
        let compiled = compile_module(CompileInput::new("main.ssrg", "test/main", source)).unwrap();
        let contract = main_contract(&compiled).unwrap();

        assert!(matches!(
            contract.failure_renderer,
            FailureRenderer::Show { ref module, ref export }
                if module == "@seseragi/runtime/show" && export == "intShow"
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
            FailureRenderer::Show { ref module, ref export }
                if module == "@seseragi/runtime/show" && export == "stringShow"
        ));
    }
}
