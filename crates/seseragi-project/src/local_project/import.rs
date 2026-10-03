use crate::{
    resolve_module_import, ImportSpecifier, LocalPackageGraph, ModuleIdentity,
    ModuleImportResolution, ModuleRoot, SourceImportError,
};

pub(super) fn resolve_import(
    packages: &LocalPackageGraph,
    current: &ModuleIdentity,
    specifier: &str,
) -> Result<ResolvedImport, ImportFailure> {
    let resolved = resolve_module_import(current.root(), current.path(), specifier);
    let (package, root, path) = match resolved {
        Ok(ModuleImportResolution::Local { root, path }) => (current.package().clone(), root, path),
        Ok(ModuleImportResolution::Standard) => return Ok(ResolvedImport::Standard),
        Err(SourceImportError::Unsupported(ImportSpecifier::Package(_))) => {
            let package_name = current.package().name().as_str();
            if specifier == package_name
                || specifier
                    .strip_prefix(package_name)
                    .is_some_and(|suffix| suffix.starts_with('/'))
            {
                if current.root() == ModuleRoot::Generated {
                    return Err(ImportFailure::new(
                        "SES-N0104",
                        "generated modules cannot import their package through its public exports",
                    ));
                }
                let suffix = specifier
                    .strip_prefix(package_name)
                    .expect("self prefix matched");
                let export = if suffix.is_empty() {
                    "."
                } else {
                    suffix
                        .strip_prefix('/')
                        .expect("self subpath starts with slash")
                };
                let package = packages
                    .package(current.package())
                    .expect("current package belongs to graph");
                let path = package
                    .manifest()
                    .exports
                    .get(export)
                    .cloned()
                    .ok_or_else(|| {
                        ImportFailure::new(
                            "SES-N0104",
                            format!("package `{package_name}` does not export `{export}`"),
                        )
                    })?;
                return Ok(ResolvedImport::Module(ModuleIdentity::new(
                    current.package().clone(),
                    ModuleRoot::Source,
                    path,
                )));
            }
            let resolved = packages
                .resolve_package_import(current.package(), specifier)
                .map_err(|error| ImportFailure::new(error.code(), error.to_string()))?;
            (
                resolved.package().clone(),
                ModuleRoot::Source,
                resolved.module().clone(),
            )
        }
        Err(SourceImportError::Unsupported(ImportSpecifier::Generated(_))) => unreachable!(),
        Err(SourceImportError::Unsupported(ImportSpecifier::Standard(_))) => unreachable!(),
        Err(SourceImportError::Unsupported(ImportSpecifier::Relative(_)))
        | Err(SourceImportError::Unsupported(ImportSpecifier::SelfPackage(_))) => unreachable!(),
        Err(SourceImportError::Invalid(reason)) => {
            return Err(ImportFailure::new("SES-N0104", reason));
        }
    };
    Ok(ResolvedImport::Module(ModuleIdentity::new(
        package, root, path,
    )))
}

pub(super) enum ResolvedImport {
    Standard,
    Module(ModuleIdentity),
}

pub(super) struct ImportFailure {
    pub(super) code: &'static str,
    pub(super) reason: String,
}

impl ImportFailure {
    fn new(code: &'static str, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }
}
