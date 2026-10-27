use crate::{InterfaceNamespaceDeprecation, SurfaceForeignMember, SurfaceModule, Visibility};

/// Retains only explicit namespace clauses, with their actual declaration
/// positions and source-produced canonical child identities.
pub fn namespace_deprecations(
    module: &str,
    surface: &SurfaceModule,
    include_private: bool,
) -> Vec<InterfaceNamespaceDeprecation> {
    let mut result = Vec::new();
    for foreign in &surface.foreign_modules {
        if include_private || foreign.visibility == Visibility::Public {
            visit(module, "", &foreign.members, surface, &mut result);
        }
    }
    result
}

fn visit(
    module: &str,
    parent: &str,
    members: &[SurfaceForeignMember],
    surface: &SurfaceModule,
    result: &mut Vec<InterfaceNamespaceDeprecation>,
) {
    for member in members {
        let SurfaceForeignMember::Namespace {
            name,
            name_span,
            members,
            span,
            ..
        } = member
        else {
            continue;
        };
        let path = joined(parent, name);
        if let Some(notice) = surface
            .deprecations
            .iter()
            .find(|entry| entry.declaration == *span)
        {
            let mut children = Vec::new();
            collect_children(module, &path, members, &mut children);
            result.push(InterfaceNamespaceDeprecation {
                symbol: format!("{module}::{path}"),
                name: path.clone(),
                declaration: *span,
                name_span: *name_span,
                deprecation: notice.deprecation.clone(),
                members: children,
                publication: None,
            });
        }
        visit(module, &path, members, surface, result);
    }
}

fn collect_children(
    module: &str,
    parent: &str,
    members: &[SurfaceForeignMember],
    result: &mut Vec<String>,
) {
    for member in members {
        match member {
            SurfaceForeignMember::Namespace { name, members, .. } => {
                collect_children(module, &joined(parent, name), members, result);
            }
            SurfaceForeignMember::Function { name, .. }
            | SurfaceForeignMember::Value { name, .. }
            | SurfaceForeignMember::OpaqueType { name, .. } => {
                result.push(format!("{module}::{}", joined(parent, name)));
            }
        }
    }
}

fn joined(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}.{name}")
    }
}
