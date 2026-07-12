use super::LinkTargetError;
use seseragi_syntax::{ModuleHeader, ModuleHeaderName, ModuleInterface, Visibility};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleLinkTarget {
    interface: ModuleInterface,
    header: Option<ModuleHeader>,
}

impl ModuleLinkTarget {
    /// Creates a target from a module compiled in the same package.
    ///
    /// Its private header is retained for diagnostics but is not copied into
    /// the public interface or exposed across package boundaries.
    pub fn same_package(
        header: ModuleHeader,
        interface: ModuleInterface,
    ) -> Result<Self, LinkTargetError> {
        validate_header(&header, &interface)?;
        Ok(Self {
            interface,
            header: Some(header),
        })
    }

    /// Creates a target from an external package's published interface.
    pub const fn external(interface: ModuleInterface) -> Self {
        Self {
            interface,
            header: None,
        }
    }

    pub const fn interface(&self) -> &ModuleInterface {
        &self.interface
    }

    pub const fn header(&self) -> Option<&ModuleHeader> {
        self.header.as_ref()
    }

    pub(crate) fn private_names(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Vec<&ModuleHeaderName> {
        let Some(header) = &self.header else {
            return Vec::new();
        };
        header
            .names
            .iter()
            .filter(|entry| entry.name == name)
            .filter(|entry| namespace.is_none_or(|namespace| entry.namespace == namespace))
            .filter(|entry| entry.visibility == Visibility::Private)
            .collect()
    }
}

fn validate_header(
    header: &ModuleHeader,
    interface: &ModuleInterface,
) -> Result<(), LinkTargetError> {
    if header.module != interface.module {
        return Err(LinkTargetError::ModuleMismatch {
            header: header.module.clone(),
            interface: interface.module.clone(),
        });
    }
    if header.source != interface.source {
        return Err(LinkTargetError::SourceMismatch {
            header: header.source.clone(),
            interface: interface.source.clone(),
        });
    }
    for entry in &header.names {
        if entry.visibility == Visibility::Public
            && !interface
                .exports
                .iter()
                .any(|export| export.namespace == entry.namespace && export.name == entry.name)
        {
            return Err(LinkTargetError::MissingPublicExport {
                module: header.module.clone(),
                namespace: entry.namespace.clone(),
                name: entry.name.clone(),
                declaration: entry.declaration,
            });
        }
    }
    Ok(())
}
