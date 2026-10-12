//! Retained generated files remain on disk, but removed entries stop authorizing imports.
use crate::{ManifestForeignTypescript, ModulePath};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Deserialize)]
struct ActiveBindings {
    schema: u32,
    entries: BTreeMap<String, ActiveEntry>,
}
#[derive(Deserialize)]
struct ActiveEntry {
    output: String,
    projection_policy: Option<String>,
}
#[derive(Deserialize)]
struct RawOwner {
    schema: u32,
    generator: seseragi_interop::ProducerIdentity,
    entry: String,
    output: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SemanticOwner {
    schema: u32,
    input_boundary: String,
    graph_digest: String,
    semantic_source: String,
    #[serde(rename = "appliedPolicies")]
    _applied_policies: Vec<seseragi_interop::AppliedProjectionPolicy>,
}

pub(crate) fn validate_generated_import_owner(
    package_root: &Path,
    generated_root: &Path,
    foreign: Option<&ManifestForeignTypescript>,
    module: &ModulePath,
) -> Result<(), String> {
    let path = module.as_str();
    let raw = fs::read_to_string(generated_root.join(format!("{path}.binding.json")))
        .ok()
        .and_then(|source| serde_json::from_str::<RawOwner>(&source).ok())
        .filter(|owner| {
            owner.schema == 1 && owner.generator.name == "seseragi-dts" && owner.output == path
        });
    let semantic_output = path.strip_suffix("/semantic");
    let semantic = semantic_output.and_then(|output| {
        fs::read_to_string(generated_root.join(format!("{output}.semantic.json")))
            .ok()
            .and_then(|source| serde_json::from_str::<SemanticOwner>(&source).ok())
            .filter(|owner| {
                owner.schema == 1
                    && owner.input_boundary == "std/json::Json"
                    && owner.graph_digest.len() == 64
                    && owner
                        .graph_digest
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit())
            })
    });
    if raw.is_none() && semantic.is_none() {
        // Handwritten modules without converter ownership metadata remain legal.
        return Ok(());
    }
    let active = foreign
        .and_then(|foreign| foreign.bindings.as_ref())
        .and_then(|bindings| fs::read_to_string(package_root.join(bindings.as_str())).ok())
        .and_then(|source| toml::from_str::<ActiveBindings>(&source).ok())
        .filter(|settings| settings.schema == 1);
    let entries = active.as_ref().map(|settings| &settings.entries);
    if raw.as_ref().is_some_and(|owner| {
        entries
            .and_then(|entries| entries.get(&owner.entry))
            .is_some_and(|entry| entry.output == path)
    }) {
        return Ok(());
    }
    if let Some(owner) = semantic {
        if entries.is_some_and(|entries| {
            entries.values().any(|entry| {
                Some(entry.output.as_str()) == semantic_output && entry.projection_policy.is_some()
            })
        }) {
            let source = fs::read_to_string(generated_root.join(format!("{path}.ssrg")))
                .map_err(|error| error.to_string())?;
            if source == owner.semantic_source {
                return Ok(());
            }
            return Err(format!("generated semantic module `gen/{path}` has been edited; regenerate it from the active projection policy"));
        }
    }
    Err(format!("generated module `gen/{path}` belongs to a removed entry, renamed output, or removed semantic policy; update the import and regenerate bindings (retained files were preserved)"))
}
