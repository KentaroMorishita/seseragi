use seseragi_provider::ServiceRequirement;
use seseragi_semantics::{ExternalTypeBinding, TypedDecl, TypedModule, TypedType};
use seseragi_syntax::Visibility;
use std::collections::BTreeSet;

/// Extracts the backend-neutral service requirements from the linked public
/// `main` Effect. Provider discovery and selection consume this metadata later;
/// this step never chooses an implementation.
pub fn main_provider_requirements(
    typed: &TypedModule,
) -> Result<Vec<ServiceRequirement>, ProviderRequirementError> {
    let main = typed
        .declarations
        .iter()
        .find(|declaration| {
            matches!(
                declaration,
                TypedDecl::EffectFn {
                    symbol,
                    visibility: Visibility::Public,
                    ..
                } if symbol.ends_with("::main")
            )
        })
        .ok_or_else(|| ProviderRequirementError::new("program must export `pub effect fn main`"))?;
    let TypedDecl::EffectFn { effect, .. } = main else {
        unreachable!()
    };
    requirements_from_environment(typed, &effect.environment)
}

fn requirements_from_environment(
    module: &TypedModule,
    environment: &TypedType,
) -> Result<Vec<ServiceRequirement>, ProviderRequirementError> {
    let TypedType::Record {
        closed: true,
        fields,
    } = environment
    else {
        return Err(ProviderRequirementError::new(
            "`main` Effect environment must be a closed record",
        ));
    };
    let mut names = BTreeSet::new();
    let mut requirements = Vec::with_capacity(fields.len());
    for field in fields {
        if field.optional {
            return Err(ProviderRequirementError::new(format!(
                "`main` environment field `{}` cannot be optional",
                field.name
            )));
        }
        if !names.insert(&field.name) {
            return Err(ProviderRequirementError::new(format!(
                "`main` environment field `{}` is duplicated",
                field.name
            )));
        }
        requirements.push(ServiceRequirement {
            field: field.name.clone(),
            service: canonical_service_identity(module, &field.type_ref).map_err(|message| {
                ProviderRequirementError::new(format!(
                    "invalid `main` environment field `{}`: {message}",
                    field.name
                ))
            })?,
        });
    }
    requirements.sort_by(|left, right| {
        left.field
            .cmp(&right.field)
            .then_with(|| left.service.cmp(&right.service))
    });
    Ok(requirements)
}

fn canonical_service_identity(
    module: &TypedModule,
    type_ref: &TypedType,
) -> Result<String, String> {
    let (name, direct_identity, arguments) = match type_ref {
        TypedType::Named { name, arguments } => (name.as_str(), None, arguments),
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => (name.as_str(), Some(canonical.as_str()), arguments),
        _ => return Err("service type must be a nominal type".to_owned()),
    };
    if !arguments.is_empty() {
        return Err("service type cannot have type arguments".to_owned());
    }
    if let Some(identity) = direct_identity {
        return Ok(stable_external_service_identity(identity));
    }

    let external = unique_external_identity(name, &module.external_type_bindings)?;
    if let Some(identity) = external {
        return Ok(stable_external_service_identity(&identity));
    }
    if let Some(identity) = local_type_identity(name, module) {
        return Ok(identity);
    }
    match name {
        "Console" | "Stdin" => Ok(format!("std/prelude::{name}")),
        "Logger" => Ok("std/log::Logger".to_owned()),
        "Dom" => Ok("std/web/dom::Dom".to_owned()),
        _ => Err(format!(
            "service type `{name}` has no canonical nominal identity"
        )),
    }
}

/// Projects a versioned local-package nominal identity onto the stable
/// service identity used by Provider Contracts.
///
/// `acme/db@1.2.3::lib::Database` becomes `acme/db::Database`, while an
/// exported submodule such as `acme/db@1.2.3::admin::Database` becomes
/// `acme/db/admin::Database`. Standard and already-stable identities pass
/// through unchanged.
pub fn stable_external_service_identity(canonical: &str) -> String {
    let Some((package, tail)) = canonical.split_once("::") else {
        return canonical.to_owned();
    };
    let Some((name, version)) = package.rsplit_once('@') else {
        return canonical.to_owned();
    };
    if semver::Version::parse(version).is_err() {
        return canonical.to_owned();
    }
    let Some((module, nominal)) = tail.rsplit_once("::") else {
        return canonical.to_owned();
    };
    if module == "lib" {
        format!("{name}::{nominal}")
    } else {
        format!("{name}/{module}::{nominal}")
    }
}

fn unique_external_identity(
    spelling: &str,
    bindings: &[ExternalTypeBinding],
) -> Result<Option<String>, String> {
    let identities = bindings
        .iter()
        .filter(|binding| binding.spelling == spelling)
        .map(|binding| binding.canonical.as_str())
        .collect::<BTreeSet<_>>();
    match identities.len() {
        0 => Ok(None),
        1 => Ok(identities.first().map(|identity| (*identity).to_owned())),
        _ => Err(format!(
            "service type `{spelling}` has ambiguous canonical identities"
        )),
    }
}

fn local_type_identity(name: &str, module: &TypedModule) -> Option<String> {
    module
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::Alias {
                symbol,
                name: candidate,
                ..
            }
            | TypedDecl::Adt {
                symbol,
                name: candidate,
                ..
            }
            | TypedDecl::Struct {
                symbol,
                name: candidate,
                ..
            } if candidate == name => Some(symbol.clone()),
            _ => None,
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequirementError {
    message: String,
}

impl ProviderRequirementError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProviderRequirementError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProviderRequirementError {}

#[cfg(test)]
mod tests {
    use super::{main_provider_requirements, stable_external_service_identity};
    use crate::{compile_module, CompileInput};
    use seseragi_provider::ServiceRequirement;

    #[test]
    fn extracts_stable_prelude_and_local_service_identities() {
        let source = r#"
pub type Clock = | Clock

pub effect fn main -> Unit
with Console, clock: Clock =
  succeed ()
"#;
        let compiled = compile_module(CompileInput::new("main.ssrg", "app/main", source)).unwrap();
        assert_eq!(
            main_provider_requirements(&compiled.typed_hir).unwrap(),
            [
                ServiceRequirement {
                    field: "clock".to_owned(),
                    service: "app/main::Clock".to_owned(),
                },
                ServiceRequirement {
                    field: "console".to_owned(),
                    service: "std/prelude::Console".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn keeps_an_imported_service_canonical_across_a_linked_project() {
        use crate::{compile_project, ProjectModuleInput};
        use seseragi_project::ModuleGraph;

        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "app/main".to_owned(),
                [("service".to_owned(), "acme/clock".to_owned())],
            )
            .unwrap();
        graph.add_module("acme/clock".to_owned(), []).unwrap();
        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "clock.ssrg",
                    "acme/clock",
                    "pub type Clock = | Clock\n",
                    "dist/clock.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "app/main",
                    "import { Clock } from \"service\"\n\npub effect fn main -> Unit with clock: Clock = succeed ()\n",
                    "dist/main.js",
                ),
            ],
        )
        .unwrap();
        let main = project.modules.get("app/main").unwrap();
        assert_eq!(
            main_provider_requirements(&main.typed_hir).unwrap(),
            [ServiceRequirement {
                field: "clock".to_owned(),
                service: "acme/clock::Clock".to_owned(),
            }]
        );
    }

    #[test]
    fn removes_package_versions_from_external_service_identities() {
        assert_eq!(
            stable_external_service_identity("seseragi/postgres@0.1.0::lib::Postgres"),
            "seseragi/postgres::Postgres"
        );
        assert_eq!(
            stable_external_service_identity("seseragi/sqlite@0.1.0::lib::Sqlite"),
            "seseragi/sqlite::Sqlite"
        );
        assert_eq!(
            stable_external_service_identity("acme/database@2.3.4::admin::Database"),
            "acme/database/admin::Database"
        );
        assert_eq!(
            stable_external_service_identity("std/http::HttpClient"),
            "std/http::HttpClient"
        );
    }
}
