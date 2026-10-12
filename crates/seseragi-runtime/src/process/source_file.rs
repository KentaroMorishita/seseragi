//! Explicit physical source context for the standalone host adapter.
//! Only statically declared host imports are copied; user directories are never staged.
use seseragi_driver::CompiledModule;
use seseragi_project::{resolve_foreign_typescript_module_for_target, runtime_import_specifiers};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

const HOST_DIRECTORY: &str = ".seseragi-foreign";

pub(super) struct PreparedSourceFile {
    pub compiled: CompiledModule,
    files: BTreeMap<PathBuf, Vec<u8>>,
}
impl PreparedSourceFile {
    pub fn stage(&self, destination: &Path, module_scope: bool) -> Result<(), String> {
        if module_scope {
            fs::write(destination.join("package.json"), b"{\"type\":\"module\"}\n")
                .map_err(|e| format!("failed to stage generated module scope: {e}"))?;
        }
        for (relative, bytes) in &self.files {
            let output = destination.join(HOST_DIRECTORY).join(relative);
            fs::create_dir_all(output.parent().expect("host output has a parent"))
                .map_err(|e| format!("failed to stage foreign host parent: {e}"))?;
            fs::write(output, bytes).map_err(|e| format!("failed to stage foreign host: {e}"))?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct SourceFileContext {
    source_file: PathBuf,
}
impl SourceFileContext {
    pub fn new(source_file: &Path) -> Result<Self, String> {
        let source_file = fs::canonicalize(source_file).map_err(|error| {
            format!("SES-F0203: failed to resolve standalone source file: {error}")
        })?;
        if !source_file.is_file() {
            return Err("SES-F0203: standalone source context requires a file".into());
        }
        Ok(Self { source_file })
    }
    pub(super) fn prepare(
        &self,
        compiled: &CompiledModule,
        browser: bool,
    ) -> Result<PreparedSourceFile, String> {
        let root = self
            .source_file
            .parent()
            .expect("a canonical source file has a parent");
        let target = if browser {
            seseragi_project::RuntimeTarget::Browser
        } else {
            seseragi_project::RuntimeTarget::Node
        };
        let mut pending = Vec::new();
        let mut specifiers = BTreeMap::new();
        for module in &compiled.typescript_ir.foreign_modules {
            let mut executable = !module.members.is_empty() || !module.values.is_empty();
            let mut namespaces = module.namespaces.iter().collect::<Vec<_>>();
            while let Some(namespace) = namespaces.pop() {
                executable |= !namespace.members.is_empty() || !namespace.values.is_empty();
                namespaces.extend(&namespace.namespaces);
            }
            if !executable {
                continue;
            }
            if module.specifier.starts_with('.') {
                let logical = relative(root, &self.source_file, &module.specifier)?;
                specifiers.insert(
                    module.specifier.clone(),
                    format!(
                        "./{HOST_DIRECTORY}/{}",
                        logical.to_string_lossy().replace('\\', "/")
                    ),
                );
            }
            pending.push((
                self.source_file.clone(),
                module.specifier.clone(),
                module.pure_load,
            ));
        }
        let mut files = BTreeMap::<PathBuf, (PathBuf, Vec<u8>, bool)>::new();
        while let Some((importer, specifier, required)) = pending.pop() {
            if specifier.starts_with("node:") {
                resolve_foreign_typescript_module_for_target(
                    root, None, &importer, &specifier, target,
                )
                .map_err(|error| error.to_string())?;
                continue;
            }
            if !specifier.starts_with('.') {
                return Err(format!("SES-F0203: standalone foreign host `{specifier}` requires a package manifest resolver"));
            }
            let logical = relative(root, &importer, &specifier)?;
            let candidate = root.join(&logical);
            // Keep absent task-load hosts at the existing lazy ModuleLoad failure boundary.
            if !candidate.exists() && !required {
                continue;
            }
            let resolved = resolve_foreign_typescript_module_for_target(
                root, None, &importer, &specifier, target,
            )
            .map_err(|error| error.to_string())?;
            if resolved.source() != candidate {
                return Err(format!("SES-F0203: standalone host imports require an explicit file path without symlink aliases: `{specifier}`"));
            }
            if let Some((source, _, previous_required)) = files.get_mut(&logical) {
                if source != resolved.source() {
                    return Err("SES-F0203: conflicting standalone host identities".into());
                }
                if *previous_required || !required {
                    continue;
                }
                *previous_required = true;
            }
            let bytes = fs::read(resolved.source())
                .map_err(|e| format!("SES-F0203: failed to read foreign host: {e}"))?;
            let imports = match resolved
                .source()
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("json") => {
                    serde_json::from_slice::<serde_json::Value>(&bytes)
                        .map_err(|e| format!("SES-F0203: invalid host JSON resource: {e}"))?;
                    Vec::new()
                }
                Some("js" | "mjs" | "cjs" | "ts" | "mts" | "cts") => {
                    let text = std::str::from_utf8(&bytes)
                        .map_err(|e| format!("SES-F0203: foreign host is not UTF-8: {e}"))?;
                    runtime_import_specifiers(text).map_err(|error| error.to_string())?
                }
                _ => {
                    return Err(format!(
                        "SES-F0203: unsupported standalone host resource {}",
                        logical.display()
                    ))
                }
            };
            files.insert(logical, (resolved.source().to_owned(), bytes, required));
            pending.extend(
                imports
                    .into_iter()
                    .map(|specifier| (resolved.source().to_owned(), specifier, required)),
            );
        }
        let mut files = files
            .into_iter()
            .map(|(path, (_, bytes, _))| (path, bytes))
            .collect::<BTreeMap<_, _>>();
        // The closest package scope controls .js / .ts evaluation. Keep only
        // the type discriminator unless that manifest was explicitly imported.
        let hosts = files.keys().cloned().collect::<Vec<_>>();
        for host in hosts {
            let (manifest, bytes) = package_scope(root, &host)?;
            files.entry(manifest).or_insert(bytes);
        }
        if !files.is_empty() {
            files
                .entry(PathBuf::from("package.json"))
                .or_insert_with(|| b"{}\n".to_vec());
        }
        Ok(PreparedSourceFile {
            compiled: seseragi_driver::relocate_foreign_outputs(compiled, &specifiers),
            files,
        })
    }
}
fn relative(root: &Path, importer: &Path, specifier: &str) -> Result<PathBuf, String> {
    let parent = importer
        .parent()
        .and_then(|path| path.strip_prefix(root).ok())
        .ok_or("SES-F0203: foreign importer is outside standalone source root")?;
    let mut relative = parent.to_owned();
    for component in Path::new(specifier).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => relative.push(value),
            Component::ParentDir if relative.pop() => {}
            _ => return Err("SES-F0203: standalone foreign imports must stay inside the source file's parent directory".into()),
        }
    }
    Ok(relative)
}

fn package_scope(root: &Path, host: &Path) -> Result<(PathBuf, Vec<u8>), String> {
    let mut scope = host.parent();
    while let Some(parent) = scope {
        let manifest = parent.join("package.json");
        let source = root.join(&manifest);
        if source.is_file() {
            return Ok((manifest, scope_bytes(&source)?));
        }
        scope = parent.parent();
    }
    for parent in root.ancestors().skip(1) {
        let source = parent.join("package.json");
        if source.is_file() {
            return Ok((PathBuf::from("package.json"), scope_bytes(&source)?));
        }
    }
    Ok((PathBuf::from("package.json"), b"{}\n".to_vec()))
}
fn scope_bytes(source: &Path) -> Result<Vec<u8>, String> {
    let metadata: serde_json::Value = serde_json::from_slice(
        &fs::read(source)
            .map_err(|e| format!("SES-F0203: failed to read host package scope: {e}"))?,
    )
    .map_err(|e| format!("SES-F0203: invalid host package scope: {e}"))?;
    let minimal = metadata.get("type").map_or_else(
        || serde_json::json!({}),
        |kind| serde_json::json!({"type": kind}),
    );
    serde_json::to_vec(&minimal)
        .map_err(|e| format!("SES-F0203: failed to encode host package scope: {e}"))
}
