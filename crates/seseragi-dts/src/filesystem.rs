//! Native filesystem adapter for the shared in-memory converter.
use crate::convert::{
    evaluation_name, parse_config, sha256, validate_config, BindingMetadata, ConversionDiagnostic,
    ConvertError, Converter, DiagnosticSeverity, HostModuleIdentity, METADATA_SCHEMA,
};
use std::fs;
use std::path::{Path, PathBuf};

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
    let converter = Converter::new(&settings_source)?;
    if let Some(entry) = &request.entry {
        if !converter.entries().contains_key(entry) {
            return Err(ConvertError::new(format!(
                "binding entry `{entry}` does not exist in {}",
                request.bindings.display()
            )));
        }
    }

    let host_manifest_path = request.package_root.join(&request.host_manifest);
    let host_manifest_source = read_utf8(&host_manifest_path, "foreign host manifest")?;
    let mut outcome = ConversionOutcome {
        converted: Vec::new(),
        diagnostics: Vec::new(),
    };
    for (id, entry) in converter.entries() {
        if request
            .entry
            .as_deref()
            .is_some_and(|selected| selected != id)
        {
            continue;
        }
        let declaration_path = request.package_root.join(&entry.declaration);
        let declaration = read_utf8(&declaration_path, "TypeScript declaration")?;
        let metadata_path = request
            .generated_root
            .join(format!("{}.binding.json", entry.output));
        let previous = fs::read_to_string(&metadata_path).ok();
        let prepared = converter.prepare(id, &declaration, previous.as_deref())?;
        // Preserve conversion-error precedence over host resolution and never write
        // any of an entry's artifacts when its conversion failed.
        if prepared.has_errors() {
            outcome.diagnostics.extend(prepared.diagnostics);
            continue;
        }
        let host_module = resolve_host_module_identity(
            &request.package_root,
            &request.host_manifest,
            &host_manifest_source,
            &entry.specifier,
        )?;
        let result = prepared.finish(host_module)?;
        outcome.diagnostics.extend(result.diagnostics);
        let generated = result
            .generated
            .expect("successful conversion has artifacts");
        let source_path = request
            .generated_root
            .join(format!("{}.ssrg", entry.output));
        let report_path = request
            .generated_root
            .join(format!("{}.report.json", entry.output));
        atomic_write_set(&[
            (&source_path, generated.source.as_bytes()),
            (&metadata_path, generated.metadata.as_bytes()),
            (&report_path, generated.report.as_bytes()),
        ])?;
        outcome.converted.push(ConvertedEntry {
            id: id.clone(),
            source: source_path,
            metadata: metadata_path,
            report: report_path,
        });
    }
    Ok(outcome)
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
                entry: id,
                message: format!(
                    "generated binding `{}` is stale; run `seseragi dts convert`",
                    entry.output
                ),
            });
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
    crate::resolve_host_module_identity(
        &package_root.join(host_manifest).to_string_lossy(),
        host_manifest_source,
        specifier,
        |path| {
            read_utf8(Path::new(path), "resolved foreign package manifest")
                .map_err(|error| error.to_string())
        },
    )
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
