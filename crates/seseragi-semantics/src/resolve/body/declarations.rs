use super::{expression, Resolver};
use crate::{ScopeKind, SymbolKind, SymbolNamespace};
use seseragi_syntax::{
    ModuleInterface, SurfaceBlockItem, SurfaceConstraint, SurfaceDecl, SurfaceImplMember,
    SurfaceMethod, SurfaceModule,
};

mod types;

pub(super) use types::resolve_type_ref;
use types::{
    declaration_scope, register_parameters, register_type_parameters, resolve_requirements,
};

pub(super) fn register_module_declarations(resolver: &mut Resolver, declarations: &[SurfaceDecl]) {
    for declaration in declarations {
        match declaration {
            SurfaceDecl::Let {
                name, name_span, ..
            } => {
                resolver.register_module(SymbolNamespace::Value, SymbolKind::Let, name, *name_span);
            }
            SurfaceDecl::Fn {
                name, name_span, ..
            } => {
                resolver.register_module(
                    SymbolNamespace::Value,
                    SymbolKind::Function,
                    name,
                    *name_span,
                );
            }
            SurfaceDecl::EffectFn {
                name, name_span, ..
            } => {
                resolver.register_module(
                    SymbolNamespace::Value,
                    SymbolKind::EffectFunction,
                    name,
                    *name_span,
                );
            }
            SurfaceDecl::Type {
                name,
                name_span,
                variants,
                ..
            } => {
                resolver.register_module(SymbolNamespace::Type, SymbolKind::Type, name, *name_span);
                for variant in variants {
                    resolver.register_module(
                        SymbolNamespace::Value,
                        SymbolKind::Constructor,
                        &variant.name,
                        variant.name_span,
                    );
                }
            }
            SurfaceDecl::Newtype {
                name, name_span, ..
            } => {
                resolver.register_module(SymbolNamespace::Type, SymbolKind::Type, name, *name_span);
                resolver.register_module(
                    SymbolNamespace::Value,
                    SymbolKind::Constructor,
                    name,
                    *name_span,
                );
            }
            SurfaceDecl::Alias {
                name, name_span, ..
            }
            | SurfaceDecl::Struct {
                name, name_span, ..
            } => {
                resolver.register_module(SymbolNamespace::Type, SymbolKind::Type, name, *name_span);
            }
            SurfaceDecl::Trait {
                name,
                name_span,
                methods,
                ..
            } => {
                resolver.register_module(
                    SymbolNamespace::Trait,
                    SymbolKind::Trait,
                    name,
                    *name_span,
                );
                for method in methods {
                    resolver.register_trait_method(name, &method.name, method.name_span);
                }
            }
            SurfaceDecl::Operator {
                spelling,
                spelling_span,
                ..
            } => {
                resolver.register_module(
                    SymbolNamespace::Operator,
                    SymbolKind::Operator,
                    spelling,
                    *spelling_span,
                );
            }
            SurfaceDecl::Impl { .. } | SurfaceDecl::Instance { .. } => {}
        }
    }
}

pub(super) fn resolve_local_function(
    resolver: &mut Resolver,
    parent: crate::ScopeId,
    function: &SurfaceBlockItem,
) {
    let SurfaceBlockItem::Function {
        type_parameters,
        parameters,
        return_type,
        constraints,
        value,
        span,
        ..
    } = function
    else {
        return;
    };
    let scope = resolver.new_scope(parent, ScopeKind::Function, *span);
    register_type_parameters(resolver, scope, type_parameters, *span);
    register_parameters(resolver, scope, parameters);
    resolve_type_ref(resolver, scope, return_type);
    resolve_constraints(resolver, scope, constraints);
    expression::resolve_expression(resolver, scope, value);
}

pub(super) fn register_imports(
    resolver: &mut Resolver,
    interface: &ModuleInterface,
    surface: &SurfaceModule,
) {
    for import in &surface.imports {
        let dependency = interface
            .dependencies
            .iter()
            .find(|dependency| dependency.specifier == import.specifier);
        for item in &import.items {
            let namespace = namespace_from_str(&item.namespace);
            let Some(namespace) = namespace else {
                continue;
            };
            let imported = dependency.and_then(|dependency| {
                dependency
                    .imports
                    .iter()
                    .find(|candidate| candidate.name == item.name)
            });
            let local = item.alias.as_deref().unwrap_or(&item.name);
            resolver.register(
                resolver.module_scope(),
                namespace,
                if namespace == SymbolNamespace::Module {
                    SymbolKind::ModuleImport
                } else {
                    SymbolKind::Imported
                },
                local,
                imported.map(|imported| imported.symbol.clone()),
                import.span,
            );
        }
    }
}

pub(super) fn resolve_declarations(resolver: &mut Resolver, declarations: &[SurfaceDecl]) {
    let module_scope = resolver.module_scope();
    for declaration in declarations {
        match declaration {
            SurfaceDecl::Let { type_ref, body, .. } => {
                if let Some(type_ref) = type_ref {
                    resolve_type_ref(resolver, module_scope, type_ref);
                }
                if let Some(body) = body {
                    expression::resolve_expression(resolver, module_scope, body);
                }
            }
            SurfaceDecl::Fn {
                type_parameters,
                parameters,
                return_type,
                constraints,
                body,
                span,
                ..
            } => {
                let scope = resolver.new_scope(module_scope, ScopeKind::Function, *span);
                register_type_parameters(resolver, scope, type_parameters, *span);
                register_parameters(resolver, scope, parameters);
                resolve_type_ref(resolver, scope, return_type);
                resolve_constraints(resolver, scope, constraints);
                if let Some(body) = body {
                    expression::resolve_expression(resolver, scope, body);
                }
            }
            SurfaceDecl::EffectFn {
                type_parameters,
                parameters,
                return_type,
                requirements,
                failure,
                constraints,
                body,
                span,
                ..
            } => {
                let scope = resolver.new_scope(module_scope, ScopeKind::Function, *span);
                register_type_parameters(resolver, scope, type_parameters, *span);
                register_parameters(resolver, scope, parameters);
                resolve_requirements(resolver, scope, requirements);
                if let Some(return_type) = return_type {
                    resolve_type_ref(resolver, scope, return_type);
                }
                if let Some(failure) = failure {
                    resolve_type_ref(resolver, scope, failure);
                }
                resolve_constraints(resolver, scope, constraints);
                if let Some(body) = body {
                    expression::resolve_expression(resolver, scope, body);
                }
            }
            SurfaceDecl::Newtype {
                type_parameters,
                representation,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                resolve_type_ref(resolver, scope, representation);
            }
            SurfaceDecl::Alias {
                type_parameters,
                target,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                resolve_type_ref(resolver, scope, target);
            }
            SurfaceDecl::Type {
                type_parameters,
                variants,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        resolve_type_ref(resolver, scope, payload);
                    }
                }
            }
            SurfaceDecl::Struct {
                type_parameters,
                fields,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                for field in fields {
                    resolve_type_ref(resolver, scope, &field.type_ref);
                }
            }
            SurfaceDecl::Trait {
                type_parameters,
                constraints,
                methods,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                resolve_constraints(resolver, scope, constraints);
                resolve_methods(resolver, scope, methods);
            }
            SurfaceDecl::Operator {
                type_parameters,
                parameters,
                return_type,
                constraints,
                body,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                register_parameters(resolver, scope, parameters);
                resolve_type_ref(resolver, scope, return_type);
                resolve_constraints(resolver, scope, constraints);
                if let Some(body) = body {
                    expression::resolve_expression(resolver, scope, body);
                }
            }
            SurfaceDecl::Impl {
                type_parameters,
                target,
                constraints,
                members,
                span,
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                resolve_type_ref(resolver, scope, target);
                resolve_constraints(resolver, scope, constraints);
                resolve_impl_members(resolver, scope, members);
            }
            SurfaceDecl::Instance {
                type_parameters,
                trait_name,
                trait_name_span,
                arguments,
                constraints,
                methods,
                span,
                ..
            } => {
                let scope = declaration_scope(resolver, module_scope, type_parameters, *span);
                resolver.reference(
                    scope,
                    SymbolNamespace::Trait,
                    trait_name,
                    *trait_name_span,
                    true,
                );
                for argument in arguments {
                    resolve_type_ref(resolver, scope, argument);
                }
                resolve_constraints(resolver, scope, constraints);
                resolve_methods(resolver, scope, methods);
            }
        }
    }
}

fn resolve_impl_members(
    resolver: &mut Resolver,
    parent: crate::ScopeId,
    members: &[SurfaceImplMember],
) {
    for member in members {
        match member {
            SurfaceImplMember::Method { method, .. } => {
                resolve_methods(resolver, parent, std::slice::from_ref(method));
            }
            SurfaceImplMember::Operator {
                self_span,
                parameters,
                return_type,
                body,
                span,
                ..
            } => {
                let member_scope = resolver.new_scope(parent, ScopeKind::Function, *span);
                resolver.register(
                    member_scope,
                    SymbolNamespace::Value,
                    SymbolKind::Parameter,
                    "self",
                    None,
                    *self_span,
                );
                register_parameters(resolver, member_scope, parameters);
                resolve_type_ref(resolver, member_scope, return_type);
                if let Some(body) = body {
                    expression::resolve_expression(resolver, member_scope, body);
                }
            }
        }
    }
}

fn resolve_methods(resolver: &mut Resolver, parent: crate::ScopeId, methods: &[SurfaceMethod]) {
    for method in methods {
        let method_scope = resolver.new_scope(parent, ScopeKind::Function, method.span);
        register_type_parameters(resolver, method_scope, &method.type_parameters, method.span);
        register_parameters(resolver, method_scope, &method.parameters);
        resolve_type_ref(resolver, method_scope, &method.return_type);
        resolve_constraints(resolver, method_scope, &method.constraints);
        if let Some(body) = &method.body {
            expression::resolve_expression(resolver, method_scope, body);
        }
    }
}

fn resolve_constraints(
    resolver: &mut Resolver,
    scope: crate::ScopeId,
    constraints: &[SurfaceConstraint],
) {
    for constraint in constraints {
        resolver.reference(
            scope,
            SymbolNamespace::Trait,
            &constraint.name,
            constraint.name_span,
            true,
        );
        for argument in &constraint.arguments {
            resolve_type_ref(resolver, scope, argument);
        }
    }
}

fn namespace_from_str(namespace: &str) -> Option<SymbolNamespace> {
    match namespace {
        "type" => Some(SymbolNamespace::Type),
        "value" => Some(SymbolNamespace::Value),
        "trait" => Some(SymbolNamespace::Trait),
        "namespace" => Some(SymbolNamespace::Module),
        "operator" => Some(SymbolNamespace::Operator),
        _ => None,
    }
}
