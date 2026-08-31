use seseragi_dts::{convert_package, ConvertRequest, DiagnosticSeverity};
use seseragi_project::parse_manifest;
use std::path::{Path, PathBuf};

pub(crate) fn dts(arguments: &[String]) -> Result<i32, String> {
    let [command, arguments @ ..] = arguments else {
        return Err(
            "invalid dts arguments; expected `seseragi dts convert [path] [--entry id]`".to_owned(),
        );
    };
    if command != "convert" {
        return Err(format!(
            "unsupported dts command `{command}`; expected `convert`"
        ));
    }
    let options = parse_convert_options(arguments)?;
    let root = package_root(&options.path)?;
    let manifest_path = root.join("seseragi.toml");
    let manifest_source = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest = parse_manifest(&manifest_source).map_err(|error| error.to_string())?;
    let foreign = manifest.foreign_typescript.ok_or_else(|| {
        "package has no [foreign.typescript] configuration for dts conversion".to_owned()
    })?;
    let bindings = foreign
        .bindings
        .ok_or_else(|| "package has no foreign.typescript.bindings settings file".to_owned())?;
    let host_manifest = PathBuf::from(foreign.manifest.as_str());
    let outcome = convert_package(&ConvertRequest {
        package_root: root.clone(),
        generated_root: root.join(manifest.layout.generated.as_str()),
        bindings: PathBuf::from(bindings.as_str()),
        host_manifest,
        entry: options.entry,
    })
    .map_err(|error| error.to_string())?;
    for diagnostic in &outcome.diagnostics {
        let severity = match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        };
        eprintln!(
            "{}:{}:{}: {severity}[{}]: {}",
            diagnostic.file, diagnostic.start, diagnostic.end, diagnostic.code, diagnostic.message
        );
    }
    if outcome.has_errors() {
        return Ok(1);
    }
    for converted in &outcome.converted {
        println!(
            "converted dts entry `{}` to {}",
            converted.id,
            relative_display(&root, &converted.source)
        );
    }
    Ok(0)
}

struct ConvertOptions {
    path: PathBuf,
    entry: Option<String>,
}

fn parse_convert_options(arguments: &[String]) -> Result<ConvertOptions, String> {
    let mut path = None;
    let mut entry = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--entry" => {
                let value = arguments.get(index + 1).ok_or_else(|| {
                    "missing value for `--entry` in `seseragi dts convert`".to_owned()
                })?;
                if entry.replace(value.clone()).is_some() {
                    return Err("duplicate `--entry` option".to_owned());
                }
                index += 2;
            }
            option if option.starts_with('-') => {
                return Err(format!("unknown dts convert option `{option}`"));
            }
            value => {
                if path.replace(PathBuf::from(value)).is_some() {
                    return Err("dts convert accepts at most one package path".to_owned());
                }
                index += 1;
            }
        }
    }
    Ok(ConvertOptions {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        entry,
    })
}

fn package_root(path: &Path) -> Result<PathBuf, String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", path.display()))?;
    crate::local_project::containing_package(&path)
        .ok_or_else(|| format!("no seseragi.toml found at or above {}", path.display()))
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
