use seseragi_driver::{compile_core_module, CompileInput};
use std::{fs, path::PathBuf, process::Command};
pub(crate) mod output;
pub(crate) mod program_wasm;

pub(crate) fn run(arguments: &[String]) -> Result<i32, String> {
    if let [operation, arguments @ ..] = arguments {
        if operation == "build-wasm" {
            return program_wasm::run(arguments);
        }
    }
    let [operation, path, flags @ ..] = arguments else {
        return Err(usage().into());
    };
    let native = match operation.as_str() {
        "emit-rust" => false,
        "build-native" => true,
        _ => return Err(usage().into()),
    };
    let mut entry = None;
    let mut destination = None;
    let mut provider = None;
    for pair in flags.chunks(2) {
        let [flag, value] = pair else {
            return Err(usage().into());
        };
        match flag.as_str() {
            "--entry" if entry.is_none() => entry = Some(value.as_str()),
            "--provider" if provider.is_none() => provider = Some(value.as_str()),
            "--output" if !native && destination.is_none() => {
                destination = Some(PathBuf::from(value))
            }
            "--out-dir" if native && destination.is_none() => {
                destination = Some(PathBuf::from(value))
            }
            _ => return Err(usage().into()),
        }
    }
    let entry = entry.ok_or_else(|| usage().to_owned())?;
    let destination = destination.ok_or_else(|| usage().to_owned())?;
    let (artifact, source_paths) = compile_native(
        path,
        entry,
        provider,
        seseragi_project::ProjectCommand::Build,
        seseragi_project::BuildProfile::Development,
    )?;
    publish_artifact(&artifact, &source_paths, &destination, native)?;
    if native {
        println!("{}", destination.join(output::EXECUTABLE).display());
    } else {
        println!("{}", destination.display());
    }
    Ok(0)
}

pub(crate) fn build_native(
    path: &std::path::Path,
    destination: &std::path::Path,
    command: seseragi_project::ProjectCommand,
    profile: seseragi_project::BuildProfile,
) -> Result<(), String> {
    let package = crate::local_project::containing_package(path);
    let path = package.as_deref().unwrap_or(path);
    let path = path.to_str().ok_or("native source path must be UTF-8")?;
    let (artifact, source_paths) = compile_native(path, "main", None, command, profile)?;
    publish_artifact(&artifact, &source_paths, destination, true)
}

fn compile_native(
    path: &str,
    entry: &str,
    provider: Option<&str>,
    command: seseragi_project::ProjectCommand,
    profile: seseragi_project::BuildProfile,
) -> Result<(seseragi_backend_rust::RustArtifact, Vec<PathBuf>), String> {
    if std::path::Path::new(path).is_dir() {
        compile_project(path, entry, provider, command, profile)
    } else {
        if !path.ends_with(".ssrg") || !std::path::Path::new(path).is_file() {
            return Err("experimental Rust target accepts a .ssrg file or package root".into());
        }
        let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let checked = compile_core_module(
            CompileInput::new(path, "experimental/main", &source).with_profile(profile),
        )
        .map_err(|diagnostics| {
            seseragi_driver::render_terminal_diagnostics(&diagnostics, &source)
        })?;
        if !checked.diagnostics.diagnostics.is_empty() {
            eprint!(
                "{}",
                seseragi_driver::render_terminal_diagnostics(&checked.diagnostics, &source)
            );
        }
        Ok((
            seseragi_backend_rust::emit_rust_module_with_options(
                &checked.core_ir,
                entry,
                &native_clock_options(provider)?,
            )
            .map_err(|e| target_error(e, Some(path)))?,
            vec![fs::canonicalize(path).map_err(|e| e.to_string())?],
        ))
    }
}

fn publish_artifact(
    artifact: &seseragi_backend_rust::RustArtifact,
    source_paths: &[PathBuf],
    destination: &std::path::Path,
    native: bool,
) -> Result<(), String> {
    // Target admission precedes filesystem changes. Generate and compile only
    // inside owned staging, then replace the checked output leaves together.
    let selected = if native {
        output::Destination::Native(destination)
    } else {
        output::Destination::Rust(destination)
    };
    output::publish(selected, source_paths, |staged| {
        fs::write(&staged.rust, &artifact.source).map_err(|e| e.to_string())?;
        fs::write(
            &staged.metadata,
            serde_json::to_string_pretty(&artifact.metadata).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if let Some(executable) = &staged.executable {
            let output = Command::new("rustc")
                .args(["--edition=2021", "--crate-name", "seseragi_program", "-O"])
                .arg(&staged.rust)
                .arg("-o")
                .arg(executable)
                .output()
                .map_err(|e| format!("could not start rustc: {e}; install a Rust toolchain"))?;
            if !output.status.success() {
                return Err(format!(
                    "Rust target compilation failed:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn target_error(
    error: seseragi_backend_rust::RustBackendError,
    source_path: Option<&str>,
) -> String {
    match &error.origin {
        Some(origin) => format!(
            "{}:{}..{}: {error}",
            source_path.unwrap_or(&origin.source),
            origin.start,
            origin.end
        ),
        None => error.to_string(),
    }
}

fn compile_project(
    path: &str,
    entry: &str,
    provider: Option<&str>,
    command: seseragi_project::ProjectCommand,
    profile: seseragi_project::BuildProfile,
) -> Result<(seseragi_backend_rust::RustArtifact, Vec<PathBuf>), String> {
    let (project, checked) = load_checked_project_entry_with_profile(path, None, command, profile)?
        .ok_or("project compiler did not produce a checked entry")?;
    let modules = checked
        .compiled
        .order
        .iter()
        .map(|id| &checked.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let root = project
        .packages()
        .package(project.packages().root())
        .expect("checked root package");
    let manifest_path = root.root().join("seseragi.toml");
    let options = native_clock_options(provider)?;
    let artifact = seseragi_backend_rust::emit_rust_project_with_options(
        &modules,
        &checked.entry_module,
        entry,
        &options,
    )
    .map_err(|error| project_target_error(error, &project))?;
    if artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "clock-sleep")
    {
        reject_clock_manifest_pin(
            root.manifest()
                .providers
                .get("std/clock::Clock")
                .map(String::as_str),
            &manifest_path,
        )?;
        for (identity, package) in project.packages().packages() {
            if !package.manifest().provider_artifacts.is_empty()
                || (identity != project.packages().root()
                    && package
                        .manifest()
                        .providers
                        .contains_key("std/clock::Clock"))
            {
                return Err(format!("{}: SES-RUST-0001: native Clock sleep requires the explicit native CLI selection; provider artifacts and dependency provider selections are not admitted", package.root().join("seseragi.toml").display()));
            }
        }
    }
    if artifact
        .metadata
        .capabilities
        .iter()
        .any(|capability| capability == "filesystem-async")
    {
        for (_, package) in project.packages().packages() {
            if package
                .manifest()
                .providers
                .contains_key("std/fs::FileSystem")
                || !package.manifest().provider_artifacts.is_empty()
            {
                return Err(format!("{}: SES-RUST-0001: unsupported native filesystem provider selection/artifact configuration; this target currently uses its built-in filesystem host", package.root().join("seseragi.toml").display()));
            }
        }
    }
    let source_paths = project
        .modules()
        .map(|(_, module)| fs::canonicalize(module.source_path()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    Ok((artifact, source_paths))
}

fn native_clock_options(
    command_line: Option<&str>,
) -> Result<seseragi_backend_rust::RustTargetOptions, String> {
    use seseragi_backend_rust::{
        NativeClockProvider, NativeProviderSelectionOrigin, RustTargetOptions,
    };
    let selected = command_line.map(|value| {
        let Some((service, provider)) = value.split_once('=') else {
            return Err("SES-RUST-0001: --provider requires std/clock::Clock=provider-identity".to_owned());
        };
        if service != "std/clock::Clock" {
            return Err(format!("SES-RUST-0001: unsupported native provider service {service}; this finite selection supports Clock sleep only"));
        }
        Ok(provider)
    }).transpose()?;
    let Some(identity) = selected else {
        return Ok(RustTargetOptions::default());
    };
    let provider =
        NativeClockProvider::from_identity(identity).map_err(|error| error.to_string())?;
    Ok(RustTargetOptions {
        clock_provider: Some(provider),
        clock_selection_origin: NativeProviderSelectionOrigin::CommandLine,
    })
}

fn reject_clock_manifest_pin(manifest: Option<&str>, path: &std::path::Path) -> Result<(), String> {
    if let Some(identity) = manifest {
        return Err(format!("{}: SES-RUST-0001: native Clock sleep does not admit the target-shared manifest provider selection {identity}; use the explicit native --provider selection without a Clock manifest pin; JavaScript providers are not substituted", path.display()));
    }
    Ok(())
}

fn project_target_error(
    error: seseragi_backend_rust::RustBackendError,
    project: &seseragi_project::LoadedLocalProject,
) -> String {
    let source = error.module.as_deref().and_then(|id| {
        project
            .modules()
            .find(|(identity, _)| seseragi_project::logical_module_id(identity) == id)
            .map(|(_, module)| module.source_path().to_string_lossy())
    });
    target_error(error, source.as_deref())
}

fn load_checked_project_entry_for_command(
    path: &str,
    module: Option<&seseragi_project::ModulePath>,
    command: seseragi_project::ProjectCommand,
) -> Result<
    Option<(
        seseragi_project::LoadedLocalProject,
        seseragi_driver::CoreLocalProject,
    )>,
    String,
> {
    load_checked_project_entry_with_profile(
        path,
        module,
        command,
        seseragi_project::BuildProfile::Development,
    )
}

fn load_checked_project_entry_with_profile(
    path: &str,
    module: Option<&seseragi_project::ModulePath>,
    command: seseragi_project::ProjectCommand,
    profile: seseragi_project::BuildProfile,
) -> Result<
    Option<(
        seseragi_project::LoadedLocalProject,
        seseragi_driver::CoreLocalProject,
    )>,
    String,
> {
    if command == seseragi_project::ProjectCommand::Dev {
        seseragi_project::read_and_validate_development_lockfile(path)
    } else {
        seseragi_project::read_and_validate_lockfile(path)
    }
    .map_err(|error| format!("{}: {error}", error.code()))?;
    let project = match module {
        Some(module) => seseragi_project::load_local_project_module(path, module),
        None => seseragi_project::load_local_project(path),
    }
    .map_err(|error| format!("{}: {error}", error.code()))?;
    let result = seseragi_driver::compile_core_local_project_with_profile(&project, profile);
    let checked = if command == seseragi_project::ProjectCommand::Dev {
        let Some(checked) = crate::local_project::render_compile_result(
            &project,
            result,
            seseragi_runtime::DiagnosticFormat::Text,
        )?
        else {
            return Ok(None);
        };
        checked
    } else {
        result.map_err(|error| {
            if let seseragi_driver::ProjectCompileError::Diagnostics { modules } = error.error() {
                modules
                    .iter()
                    .map(|diagnostics| {
                        let source = project
                            .modules()
                            .find(|(identity, _)| {
                                seseragi_project::logical_module_id(identity) == diagnostics.module
                            })
                            .expect("diagnostics belong to a loaded module")
                            .1
                            .source();
                        seseragi_driver::render_terminal_diagnostics(
                            &diagnostics.diagnostics,
                            source,
                        )
                    })
                    .collect::<String>()
            } else {
                format!("project compiler rejected package: {:?}", error.error())
            }
        })?
    };
    for (id, module) in &checked.compiled.modules {
        if !module.diagnostics.diagnostics.is_empty() {
            let source = project
                .modules()
                .find(|(identity, _)| seseragi_project::logical_module_id(identity) == *id)
                .expect("checked module was loaded")
                .1
                .source();
            eprint!(
                "{}",
                seseragi_driver::render_terminal_diagnostics(&module.diagnostics, source)
            );
        }
    }
    Ok(Some((project, checked)))
}

fn usage() -> &'static str {
    "usage: seseragi experimental emit-rust source.ssrg|package-root --entry function --output program.rs [--provider std/clock::Clock=seseragi/runtime-native#clock-sleep] | build-native source.ssrg|package-root --entry function --out-dir path [--provider std/clock::Clock=seseragi/runtime-native#clock-sleep]"
}
