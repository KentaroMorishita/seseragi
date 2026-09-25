use std::path::Path;

use seseragi_driver::{
    analyze_module, analyze_project, CompileInput, ProjectCompileError, ProjectModuleInput,
};
use seseragi_project::{logical_module_id, logical_package_scope, ModuleGraph};
use seseragi_runtime::DiagnosticFormat;
use seseragi_syntax::{DiagnosticArtifact, DiagnosticSeverity};

use crate::diagnostics::{render_diagnostics, DiagnosticDocument};

struct LintDocument {
    path: String,
    source: String,
    artifact: DiagnosticArtifact,
}

pub(crate) fn lint(arguments: &[String]) -> Result<i32, String> {
    let mut path = None;
    let mut format = DiagnosticFormat::Text;
    let mut format_seen = false;
    let mut deny_warnings = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--diagnostic-format" => {
                if format_seen {
                    return Err("--diagnostic-format may only be specified once".to_owned());
                }
                format_seen = true;
                index += 1;
                format = match arguments.get(index).map(String::as_str) {
                    Some("text") => DiagnosticFormat::Text,
                    Some("json") => DiagnosticFormat::Json,
                    _ => return Err("--diagnostic-format requires text or json".to_owned()),
                };
            }
            "--deny-warnings" => {
                if deny_warnings {
                    return Err("--deny-warnings may only be specified once".to_owned());
                }
                deny_warnings = true;
            }
            argument if argument.starts_with('-') => {
                return Err(format!("unknown lint option `{argument}`"));
            }
            argument if path.is_none() => path = Some(argument.to_owned()),
            argument => return Err(format!("unexpected lint argument `{argument}`")),
        }
        index += 1;
    }
    let path = path.ok_or_else(|| "lint requires a source file or package path".to_owned())?;
    let path = Path::new(&path);
    if !path.exists() {
        return Err(format!("lint path does not exist: {}", path.display()));
    }
    if path.is_file() && path.extension().and_then(|extension| extension.to_str()) != Some("ssrg") {
        return Err("lint expects a .ssrg source file or package".to_owned());
    }
    let documents = if let Some(package) = crate::local_project::containing_package(path) {
        lint_package(&package)?
    } else if path.is_dir() {
        lint_package(path)?
    } else {
        lint_file(path)?
    };
    let has_error = documents.iter().any(|document| {
        document
            .artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    });
    let has_warning = documents.iter().any(|document| {
        document
            .artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
    });
    let references = documents
        .iter()
        .map(|document| DiagnosticDocument {
            path: &document.path,
            source: &document.source,
            artifact: &document.artifact,
        })
        .collect::<Vec<_>>();
    eprint!("{}", render_diagnostics(format, &references)?);
    Ok(if has_error {
        2
    } else if has_warning && deny_warnings {
        1
    } else {
        0
    })
}

fn lint_file(path: &Path) -> Result<Vec<LintDocument>, String> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("ssrg") {
        return Err("lint expects a .ssrg source file or package".to_owned());
    }
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let name = path.to_string_lossy().into_owned();
    let analysis = analyze_module(CompileInput::new(&name, "single-file/main", &source));
    Ok(vec![LintDocument {
        path: name,
        source,
        artifact: analysis.authoring_diagnostics(),
    }])
}

fn lint_package(path: &Path) -> Result<Vec<LintDocument>, String> {
    let project = seseragi_project::load_local_documents(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let mut graph = ModuleGraph::new();
    for (identity, _) in project.modules() {
        let dependencies = project
            .graph()
            .dependencies_for(identity)
            .expect("loaded local document graph contains every source module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, logical_module_id(&dependency)));
        graph
            .add_module(logical_module_id(identity), dependencies)
            .expect("loaded local document graph was already validated");
    }
    let inputs = project.modules().map(|(identity, module)| {
        let module_id = logical_module_id(identity);
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            &module_id,
            module.source(),
            format!("lint/{module_id}.js"),
        )
        .with_package_scope(logical_package_scope(identity.package()))
    });
    let mut analyzed = match analyze_project(graph, inputs) {
        Ok(analyzed) => analyzed,
        Err(error) => match error {
            ProjectCompileError::Diagnostics { modules } => {
                let mut documents = Vec::new();
                for entry in modules {
                    let (_, module) = project
                        .modules()
                        .find(|(identity, _)| logical_module_id(identity) == entry.module)
                        .ok_or_else(|| {
                            format!("compiler diagnostic module is missing: {}", entry.module)
                        })?;
                    documents.push(LintDocument {
                        path: entry.diagnostics.source.clone(),
                        source: module.source().to_owned(),
                        artifact: entry.diagnostics.clone(),
                    });
                }
                documents.sort_by(|left, right| left.path.cmp(&right.path));
                return Ok(documents);
            }
            other => return Err(format!("project analyzer rejected package: {other:?}")),
        },
    };
    let mut documents = Vec::new();
    for (identity, module) in project.modules() {
        let module_id = logical_module_id(identity);
        let analysis = analyzed
            .documents
            .remove(&module_id)
            .ok_or_else(|| format!("project analysis is missing: {module_id}"))?;
        let own_package = identity.package() == project.packages().root();
        let artifact = if own_package {
            analysis.authoring_diagnostics()
        } else {
            analysis.diagnostics.clone()
        };
        if !own_package && artifact.diagnostics.is_empty() {
            continue;
        }
        documents.push(LintDocument {
            path: module.source_path().to_string_lossy().into_owned(),
            source: module.source().to_owned(),
            artifact,
        });
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}
