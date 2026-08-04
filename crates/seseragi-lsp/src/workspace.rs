use crate::features::DocumentState;
use seseragi_driver::{analyze_project, ProjectModuleInput};
use seseragi_project::{
    load_local_project_with_overlays, load_workspace_project, LoadedLocalProject,
    LoadedWorkspaceProject, ModuleGraph, ModuleIdentity, ModulePath, PackageIdentity,
    SourceOverlay,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProjectKey {
    LocalPackage { root: PathBuf },
    WorkspaceSource { root: PathBuf, entry: PathBuf },
}

impl ProjectKey {
    pub(crate) fn root(&self) -> &Path {
        match self {
            Self::LocalPackage { root } | Self::WorkspaceSource { root, .. } => root,
        }
    }

    pub(crate) fn contains_path(&self, path: &Path) -> bool {
        path.starts_with(self.root())
    }

    pub(crate) fn entry(&self) -> Option<&Path> {
        match self {
            Self::WorkspaceSource { entry, .. } => Some(entry),
            Self::LocalPackage { .. } => None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct OpenDocument {
    pub(crate) version: i64,
    pub(crate) source: String,
    pub(crate) path: Option<PathBuf>,
}

#[derive(Clone, Default)]
pub(crate) struct ProjectSnapshot {
    pub(crate) documents: BTreeMap<String, DocumentState>,
}

pub(crate) fn file_path(uri: &str) -> Option<PathBuf> {
    let url = Url::parse(uri).ok()?;
    (url.scheme() == "file").then_some(())?;
    let path = url.to_file_path().ok()?;
    Some(fs::canonicalize(&path).unwrap_or(path))
}

pub(crate) fn workspace_folder_paths(
    folders: impl IntoIterator<Item = String>,
    root_uri: Option<String>,
) -> Vec<PathBuf> {
    let mut paths = folders
        .into_iter()
        .filter_map(|uri| file_path(&uri))
        .collect::<Vec<_>>();
    if paths.is_empty() {
        paths.extend(root_uri.as_deref().and_then(file_path));
    }
    paths.sort();
    paths.dedup();
    paths
}

pub(crate) fn project_key_for(path: &Path, workspace_folders: &[PathBuf]) -> Option<ProjectKey> {
    let root = workspace_folders
        .iter()
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())?
        .clone();
    if let Some(package_root) = manifest_root(path, &root) {
        return Some(ProjectKey::LocalPackage { root: package_root });
    }
    Some(ProjectKey::WorkspaceSource {
        root,
        entry: path.to_owned(),
    })
}

pub(crate) fn analyze(
    key: &ProjectKey,
    open_documents: &BTreeMap<String, OpenDocument>,
) -> Result<ProjectSnapshot, String> {
    match key {
        ProjectKey::LocalPackage { root } => {
            let overlays = open_documents
                .values()
                .filter_map(|document| {
                    let path = document.path.as_ref()?;
                    path.starts_with(root)
                        .then(|| SourceOverlay::new(path, &document.source))
                })
                .collect::<Vec<_>>();
            let project = load_local_project_with_overlays(root, overlays)
                .map_err(|error| error.to_string())?;
            analyze_local_package(&project, open_documents)
        }
        ProjectKey::WorkspaceSource { root, entry } => {
            let overlays = open_documents
                .values()
                .filter_map(|document| {
                    let path = document.path.as_ref()?;
                    path.starts_with(root)
                        .then(|| SourceOverlay::new(path, &document.source))
                })
                .collect::<Vec<_>>();
            let project =
                load_workspace_project(root, entry, overlays).map_err(|error| error.to_string())?;
            analyze_workspace_source(&project, open_documents)
        }
    }
}

fn manifest_root(path: &Path, workspace_root: &Path) -> Option<PathBuf> {
    let mut current = path.parent()?.to_owned();
    loop {
        if current.join("seseragi.toml").is_file() {
            return Some(fs::canonicalize(current).ok()?);
        }
        if current == workspace_root {
            return None;
        }
        current = current.parent()?.to_owned();
        if !current.starts_with(workspace_root) {
            return None;
        }
    }
}

fn analyze_workspace_source(
    project: &LoadedWorkspaceProject,
    open_documents: &BTreeMap<String, OpenDocument>,
) -> Result<ProjectSnapshot, String> {
    let mut graph = ModuleGraph::new();
    for (path, _) in project.modules() {
        let dependencies = project
            .graph()
            .dependencies_for(path)
            .expect("loaded workspace graph contains every source module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, workspace_module_id(&dependency)));
        graph
            .add_module(workspace_module_id(path), dependencies)
            .map_err(|error| format!("invalid workspace graph: {error:?}"))?;
    }
    let inputs = project.modules().map(|(path, module)| {
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            workspace_module_id(path),
            module.source(),
            format!("lsp/{}.js", path.as_str()),
        )
    });
    let mut analyzed = analyze_project(graph, inputs).map_err(|error| format!("{error:?}"))?;
    let mut documents = BTreeMap::new();
    for (path, module) in project.modules() {
        let analysis = analyzed
            .documents
            .remove(&workspace_module_id(path))
            .expect("project analysis contains every workspace module");
        let (uri, version) = uri_and_version(module.source_path(), open_documents)?;
        documents.insert(
            uri,
            DocumentState::from_analysis(version, module.source().to_owned(), analysis),
        );
    }
    Ok(ProjectSnapshot { documents })
}

fn analyze_local_package(
    project: &LoadedLocalProject,
    open_documents: &BTreeMap<String, OpenDocument>,
) -> Result<ProjectSnapshot, String> {
    let mut graph = ModuleGraph::new();
    for (identity, _) in project.modules() {
        let dependencies = project
            .graph()
            .dependencies_for(identity)
            .expect("loaded package graph contains every source module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, local_module_id(&dependency)));
        graph
            .add_module(local_module_id(identity), dependencies)
            .map_err(|error| format!("invalid local package graph: {error:?}"))?;
    }
    let inputs = project.modules().map(|(identity, module)| {
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            local_module_id(identity),
            module.source(),
            local_output_path(identity),
        )
        .with_package_scope(package_scope(identity.package()))
    });
    let mut analyzed = analyze_project(graph, inputs).map_err(|error| format!("{error:?}"))?;
    let mut documents = BTreeMap::new();
    for (identity, module) in project.modules() {
        let analysis = analyzed
            .documents
            .remove(&local_module_id(identity))
            .expect("project analysis contains every local source module");
        let (uri, version) = uri_and_version(module.source_path(), open_documents)?;
        documents.insert(
            uri,
            DocumentState::from_analysis(version, module.source().to_owned(), analysis),
        );
    }
    Ok(ProjectSnapshot { documents })
}

fn uri_and_version(
    path: &Path,
    open_documents: &BTreeMap<String, OpenDocument>,
) -> Result<(String, Option<i64>), String> {
    if let Some((uri, document)) = open_documents
        .iter()
        .find(|(_, document)| document.path.as_deref() == Some(path))
    {
        return Ok((uri.clone(), Some(document.version)));
    }
    let uri = Url::from_file_path(path)
        .map_err(|_| {
            format!(
                "cannot encode workspace source as a file URI: {}",
                path.display()
            )
        })?
        .into();
    Ok((uri, None))
}

fn workspace_module_id(path: &ModulePath) -> String {
    format!("workspace/{}", path.as_str())
}

fn local_module_id(identity: &ModuleIdentity) -> String {
    format!(
        "{}::{}",
        package_scope(identity.package()),
        identity.path().as_str()
    )
}

fn package_scope(package: &PackageIdentity) -> String {
    format!("{}@{}", package.name().as_str(), package.version())
}

fn local_output_path(identity: &ModuleIdentity) -> String {
    format!(
        "lsp/packages/{}/{}/{}.js",
        identity.package().name().as_str(),
        identity.package().version(),
        identity.path().as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_the_innermost_matching_workspace_folder_without_scanning_siblings() {
        let outer =
            std::env::temp_dir().join(format!("seseragi-lsp-multi-root-{}", std::process::id()));
        let nested = outer.join("nested");
        let key = project_key_for(&nested.join("main.ssrg"), &[outer.clone(), nested.clone()])
            .expect("nested source is inside a workspace folder");

        assert_eq!(
            key,
            ProjectKey::WorkspaceSource {
                root: nested,
                entry: outer.join("nested/main.ssrg"),
            }
        );
    }
}
