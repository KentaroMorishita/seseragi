use crate::{ResolvedModule, SymbolId, SymbolKind, SymbolNamespace};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, SurfaceBlockItem,
    SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr, SurfaceImplMember,
    SurfaceTemplatePart, TypeRef, Visibility,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect_alias_diagnostics(
    resolved: &ResolvedModule,
    diagnostics: &mut Vec<Diagnostic>,
) {
    collect_arity_diagnostics(resolved, diagnostics);
    collect_cycle_diagnostics(resolved, diagnostics);
    collect_private_exposure_diagnostics(resolved, diagnostics);
    collect_requirement_merge_diagnostics(resolved, diagnostics);
}

fn collect_requirement_merge_diagnostics(
    resolved: &ResolvedModule,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut allowed = BTreeSet::new();
    walk_module_types(&resolved.declarations, &mut |type_ref| {
        let TypeRef::Named {
            name, arguments, ..
        } = type_ref
        else {
            return;
        };
        if matches!(name.as_str(), "Effect" | "Stream") {
            if let Some(environment) = arguments.first() {
                mark_allowed_requirement_merges(environment, &mut allowed);
            }
        }
    });

    let resolution = crate::typed::TypedResolution::new(resolved);
    walk_module_types(&resolved.declarations, &mut |type_ref| {
        let TypeRef::RequirementMerge { operands, span } = type_ref else {
            return;
        };
        if !allowed.contains(&(span.start, span.end))
            || operands
                .iter()
                .any(|operand| !valid_requirement_operand(operand, resolved, &resolution))
        {
            diagnostics.push(error(
                "SES-T0501",
                "type.requirement-merge-invalid-position",
                *span,
            ));
            return;
        }

        let normalized = resolution.semantic_value_from_type_ref(type_ref).type_ref;
        let crate::TypedType::Record { fields, .. } = normalized else {
            return;
        };
        for (index, field) in fields.iter().enumerate() {
            if fields[..index]
                .iter()
                .any(|previous| previous.name == field.name && previous.type_ref != field.type_ref)
            {
                diagnostics.push(error(
                    "SES-E0001",
                    "effect.requirement-merge-field-conflict",
                    *span,
                ));
                return;
            }
        }
    });
}

fn mark_allowed_requirement_merges(type_ref: &TypeRef, allowed: &mut BTreeSet<(usize, usize)>) {
    let TypeRef::RequirementMerge { operands, span } = type_ref else {
        return;
    };
    allowed.insert((span.start, span.end));
    for operand in operands {
        mark_allowed_requirement_merges(operand, allowed);
    }
}

fn valid_requirement_operand(
    operand: &TypeRef,
    resolved: &ResolvedModule,
    resolution: &crate::typed::TypedResolution<'_>,
) -> bool {
    match operand {
        TypeRef::Record { fields, .. } => fields.iter().all(|field| !field.optional),
        TypeRef::Named {
            arguments, span, ..
        } if arguments.is_empty() => {
            let is_parameter = resolution
                .target(*span, SymbolNamespace::Type)
                .and_then(|target| resolved.symbols.iter().find(|symbol| symbol.id == target))
                .is_some_and(|symbol| symbol.kind == SymbolKind::TypeParameter);
            if is_parameter {
                return true;
            }
            matches!(
                resolution.semantic_value_from_type_ref(operand).type_ref,
                crate::TypedType::Record { fields, .. }
                    if fields.iter().all(|field| !field.optional)
            )
        }
        TypeRef::RequirementMerge { operands, .. } => operands
            .iter()
            .all(|operand| valid_requirement_operand(operand, resolved, resolution)),
        _ => false,
    }
}

fn collect_arity_diagnostics(resolved: &ResolvedModule, diagnostics: &mut Vec<Diagnostic>) {
    walk_module_types(&resolved.declarations, &mut |type_ref| {
        let TypeRef::Named {
            arguments, span, ..
        } = type_ref
        else {
            return;
        };
        let Some(target) = type_target(resolved, *span) else {
            return;
        };
        let Some(parameters) = alias_parameters(resolved, target) else {
            return;
        };
        if arguments.len() != parameters.len() {
            diagnostics.push(error("SES-T0601", "alias.arity-mismatch", *span));
            return;
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            let mismatch = match remaining_type_arity(resolved, argument) {
                RemainingTypeArity::Known(actual) => actual != parameter.arity,
                RemainingTypeArity::Invalid => true,
                RemainingTypeArity::Unknown => false,
            };
            if mismatch {
                diagnostics.push(error(
                    "SES-T0604",
                    "alias.kind-mismatch",
                    type_ref_span(argument),
                ));
            }
        }
    });
}

fn collect_cycle_diagnostics(resolved: &ResolvedModule, diagnostics: &mut Vec<Diagnostic>) {
    let aliases = local_aliases(resolved);
    let owners = aliases.keys().copied().collect::<BTreeSet<_>>();
    let graph = aliases
        .iter()
        .map(|(owner, declaration)| {
            let mut edges = BTreeSet::new();
            walk_type(&declaration.target, &mut |type_ref| {
                if let TypeRef::Named { span, .. } = type_ref {
                    if let Some(target) =
                        type_target(resolved, *span).filter(|target| owners.contains(target))
                    {
                        edges.insert(target);
                    }
                }
            });
            (*owner, edges)
        })
        .collect::<BTreeMap<_, _>>();
    for (owner, declaration) in aliases {
        if reaches(&graph, owner, owner, &mut BTreeSet::new()) {
            diagnostics.push(error("SES-T0602", "alias.cycle", declaration.name_span));
        }
    }
}

fn collect_private_exposure_diagnostics(
    resolved: &ResolvedModule,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let aliases = local_aliases(resolved);
    for declaration in aliases
        .values()
        .filter(|alias| alias.visibility == Visibility::Public)
    {
        if let Some(origin) = private_nominal_in(
            resolved,
            &aliases,
            &declaration.target,
            &mut BTreeSet::new(),
        ) {
            diagnostics.push(error("SES-T0603", "alias.private-type-exposure", origin));
        }
    }
}

#[derive(Clone, Copy)]
struct LocalAlias<'a> {
    visibility: Visibility,
    name_span: ByteSpan,
    target: &'a TypeRef,
}

fn local_aliases(resolved: &ResolvedModule) -> BTreeMap<SymbolId, LocalAlias<'_>> {
    resolved
        .declarations
        .iter()
        .filter_map(|declaration| {
            let SurfaceDecl::Alias {
                visibility,
                name_span,
                target,
                ..
            } = declaration
            else {
                return None;
            };
            let owner = resolved
                .symbols
                .iter()
                .find(|symbol| symbol.kind == SymbolKind::Type && symbol.origin == *name_span)?;
            Some((
                owner.id,
                LocalAlias {
                    visibility: *visibility,
                    name_span: *name_span,
                    target,
                },
            ))
        })
        .collect()
}

fn private_nominal_in(
    resolved: &ResolvedModule,
    aliases: &BTreeMap<SymbolId, LocalAlias<'_>>,
    type_ref: &TypeRef,
    visited: &mut BTreeSet<SymbolId>,
) -> Option<ByteSpan> {
    let mut found = None;
    walk_type(type_ref, &mut |type_ref| {
        if found.is_some() {
            return;
        }
        let TypeRef::Named { span, .. } = type_ref else {
            return;
        };
        let Some(target) = type_target(resolved, *span) else {
            return;
        };
        if let Some(alias) = aliases.get(&target) {
            if visited.insert(target) {
                found = private_nominal_in(resolved, aliases, alias.target, visited);
                visited.remove(&target);
            }
            return;
        }
        let Some(symbol) = resolved.symbols.iter().find(|symbol| symbol.id == target) else {
            return;
        };
        if symbol.origin.start == 0 && symbol.origin.end == 0 {
            return;
        }
        if resolved.declarations.iter().any(|declaration| {
            matches!(
                declaration,
                SurfaceDecl::Type { visibility: Visibility::Private, name_span, .. }
                    | SurfaceDecl::Newtype { visibility: Visibility::Private, name_span, .. }
                    | SurfaceDecl::Struct { visibility: Visibility::Private, name_span, .. }
                    if *name_span == symbol.origin
            )
        }) {
            found = Some(*span);
        }
    });
    found
}

fn reaches(
    graph: &BTreeMap<SymbolId, BTreeSet<SymbolId>>,
    start: SymbolId,
    current: SymbolId,
    visited: &mut BTreeSet<SymbolId>,
) -> bool {
    if !visited.insert(current) {
        return false;
    }
    let result = graph.get(&current).is_some_and(|edges| {
        edges.contains(&start)
            || edges
                .iter()
                .copied()
                .any(|next| reaches(graph, start, next, visited))
    });
    visited.remove(&current);
    result
}

fn alias_parameters(
    resolved: &ResolvedModule,
    target: SymbolId,
) -> Option<Vec<seseragi_syntax::TypeParameter>> {
    let canonical = resolved
        .symbols
        .iter()
        .find(|symbol| symbol.id == target)
        .and_then(|symbol| symbol.canonical.as_deref());
    if canonical == Some("std/prelude::Task") {
        return Some(vec![seseragi_syntax::TypeParameter::value("A")]);
    }
    if let Some(parameters) = resolved.declarations.iter().find_map(|declaration| {
        let SurfaceDecl::Alias {
            name_span,
            type_parameters,
            ..
        } = declaration
        else {
            return None;
        };
        resolved
            .symbols
            .iter()
            .any(|symbol| symbol.id == target && symbol.origin == *name_span)
            .then_some(type_parameters.clone())
    }) {
        return Some(parameters);
    }
    resolved
        .imports
        .iter()
        .find(|import| {
            import.symbol == target && import.export.declaration_kind.as_deref() == Some("alias")
        })
        .map(|import| import.export.scheme.type_parameters.clone())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RemainingTypeArity {
    Known(u32),
    Invalid,
    Unknown,
}

fn remaining_type_arity(resolved: &ResolvedModule, type_ref: &TypeRef) -> RemainingTypeArity {
    let TypeRef::Named {
        name,
        arguments,
        span,
    } = type_ref
    else {
        return match type_ref {
            TypeRef::Record { .. }
            | TypeRef::Tuple { .. }
            | TypeRef::Function { .. }
            | TypeRef::RequirementMerge { .. } => RemainingTypeArity::Known(0),
            TypeRef::Hole { .. } => RemainingTypeArity::Unknown,
            TypeRef::Named { .. } => unreachable!(),
        };
    };
    let Some(declared) = named_type_arity(resolved, name, *span) else {
        return RemainingTypeArity::Unknown;
    };
    if arguments.len() > declared as usize {
        return RemainingTypeArity::Invalid;
    }
    let consumed = arguments
        .iter()
        .filter(|argument| !matches!(argument, TypeRef::Hole { .. }))
        .count() as u32;
    declared
        .checked_sub(consumed)
        .map(RemainingTypeArity::Known)
        .unwrap_or(RemainingTypeArity::Invalid)
}

fn named_type_arity(resolved: &ResolvedModule, name: &str, origin: ByteSpan) -> Option<u32> {
    let target = type_target(resolved, origin)?;
    let symbol = resolved.symbols.iter().find(|symbol| symbol.id == target)?;
    if symbol.kind == SymbolKind::TypeParameter {
        return resolved
            .declarations
            .iter()
            .filter(|declaration| declaration.span() == symbol.origin)
            .filter_map(declaration_type_parameters)
            .flatten()
            .find(|parameter| parameter.name == symbol.spelling)
            .map(|parameter| parameter.arity);
    }
    if symbol
        .canonical
        .as_deref()
        .is_some_and(|canonical| canonical.starts_with("std/prelude::"))
    {
        return crate::prelude::type_constructor_arity(name);
    }
    if let Some(arity) = resolved.declarations.iter().find_map(|declaration| {
        let (name_span, parameters) = match declaration {
            SurfaceDecl::Newtype {
                name_span,
                type_parameters,
                ..
            }
            | SurfaceDecl::Alias {
                name_span,
                type_parameters,
                ..
            }
            | SurfaceDecl::Type {
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
        (*name_span == symbol.origin).then_some(parameters.len() as u32)
    }) {
        return Some(arity);
    }
    resolved
        .imports
        .iter()
        .find(|import| import.symbol == target && import.export.namespace == "type")
        .and_then(|import| match import.export.representation.as_ref() {
            Some(seseragi_syntax::InterfaceType::TypeConstructor { arity, .. }) => Some(*arity),
            _ => Some(import.export.scheme.type_parameters.len() as u32),
        })
}

fn declaration_type_parameters(
    declaration: &SurfaceDecl,
) -> Option<std::slice::Iter<'_, seseragi_syntax::TypeParameter>> {
    match declaration {
        SurfaceDecl::EffectFn {
            type_parameters, ..
        }
        | SurfaceDecl::Fn {
            type_parameters, ..
        }
        | SurfaceDecl::Newtype {
            type_parameters, ..
        }
        | SurfaceDecl::Alias {
            type_parameters, ..
        }
        | SurfaceDecl::Type {
            type_parameters, ..
        }
        | SurfaceDecl::Struct {
            type_parameters, ..
        }
        | SurfaceDecl::Trait {
            type_parameters, ..
        }
        | SurfaceDecl::Operator {
            type_parameters, ..
        }
        | SurfaceDecl::Impl {
            type_parameters, ..
        }
        | SurfaceDecl::Instance {
            type_parameters, ..
        } => Some(type_parameters.iter()),
        SurfaceDecl::Let { .. } => None,
    }
}

fn type_ref_span(type_ref: &TypeRef) -> ByteSpan {
    match type_ref {
        TypeRef::Named { span, .. }
        | TypeRef::Hole { span }
        | TypeRef::Record { span, .. }
        | TypeRef::Tuple { span, .. }
        | TypeRef::Function { span, .. }
        | TypeRef::RequirementMerge { span, .. } => *span,
    }
}

fn type_target(resolved: &ResolvedModule, origin: ByteSpan) -> Option<SymbolId> {
    resolved
        .references
        .iter()
        .find(|reference| {
            reference.namespace == SymbolNamespace::Type && reference.origin == origin
        })
        .and_then(|reference| reference.target)
}

fn walk_module_types(declarations: &[SurfaceDecl], visit: &mut impl FnMut(&TypeRef)) {
    for declaration in declarations {
        match declaration {
            SurfaceDecl::Let { type_ref, .. } => type_ref
                .iter()
                .for_each(|type_ref| walk_type(type_ref, visit)),
            SurfaceDecl::EffectFn {
                parameters,
                return_type,
                requirements,
                failure,
                constraints,
                ..
            } => {
                for parameter in parameters {
                    walk_type(&parameter.type_ref, visit);
                }
                return_type
                    .iter()
                    .for_each(|type_ref| walk_type(type_ref, visit));
                for requirement in requirements {
                    if let seseragi_syntax::SurfaceRequirement::Field { type_ref, .. } = requirement
                    {
                        walk_type(type_ref, visit);
                    }
                }
                failure
                    .iter()
                    .for_each(|type_ref| walk_type(type_ref, visit));
                for constraint in constraints {
                    for argument in &constraint.arguments {
                        walk_type(argument, visit);
                    }
                }
            }
            SurfaceDecl::Fn {
                parameters,
                return_type,
                constraints,
                ..
            }
            | SurfaceDecl::Operator {
                parameters,
                return_type,
                constraints,
                ..
            } => {
                for parameter in parameters {
                    walk_type(&parameter.type_ref, visit);
                }
                walk_type(return_type, visit);
                for constraint in constraints {
                    for argument in &constraint.arguments {
                        walk_type(argument, visit);
                    }
                }
            }
            SurfaceDecl::Newtype { representation, .. } => walk_type(representation, visit),
            SurfaceDecl::Alias { target, .. } => walk_type(target, visit),
            SurfaceDecl::Type { variants, .. } => {
                for variant in variants {
                    variant
                        .payload
                        .iter()
                        .for_each(|type_ref| walk_type(type_ref, visit));
                }
            }
            SurfaceDecl::Struct { fields, .. } => {
                for field in fields {
                    walk_type(&field.type_ref, visit);
                }
            }
            SurfaceDecl::Trait {
                constraints,
                methods,
                ..
            }
            | SurfaceDecl::Instance {
                constraints,
                methods,
                ..
            } => {
                for constraint in constraints {
                    for argument in &constraint.arguments {
                        walk_type(argument, visit);
                    }
                }
                for method in methods {
                    for parameter in &method.parameters {
                        walk_type(&parameter.type_ref, visit);
                    }
                    walk_type(&method.return_type, visit);
                    for constraint in &method.constraints {
                        for argument in &constraint.arguments {
                            walk_type(argument, visit);
                        }
                    }
                }
            }
            SurfaceDecl::Impl {
                target,
                constraints,
                members,
                ..
            } => {
                walk_type(target, visit);
                for constraint in constraints {
                    for argument in &constraint.arguments {
                        walk_type(argument, visit);
                    }
                }
                for member in members {
                    match member {
                        SurfaceImplMember::Method { method, .. } => {
                            for parameter in &method.parameters {
                                walk_type(&parameter.type_ref, visit);
                            }
                            walk_type(&method.return_type, visit);
                            for constraint in &method.constraints {
                                for argument in &constraint.arguments {
                                    walk_type(argument, visit);
                                }
                            }
                        }
                        SurfaceImplMember::Operator {
                            parameters,
                            return_type,
                            ..
                        } => {
                            for parameter in parameters {
                                walk_type(&parameter.type_ref, visit);
                            }
                            walk_type(return_type, visit);
                        }
                    }
                }
            }
        }
        walk_declaration_expression_types(declaration, visit);
    }
}

fn walk_declaration_expression_types(declaration: &SurfaceDecl, visit: &mut impl FnMut(&TypeRef)) {
    match declaration {
        SurfaceDecl::Let { body, .. }
        | SurfaceDecl::EffectFn { body, .. }
        | SurfaceDecl::Fn { body, .. }
        | SurfaceDecl::Operator { body, .. } => {
            if let Some(body) = body {
                walk_expression_types(body, visit);
            }
        }
        SurfaceDecl::Trait { methods, .. } | SurfaceDecl::Instance { methods, .. } => {
            for method in methods {
                if let Some(body) = &method.body {
                    walk_expression_types(body, visit);
                }
            }
        }
        SurfaceDecl::Impl { members, .. } => {
            for member in members {
                let body = match member {
                    SurfaceImplMember::Method { method, .. } => method.body.as_ref(),
                    SurfaceImplMember::Operator { body, .. } => body.as_ref(),
                };
                if let Some(body) = body {
                    walk_expression_types(body, visit);
                }
            }
        }
        SurfaceDecl::Newtype { .. }
        | SurfaceDecl::Alias { .. }
        | SurfaceDecl::Type { .. }
        | SurfaceDecl::Struct { .. } => {}
    }
}

fn walk_expression_types(expression: &SurfaceExpr, visit: &mut impl FnMut(&TypeRef)) {
    match expression {
        SurfaceExpr::Template { parts, .. } => {
            for part in parts {
                if let SurfaceTemplatePart::Interpolation { value, .. } = part {
                    walk_expression_types(value, visit);
                }
            }
        }
        SurfaceExpr::Member { receiver, .. }
        | SurfaceExpr::Prefix {
            operand: receiver, ..
        }
        | SurfaceExpr::Grouped {
            value: receiver, ..
        } => walk_expression_types(receiver, visit),
        SurfaceExpr::Lambda {
            parameter, body, ..
        } => {
            if let Some(type_ref) = &parameter.type_ref {
                walk_type(type_ref, visit);
            }
            walk_expression_types(body, visit);
        }
        SurfaceExpr::Application {
            function, argument, ..
        }
        | SurfaceExpr::Assignment {
            target: function,
            value: argument,
            ..
        }
        | SurfaceExpr::Index {
            receiver: function,
            index: argument,
            ..
        }
        | SurfaceExpr::Binary {
            left: function,
            right: argument,
            ..
        } => {
            walk_expression_types(function, visit);
            walk_expression_types(argument, visit);
        }
        SurfaceExpr::EffectfulFor { source, body, .. } => {
            walk_expression_types(source, visit);
            walk_expression_types(body, visit);
        }
        SurfaceExpr::Tuple { elements, .. }
        | SurfaceExpr::Array { elements, .. }
        | SurfaceExpr::List { elements, .. } => {
            for element in elements {
                walk_expression_types(element, visit);
            }
        }
        SurfaceExpr::Record { items, .. } => {
            for item in items {
                walk_expression_types(item.value(), visit);
            }
        }
        SurfaceExpr::Struct {
            type_arguments,
            items,
            ..
        } => {
            if let Some(type_arguments) = type_arguments {
                for type_argument in type_arguments {
                    walk_type(type_argument, visit);
                }
            }
            for item in items {
                walk_expression_types(item.value(), visit);
            }
        }
        SurfaceExpr::ArrayComprehension {
            element, clauses, ..
        }
        | SurfaceExpr::ListComprehension {
            element, clauses, ..
        } => {
            walk_expression_types(element, visit);
            for clause in clauses {
                match clause {
                    SurfaceComprehensionClause::Generator { source, .. } => {
                        walk_expression_types(source, visit)
                    }
                    SurfaceComprehensionClause::Guard { condition, .. } => {
                        walk_expression_types(condition, visit)
                    }
                }
            }
        }
        SurfaceExpr::InfixChain { first, steps, .. } => {
            walk_expression_types(first, visit);
            for step in steps {
                walk_expression_types(&step.operand, visit);
            }
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            walk_expression_types(condition, visit);
            walk_expression_types(then_branch, visit);
            walk_expression_types(else_branch, visit);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            walk_expression_types(scrutinee, visit);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    walk_expression_types(guard, visit);
                }
                walk_expression_types(&arm.body, visit);
            }
        }
        SurfaceExpr::Block { items, result, .. } => {
            for item in items {
                match item {
                    SurfaceBlockItem::Let {
                        type_ref, value, ..
                    } => {
                        if let Some(type_ref) = type_ref {
                            walk_type(type_ref, visit);
                        }
                        walk_expression_types(value, visit);
                    }
                    SurfaceBlockItem::Function {
                        parameters,
                        return_type,
                        constraints,
                        value,
                        ..
                    } => {
                        for parameter in parameters {
                            walk_type(&parameter.type_ref, visit);
                        }
                        walk_type(return_type, visit);
                        for constraint in constraints {
                            for argument in &constraint.arguments {
                                walk_type(argument, visit);
                            }
                        }
                        walk_expression_types(value, visit);
                    }
                }
            }
            walk_expression_types(result, visit);
        }
        SurfaceExpr::Do { items, result, .. } => {
            for item in items {
                let value = match item {
                    SurfaceDoItem::Bind { value, .. }
                    | SurfaceDoItem::Let { value, .. }
                    | SurfaceDoItem::Expression { value, .. } => value,
                };
                walk_expression_types(value, visit);
            }
            if let Some(result) = result {
                walk_expression_types(result, visit);
            }
        }
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::Float { .. }
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Char { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Name { .. }
        | SurfaceExpr::Error { .. } => {}
    }
}

fn walk_type(type_ref: &TypeRef, visit: &mut impl FnMut(&TypeRef)) {
    visit(type_ref);
    match type_ref {
        TypeRef::Named { arguments, .. } => {
            for argument in arguments {
                walk_type(argument, visit);
            }
        }
        TypeRef::Record { fields, .. } => {
            for field in fields {
                walk_type(&field.type_ref, visit);
            }
        }
        TypeRef::Tuple { elements, .. } => {
            for element in elements {
                walk_type(element, visit);
            }
        }
        TypeRef::Function {
            parameter, result, ..
        } => {
            walk_type(parameter, visit);
            walk_type(result, visit);
        }
        TypeRef::RequirementMerge { operands, .. } => {
            for operand in operands {
                walk_type(operand, visit);
            }
        }
        TypeRef::Hole { .. } => {}
    }
}

fn error(code: &str, message_key: &str, span: ByteSpan) -> Diagnostic {
    Diagnostic {
        type_difference: None,
        id: String::new(),
        code: code.to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: ByteRange {
            start: span.start,
            end: span.end,
        },
        related: Vec::new(),
        fixes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn enforces_restricted_requirement_merge_positions_and_fields() {
        let valid = crate::semantic_diagnostics(
            "artifact/valid-requirement-merge/main.ssrg",
            concat!(
                "pub struct Clock {}\n",
                "pub alias Timed<R, E, A> = Effect<R & { clock: Clock }, E, A>\n",
                "pub alias Same = Effect<{ clock: Clock } & { clock: Clock }, Never, Unit>\n",
            ),
        );
        assert!(valid.diagnostics.is_empty(), "{:#?}", valid.diagnostics);

        let invalid_position = crate::semantic_diagnostics(
            "artifact/invalid-requirement-merge/main.ssrg",
            "alias Invalid = Int & String\n",
        );
        assert_eq!(invalid_position.diagnostics.len(), 1);
        assert_eq!(invalid_position.diagnostics[0].code, "SES-T0501");

        let conflict = crate::semantic_diagnostics(
            "artifact/conflicting-requirement-merge/main.ssrg",
            "alias Invalid = Effect<{ service: Int } & { service: String }, Never, Unit>\n",
        );
        assert!(conflict
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-E0001"));

        let optional = crate::semantic_diagnostics(
            "artifact/optional-requirement-merge/main.ssrg",
            "alias Invalid = Effect<{ service?: Int } & {}, Never, Unit>\n",
        );
        assert!(optional
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-T0501"));
    }

    #[test]
    fn reports_alias_arity_cycles_and_private_exposure() {
        let arity = crate::semantic_diagnostics(
            "artifact/alias-arity/main.ssrg",
            "alias Pair<A> = { left: A, right: A }\npub fn broken value: Pair<Int, String> -> Int = 0\n",
        );
        assert!(arity.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0601" && diagnostic.message_key == "alias.arity-mismatch"
        }));

        let cycle = crate::semantic_diagnostics(
            "artifact/alias-cycle/main.ssrg",
            "alias First = Second\nalias Second = First\n",
        );
        assert_eq!(
            cycle
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.message_key == "alias.cycle")
                .count(),
            2
        );

        let exposure = crate::semantic_diagnostics(
            "artifact/alias-private/main.ssrg",
            "struct Secret { value: Int }\npub alias Public = Secret\n",
        );
        assert!(exposure.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0603"
                && diagnostic.message_key == "alias.private-type-exposure"
        }));
    }

    #[test]
    fn accepts_task_with_one_type_argument() {
        let artifact = crate::semantic_diagnostics(
            "artifact/task-alias/main.ssrg",
            "pub fn identity task: Task<Int> -> Task<Int> = task\n",
        );
        assert!(
            artifact.diagnostics.is_empty(),
            "{:#?}",
            artifact.diagnostics
        );
    }

    #[test]
    fn reports_nested_local_alias_arity_mismatches_without_ignoring_annotations() {
        let artifact = crate::semantic_diagnostics(
            "artifact/local-alias-arity/main.ssrg",
            concat!(
                "alias Pair<A> = { left: A, right: A }\n",
                "fn broken -> Int = {\n",
                "  let value: { direct: Pair<Int, String>, nested: Array<(Pair<Int, String>, Int -> Pair<Int, String>)> } = ()\n",
                "  fn local item: Pair<Int, String> -> Pair<Int, String>\n",
                "    where Show<Pair<Int, String>> = item\n",
                "  0\n",
                "}\n",
            ),
        );
        let alias_diagnostics = artifact
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "SES-T0601")
            .collect::<Vec<_>>();

        assert_eq!(alias_diagnostics.len(), 6, "{:#?}", artifact.diagnostics);
        assert!(alias_diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message_key == "alias.arity-mismatch"));
    }

    #[test]
    fn accepts_nested_local_alias_applications_with_the_declared_arity() {
        let artifact = crate::semantic_diagnostics(
            "artifact/valid-local-alias-arity/main.ssrg",
            concat!(
                "alias Pair<A> = { left: A, right: A }\n",
                "fn valid -> Int = {\n",
                "  let value: { direct: Pair<Int>, nested: Array<(Pair<Int>, Int -> Pair<Int>)> } = ()\n",
                "  fn local item: Pair<Int> -> Pair<Int>\n",
                "    where Show<Pair<Int>> = item\n",
                "  0\n",
                "}\n",
            ),
        );

        assert!(
            artifact
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "SES-T0601"),
            "{:#?}",
            artifact.diagnostics
        );
    }

    #[test]
    fn accepts_higher_kinded_alias_arguments_with_matching_remaining_arity() {
        let artifact = crate::semantic_diagnostics(
            "artifact/valid-higher-kinded-alias/main.ssrg",
            concat!(
                "alias StateT<S, M<_>, A> = S -> M<(A, S)>\n",
                "alias OptionalState<S, A> = StateT<S, Maybe, A>\n",
                "alias EitherState<E, S, A> = StateT<S, Either<E, _>, A>\n",
                "type Box<A> = | Boxed A\n",
                "alias BoxState<S, A> = StateT<S, Box, A>\n",
                "alias Rebind<F<_>, A> = StateT<Int, F, A>\n",
            ),
        );

        assert!(
            artifact.diagnostics.is_empty(),
            "{:#?}",
            artifact.diagnostics
        );
    }

    #[test]
    fn reports_higher_kinded_alias_argument_kind_mismatches() {
        let artifact = crate::semantic_diagnostics(
            "artifact/higher-kinded-alias-kind/main.ssrg",
            concat!(
                "alias StateT<S, M<_>, A> = S -> M<(A, S)>\n",
                "alias ValueAsConstructor<S, A> = StateT<S, Int, A>\n",
                "alias AppliedConstructor<S, A> = StateT<S, Maybe<Int>, A>\n",
                "alias ExcessConstructor<S, A> = StateT<S, Either, A>\n",
                "alias ConstructorAsValue<A> = StateT<Maybe, Maybe, A>\n",
                "alias OverAppliedConstructor<S, A> = StateT<S, Maybe<Int, String>, A>\n",
            ),
        );
        let kind_diagnostics = artifact
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "SES-T0604")
            .collect::<Vec<_>>();

        assert_eq!(kind_diagnostics.len(), 5, "{:#?}", artifact.diagnostics);
        assert!(kind_diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message_key == "alias.kind-mismatch"));
    }

    #[test]
    fn preserves_cycle_diagnostics_for_higher_kinded_aliases() {
        let artifact = crate::semantic_diagnostics(
            "artifact/higher-kinded-alias-cycle/main.ssrg",
            "alias Loop<F<_>, A> = Loop<F, A>\n",
        );

        assert_eq!(
            artifact
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "SES-T0602")
                .count(),
            1,
            "{:#?}",
            artifact.diagnostics
        );
        assert!(artifact
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "SES-T0604"));
    }

    #[test]
    fn treats_nested_task_and_effect_as_the_same_type_in_both_directions() {
        let artifact = crate::semantic_diagnostics(
            "artifact/nested-task-alias/main.ssrg",
            concat!(
                "fn asEffect value: Array<Task<Unit>>\n",
                "  -> Array<Effect<{}, Never, Unit>> = value\n",
                "fn asTask value: Array<Effect<{}, Never, Unit>>\n",
                "  -> Array<Task<Unit>> = value\n",
            ),
        );
        assert!(
            artifact.diagnostics.is_empty(),
            "{:#?}",
            artifact.diagnostics
        );
    }
}
