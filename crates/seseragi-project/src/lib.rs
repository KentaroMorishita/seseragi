//! Package and module identity owned by the project layer.
//!
//! Compiler stages consume identities produced here; they do not infer them
//! from diagnostic source labels or process working directories.

mod graph;
mod identity;
mod link;
mod loader;
mod local_graph;
mod local_project;
mod lockfile;
mod manifest;
mod module_path;
mod package_name;
mod source_import;
mod specifier;
mod standard;
mod target;
mod virtual_package;
mod workspace;

pub use graph::{ModuleGraph, ModuleGraphError};
pub use identity::{
    ModuleIdentity, ModuleRoot, PackageIdentity, PackageSourceIdentity, SourceIdentityError,
};
pub fn logical_package_scope(package: &PackageIdentity) -> String {
    format!("{}@{}", package.name().as_str(), package.version())
}

pub fn logical_module_id(module: &ModuleIdentity) -> String {
    format!(
        "{}::{}",
        logical_package_scope(module.package()),
        module.path().as_str()
    )
}
pub use link::{
    link_module, LinkError, LinkTargetError, LinkedDependency, LinkedImport, LinkedModule,
    ModuleLinkTarget,
};
pub use loader::{
    load_package, LoadedModule, LoadedPackage, PackageLoadError, IMPLEMENTED_LANGUAGE_VERSION,
};
pub use local_graph::{
    discover_local_package_graph, LocalPackageGraph, LocalPackageGraphError, LocalPackageManifest,
    PackageImportError, ResolvedPackageImport,
};
pub use local_project::{
    load_local_project, load_local_project_with_overlays, load_local_tests, LoadedLocalProject,
    LoadedLocalTests, LocalProjectLoadError,
};
pub use lockfile::{
    generate_lockfile, parse_lockfile, read_and_validate_development_lockfile,
    read_and_validate_lockfile, write_lockfile, LockDependency, LockError, LockHostPackage,
    LockPackage, LockProviderSelection, LockSourceKind, Lockfile,
};
pub use manifest::{
    parse_manifest, DependencyKey, DependencyPath, DependencyVersionRequirement,
    LanguageRequirement, LayoutPath, Manifest, ManifestDependency, ManifestError, ManifestLayout,
    ManifestPackage, ManifestRun, ManifestTest, ProviderArtifactPath, RunSeed, SignalMode,
    TargetId,
};
pub use module_path::{ModulePath, ModulePathError};
pub use package_name::{PackageName, PackageNameError};
pub use source_import::{resolve_source_import, SourceImportError, SourceImportResolution};
pub use specifier::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, RelativeSpecifierError,
    SpecifierError,
};
pub use standard::{
    is_available_standard_module, is_standard_module, is_standard_void_html_tag, standard_html_tag,
    standard_html_tag_props, standard_module_interfaces, standard_module_registry_surface,
    standard_module_status, standard_module_target, StandardHtmlTag, StandardHtmlTagKind,
    StandardModuleRegistrySurface, StandardModuleStatus, StandardModuleSurface,
    StandardPreludeBoundary, STANDARD_HTML_TAGS,
};
pub use target::{
    select_project_target, ProjectCommand, ProjectTarget, TargetSelection, TargetSelectionError,
    TargetSelectionSource,
};
pub use virtual_package::{
    load_virtual_package, LoadedVirtualPackage, VirtualPackageLoadError, VirtualPackageModule,
    VirtualSourceFile,
};
pub use workspace::{
    load_workspace_project, LoadedWorkspaceProject, SourceOverlay, WorkspaceModule,
    WorkspaceProjectLoadError,
};
