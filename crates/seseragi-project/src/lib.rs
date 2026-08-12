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
mod manifest;
mod module_path;
mod package_name;
mod specifier;
mod standard;
mod workspace;

pub use graph::{ModuleGraph, ModuleGraphError};
pub use identity::{
    ModuleIdentity, ModuleRoot, PackageIdentity, PackageSourceIdentity, SourceIdentityError,
};
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
    load_local_project, load_local_project_with_overlays, LoadedLocalProject, LocalProjectLoadError,
};
pub use manifest::{
    parse_manifest, DependencyKey, DependencyPath, DependencyVersionRequirement,
    LanguageRequirement, LayoutPath, Manifest, ManifestDependency, ManifestError, ManifestLayout,
    ManifestPackage, ManifestRun, ProviderArtifactPath, RunSeed, SignalMode, TargetId,
};
pub use module_path::{ModulePath, ModulePathError};
pub use package_name::{PackageName, PackageNameError};
pub use specifier::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, RelativeSpecifierError,
    SpecifierError,
};
pub use standard::{
    is_standard_module, is_standard_void_html_tag, standard_html_tag, standard_html_tag_props,
    standard_module_interfaces, standard_module_target, StandardHtmlTag, StandardHtmlTagKind,
    STANDARD_HTML_TAGS,
};
pub use workspace::{
    load_workspace_project, LoadedWorkspaceProject, SourceOverlay, WorkspaceModule,
    WorkspaceProjectLoadError,
};
