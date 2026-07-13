use super::model::{
    DeferredTables, LanguageRequirement, LayoutPath, Manifest, ManifestLayout, ManifestPackage,
    ManifestRun, RunSeed, SignalMode, TargetId,
};
use super::ManifestError;
use crate::{ModulePath, PackageName};
use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;

pub fn parse_manifest(source: &str) -> Result<Manifest, ManifestError> {
    let raw = toml::from_str::<RawManifest>(source)
        .map_err(|error| ManifestError::toml(error, source))?;
    let package = parse_package(raw.package)?;
    let layout = parse_layout(raw.layout.unwrap_or_default())?;
    let exports = parse_exports(raw.exports)?;
    let run = raw.run.map(parse_run).transpose()?;

    Ok(Manifest {
        package,
        layout,
        exports,
        run,
        deferred: DeferredTables {
            dependencies: raw.dependencies,
            foreign: raw.foreign,
            test: raw.test,
            benchmark: raw.benchmark,
            tool: raw.tool,
        },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    package: RawPackage,
    #[serde(default)]
    layout: Option<RawLayout>,
    #[serde(default)]
    exports: BTreeMap<String, String>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(default)]
    foreign: Option<toml::Table>,
    #[serde(default)]
    run: Option<RawRun>,
    #[serde(default)]
    test: Option<toml::Table>,
    #[serde(default)]
    benchmark: Option<toml::Table>,
    #[serde(default)]
    tool: Option<toml::Table>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackage {
    name: String,
    version: String,
    language: String,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawLayout {
    source: String,
    tests: String,
    benchmarks: String,
    generated: String,
}

impl Default for RawLayout {
    fn default() -> Self {
        Self {
            source: "src".to_owned(),
            tests: "tests".to_owned(),
            benchmarks: "benchmarks".to_owned(),
            generated: ".seseragi/generated".to_owned(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRun {
    entry: String,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    signal_mode: RawSignalMode,
    #[serde(default)]
    shutdown_grace_ms: Option<u64>,
    #[serde(default)]
    hash_seed: RawSeed,
    #[serde(default)]
    random_seed: RawSeed,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RawSignalMode {
    #[default]
    Cancel,
    Forward,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum RawSeed {
    Name(String),
    Fixed(i64),
}

impl Default for RawSeed {
    fn default() -> Self {
        Self::Name("entropy".to_owned())
    }
}

fn parse_package(raw: RawPackage) -> Result<ManifestPackage, ManifestError> {
    let name = PackageName::parse(&raw.name).map_err(ManifestError::PackageName)?;
    let version = Version::parse(&raw.version)
        .map_err(|_| ManifestError::InvalidVersion(raw.version.clone()))?;
    validate_language_requirement(&raw.language)?;
    Ok(ManifestPackage {
        name,
        version,
        language: LanguageRequirement::new(raw.language),
    })
}

fn validate_language_requirement(value: &str) -> Result<(), ManifestError> {
    super::requirement::validate(value)
        .then_some(())
        .ok_or_else(|| ManifestError::InvalidLanguageRequirement(value.to_owned()))
}

fn parse_layout(raw: RawLayout) -> Result<ManifestLayout, ManifestError> {
    let layout = ManifestLayout {
        source: parse_layout_path("layout.source", raw.source)?,
        tests: parse_layout_path("layout.tests", raw.tests)?,
        benchmarks: parse_layout_path("layout.benchmarks", raw.benchmarks)?,
        generated: parse_layout_path("layout.generated", raw.generated)?,
    };
    let roots = [
        ("layout.source", &layout.source),
        ("layout.tests", &layout.tests),
        ("layout.benchmarks", &layout.benchmarks),
        ("layout.generated", &layout.generated),
    ];
    for (index, (left_name, left)) in roots.iter().enumerate() {
        for (right_name, right) in &roots[index + 1..] {
            if paths_overlap(left, right) {
                return Err(ManifestError::OverlappingLayoutRoots {
                    left: left_name,
                    right: right_name,
                });
            }
        }
    }
    Ok(layout)
}

fn parse_layout_path(field: &'static str, value: String) -> Result<LayoutPath, ManifestError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err(ManifestError::InvalidLayoutPath { field, value });
    }
    Ok(LayoutPath::new(value))
}

fn paths_overlap(left: &LayoutPath, right: &LayoutPath) -> bool {
    let left = left.segments().collect::<Vec<_>>();
    let right = right.segments().collect::<Vec<_>>();
    left.starts_with(&right) || right.starts_with(&left)
}

fn parse_exports(
    raw: BTreeMap<String, String>,
) -> Result<BTreeMap<String, ModulePath>, ManifestError> {
    raw.into_iter()
        .map(|(key, target)| {
            if key != "." {
                ModulePath::parse(&key)
                    .map_err(|error| ManifestError::InvalidExportKey(key.clone(), error))?;
            }
            let target = ModulePath::parse(&target)
                .map_err(|error| ManifestError::InvalidExportTarget(target, error))?;
            Ok((key, target))
        })
        .collect()
}

fn parse_run(raw: RawRun) -> Result<ManifestRun, ManifestError> {
    let entry = ModulePath::parse(&raw.entry)
        .map_err(|error| ManifestError::InvalidRunEntry(raw.entry, error))?;
    let target = raw.target.map(parse_target).transpose()?;
    let signal_mode = match raw.signal_mode {
        RawSignalMode::Cancel => SignalMode::Cancel,
        RawSignalMode::Forward => SignalMode::Forward,
    };
    if signal_mode == SignalMode::Forward && raw.shutdown_grace_ms.is_some() {
        return Err(ManifestError::ShutdownGraceWithForwardSignal);
    }
    Ok(ManifestRun {
        entry,
        target,
        signal_mode,
        shutdown_grace_ms: match (signal_mode, raw.shutdown_grace_ms) {
            (SignalMode::Cancel, value) => Some(value.unwrap_or(10_000)),
            (SignalMode::Forward, _) => None,
        },
        hash_seed: parse_seed("run.hash_seed", raw.hash_seed)?,
        random_seed: parse_seed("run.random_seed", raw.random_seed)?,
    })
}

fn parse_target(value: String) -> Result<TargetId, ManifestError> {
    if valid_target_id(&value) {
        Ok(TargetId::new(value))
    } else {
        Err(ManifestError::InvalidTarget(value))
    }
}

fn valid_target_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn parse_seed(field: &'static str, seed: RawSeed) -> Result<RunSeed, ManifestError> {
    match seed {
        RawSeed::Name(name) if name == "entropy" => Ok(RunSeed::Entropy),
        RawSeed::Name(value) => Err(ManifestError::InvalidSeed { field, value }),
        RawSeed::Fixed(value) => Ok(RunSeed::Fixed(value)),
    }
}
