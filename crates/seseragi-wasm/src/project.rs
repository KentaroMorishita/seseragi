use serde::{Deserialize, Serialize};
use seseragi_driver::{
    analyze_project as analyze_driver_project, compile_project as compile_driver_project,
    format_module, ProjectCompileError, ProjectModuleInput,
};
use seseragi_lowering::GeneratedBundle;
use seseragi_project::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, LinkError, ModuleGraph,
    ModuleGraphError, ModulePath,
};
use seseragi_runtime::{project_main_contract, MainContract};
use seseragi_syntax::{
    parse_diagnostics, parse_unlinked_module_interface, ByteSpan, DiagnosticArtifact,
    DiagnosticSeverity,
};
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;

const PROJECT_SCHEMA: u32 = 1;
const PACKAGE_SCOPE: &str = "playground";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRequest {
    schema: u32,
    entry: String,
    files: Vec<ProjectSource>,
}

#[derive(Clone, Deserialize)]
struct ProjectSource {
    path: String,
    source: String,
}

struct BrowserProject {
    graph: ModuleGraph<String>,
    inputs: Vec<ProjectModuleInput>,
    entry_module: String,
    paths: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Serialize)]
struct SourceRange {
    start: usize,
    end: usize,
}

impl From<ByteSpan> for SourceRange {
    fn from(value: ByteSpan) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectProblem {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary: Option<SourceRange>,
}

impl ProjectProblem {
    fn workspace(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            path: None,
            primary: None,
        }
    }

    fn file(
        code: &str,
        message: impl Into<String>,
        path: impl Into<String>,
        primary: Option<ByteSpan>,
    ) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            path: Some(path.into()),
            primary: primary.map(SourceRange::from),
        }
    }
}

#[derive(Serialize)]
struct ProjectFileDiagnostics {
    path: String,
    diagnostics: DiagnosticArtifact,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectGeneratedModule {
    path: String,
    module: String,
    generated: GeneratedBundle,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectEntry {
    path: String,
    module: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<MainContract>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum ProjectCompileResponse {
    Success {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        modules: Vec<ProjectGeneratedModule>,
        entry: ProjectEntry,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        problems: Vec<ProjectProblem>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectAnalysisDocument {
    path: String,
    module: String,
    document: seseragi_driver::AnalysisDocument,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum ProjectAnalysisResponse {
    Success {
        schema: u32,
        documents: Vec<ProjectAnalysisDocument>,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        problems: Vec<ProjectProblem>,
    },
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum ProjectFormatResponse {
    Success {
        schema: u32,
        path: String,
        source: String,
        changed: bool,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        problems: Vec<ProjectProblem>,
    },
}

struct ProjectFailure {
    diagnostics: Vec<ProjectFileDiagnostics>,
    problems: Vec<ProjectProblem>,
}

impl ProjectFailure {
    fn problem(problem: ProjectProblem) -> Self {
        Self {
            diagnostics: Vec::new(),
            problems: vec![problem],
        }
    }

    fn diagnostics(path: impl Into<String>, diagnostics: DiagnosticArtifact) -> Self {
        Self {
            diagnostics: vec![ProjectFileDiagnostics {
                path: path.into(),
                diagnostics,
            }],
            problems: Vec::new(),
        }
    }
}

/// Compiles an in-memory browser workspace through the shared project driver.
#[wasm_bindgen]
pub fn compile_project(request: &str) -> String {
    let response = match prepare_project(request) {
        Ok(project) => compile_prepared_project(project),
        Err(failure) => ProjectCompileResponse::Failure {
            schema: PROJECT_SCHEMA,
            diagnostics: failure.diagnostics,
            problems: failure.problems,
        },
    };
    serde_json::to_string(&response).expect("project compile response must serialize")
}

/// Analyzes every source in an in-memory browser workspace after linking its
/// local imports to the same typed public interfaces used by compilation.
#[wasm_bindgen]
pub fn analyze_project(request: &str) -> String {
    let response = match prepare_project(request) {
        Ok(project) => analyze_prepared_project(project),
        Err(failure) => ProjectAnalysisResponse::Failure {
            schema: PROJECT_SCHEMA,
            diagnostics: failure.diagnostics,
            problems: failure.problems,
        },
    };
    serde_json::to_string(&response).expect("project analysis response must serialize")
}

/// Formats one path selected from the versioned workspace request.
#[wasm_bindgen]
pub fn format_project_file(request: &str, path: &str) -> String {
    let response =
        match parse_request(request).and_then(|request| select_format_source(request, path)) {
            Ok(source) => match format_module(path, &source) {
                Ok(formatted) => ProjectFormatResponse::Success {
                    schema: PROJECT_SCHEMA,
                    path: path.to_owned(),
                    source: formatted.text,
                    changed: formatted.changed,
                },
                Err(diagnostics) => ProjectFormatResponse::Failure {
                    schema: PROJECT_SCHEMA,
                    diagnostics: vec![ProjectFileDiagnostics {
                        path: path.to_owned(),
                        diagnostics,
                    }],
                    problems: Vec::new(),
                },
            },
            Err(failure) => ProjectFormatResponse::Failure {
                schema: PROJECT_SCHEMA,
                diagnostics: failure.diagnostics,
                problems: failure.problems,
            },
        };
    serde_json::to_string(&response).expect("project format response must serialize")
}

fn compile_prepared_project(project: BrowserProject) -> ProjectCompileResponse {
    let BrowserProject {
        graph,
        inputs,
        entry_module,
        paths,
    } = project;
    match compile_driver_project(graph, inputs) {
        Ok(compiled) => {
            let diagnostics = compiled
                .order
                .iter()
                .filter_map(|module| {
                    let compiled_module = compiled.modules.get(module)?;
                    (!compiled_module.diagnostics.diagnostics.is_empty()).then(|| {
                        ProjectFileDiagnostics {
                            path: path_for_module(&paths, module),
                            diagnostics: compiled_module.diagnostics.clone(),
                        }
                    })
                })
                .collect();
            let modules = compiled
                .order
                .iter()
                .map(|module| ProjectGeneratedModule {
                    path: path_for_module(&paths, module),
                    module: module.clone(),
                    generated: compiled
                        .modules
                        .get(module)
                        .expect("compiled order names a module")
                        .generated
                        .clone(),
                })
                .collect();
            let (contract, error) = match project_main_contract(&compiled, &entry_module) {
                Ok(contract) => (Some(contract), None),
                Err(error) => (None, Some(error)),
            };
            ProjectCompileResponse::Success {
                schema: PROJECT_SCHEMA,
                diagnostics,
                modules,
                entry: ProjectEntry {
                    path: path_for_module(&paths, &entry_module),
                    module: entry_module,
                    contract,
                    error,
                },
            }
        }
        Err(error) => {
            let failure = driver_failure(error, &paths);
            ProjectCompileResponse::Failure {
                schema: PROJECT_SCHEMA,
                diagnostics: failure.diagnostics,
                problems: failure.problems,
            }
        }
    }
}

fn analyze_prepared_project(project: BrowserProject) -> ProjectAnalysisResponse {
    let BrowserProject {
        graph,
        inputs,
        paths,
        ..
    } = project;
    match analyze_driver_project(graph, inputs) {
        Ok(analyzed) => ProjectAnalysisResponse::Success {
            schema: PROJECT_SCHEMA,
            documents: analyzed
                .order
                .iter()
                .map(|module| ProjectAnalysisDocument {
                    path: path_for_module(&paths, module),
                    module: module.clone(),
                    document: analyzed
                        .documents
                        .get(module)
                        .expect("analysis order names a document")
                        .clone(),
                })
                .collect(),
        },
        Err(error) => {
            let failure = driver_failure(error, &paths);
            ProjectAnalysisResponse::Failure {
                schema: PROJECT_SCHEMA,
                diagnostics: failure.diagnostics,
                problems: failure.problems,
            }
        }
    }
}

fn prepare_project(request: &str) -> Result<BrowserProject, ProjectFailure> {
    let request = parse_request(request)?;
    if request.files.is_empty() {
        return Err(ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            "workspace must contain at least one source file",
        )));
    }

    let mut sources = BTreeMap::<String, ProjectSource>::new();
    let mut modules_by_path = BTreeMap::<String, String>::new();
    let mut paths = BTreeMap::<String, String>::new();
    for file in request.files {
        let (canonical_path, module_path, module) = source_identity(&file.path)?;
        if canonical_path != file.path {
            return Err(ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("source path must be normalized as `{canonical_path}`"),
                file.path,
                None,
            )));
        }
        if sources.contains_key(&canonical_path) || paths.contains_key(&module) {
            return Err(ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                "workspace contains a duplicate source path or module identity",
                canonical_path,
                None,
            )));
        }
        modules_by_path.insert(module_path, module.clone());
        paths.insert(module, canonical_path.clone());
        sources.insert(canonical_path, file);
    }

    let (entry_path, _, entry_module) = source_identity(&request.entry)?;
    if entry_path != request.entry || !sources.contains_key(&entry_path) {
        return Err(ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!(
                "entry source `{}` is not present in the workspace",
                request.entry
            ),
        )));
    }

    let mut graph = ModuleGraph::new();
    let mut inputs = Vec::with_capacity(sources.len());
    for (path, file) in &sources {
        let (_, current_path, module) = source_identity(path)?;
        let diagnostics = parse_diagnostics(path, &file.source);
        if diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        {
            return Err(ProjectFailure::diagnostics(path, diagnostics));
        }
        let unlinked = parse_unlinked_module_interface(path, &module, &file.source);
        let mut dependencies = BTreeMap::new();
        for import in unlinked.imports {
            let target_path = match classify_specifier(&import.specifier) {
                Ok(ImportSpecifier::Standard(_)) => continue,
                Ok(ImportSpecifier::Relative(specifier)) => resolve_relative_specifier(
                    &ModulePath::parse(&current_path).expect("validated path"),
                    &specifier,
                )
                .map_err(|error| {
                    ProjectFailure::problem(ProjectProblem::file(
                        "SES-N0104",
                        format!("cannot resolve import `{}`: {error}", import.specifier),
                        path,
                        Some(import.span),
                    ))
                })?
                .as_str()
                .to_owned(),
                Ok(ImportSpecifier::SelfPackage(target)) => ModulePath::parse(&target)
                    .map_err(|error| {
                        ProjectFailure::problem(ProjectProblem::file(
                            "SES-N0104",
                            format!("cannot resolve import `{}`: {error}", import.specifier),
                            path,
                            Some(import.span),
                        ))
                    })?
                    .as_str()
                    .to_owned(),
                Ok(ImportSpecifier::Generated(_)) | Ok(ImportSpecifier::Package(_)) => {
                    return Err(ProjectFailure::problem(ProjectProblem::file(
                        "SES-N0104",
                        format!(
                            "browser workspace does not resolve import `{}`",
                            import.specifier
                        ),
                        path,
                        Some(import.span),
                    )));
                }
                Err(error) => {
                    return Err(ProjectFailure::problem(ProjectProblem::file(
                        "SES-N0104",
                        format!("cannot resolve import `{}`: {error}", import.specifier),
                        path,
                        Some(import.span),
                    )));
                }
            };
            let Some(dependency) = modules_by_path.get(&target_path) else {
                return Err(ProjectFailure::problem(ProjectProblem::file(
                    "SES-N0104",
                    format!(
                        "import `{}` resolves to missing module `{target_path}`",
                        import.specifier
                    ),
                    path,
                    Some(import.span),
                )));
            };
            if dependencies
                .insert(import.specifier.clone(), dependency.clone())
                .is_some()
            {
                return Err(ProjectFailure::problem(ProjectProblem::file(
                    "SES-N0101",
                    format!("duplicate import specifier `{}`", import.specifier),
                    path,
                    Some(import.span),
                )));
            }
        }
        graph
            .add_module(module.clone(), dependencies)
            .map_err(|error| graph_failure(error, &paths))?;
        inputs.push(
            ProjectModuleInput::new(path, &module, &file.source, format!("{current_path}.js"))
                .with_package_scope(PACKAGE_SCOPE),
        );
    }
    graph
        .topological_order()
        .map_err(|error| graph_failure(error, &paths))?;

    Ok(BrowserProject {
        graph,
        inputs,
        entry_module,
        paths,
    })
}

fn parse_request(request: &str) -> Result<ProjectRequest, ProjectFailure> {
    let request: ProjectRequest = serde_json::from_str(request).map_err(|error| {
        ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!("invalid project request JSON: {error}"),
        ))
    })?;
    if request.schema != PROJECT_SCHEMA {
        return Err(ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!("project request schema must be {PROJECT_SCHEMA}"),
        )));
    }
    Ok(request)
}

fn select_format_source(request: ProjectRequest, path: &str) -> Result<String, ProjectFailure> {
    let (canonical, _, _) = source_identity(path)?;
    if canonical != path {
        return Err(ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            format!("source path must be normalized as `{canonical}`"),
            path,
            None,
        )));
    }
    let mut seen = BTreeSet::new();
    let mut selected = None;
    for file in request.files {
        let (file_path, _, _) = source_identity(&file.path)?;
        if file_path != file.path || !seen.insert(file_path.clone()) {
            return Err(ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                "workspace contains a non-normalized or duplicate source path",
                file.path,
                None,
            )));
        }
        if file_path == path {
            selected = Some(file.source);
        }
    }
    selected.ok_or_else(|| {
        ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            "format target is not present in the workspace",
            path,
            None,
        ))
    })
}

fn source_identity(path: &str) -> Result<(String, String, String), ProjectFailure> {
    if path.contains('\0') {
        return Err(ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            "source path must not contain NUL",
            path,
            None,
        )));
    }
    let Some(module_path) = path.strip_suffix(".ssrg") else {
        return Err(ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            "source path must end in `.ssrg`",
            path,
            None,
        )));
    };
    let module_path = ModulePath::parse(module_path).map_err(|error| {
        ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            format!("invalid source path `{path}`: {error}"),
            path,
            None,
        ))
    })?;
    let module_path = module_path.as_str().to_owned();
    let canonical_path = format!("{module_path}.ssrg");
    let module = format!("{PACKAGE_SCOPE}/{module_path}");
    Ok((canonical_path, module_path, module))
}

fn graph_failure(
    error: ModuleGraphError<String>,
    paths: &BTreeMap<String, String>,
) -> ProjectFailure {
    let (code, message, module) = match error {
        ModuleGraphError::Cycle { modules } => (
            "SES-N0103",
            format!(
                "module import graph contains a cycle: {}",
                modules.join(" -> ")
            ),
            modules.first().cloned(),
        ),
        ModuleGraphError::MissingModule { module, dependency } => (
            "SES-N0104",
            format!("module `{module}` depends on missing module `{dependency}`"),
            Some(module),
        ),
        ModuleGraphError::DuplicateModule { module } => (
            "SES-K0001",
            format!("duplicate module `{module}`"),
            Some(module),
        ),
        ModuleGraphError::DuplicateSpecifier { module, specifier } => (
            "SES-N0101",
            format!("module `{module}` contains duplicate specifier `{specifier}`"),
            Some(module),
        ),
    };
    ProjectFailure::problem(ProjectProblem {
        code: code.to_owned(),
        message,
        path: module.map(|module| path_for_module(paths, &module)),
        primary: None,
    })
}

fn driver_failure(error: ProjectCompileError, paths: &BTreeMap<String, String>) -> ProjectFailure {
    match error {
        ProjectCompileError::Diagnostics {
            module,
            diagnostics,
        } => ProjectFailure::diagnostics(path_for_module(paths, &module), diagnostics),
        ProjectCompileError::Graph(error) => graph_failure(error, paths),
        ProjectCompileError::Link { module, errors } => ProjectFailure {
            diagnostics: Vec::new(),
            problems: errors
                .into_iter()
                .map(|error| link_problem(error, path_for_module(paths, &module)))
                .collect(),
        },
        ProjectCompileError::DuplicateInput { module } => {
            ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("duplicate project input `{module}`"),
                path_for_module(paths, &module),
                None,
            ))
        }
        ProjectCompileError::UnexpectedInput { module } => {
            ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("unexpected project input `{module}`"),
                path_for_module(paths, &module),
                None,
            ))
        }
        ProjectCompileError::MissingInput { module } => {
            ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("missing project input `{module}`"),
                path_for_module(paths, &module),
                None,
            ))
        }
        other => ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!("project compiler rejected the workspace: {other:?}"),
        )),
    }
}

fn link_problem(error: LinkError, path: String) -> ProjectProblem {
    let (code, message) = match &error {
        LinkError::UnresolvedSpecifier { specifier, .. } => (
            "SES-N0104",
            format!("module specifier `{specifier}` could not be resolved"),
        ),
        LinkError::MissingExport { module, name, .. } => (
            "SES-N0104",
            format!("module `{module}` does not export `{name}`"),
        ),
        LinkError::PrivateExport { module, name, .. } => (
            "SES-N0102",
            format!("module `{module}` keeps `{name}` private"),
        ),
        LinkError::DuplicateImport { local_name, .. } => (
            "SES-N0101",
            format!("import name `{local_name}` is ambiguous"),
        ),
        LinkError::MissingNamespaceAlias { .. } => {
            ("SES-N0104", "namespace import requires an alias".to_owned())
        }
        LinkError::UnsupportedImportNamespace { namespace, .. } => (
            "SES-N0104",
            format!("import namespace `{namespace}` is unsupported"),
        ),
    };
    ProjectProblem::file(code, message, path, Some(error.origin()))
}

fn path_for_module(paths: &BTreeMap<String, String>, module: &str) -> String {
    paths
        .get(module)
        .cloned()
        .unwrap_or_else(|| format!("{module}.ssrg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn request(files: Value, entry: &str) -> String {
        json!({ "schema": 1, "entry": entry, "files": files }).to_string()
    }

    #[test]
    fn compiles_and_analyzes_a_multi_module_browser_workspace() {
        let request = request(
            json!([
                {
                    "path": "domain.ssrg",
                    "source": "pub fn double value: Int -> Int = value + value\n"
                },
                {
                    "path": "main.ssrg",
                    "source": "import { double } from \"./domain\"\n\npub effect fn main -> Unit with Console fails ConsoleError =\n  double 21 |> debug |> println\n"
                }
            ]),
            "main.ssrg",
        );

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "success");
        assert_eq!(compiled["modules"].as_array().unwrap().len(), 2);
        assert_eq!(compiled["entry"]["path"], "main.ssrg");
        assert_eq!(
            compiled["entry"]["contract"]["environment"][0]["service"],
            "console"
        );
        assert!(compiled["modules"][1]["generated"]["typescript"]
            .as_str()
            .unwrap()
            .contains("from \"./domain.js\""));

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "success");
        assert_eq!(analyzed["documents"].as_array().unwrap().len(), 2);
        let main = analyzed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|document| document["path"] == "main.ssrg")
            .unwrap();
        assert!(main["document"]["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|symbol| symbol["name"] == "double" && symbol["module"] == "playground/domain"));
    }

    #[test]
    fn selects_a_failure_dictionary_from_a_generated_provider_module() {
        let request = request(
            json!([
                {
                    "path": "domain/provider.ssrg",
                    "source": "pub type InputError deriving Show =\n  | InvalidInput String\n"
                },
                {
                    "path": "effects/facade.ssrg",
                    "source": "import { InputError, InvalidInput } from \"../domain/provider\"\n\npub effect fn reject input: String =\n  fail (InvalidInput input)\n"
                },
                {
                    "path": "app/main.ssrg",
                    "source": "import { reject } from \"../effects/facade\"\n\npub effect fn main =\n  do {\n    reject \"lizard\"\n    succeed ()\n  }\n"
                }
            ]),
            "app/main.ssrg",
        );

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "success");
        assert_eq!(
            compiled["entry"]["contract"]["failureRenderer"]["module"], "./domain/provider.ts",
            "{compiled}"
        );
        assert_eq!(
            compiled["entry"]["contract"]["failureRenderer"]["export"],
            "__ssrg$instance$Show$0"
        );
    }

    #[test]
    fn reports_missing_modules_cycles_duplicates_and_file_diagnostics() {
        let missing = request(
            json!([{ "path": "main.ssrg", "source": "import { value } from \"./missing\"\n" }]),
            "main.ssrg",
        );
        let missing: Value = serde_json::from_str(&compile_project(&missing)).unwrap();
        assert_eq!(missing["status"], "failure");
        assert_eq!(missing["problems"][0]["code"], "SES-N0104");
        assert_eq!(missing["problems"][0]["path"], "main.ssrg");
        assert!(missing["problems"][0]["primary"]["end"].as_u64().unwrap() > 0);

        let cycle = request(
            json!([
                { "path": "a.ssrg", "source": "import { b } from \"./b\"\npub let a = b\n" },
                { "path": "b.ssrg", "source": "import { a } from \"./a\"\npub let b = a\n" }
            ]),
            "a.ssrg",
        );
        let cycle: Value = serde_json::from_str(&analyze_project(&cycle)).unwrap();
        assert_eq!(cycle["problems"][0]["code"], "SES-N0103");

        let duplicate = request(
            json!([
                { "path": "main.ssrg", "source": "pub let first = 1\n" },
                { "path": "main.ssrg", "source": "pub let second = 2\n" }
            ]),
            "main.ssrg",
        );
        let duplicate: Value = serde_json::from_str(&compile_project(&duplicate)).unwrap();
        assert_eq!(duplicate["problems"][0]["code"], "SES-K0001");

        let invalid = request(
            json!([{ "path": "main.ssrg", "source": "pub let broken: Int =\n" }]),
            "main.ssrg",
        );
        let invalid: Value = serde_json::from_str(&analyze_project(&invalid)).unwrap();
        assert_eq!(invalid["diagnostics"][0]["path"], "main.ssrg");
        assert!(
            invalid["diagnostics"][0]["diagnostics"]["diagnostics"][0]["primary"]["end"]
                .as_u64()
                .is_some()
        );
    }

    #[test]
    fn formats_one_file_through_the_workspace_request_shape() {
        let request = request(
            json!([{ "path": "main.ssrg", "source": "pub let value: Int = 1   \r\n" }]),
            "main.ssrg",
        );
        let formatted: Value =
            serde_json::from_str(&format_project_file(&request, "main.ssrg")).unwrap();

        assert_eq!(formatted["status"], "success");
        assert_eq!(formatted["path"], "main.ssrg");
        assert_eq!(formatted["source"], "pub let value: Int = 1\n");
        assert_eq!(formatted["changed"], true);
    }
}
