use super::*;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvertRequest {
    pub package_root: PathBuf,
    pub generated_root: PathBuf,
    pub bindings: PathBuf,
    pub host_manifest: PathBuf,
    pub entry: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionOutcome {
    pub converted: Vec<ConvertedEntry>,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub compatibility: BTreeMap<String, SemanticCompatibilityReport>,
}

impl ConversionOutcome {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvertedEntry {
    pub id: String,
    pub source: PathBuf,
    pub metadata: PathBuf,
    pub report: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError {
    pub entry: String,
    pub message: String,
}

pub fn convert_package(request: &ConvertRequest) -> Result<ConversionOutcome, ConvertError> {
    let bindings_path = request.package_root.join(&request.bindings);
    let settings_source = read_utf8(&bindings_path, "binding settings")?;
    let config = parse_config(&settings_source)?;
    validate_config(&config)?;
    if let Some(entry) = &request.entry {
        if !config.entries.contains_key(entry) {
            return Err(ConvertError::new(format!(
                "binding entry `{entry}` does not exist in {}",
                request.bindings.display()
            )));
        }
    }

    let settings_digest = sha256(&settings_source);
    let host_manifest_path = request.package_root.join(&request.host_manifest);
    let host_manifest_source = read_utf8(&host_manifest_path, "foreign host manifest")?;
    let mut outcome = ConversionOutcome {
        converted: Vec::new(),
        diagnostics: Vec::new(),
        compatibility: BTreeMap::new(),
    };
    for (id, entry) in &config.entries {
        if request
            .entry
            .as_deref()
            .is_some_and(|selected| selected != id)
        {
            continue;
        }
        let declaration_path = request.package_root.join(&entry.declaration);
        let declaration = read_utf8(&declaration_path, "TypeScript declaration")?;
        let previous = fs::read_to_string(
            request
                .generated_root
                .join(format!("{}.binding.json", entry.output)),
        )
        .ok();
        let mut converted = convert_entry_with_host(
            id,
            entry,
            &declaration,
            &settings_digest,
            previous.as_deref(),
            || {
                resolve_host_module_identity(
                    &request.package_root,
                    &request.host_manifest,
                    &host_manifest_source,
                    &entry.specifier,
                )
            },
        )?;
        let artifacts = runtime_artifacts(request, entry);
        crate::attach_runtime_linkage(&mut converted, entry, &artifacts)?;
        if let Some(path) = &entry.projection_policy {
            let source = read_utf8(&request.package_root.join(path), "projection policy")?;
            crate::apply_projection_policy(&mut converted, entry, &source)
                .map_err(ConvertError::new)?;
        }
        let semantic_path = request
            .generated_root
            .join(format!("{}/semantic.ssrg", entry.output));
        let semantic_metadata_path = request
            .generated_root
            .join(format!("{}.semantic.json", entry.output));
        let ownership = semantic_output_ownership(&semantic_path, &semantic_metadata_path);
        if converted.generated.is_some()
            && (matches!(ownership, SemanticOutputOwnership::ModifiedOwned)
                || (entry.projection_policy.is_some()
                    && matches!(ownership, SemanticOutputOwnership::Unowned)))
        {
            crate::projection_policy::reject_projection_output(&mut converted,semantic_path.strip_prefix(&request.package_root).unwrap_or(&semantic_path).to_string_lossy().into_owned(),"semantic output is edited or unowned; preserve it and choose another output or explicitly restore the matching generated source before conversion".into()).map_err(ConvertError::new)?;
        }
        outcome.diagnostics.extend(converted.diagnostics);
        if let Some(report) = converted.compatibility {
            outcome.compatibility.insert(id.clone(), report);
        }
        let Some(generated) = converted.generated else {
            continue;
        };
        let source_path = request
            .generated_root
            .join(format!("{}.ssrg", entry.output));
        let metadata_path = request
            .generated_root
            .join(format!("{}.binding.json", entry.output));
        let report_path = request
            .generated_root
            .join(format!("{}.report.json", entry.output));
        let semantic_metadata = generated.adapters.as_ref().map(json_text).transpose()?;
        let mut writes: Vec<(&Path, &[u8])> = vec![
            (&source_path, generated.source.as_bytes()),
            (&metadata_path, generated.metadata.as_bytes()),
            (&report_path, generated.report.as_bytes()),
        ];
        if let (Some(adapters), Some(metadata)) = (&generated.adapters, &semantic_metadata) {
            writes.push((&semantic_path, adapters.semantic_source.as_bytes()));
            writes.push((&semantic_metadata_path, metadata.as_bytes()));
        }
        atomic_write_set(&writes)?;
        if generated.adapters.is_none()
            && matches!(ownership, SemanticOutputOwnership::Owned)
            && !config
                .entries
                .values()
                .any(|other| other.output == format!("{}/semantic", entry.output))
        {
            for path in [&semantic_path, &semantic_metadata_path] {
                if let Err(error) = fs::remove_file(path) {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        return Err(ConvertError::new(format!(
                            "failed to remove stale owned semantic artifact {}: {error}",
                            path.display()
                        )));
                    }
                }
            }
        }
        outcome.converted.push(ConvertedEntry {
            id: id.clone(),
            source: source_path,
            metadata: metadata_path,
            report: report_path,
        });
    }
    Ok(outcome)
}

#[derive(Clone, Copy)]
enum SemanticOutputOwnership {
    Absent,
    Owned,
    ModifiedOwned,
    Unowned,
}
fn semantic_output_ownership(source: &Path, metadata: &Path) -> SemanticOutputOwnership {
    // Symlink artifacts/parents are not owned just because their contents look
    // like metadata. Never follow them for destructive cleanup.
    let root = metadata.parent();
    let linked_parent = source.parent().is_some_and(|parent| {
        parent
            .ancestors()
            .take_while(|path| root.is_none_or(|root| path.starts_with(root)))
            .any(|path| fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()))
    });
    if linked_parent
        || [source, metadata]
            .iter()
            .any(|path| fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()))
    {
        return SemanticOutputOwnership::Unowned;
    }
    let actual = fs::read_to_string(source);
    let meta = fs::read_to_string(metadata);
    if actual
        .as_ref()
        .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        && meta
            .as_ref()
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    {
        return SemanticOutputOwnership::Absent;
    }
    let Ok(previous) = meta.and_then(|text| {
        serde_json::from_str::<seseragi_projection::SemanticAdapters>(&text)
            .map_err(std::io::Error::other)
    }) else {
        return SemanticOutputOwnership::Unowned;
    };
    if previous.schema != 1
        || previous.input_boundary != "std/json::Json"
        || previous.graph_digest.len() != 64
        || !previous
            .graph_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return SemanticOutputOwnership::Unowned;
    }
    match actual {
        Ok(actual) if actual == previous.semantic_source => SemanticOutputOwnership::Owned,
        Ok(_) => SemanticOutputOwnership::ModifiedOwned,
        Err(_) => SemanticOutputOwnership::Unowned,
    }
}

pub fn validate_generated_bindings(
    package_root: &Path,
    generated_root: &Path,
    bindings: &Path,
    host_manifest: &Path,
) -> Result<(), Vec<ValidationError>> {
    let settings_path = package_root.join(bindings);
    let settings_source = match fs::read_to_string(&settings_path) {
        Ok(source) => source,
        Err(error) => {
            return Err(vec![ValidationError {
                entry: "configuration".to_owned(),
                message: format!("failed to read {}: {error}", settings_path.display()),
            }]);
        }
    };
    let config = match parse_config(&settings_source).and_then(|config| {
        validate_config(&config)?;
        Ok(config)
    }) {
        Ok(config) => config,
        Err(error) => {
            return Err(vec![ValidationError {
                entry: "configuration".to_owned(),
                message: error.to_string(),
            }]);
        }
    };
    let settings_digest = sha256(&settings_source);
    let host_manifest_path = package_root.join(host_manifest);
    let host_manifest_source = match fs::read_to_string(&host_manifest_path) {
        Ok(source) => source,
        Err(error) => {
            return Err(vec![ValidationError {
                entry: "configuration".to_owned(),
                message: format!("failed to read {}: {error}", host_manifest_path.display()),
            }]);
        }
    };
    let mut errors = Vec::new();
    for (id, entry) in config.entries {
        let metadata_path = generated_root.join(format!("{}.binding.json", entry.output));
        let source_path = generated_root.join(format!("{}.ssrg", entry.output));
        let report_path = generated_root.join(format!("{}.report.json", entry.output));
        let Some(metadata) = read_previous_metadata(&metadata_path) else {
            errors.push(ValidationError {
                entry: id,
                message: format!(
                    "generated binding metadata is missing or invalid at {}; run `seseragi dts convert`",
                    metadata_path.display()
                ),
            });
            continue;
        };
        let declaration_path = package_root.join(&entry.declaration);
        let input_digest = fs::read_to_string(&declaration_path)
            .map(|source| sha256(&source))
            .unwrap_or_default();
        let host_module = resolve_host_module_identity(
            package_root,
            host_manifest,
            &host_manifest_source,
            &entry.specifier,
        )
        .ok();
        if metadata.schema != METADATA_SCHEMA
            || metadata.entry != id
            || metadata.output != entry.output
            || metadata.specifier != entry.specifier
            || host_module.as_ref() != Some(&metadata.host_module)
            || metadata.evaluation != evaluation_name(entry.evaluation)
            || metadata.settings_digest != settings_digest
            || metadata.input_digest != input_digest
            || !source_path.is_file()
            || !report_path.is_file()
        {
            errors.push(ValidationError {
                entry: id.clone(),
                message: format!(
                    "generated binding `{}` is stale; run `seseragi dts convert`",
                    entry.output
                ),
            });
        }

        if let Ok(source) = fs::read_to_string(&metadata_path) {
            let foreign = fs::read_to_string(package_root.join("seseragi.toml"))
                .ok()
                .and_then(|source| seseragi_project::parse_manifest(&source).ok())
                .and_then(|manifest| manifest.foreign_typescript);
            for target in [
                seseragi_interop::RuntimeTarget::Node,
                seseragi_interop::RuntimeTarget::Bun,
            ] {
                if let Err(problems) = seseragi_project::validate_binding_runtime_linkage(
                    &source,
                    target,
                    true,
                    |specifier| {
                        seseragi_project::resolve_foreign_runtime_artifact_for_target(
                            package_root,
                            foreign.as_ref(),
                            &source_path,
                            specifier,
                            target,
                        )
                    },
                ) {
                    errors.extend(problems.into_iter().map(|problem| ValidationError {
                        entry: id.clone(),
                        message: format!("{problem}; run `seseragi dts convert`"),
                    }));
                }
            }
        }

        if let Some(policy) = &entry.projection_policy {
            let checked = (|| -> Result<(), String> {
                let source = fs::read_to_string(&declaration_path).map_err(|e| e.to_string())?;
                let mut converted =
                    convert_entry_with_host(&id, &entry, &source, &settings_digest, None, || {
                        host_module.clone().ok_or_else(|| {
                            ConvertError::new("foreign host identity is unavailable")
                        })
                    })
                    .map_err(|error| error.to_string())?;
                let request = ConvertRequest {
                    package_root: package_root.to_owned(),
                    generated_root: generated_root.to_owned(),
                    bindings: bindings.to_owned(),
                    host_manifest: host_manifest.to_owned(),
                    entry: Some(id.clone()),
                };
                crate::attach_runtime_linkage(
                    &mut converted,
                    &entry,
                    &runtime_artifacts(&request, &entry),
                )
                .map_err(|error| error.to_string())?;
                let graph = converted
                    .semantic
                    .ok_or("current declaration has no semantic graph")?;
                let policies: seseragi_interop::ProjectionPolicySet = serde_json::from_str(
                    &fs::read_to_string(package_root.join(policy)).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let expected = seseragi_projection::generate_adapters(&graph, &policies)?;
                let actual_source = fs::read_to_string(
                    generated_root.join(format!("{}/semantic.ssrg", entry.output)),
                )
                .map_err(|e| e.to_string())?;
                let actual_metadata: seseragi_projection::SemanticAdapters = serde_json::from_str(
                    &fs::read_to_string(
                        generated_root.join(format!("{}.semantic.json", entry.output)),
                    )
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if expected != actual_metadata || expected.semantic_source != actual_source {
                    return Err("semantic adapter output differs from current policy".into());
                }
                Ok(())
            })();
            if let Err(message) = checked {
                errors.push(ValidationError { entry: id.clone(),message:format!("semantic policy `{policy}` is missing/invalid/stale: {message}; run `seseragi dts convert`") });
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn resolve_host_module_identity(
    package_root: &Path,
    host_manifest: &Path,
    host_manifest_source: &str,
    specifier: &str,
) -> Result<HostModuleIdentity, ConvertError> {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return Ok(HostModuleIdentity {
            specifier: specifier.to_owned(),
            exact_identity: format!("workspace:{specifier}"),
        });
    }
    let package_name = if specifier.starts_with('@') {
        specifier.split('/').take(2).collect::<Vec<_>>().join("/")
    } else {
        specifier.split('/').next().unwrap_or(specifier).to_owned()
    };
    let subpath = specifier
        .strip_prefix(&package_name)
        .unwrap_or_default()
        .trim_start_matches('/');
    let root_manifest: serde_json::Value =
        serde_json::from_str(host_manifest_source).map_err(|error| {
            ConvertError::new(format!(
                "invalid foreign host manifest {}: {error}",
                package_root.join(host_manifest).display()
            ))
        })?;
    let manifest = if root_manifest.get("name").and_then(|value| value.as_str())
        == Some(package_name.as_str())
    {
        root_manifest
    } else {
        let package_manifest = package_root
            .join(host_manifest)
            .parent()
            .expect("manifest has a package-relative parent")
            .join("node_modules")
            .join(&package_name)
            .join("package.json");
        let source = read_utf8(&package_manifest, "resolved foreign package manifest")?;
        serde_json::from_str(&source).map_err(|error| {
            ConvertError::new(format!(
                "invalid resolved foreign package manifest {}: {error}",
                package_manifest.display()
            ))
        })?
    };
    host_identity(specifier, &package_name, subpath, &manifest)
}

fn read_utf8(path: &Path, label: &str) -> Result<String, ConvertError> {
    fs::read_to_string(path).map_err(|error| {
        ConvertError::new(format!(
            "failed to read {label} {}: {error}",
            path.display()
        ))
    })
}

fn atomic_write_set(entries: &[(&Path, &[u8])]) -> Result<(), ConvertError> {
    let nonce = format!("{}.tmp", std::process::id());
    let mut temporary = Vec::new();
    for (path, bytes) in entries {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ConvertError::new(format!("failed to create {}: {error}", parent.display()))
            })?;
        }
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!("{value}.{nonce}"))
            .unwrap_or_else(|| nonce.clone());
        let temp = path.with_extension(extension);
        fs::write(&temp, bytes).map_err(|error| {
            ConvertError::new(format!("failed to write {}: {error}", temp.display()))
        })?;
        temporary.push((temp, path.to_path_buf()));
    }
    for (temp, path) in &temporary {
        if cfg!(windows) && path.exists() {
            fs::remove_file(path).map_err(|error| {
                ConvertError::new(format!("failed to replace {}: {error}", path.display()))
            })?;
        }
        fs::rename(temp, path).map_err(|error| {
            ConvertError::new(format!("failed to replace {}: {error}", path.display()))
        })?;
    }
    Ok(())
}

fn read_previous_metadata(path: &Path) -> Option<BindingMetadata> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fixture_root(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/spec/fixtures/projects")
            .join(name)
    }

    fn convert_fixture(name: &str) -> (TempDir, ConversionOutcome) {
        let temporary = TempDir::new().unwrap();
        let request = ConvertRequest {
            package_root: fixture_root(name),
            generated_root: temporary.path().join("generated"),
            bindings: PathBuf::from("seseragi.bindings.toml"),
            host_manifest: PathBuf::from("host/package.json"),
            entry: None,
        };
        let outcome = convert_package(&request).unwrap();
        (temporary, outcome)
    }

    #[test]
    fn matches_existing_conversion_snapshots() {
        for (fixture, output) in [
            ("dts-basic-conversion", "fixture-api"),
            ("dts-callback-during-call", "callback-api"),
            ("dts-declaration-merge", "merge-api"),
            ("dts-generated-name", "naming-api"),
            ("dts-namespace-runtime", "analytics"),
        ] {
            let (temporary, outcome) = convert_fixture(fixture);
            assert!(!outcome.has_errors(), "{:?}", outcome.diagnostics);
            let actual = fs::read_to_string(
                temporary
                    .path()
                    .join("generated")
                    .join(format!("{output}.ssrg")),
            )
            .unwrap();
            let expected = fs::read_to_string(
                fixture_root(fixture)
                    .join("expected")
                    .join(format!("{output}.ssrg")),
            )
            .unwrap();
            assert_eq!(actual, expected, "{fixture}");
        }
    }

    #[test]
    fn preserves_conversion_error_spans_without_updating_outputs() {
        for (fixture, code, start, end) in [
            ("dts-unsupported-any", "SES-F0101", 38, 41),
            ("dts-callback-missing-release", "SES-F0102", 30, 38),
        ] {
            let (temporary, outcome) = convert_fixture(fixture);
            assert!(outcome.has_errors());
            let diagnostic = outcome
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code == code)
                .unwrap();
            assert_eq!((diagnostic.start, diagnostic.end), (start, end));
            assert!(!temporary.path().join("generated").exists());
        }
    }
}

fn runtime_artifacts(
    request: &ConvertRequest,
    entry: &EntryConfig,
) -> BTreeMap<seseragi_interop::RuntimeTarget, seseragi_interop::RuntimeArtifactIdentity> {
    let foreign = fs::read_to_string(request.package_root.join("seseragi.toml"))
        .ok()
        .and_then(|source| seseragi_project::parse_manifest(&source).ok())
        .and_then(|manifest| manifest.foreign_typescript);
    let importer = request
        .generated_root
        .join(format!("{}.ssrg", entry.output));
    [
        seseragi_interop::RuntimeTarget::Browser,
        seseragi_interop::RuntimeTarget::Node,
        seseragi_interop::RuntimeTarget::Bun,
    ]
    .into_iter()
    .filter_map(|target| {
        seseragi_project::resolve_foreign_runtime_artifact_for_target(
            &request.package_root,
            foreign.as_ref(),
            &importer,
            &entry.specifier,
            target,
        )
        .ok()
        .map(|artifact| (target, artifact))
    })
    .collect()
}
