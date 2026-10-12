//! Browser snapshot adapter for the same source-neutral package export policy.
use crate::{RuntimeLinkageDiagnostic, RuntimeTarget};
use serde::{Deserialize, Serialize};
use seseragi_interop::select_package_export;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VirtualRuntimeImport {
    pub importer: String,
    pub specifier: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VirtualRuntimeModule {
    pub path: String,
    pub source: String,
    pub exact_identity: String,
    pub content_digest: String,
    pub imports: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VirtualRuntimeBundle {
    /// Digest of the complete resolved import closure, including dependency bytes.
    pub snapshot_digest: String,
    pub modules: BTreeMap<String, VirtualRuntimeModule>,
    /// Keys are generated importer paths; values map specifiers to snapshot paths.
    pub roots: BTreeMap<String, BTreeMap<String, String>>,
}
fn problem(reason: impl Into<String>) -> RuntimeLinkageDiagnostic {
    RuntimeLinkageDiagnostic::new("SES-F0203", Some(RuntimeTarget::Browser), reason)
}
fn normalized(path: &str) -> Result<String, RuntimeLinkageDiagnostic> {
    if path.starts_with('/') || path.contains(['\\', '\0', '?', '#', '%']) {
        return Err(problem(
            "runtime path must be a portable snapshot-relative path",
        ));
    }
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(problem("runtime import escapes snapshot root"));
                }
            }
            part => parts.push(part),
        }
    }
    if parts.is_empty() {
        return Err(problem("runtime path is empty"));
    }
    Ok(parts.join("/"))
}
fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(prefix, _)| prefix)
}
fn joined(base: &str, path: &str) -> Result<String, RuntimeLinkageDiagnostic> {
    if base.is_empty() {
        normalized(path)
    } else {
        normalized(&format!("{base}/{path}"))
    }
}
fn file(files: &BTreeMap<String, String>, path: &str) -> Option<String> {
    if files.contains_key(path) {
        return Some(path.to_owned());
    }
    if path
        .rsplit('/')
        .next()
        .is_some_and(|name| !name.contains('.'))
    {
        for ext in ["mjs", "js", "cjs", "mts", "ts", "cts"] {
            let candidate = format!("{path}.{ext}");
            if files.contains_key(&candidate) {
                return Some(candidate);
            }
        }
        for ext in ["mjs", "js", "cjs", "mts", "ts", "cts"] {
            let candidate = format!("{path}/index.{ext}");
            if files.contains_key(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}
struct Resolver<'a> {
    files: &'a BTreeMap<String, String>,
    host_root: &'a str,
}
impl Resolver<'_> {
    fn resolve(
        &self,
        importer: &str,
        specifier: &str,
    ) -> Result<(String, String), RuntimeLinkageDiagnostic> {
        if specifier.starts_with("node:") || specifier.starts_with("bun:") {
            return Err(problem(format!(
                "runtime `{specifier}` is unavailable in browser"
            )));
        }
        if specifier.starts_with('.') {
            let candidate = joined(parent(importer), specifier)?;
            let path = file(self.files, &candidate).ok_or_else(|| {
                problem(format!("runtime snapshot module `{candidate}` is missing"))
            })?;
            return Ok((path.clone(), format!("workspace:{path}")));
        }
        let parts = specifier.split('/').collect::<Vec<_>>();
        let count = if specifier.starts_with('@') { 2 } else { 1 };
        if parts.len() < count
            || parts
                .iter()
                .any(|p| p.is_empty() || *p == "." || *p == ".." || p.contains(['\\', ':', '%']))
        {
            return Err(problem("invalid runtime package specifier"));
        }
        let name = parts[..count].join("/");
        let subpath = (parts.len() > count).then(|| parts[count..].join("/"));
        let mut ancestors = parent(importer).to_owned();
        let mut candidates = Vec::new();
        loop {
            candidates.push(if ancestors.is_empty() {
                format!("node_modules/{name}")
            } else {
                format!("{ancestors}/node_modules/{name}")
            });
            if ancestors.is_empty() {
                break;
            }
            ancestors = parent(&ancestors).to_owned();
        }
        if !self.host_root.is_empty() {
            candidates.push(format!("{}/node_modules/{name}", self.host_root));
        }
        let root = candidates
            .into_iter()
            .find(|root| self.files.contains_key(&format!("{root}/package.json")))
            .ok_or_else(|| {
                problem(format!(
                    "runtime package `{name}` is absent from the workspace snapshot"
                ))
            })?;
        let manifest = &self.files[&format!("{root}/package.json")];
        let selected =
            select_package_export(manifest, &name, subpath.as_deref(), RuntimeTarget::Browser)?;
        let candidate = joined(&root, &selected.path)?;
        let path = file(self.files, &candidate).ok_or_else(|| {
            problem(format!(
                "browser package export `{candidate}` is absent from the snapshot"
            ))
        })?;
        if !path.starts_with(&format!("{root}/")) {
            return Err(problem("runtime package export escapes package root"));
        }
        let relative = path.strip_prefix(&format!("{root}/")).unwrap();
        Ok((
            path.clone(),
            format!("npm:{name}@{}/{relative}", selected.version),
        ))
    }
}
/// All imports are parsed and resolved before any module executes. Snapshot
/// bytes supply SHA-256 identities; registry access and remote fetch are absent.
pub fn resolve_browser_runtime_snapshot(
    files: &BTreeMap<String, String>,
    imports: &[VirtualRuntimeImport],
    host_root: &str,
) -> Result<VirtualRuntimeBundle, Vec<RuntimeLinkageDiagnostic>> {
    let failure = |error| vec![error];
    for path in files.keys() {
        if normalized(path).map_err(failure)? != *path {
            return Err(vec![problem(format!(
                "noncanonical or aliased runtime snapshot path `{path}`"
            ))]);
        }
    }
    let host_root = if host_root.is_empty() {
        String::new()
    } else {
        normalized(host_root).map_err(failure)?
    };
    let resolver = Resolver {
        files,
        host_root: &host_root,
    };
    let mut bundle = VirtualRuntimeBundle {
        snapshot_digest: String::new(),
        modules: BTreeMap::new(),
        roots: BTreeMap::new(),
    };
    let mut pending = Vec::new();
    for import in imports {
        let importer = normalized(&import.importer).map_err(failure)?;
        let (path, identity) = resolver
            .resolve(&importer, &import.specifier)
            .map_err(failure)?;
        bundle
            .roots
            .entry(import.importer.clone())
            .or_default()
            .insert(import.specifier.clone(), path.clone());
        pending.push((path, identity));
    }
    let mut seen = BTreeSet::new();
    while let Some((path, identity)) = pending.pop() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let source = &files[&path];
        let dependencies = runtime_import_specifiers(source).map_err(failure)?;
        let mut edges = BTreeMap::new();
        for specifier in dependencies {
            let (target, identity) = resolver.resolve(&path, &specifier).map_err(failure)?;
            edges.insert(specifier, target.clone());
            pending.push((target, identity));
        }
        bundle.modules.insert(
            path.clone(),
            VirtualRuntimeModule {
                path,
                source: source.clone(),
                exact_identity: identity,
                content_digest: format!("sha256:{:x}", Sha256::digest(source.as_bytes())),
                imports: edges,
            },
        );
    }
    bundle.snapshot_digest = format!(
        "sha256:{:x}",
        Sha256::digest(
            serde_json::to_vec(&bundle.modules).map_err(|e| vec![problem(e.to_string())])?
        )
    );
    Ok(bundle)
}
/// Parse executable import edges, including literal import()/require(). Type
/// imports and comments are not runtime dependencies. Computed imports fail
/// before execution because their runtime identity cannot be pinned.
pub fn runtime_import_specifiers(source: &str) -> Result<Vec<String>, RuntimeLinkageDiagnostic> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .map_err(|e| problem(e.to_string()))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| problem("failed to parse runtime source"))?;
    if tree.root_node().has_error() {
        return Err(problem("runtime source has invalid TypeScript syntax"));
    }
    let mut pending = vec![tree.root_node()];
    let mut imports = BTreeSet::new();
    while let Some(node) = pending.pop() {
        if matches!(node.kind(), "import_statement" | "export_statement") {
            if statement_type_only(node) {
                continue;
            }
            if let Some(value) = node.child_by_field_name("source") {
                imports.insert(string_literal(value, source)?);
            }
        }
        if node.kind() == "call_expression" {
            let function = node
                .child_by_field_name("function")
                .and_then(|node| node.utf8_text(source.as_bytes()).ok())
                .unwrap_or_default();
            if matches!(function, "import" | "require" | "import.meta.resolve") {
                let arguments = node
                    .child_by_field_name("arguments")
                    .ok_or_else(|| problem("runtime import has no arguments"))?;
                if arguments.named_child_count() != 1 {
                    return Err(problem(
                        "runtime imports require exactly one literal specifier",
                    ));
                }
                imports.insert(string_literal(arguments.named_child(0).unwrap(), source)?);
            }
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    Ok(imports.into_iter().collect())
}
fn string_literal(
    node: tree_sitter::Node<'_>,
    source: &str,
) -> Result<String, RuntimeLinkageDiagnostic> {
    if node.kind() != "string" {
        return Err(problem(
            "computed runtime imports have unresolved exact identity",
        ));
    }
    let raw = node
        .utf8_text(source.as_bytes())
        .map_err(|e| problem(e.to_string()))?;
    // Import specifiers intentionally reject escapes rather than reinterpret
    // JS escapes differently on the native and browser host.
    let value = raw
        .get(1..raw.len().saturating_sub(1))
        .ok_or_else(|| problem("invalid runtime import string"))?;
    if value.is_empty() || value.contains('\\') {
        return Err(problem(
            "escaped or empty runtime import specifier is unsupported",
        ));
    }
    Ok(value.to_owned())
}

fn has_type_keyword(node: tree_sitter::Node<'_>) -> bool {
    let mut cursor = node.walk();
    let type_only = node
        .children(&mut cursor)
        .any(|child| child.kind() == "type");
    type_only
}
fn statement_type_only(node: tree_sitter::Node<'_>) -> bool {
    if has_type_keyword(node) {
        return true;
    }
    let mut cursor = node.walk();
    let clause = node
        .named_children(&mut cursor)
        .find(|child| matches!(child.kind(), "import_clause" | "export_clause"));
    let Some(clause) = clause else {
        return false;
    };
    let mut pending = vec![clause];
    let mut found = false;
    while let Some(node) = pending.pop() {
        match node.kind() {
            "import_specifier" | "export_specifier" => {
                if !has_type_keyword(node) {
                    return false;
                }
                found = true;
            }
            "identifier" | "namespace_import" | "namespace_export" => return false,
            "comment" => {}
            _ => {
                let mut cursor = node.walk();
                pending.extend(node.named_children(&mut cursor));
            }
        }
    }
    found
}
