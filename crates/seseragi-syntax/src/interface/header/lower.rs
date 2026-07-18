use crate::surface::{ByteSpan, SurfaceDecl, SurfaceModule, Visibility};

use super::{ModuleHeader, ModuleHeaderName};

pub(super) fn module_header_from_surface(module: &str, surface: &SurfaceModule) -> ModuleHeader {
    ModuleHeader {
        schema: 1,
        module: module.to_owned(),
        source: surface.source.clone(),
        names: surface
            .declarations
            .iter()
            .flat_map(|declaration| names_from_declaration(module, declaration))
            .collect(),
    }
}

pub(super) fn empty_module_header(module: String, source: String) -> ModuleHeader {
    ModuleHeader {
        schema: 1,
        module,
        source,
        names: Vec::new(),
    }
}

fn names_from_declaration(module: &str, declaration: &SurfaceDecl) -> Vec<ModuleHeaderName> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            name_span,
            ..
        } => vec![name_entry(
            module,
            "value",
            name,
            *visibility,
            "let",
            *name_span,
        )],
        SurfaceDecl::EffectFn {
            visibility,
            name,
            name_span,
            ..
        } => vec![name_entry(
            module,
            "value",
            name,
            *visibility,
            "effect-function",
            *name_span,
        )],
        SurfaceDecl::Fn {
            visibility,
            name,
            name_span,
            ..
        } => vec![name_entry(
            module,
            "value",
            name,
            *visibility,
            "function",
            *name_span,
        )],
        SurfaceDecl::Newtype {
            visibility,
            opaque,
            name,
            name_span,
            ..
        } => nominal_with_constructors(
            module,
            name,
            *name_span,
            *visibility,
            *opaque,
            "newtype",
            std::iter::once((name.as_str(), *name_span)),
        ),
        SurfaceDecl::Alias {
            visibility,
            name,
            name_span,
            ..
        } => vec![name_entry(
            module,
            "type",
            name,
            *visibility,
            "alias",
            *name_span,
        )],
        SurfaceDecl::Type {
            visibility,
            opaque,
            name,
            name_span,
            variants,
            ..
        } => nominal_with_constructors(
            module,
            name,
            *name_span,
            *visibility,
            *opaque,
            "type",
            variants
                .iter()
                .map(|variant| (variant.name.as_str(), variant.name_span)),
        ),
        SurfaceDecl::Struct {
            visibility,
            opaque,
            name,
            name_span,
            ..
        } => vec![name_entry(
            module,
            "type",
            name,
            *visibility,
            if *opaque { "opaque-struct" } else { "struct" },
            *name_span,
        )],
        SurfaceDecl::Trait {
            visibility,
            name,
            name_span,
            ..
        } => vec![name_entry(
            module,
            "trait",
            name,
            *visibility,
            "trait",
            *name_span,
        )],
        SurfaceDecl::Operator {
            visibility,
            spelling,
            span,
            ..
        } => vec![name_entry(
            module,
            "operator",
            spelling,
            *visibility,
            "custom-operator",
            *span,
        )],
        SurfaceDecl::Impl { .. } | SurfaceDecl::Instance { .. } => Vec::new(),
    }
}

fn nominal_with_constructors<'a>(
    module: &str,
    name: &str,
    name_span: ByteSpan,
    visibility: Visibility,
    opaque: bool,
    declaration_kind: &str,
    constructors: impl Iterator<Item = (&'a str, ByteSpan)>,
) -> Vec<ModuleHeaderName> {
    let owner = format!("{module}::{name}");
    let mut names = vec![name_entry(
        module,
        "type",
        name,
        visibility,
        if opaque {
            match declaration_kind {
                "type" => "opaque-type",
                other => other,
            }
        } else {
            declaration_kind
        },
        name_span,
    )];
    let constructor_visibility = if visibility == Visibility::Public && !opaque {
        Visibility::Public
    } else {
        Visibility::Private
    };
    names.extend(constructors.map(|(constructor, span)| {
        let mut entry = name_entry(
            module,
            "value",
            constructor,
            constructor_visibility,
            "constructor",
            span,
        );
        entry.constructor_of = Some(owner.clone());
        entry
    }));
    names
}

fn name_entry(
    module: &str,
    namespace: &str,
    name: &str,
    visibility: Visibility,
    declaration_kind: &str,
    declaration: ByteSpan,
) -> ModuleHeaderName {
    ModuleHeaderName {
        symbol: match namespace {
            "trait" => format!("{module}::trait({name})"),
            "operator" => format!("{module}::operator({name})"),
            _ => format!("{module}::{name}"),
        },
        namespace: namespace.to_owned(),
        name: name.to_owned(),
        visibility,
        declaration_kind: declaration_kind.to_owned(),
        declaration,
        constructor_of: None,
    }
}
