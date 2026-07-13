use crate::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, LocalPackageGraph,
    ModuleIdentity, ModulePath, ModuleRoot,
};

pub(super) fn resolve_import(
    packages: &LocalPackageGraph,
    current: &ModuleIdentity,
    specifier: &str,
) -> Result<ModuleIdentity, ImportFailure> {
    let kind = classify_specifier(specifier)
        .map_err(|error| ImportFailure::new("SES-N0104", error.to_string()))?;
    let (package, path) = match kind {
        ImportSpecifier::Relative(value) => (
            current.package().clone(),
            resolve_relative_specifier(current.path(), &value)
                .map_err(|error| ImportFailure::new("SES-N0104", error.to_string()))?,
        ),
        ImportSpecifier::SelfPackage(value) => (
            current.package().clone(),
            ModulePath::parse(&value)
                .map_err(|error| ImportFailure::new("SES-N0104", error.to_string()))?,
        ),
        ImportSpecifier::Package(_) => {
            let resolved = packages
                .resolve_package_import(current.package(), specifier)
                .map_err(|error| ImportFailure::new(error.code(), error.to_string()))?;
            (resolved.package().clone(), resolved.module().clone())
        }
        unsupported @ (ImportSpecifier::Standard(_) | ImportSpecifier::Generated(_)) => {
            return Err(ImportFailure::new(
                "SES-K0001",
                format!("unsupported source import {unsupported:?}"),
            ));
        }
    };
    Ok(ModuleIdentity::new(package, ModuleRoot::Source, path))
}

pub(super) struct ImportFailure {
    pub(super) code: &'static str,
    pub(super) reason: String,
}

impl ImportFailure {
    fn new(code: &'static str, reason: String) -> Self {
        Self { code, reason }
    }
}
