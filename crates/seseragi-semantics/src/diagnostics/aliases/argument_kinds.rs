//! Check explicit nominal type arguments even when no value uses the annotation.
use super::{error, type_ref_span, type_target, walk_module_types, TypeAnnotationVisitor};
use crate::{ResolvedModule, SymbolId, SymbolKind};
use seseragi_syntax::{ByteSpan, Diagnostic, SurfaceDecl, TypeParameter, TypeRef};
use std::collections::BTreeMap;

pub(super) fn collect_nominal_argument_kind_diagnostics(
    resolved: &ResolvedModule,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut binders = BinderKinds {
        resolved,
        arities: BTreeMap::new(),
    };
    walk_module_types(&resolved.declarations, &mut binders);
    walk_module_types(&resolved.declarations, &mut |type_ref: &TypeRef| {
        let TypeRef::Named {
            arguments, span, ..
        } = type_ref
        else {
            return;
        };
        let Some(owner) = type_target(resolved, *span) else {
            return;
        };
        let Some(parameters) = nominal_parameters(resolved, owner) else {
            return;
        };
        for (argument, parameter) in arguments.iter().zip(parameters) {
            let mismatch = match remaining_kind(resolved, &binders.arities, argument) {
                RemainingKind::Known(kinds) => {
                    kinds.len() != parameter.arity as usize || kinds.iter().any(|kind| *kind != 0)
                }
                RemainingKind::Invalid => true,
                RemainingKind::Unknown => false,
            };
            if mismatch {
                diagnostics.push(error(
                    "SES-T0604",
                    "type.argument-kind-mismatch",
                    type_ref_span(argument),
                ));
            }
        }
    });
}

struct BinderKinds<'a> {
    resolved: &'a ResolvedModule,
    arities: BTreeMap<SymbolId, u32>,
}

impl TypeAnnotationVisitor for BinderKinds<'_> {
    fn visit_type_parameters(&mut self, parameters: &[TypeParameter], origin: ByteSpan) {
        for symbol in
            self.resolved.symbols.iter().filter(|symbol| {
                symbol.kind == SymbolKind::TypeParameter && symbol.origin == origin
            })
        {
            if let Some(parameter) = parameters.iter().find(|p| p.name == symbol.spelling) {
                // Only the declaration that introduced this resolved binder
                // supplies its kind. Sibling and same-named nominal owners
                // retain separate SymbolIds.
                self.arities.insert(symbol.id, parameter.arity);
            }
        }
    }
}

enum RemainingKind {
    Known(Vec<u32>),
    Invalid,
    Unknown,
}

fn remaining_kind(
    resolved: &ResolvedModule,
    binders: &BTreeMap<SymbolId, u32>,
    type_ref: &TypeRef,
) -> RemainingKind {
    let TypeRef::Named {
        arguments, span, ..
    } = type_ref
    else {
        return match type_ref {
            TypeRef::Hole { .. } => RemainingKind::Unknown,
            _ => RemainingKind::Known(Vec::new()),
        };
    };
    let Some(target) = type_target(resolved, *span) else {
        return RemainingKind::Unknown;
    };
    let Some(parameters) = parameter_kinds(resolved, binders, target) else {
        return RemainingKind::Unknown;
    };
    if arguments.len() > parameters.len() {
        return RemainingKind::Invalid;
    }
    let remaining = arguments
        .iter()
        .zip(&parameters)
        .filter_map(|(argument, kind)| matches!(argument, TypeRef::Hole { .. }).then_some(*kind))
        .chain(parameters[arguments.len()..].iter().copied())
        .collect();
    RemainingKind::Known(remaining)
}

fn parameter_kinds(
    resolved: &ResolvedModule,
    binders: &BTreeMap<SymbolId, u32>,
    target: SymbolId,
) -> Option<Vec<u32>> {
    let symbol = resolved.symbols.iter().find(|symbol| symbol.id == target)?;
    if symbol.kind == SymbolKind::TypeParameter {
        // F<_, _> accepts two value types; it is not an arbitrary higher-kind
        // constructor merely because another declaration has the same arity.
        return binders.get(&target).map(|arity| vec![0; *arity as usize]);
    }
    if let Some(parameters) = nominal_parameters(resolved, target) {
        return Some(parameters.iter().map(|p| p.arity).collect());
    }
    if let Some(parameters) = super::alias_parameters(resolved, target) {
        return Some(parameters.iter().map(|p| p.arity).collect());
    }
    if let Some(name) = symbol.canonical.as_deref()?.strip_prefix("std/prelude::") {
        return crate::prelude::type_constructor_arity(name).map(|arity| vec![0; arity as usize]);
    }
    // Newtype constructors can be supplied as arguments, without broadening
    // this annotation check into newtype representation validation.
    resolved.declarations.iter().find_map(|declaration| {
        let SurfaceDecl::Newtype {
            name_span,
            type_parameters,
            ..
        } = declaration
        else {
            return None;
        };
        (*name_span == symbol.origin).then(|| type_parameters.iter().map(|p| p.arity).collect())
    })
}

fn nominal_parameters(resolved: &ResolvedModule, owner: SymbolId) -> Option<&[TypeParameter]> {
    let symbol = resolved.symbols.iter().find(|symbol| symbol.id == owner)?;
    if let Some(parameters) = resolved.declarations.iter().find_map(|declaration| {
        let (name_span, type_parameters) = match declaration {
            SurfaceDecl::Type {
                name_span,
                type_parameters,
                ..
            }
            | SurfaceDecl::Struct {
                name_span,
                type_parameters,
                ..
            } => (name_span, type_parameters),
            _ => return None,
        };
        (*name_span == symbol.origin).then_some(type_parameters.as_slice())
    }) {
        return Some(parameters);
    }
    resolved
        .imports
        .iter()
        .find(|import| {
            import.symbol == owner
                && import.export.namespace == "type"
                && matches!(
                    import.export.declaration_kind.as_deref(),
                    Some("type" | "opaque-type" | "struct" | "opaque-struct" | "newtype")
                )
        })
        .map(|import| import.export.scheme.type_parameters.as_slice())
}
