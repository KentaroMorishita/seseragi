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
    declaration: String,
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
    let settings_source = foreign
        .and_then(|foreign| foreign.bindings.as_ref())
        .and_then(|bindings| fs::read_to_string(package_root.join(bindings.as_str())).ok());
    let active = settings_source
        .as_deref()
        .and_then(|source| toml::from_str::<ActiveBindings>(source).ok())
        .filter(|settings| settings.schema == 1);
    let entries = active.as_ref().map(|settings| &settings.entries);
    if let Some(entry) = raw.as_ref().and_then(|owner| {
        entries
            .and_then(|entries| entries.get(&owner.entry))
            .filter(|entry| entry.output == path)
    }) {
        return validate_projection_owner(
            package_root,
            generated_root,
            settings_source.as_deref().unwrap(),
            entry,
        );
    }
    if let Some(owner) = semantic {
        if let Some(entry) = entries.and_then(|entries| {
            entries.values().find(|entry| {
                Some(entry.output.as_str()) == semantic_output && entry.projection_policy.is_some()
            })
        }) {
            let source = fs::read_to_string(generated_root.join(format!("{path}.ssrg")))
                .map_err(|error| error.to_string())?;
            if source == owner.semantic_source {
                return validate_projection_owner(
                    package_root,
                    generated_root,
                    settings_source.as_deref().unwrap(),
                    entry,
                );
            }
            return Err(format!("generated semantic module `gen/{path}` has been edited; regenerate it from the active projection policy"));
        }
    }
    Err(format!("generated module `gen/{path}` belongs to a removed entry, renamed output, or removed semantic policy; update the import and regenerate bindings (retained files were preserved)"))
}

fn validate_projection_owner(
    package_root: &Path,
    generated_root: &Path,
    settings: &str,
    entry: &ActiveEntry,
) -> Result<(), String> {
    let Some(policy) = &entry.projection_policy else {
        return Ok(());
    };
    let metadata =
        fs::read_to_string(generated_root.join(format!("{}.binding.json", entry.output)))
            .map_err(|error| format!("missing semantic adapter binding metadata: {error}"))?;
    crate::generated_projection::validate_generated_projection(
        package_root, generated_root, settings, &entry.output, &entry.declaration, policy, &metadata,
    ).map_err(|error| format!("semantic policy `{policy}` is missing/invalid/stale: {error}; run `seseragi dts convert`"))
}
