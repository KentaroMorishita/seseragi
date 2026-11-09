use seseragi_backend_rust::{
    ProgramWasmArtifact, PROGRAM_WASM_ABI, PROGRAM_WASM_EFFECT_ABI, PROGRAM_WASM_HOST_EFFECT_ABI,
};
use seseragi_driver::{compile_core_module, CompileInput};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const FILES: [&str; 4] = ["program.rs", "program.wasm", "program.mjs", "program.d.mts"];
static SERIAL: AtomicUsize = AtomicUsize::new(0);
const USAGE: &str = "usage: seseragi experimental build-wasm source.ssrg|package-root --export function|function<Type>|name=function<Type> [--export other] [--profile development|release] --out-dir path";

pub(crate) struct OwnedDirectory(pub(crate) PathBuf, bool);
impl OwnedDirectory {
    pub(crate) fn create(parent: &Path) -> Result<Self, String> {
        for _ in 0..1000 {
            let path = parent.join(format!(
                ".seseragi-program-wasm-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path, true)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("could not reserve program WASM staging directory".into())
    }
}
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if self.1 && self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The selected exports. Each `function<Type>` export becomes a checked
/// binding added after the entry module, `let __seseragi_export_N =
/// function<Type>`, so the type checker selects its trait instances exactly
/// as for a call in the source. An instance that does not exist is the
/// checker's ordinary diagnostic on that line.
struct Selection {
    names: Vec<String>,
    /// The `function<Type>` exports, with the text each was written as.
    instantiated: Vec<(String, seseragi_project::ProgramExport)>,
}

/// The source added after the entry module and the bindings it defines.
struct Bindings {
    extension: String,
    instantiations: Vec<seseragi_backend_rust::ProgramWasmInstantiation>,
}

impl Selection {
    /// Binds each instantiated export after `entry`, the entry module's
    /// source. A binding takes the next `__seseragi_export_N` that appears
    /// nowhere in that source, so it cannot collide with a definition or an
    /// import of the entry module.
    fn bind(&self, entry: &str) -> Bindings {
        let mut bindings = Bindings {
            extension: String::new(),
            instantiations: Vec::new(),
        };
        let mut next = 0usize;
        for (text, export) in &self.instantiated {
            let binding = loop {
                let candidate = format!("__seseragi_export_{next}");
                next += 1;
                if !entry.contains(&candidate) && !bindings.extension.contains(&candidate) {
                    break candidate;
                }
            };
            let arguments = export
                .type_arguments
                .as_deref()
                .expect("an instantiated export has type arguments");
            bindings.extension.push_str(&format!(
                "\n// program WASM export `{text}`\nlet {binding} = {}<{arguments}>\n",
                export.function
            ));
            bindings
                .instantiations
                .push(seseragi_backend_rust::ProgramWasmInstantiation {
                    name: export.name.clone(),
                    binding,
                    function: export.function.clone(),
                });
        }
        bindings
    }
}

fn selection(exports: &[String]) -> Result<Selection, String> {
    let mut selection = Selection {
        names: Vec::new(),
        instantiated: Vec::new(),
    };
    for text in exports {
        let export = seseragi_project::ProgramExport::parse(text)
            .map_err(|reason| format!("SES-RUST-0001: --export {reason}"))?;
        if selection.names.contains(&export.name) {
            return Err(format!(
                "SES-RUST-0001: --export `{}` repeats the export name `{}`; name each instantiation as name=function<Type>",
                text.trim(),
                export.name
            ));
        }
        selection.names.push(export.name.clone());
        if export.type_arguments.is_some() {
            selection
                .instantiated
                .push((text.trim().to_owned(), export));
        }
    }
    Ok(selection)
}

/// The program passes values in the shape of the TS target built with the
/// same profile; the release profile erases newtype wrappers.
fn options(profile: seseragi_project::BuildProfile) -> seseragi_backend_rust::ProgramWasmOptions {
    seseragi_backend_rust::ProgramWasmOptions {
        erase_newtypes: profile == seseragi_project::BuildProfile::Release,
        instantiations: Vec::new(),
    }
}

fn compile(
    path: &str,
    exports: &[String],
    options: seseragi_backend_rust::ProgramWasmOptions,
) -> Result<(ProgramWasmArtifact, Vec<(PathBuf, Vec<u8>)>), String> {
    if Path::new(path).is_dir() {
        compile_package(
            path,
            exports,
            None,
            seseragi_project::ProjectCommand::Build,
            options,
        )?
        .ok_or_else(|| "program WASM compiler did not produce a checked output".to_owned())
    } else {
        compile_file(path, exports, options)
    }
}

fn compile_package(
    path: &str,
    exports: &[String],
    module: Option<&seseragi_project::ModulePath>,
    command: seseragi_project::ProjectCommand,
    options: seseragi_backend_rust::ProgramWasmOptions,
) -> Result<Option<(ProgramWasmArtifact, Vec<(PathBuf, Vec<u8>)>)>, String> {
    let selection = selection(exports)?;
    let bound = std::cell::RefCell::new(None);
    let bind = |entry: &str| {
        let bindings = selection.bind(entry);
        let extension = bindings.extension.clone();
        *bound.borrow_mut() = Some(bindings.instantiations);
        extension
    };
    let Some((project, checked)) =
        super::load_checked_project_entry_for_command(path, module, command, &bind)?
    else {
        return Ok(None);
    };
    let options = seseragi_backend_rust::ProgramWasmOptions {
        instantiations: bound.into_inner().unwrap_or_default(),
        ..options
    };
    let exports = &selection.names;
    for (_, package) in project.packages().packages() {
        if module.is_none()
            && (!package.manifest().providers.is_empty()
                || !package.manifest().provider_artifacts.is_empty())
        {
            return Err(format!(
                "{}: SES-RUST-0001: this program WASM ABI does not accept provider configuration",
                package.root().join("seseragi.toml").display()
            ));
        }
    }
    let modules = checked
        .compiled
        .order
        .iter()
        .map(|id| &checked.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let source_files = project
        .modules()
        .map(|(identity, module)| {
            (
                seseragi_project::logical_module_id(identity),
                seseragi_backend_rust::RustSourceFile {
                    path: module.runtime_source_path(
                        project
                            .packages()
                            .package(identity.package())
                            .expect("checked source package")
                            .root(),
                    ),
                    text: module.source().to_owned(),
                },
            )
        })
        .collect();
    let artifact = seseragi_backend_rust::emit_program_wasm_with_options(
        &modules,
        &checked.entry_module,
        exports,
        source_files,
        options,
    )
    .map_err(|error| super::project_target_error(error, &project))?;
    let sources = project
        .modules()
        .map(|(_, module)| {
            // The receipt records the files on disk, not the checked
            // export bindings added after the entry module.
            fs::canonicalize(module.source_path())
                .and_then(|path| fs::read(&path).map(|bytes| (path, bytes)))
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut sources = sources;
    for (_, package) in project.packages().packages() {
        let path = package.root().join("seseragi.toml");
        sources.push((path.clone(), fs::read(path).map_err(|e| e.to_string())?));
    }
    Ok(Some((artifact, sources)))
}

fn compile_file(
    path: &str,
    exports: &[String],
    options: seseragi_backend_rust::ProgramWasmOptions,
) -> Result<(ProgramWasmArtifact, Vec<(PathBuf, Vec<u8>)>), String> {
    if !path.ends_with(".ssrg") || !Path::new(path).is_file() {
        return Err("program WASM accepts a .ssrg source or package root".into());
    }
    let original = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let selection = selection(exports)?;
    let bindings = selection.bind(&original);
    let options = seseragi_backend_rust::ProgramWasmOptions {
        instantiations: bindings.instantiations,
        ..options
    };
    let exports = &selection.names;
    let source = format!("{original}{}", bindings.extension);
    let checked = compile_core_module(CompileInput::new(path, "experimental/main", &source))
        .map_err(|diagnostics| {
            seseragi_driver::render_terminal_diagnostics(&diagnostics, &source)
        })?;
    if !checked.diagnostics.diagnostics.is_empty() {
        eprint!(
            "{}",
            seseragi_driver::render_terminal_diagnostics(&checked.diagnostics, &source)
        );
    }
    let artifact = seseragi_backend_rust::emit_program_wasm_with_options(
        &[&checked.core_ir],
        "experimental/main",
        exports,
        [(
            "experimental/main".to_owned(),
            seseragi_backend_rust::RustSourceFile {
                path: seseragi_project::standalone_runtime_source_path(path),
                text: source.clone(),
            },
        )]
        .into(),
        options,
    )
    .map_err(|e| super::target_error(e, Some(path)))?;
    Ok((
        artifact,
        vec![(
            fs::canonicalize(path).map_err(|e| e.to_string())?,
            original.into_bytes(),
        )],
    ))
}

fn owned_output(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("program WASM output must be an owned directory, not a file or symlink".into());
    }
    let entries = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if entries.is_empty() {
        return Ok(());
    }
    if entries.len() != FILES.len() + 1
        || entries.iter().any(|entry| {
            let name = entry.file_name();
            !FILES.iter().any(|expected| name == *expected) && name != "program-wasm.json"
                || entry.file_type().map_or(true, |kind| !kind.is_file())
        })
    {
        return Err("refusing to replace unowned program WASM output contents".into());
    }
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(path.join("program-wasm.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if (manifest["abi"] != PROGRAM_WASM_ABI
        && manifest["abi"] != PROGRAM_WASM_EFFECT_ABI
        && manifest["abi"] != PROGRAM_WASM_HOST_EFFECT_ABI)
        || manifest["schema"] != 1
    {
        return Err("refusing to replace an unknown program WASM artifact".into());
    }
    let files = manifest["files"]
        .as_array()
        .filter(|v| v.len() == FILES.len())
        .ok_or("unproven program WASM output file ownership")?;
    for name in FILES {
        let file = files
            .iter()
            .find(|file| file["path"] == name)
            .ok_or("unproven program WASM output file ownership")?;
        if file["sha256"] != digest(&fs::read(path.join(name)).map_err(|e| e.to_string())?) {
            return Err(format!(
                "refusing to replace modified program WASM output: {name}"
            ));
        }
    }
    Ok(())
}

fn write_artifact(
    artifact: &ProgramWasmArtifact,
    sources: &[(PathBuf, Vec<u8>)],
    directory: &Path,
) -> Result<(), String> {
    for (name, content) in [
        ("program.rs", artifact.source.as_str()),
        ("program.mjs", artifact.javascript.as_str()),
        ("program.d.mts", artifact.declarations.as_str()),
    ] {
        fs::write(directory.join(name), content).map_err(|e| e.to_string())?;
    }
    let version = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| {
            format!(
                "could not start rustc: {e}; install a Rust toolchain with wasm32-unknown-unknown"
            )
        })?;
    if !version.status.success() {
        return Err("could not measure the program Rust compiler".into());
    }
    let options = [
        "--edition=2021",
        "--crate-name",
        "seseragi_program",
        "--target",
        "wasm32-unknown-unknown",
        "--crate-type",
        "cdylib",
        "-O",
        "-C",
        "panic=abort",
        "-C",
        "strip=symbols",
    ];
    let result = Command::new("rustc")
        .args(options)
        .arg(directory.join("program.rs"))
        .arg("-o")
        .arg(directory.join("program.wasm"))
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(format!(
            "program WASM compilation failed (requires wasm32-unknown-unknown):\n{}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let mut metadata = serde_json::to_value(&artifact.metadata).map_err(|e| e.to_string())?;
    metadata["compilerVersion"] = env!("CARGO_PKG_VERSION").into();
    metadata["rustc"] = serde_json::json!({"version":String::from_utf8_lossy(&version.stdout).trim(),"target":"wasm32-unknown-unknown","flags":options});
    metadata["files"] = serde_json::Value::Array(
        FILES
            .iter()
            .map(|name| {
                let bytes = fs::read(directory.join(name)).map_err(|e| e.to_string())?;
                Ok(serde_json::json!({"path":name,"bytes":bytes.len(),"sha256":digest(&bytes)}))
            })
            .collect::<Result<Vec<_>, String>>()?,
    );
    metadata["sources"] = serde_json::Value::Array(
        sources
            .iter()
            .map(|(path, bytes)| {
                if fs::read(path).map_err(|e| e.to_string())? != *bytes {
                    return Err(format!(
                        "source changed during program WASM build: {}",
                        path.display()
                    ));
                }
                Ok(serde_json::json!({"path":path,"sha256":digest(bytes)}))
            })
            .collect::<Result<Vec<_>, String>>()?,
    );
    fs::write(
        directory.join("program-wasm.json"),
        serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn package_assets(
    root: &Path,
    programs: &std::collections::BTreeMap<String, seseragi_project::ManifestProgramWasm>,
    command: seseragi_project::ProjectCommand,
    profile: seseragi_project::BuildProfile,
) -> Result<Option<Vec<seseragi_runtime::ProgramWasmAssets>>, String> {
    // Check every selected import closure before starting rustc or producing files.
    let mut checked = Vec::new();
    for (name, config) in programs {
        let Some((artifact, sources)) = compile_package(
            &root.to_string_lossy(),
            &config.exports,
            Some(&config.module),
            command,
            options(profile),
        )
        .map_err(|error| {
            format!(
                "{}: web.programs.{name}: {error}",
                root.join("seseragi.toml").display()
            )
        })?
        else {
            return Ok(None);
        };
        checked.push((name.clone(), artifact, sources));
    }
    let mut result = Vec::new();
    for (name, artifact, sources) in checked {
        let temporary = OwnedDirectory::create(&std::env::temp_dir())?;
        write_artifact(&artifact, &sources, &temporary.0)?;
        let files = FILES
            .into_iter()
            .chain(["program-wasm.json"])
            .map(|name| {
                fs::read(temporary.0.join(name))
                    .map(|bytes| (name.to_owned(), bytes))
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<_, _>>()?;
        result.push(seseragi_runtime::ProgramWasmAssets {
            name,
            files,
            inputs: sources,
        });
    }
    Ok(Some(result))
}

pub(super) fn run(arguments: &[String]) -> Result<i32, String> {
    let [path, flags @ ..] = arguments else {
        return Err(USAGE.into());
    };
    let mut exports = Vec::new();
    let mut destination = None;
    let mut profile = None;
    for pair in flags.chunks(2) {
        let [flag, value] = pair else {
            return Err(USAGE.into());
        };
        match flag.as_str() {
            "--export" => exports.push(value.clone()),
            "--out-dir" if destination.is_none() => destination = Some(PathBuf::from(value)),
            "--profile" if profile.is_none() => {
                profile = Some(seseragi_project::BuildProfile::parse(value)?)
            }
            _ => return Err(USAGE.into()),
        }
    }
    let destination = destination.ok_or(USAGE)?;
    // Entire checked target admission precedes any generated output or rustc.
    let (artifact, sources) = compile(path, &exports, options(profile.unwrap_or_default()))?;
    owned_output(&destination)?;
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = destination
        .file_name()
        .ok_or("invalid program WASM output path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    let destination = parent.join(name);
    let stage = OwnedDirectory::create(&parent)?;
    write_artifact(&artifact, &sources, &stage.0)?;
    // Stage compilation failures preserve the preceding complete artifact.
    owned_output(&destination)?;
    let mut backup = OwnedDirectory::create(&parent)?;
    fs::remove_dir(&backup.0).map_err(|e| e.to_string())?;
    let previous = destination.exists();
    if previous {
        fs::rename(&destination, &backup.0).map_err(|e| e.to_string())?;
    }
    if let Err(error) = fs::rename(&stage.0, &destination) {
        if previous {
            if let Err(rollback) = fs::rename(&backup.0, &destination) {
                backup.1 = false;
                return Err(format!("{error}; output rollback failed: {rollback}; preceding artifact preserved at {}", backup.0.display()));
            }
        }
        return Err(error.to_string());
    }
    if previous {
        if let Err(error) = fs::remove_dir_all(&backup.0) {
            backup.1 = false;
            return Err(format!(
                "artifact published; old owned output cleanup failed at {}: {error}",
                backup.0.display()
            ));
        }
    }
    println!("{}", destination.join("program.wasm").display());
    Ok(0)
}
