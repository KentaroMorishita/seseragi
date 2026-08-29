use seseragi_driver::{
    compile_project, CompiledProject, LinkedCompileError, ProjectCompileError, ProjectModuleInput,
};
use seseragi_project::{
    logical_module_id, logical_package_scope, LoadedLocalDocuments, ModuleGraph, ModuleIdentity,
};
use seseragi_syntax::{
    extract_document_tests, DiagnosticArtifact, DiagnosticSeverity, DocumentTestBlock,
    DocumentTestMode,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SYNTHETIC_SPECIFIER: &str = "__documented";

pub(crate) fn doc(arguments: &[String]) -> Result<i32, String> {
    let invocation = Invocation::parse(arguments)?;
    if !invocation.test {
        return Err(
            "document artifact generation has no normative wire schema yet; use `seseragi doc --test`"
                .to_owned(),
        );
    }
    let root = crate::local_project::containing_package(&invocation.path)
        .unwrap_or_else(|| invocation.path.clone());
    if !root.join("seseragi.toml").is_file() {
        return Err(format!(
            "doc expects a package containing seseragi.toml: {}",
            root.display()
        ));
    }
    seseragi_project::read_and_validate_lockfile(&root)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let project = seseragi_project::load_local_documents(&root)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let base =
        compile_documents(&project, None).map_err(|error| format_project_error(&project, error))?;
    let root_manifest = project
        .packages()
        .package(project.packages().root())
        .expect("root manifest")
        .manifest();
    let target = invocation
        .target
        .as_deref()
        .or_else(|| {
            root_manifest
                .test
                .as_ref()
                .and_then(|test| test.target.as_ref())
                .map(|value| value.as_str())
        })
        .or_else(|| {
            root_manifest
                .run
                .as_ref()
                .and_then(|run| run.target.as_ref())
                .map(|value| value.as_str())
        });
    let has_run_block = project.roots().any(|identity| {
        let module = project.module(identity).expect("document root is loaded");
        extract_document_tests(module.source_path().to_string_lossy(), module.source())
            .iter()
            .any(|block| matches!(block.mode, DocumentTestMode::Run))
    });
    if has_run_block {
        let selected = target.ok_or_else(|| {
            "doc run target is required; pass `--target node` or set test.target/run.target"
                .to_owned()
        })?;
        if !matches!(selected, "node" | "process" | "test-js") {
            return Err(format!(
                "doc target `{selected}` is unsupported; expected `node`"
            ));
        }
    }

    let mut passed = 0;
    let mut failed = 0;
    for identity in project.roots() {
        let module = project.module(identity).expect("document root is loaded");
        let blocks =
            extract_document_tests(module.source_path().to_string_lossy(), module.source());
        for block in blocks {
            let id = format!(
                "{}::{}#{}",
                identity.path().as_str(),
                block.declaration,
                block.ordinal
            );
            match run_block(&project, &base, identity, &block, target, &root) {
                Ok(()) => {
                    println!("PASS {id} {}", block.mode.label());
                    passed += 1;
                }
                Err(message) => {
                    println!("FAIL {id} {}", block.mode.label());
                    eprintln!("{message}");
                    failed += 1;
                }
            }
        }
    }
    println!("{passed} passed; {failed} failed");
    Ok(if failed == 0 { 0 } else { 1 })
}

fn run_block(
    project: &LoadedLocalDocuments,
    base: &CompiledProject,
    documented: &ModuleIdentity,
    block: &DocumentTestBlock,
    target: Option<&str>,
    root: &Path,
) -> Result<(), String> {
    let documented_id = logical_module_id(documented);
    let import = public_import(base, &documented_id)?;
    let prefix = if import.is_empty() {
        String::new()
    } else {
        format!("{import}\n")
    };
    let synthetic = format!(
        "{}::doc/{}/{}",
        logical_package_scope(project.packages().root()),
        documented.path().as_str(),
        block.ordinal
    );
    let compiled = compile_documents(
        project,
        Some(SyntheticInput {
            id: synthetic.clone(),
            documented: documented_id,
            source: format!("{prefix}{}", block.source),
            has_import: !import.is_empty(),
        }),
    );
    match &block.mode {
        DocumentTestMode::CompileFail { code } => match compiled {
            Ok(_) => Err(format!(
                "{}: documentation block compiled successfully; expected error[{code}]",
                location(project, documented, block, 0)
            )),
            Err(error) => {
                let Some(diagnostics) = synthetic_diagnostics(&error, &synthetic) else {
                    return Err(format_project_error(project, error));
                };
                if diagnostics.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == *code && diagnostic.severity == DiagnosticSeverity::Error
                }) {
                    Ok(())
                } else {
                    Err(render_block_diagnostics(
                        project,
                        documented,
                        block,
                        diagnostics,
                        prefix.len(),
                        Some(code),
                    ))
                }
            }
        },
        DocumentTestMode::Check => match compiled {
            Ok(_) => Ok(()),
            Err(error) => {
                block_compile_failure(project, documented, block, error, &synthetic, prefix.len())
            }
        },
        DocumentTestMode::Run => {
            let _target = target.expect("run target was validated before execution");
            let compiled = compiled.map_err(|error| {
                block_compile_failure(project, documented, block, error, &synthetic, prefix.len())
                    .unwrap_err()
            })?;
            let outcome =
                seseragi_runtime::run_document_entry_in_directory(&compiled, &synthetic, root)
                    .map_err(|error| {
                        format!("{}: {error}", location(project, documented, block, 0))
                    })?;
            let expected = block.expected_stdout.as_deref().unwrap_or("");
            if outcome.exit_code != 0 {
                return Err(format!(
                    "{}: run exited with {}\n{}",
                    location(project, documented, block, 0),
                    outcome.exit_code,
                    outcome.stderr
                ));
            }
            if outcome.stdout != expected {
                return Err(format!(
                    "{}: stdout mismatch\n  expected: {:?}\n  actual:   {:?}",
                    location(project, documented, block, 0),
                    expected,
                    outcome.stdout
                ));
            }
            Ok(())
        }
    }
}

struct SyntheticInput {
    id: String,
    documented: String,
    source: String,
    has_import: bool,
}

fn compile_documents(
    project: &LoadedLocalDocuments,
    synthetic: Option<SyntheticInput>,
) -> Result<CompiledProject, ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    for (identity, _) in project.modules() {
        graph
            .add_module(
                logical_module_id(identity),
                project
                    .graph()
                    .dependencies_for(identity)
                    .expect("loaded graph")
                    .into_iter()
                    .map(|(specifier, dependency)| (specifier, logical_module_id(&dependency))),
            )
            .expect("validated graph");
    }
    if let Some(input) = &synthetic {
        graph
            .add_module(
                input.id.clone(),
                input
                    .has_import
                    .then(|| (SYNTHETIC_SPECIFIER.to_owned(), input.documented.clone())),
            )
            .expect("unique synthetic module");
    }
    let mut inputs = project
        .modules()
        .map(|(identity, module)| {
            ProjectModuleInput::new(
                module.source_path().to_string_lossy(),
                logical_module_id(identity),
                module.source(),
                output_path(identity),
            )
            .with_package_scope(logical_package_scope(identity.package()))
        })
        .collect::<Vec<_>>();
    if let Some(input) = synthetic {
        inputs.push(
            ProjectModuleInput::new(
                format!("<{}>", input.id),
                &input.id,
                input.source,
                format!("dist/doc/{}.js", sanitize(&input.id)),
            )
            .with_package_scope(logical_package_scope(project.packages().root())),
        );
    }
    compile_project(graph, inputs)
}

fn public_import(project: &CompiledProject, module: &str) -> Result<String, String> {
    let interface = &project
        .modules
        .get(module)
        .ok_or_else(|| format!("compiled documentation omitted {module}"))?
        .typed_interface;
    let mut names = BTreeSet::new();
    let mut operators = interface
        .operators
        .iter()
        .map(|operator| operator.spelling.clone())
        .collect::<BTreeSet<_>>();
    for export in &interface.exports {
        if export.namespace == "operator" {
            operators.insert(export.name.clone());
        } else {
            names.insert(export.name.clone());
        }
    }
    let mut items = names.into_iter().collect::<Vec<_>>();
    items.extend(
        operators
            .into_iter()
            .map(|operator| format!("operator {operator}")),
    );
    if items.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!(
            "import {{ {} }} from \"{SYNTHETIC_SPECIFIER}\"",
            items.join(", ")
        ))
    }
}

fn synthetic_diagnostics<'a>(
    error: &'a ProjectCompileError,
    module: &str,
) -> Option<&'a DiagnosticArtifact> {
    match error {
        ProjectCompileError::Diagnostics { modules } => modules
            .iter()
            .find(|entry| entry.module == module)
            .map(|entry| &entry.diagnostics),
        ProjectCompileError::Compile {
            module: owner,
            error: LinkedCompileError::Diagnostics(diagnostics),
        } if owner == module => Some(diagnostics),
        _ => None,
    }
}

fn block_compile_failure(
    project: &LoadedLocalDocuments,
    documented: &ModuleIdentity,
    block: &DocumentTestBlock,
    error: ProjectCompileError,
    synthetic: &str,
    prefix: usize,
) -> Result<(), String> {
    match synthetic_diagnostics(&error, synthetic) {
        Some(diagnostics) => Err(render_block_diagnostics(
            project,
            documented,
            block,
            diagnostics,
            prefix,
            None,
        )),
        None => Err(format_project_error(project, error)),
    }
}

fn render_block_diagnostics(
    project: &LoadedLocalDocuments,
    documented: &ModuleIdentity,
    block: &DocumentTestBlock,
    diagnostics: &DiagnosticArtifact,
    prefix: usize,
    expected: Option<&str>,
) -> String {
    let mut rendered = String::new();
    if let Some(code) = expected {
        rendered.push_str(&format!(
            "{}: expected error[{code}] was not produced\n",
            location(project, documented, block, 0)
        ));
    }
    for diagnostic in &diagnostics.diagnostics {
        let offset = diagnostic.primary.start.saturating_sub(prefix);
        rendered.push_str(&format!(
            "{}: {:?}[{}]: {}\n",
            location(project, documented, block, offset),
            diagnostic.severity,
            diagnostic.code,
            diagnostic.message()
        ));
    }
    rendered
}

fn location(
    project: &LoadedLocalDocuments,
    documented: &ModuleIdentity,
    block: &DocumentTestBlock,
    offset: usize,
) -> String {
    let module = project
        .module(documented)
        .expect("documented module exists");
    let byte = block.original_offset(offset).min(module.source().len());
    let before = &module.source()[..byte];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(1, |value| value.chars().count() + 1);
    format!("{}:{line}:{column}", module.source_path().display())
}

fn format_project_error(_project: &LoadedLocalDocuments, error: ProjectCompileError) -> String {
    format!("documentation compiler rejected package: {error:?}")
}

fn output_path(identity: &ModuleIdentity) -> String {
    format!(
        "dist/packages/{}/{}/{}.js",
        identity.package().name().as_str(),
        identity.package().version(),
        identity.path().as_str()
    )
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

struct Invocation {
    path: PathBuf,
    test: bool,
    target: Option<String>,
}

impl Invocation {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut result = Self {
            path: PathBuf::from("."),
            test: false,
            target: None,
        };
        let mut saw_path = false;
        let mut index = 0;
        while index < arguments.len() {
            match arguments[index].as_str() {
                "--test" => {
                    result.test = true;
                    index += 1;
                }
                "--target" => {
                    result.target = Some(
                        arguments
                            .get(index + 1)
                            .ok_or_else(|| "--target requires a value".to_owned())?
                            .clone(),
                    );
                    index += 2;
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown doc option `{value}`"));
                }
                value if !saw_path => {
                    result.path = PathBuf::from(value);
                    saw_path = true;
                    index += 1;
                }
                value => return Err(format!("unexpected doc argument `{value}`")),
            }
        }
        Ok(result)
    }
}
