use super::*;
use seseragi_syntax::SurfaceImplMember;

/// A member has an API owner even when it is deliberately absent from lexical
/// value scope. These identities come from resolved traits or typed inherent
/// declarations, never from an unqualified call's display name.
pub(in crate::query) struct Member {
    pub identity: String,
    pub name: String,
    pub owner: String,
    pub inherent: bool,
    pub resolved_symbol: Option<SymbolId>,
    pub definition: Option<ByteSpan>,
    pub own_declaration: Option<ByteSpan>,
    pub deprecation: Option<Deprecation>,
    pub callable: Option<AnalysisCallable>,
}

pub(in crate::query) fn catalog(resolved: &ResolvedModule, typed: &TypedModule) -> Vec<Member> {
    let notices = resolved
        .deprecations
        .iter()
        .map(|entry| {
            (
                (entry.declaration.start, entry.declaration.end),
                &entry.deprecation,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut result = BTreeMap::<String, Member>::new();
    for declaration in &resolved.declarations {
        match declaration {
            SurfaceDecl::Trait { methods, .. } => {
                for method in methods {
                    let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::TraitMethod && symbol.origin == method.name_span
                    }) else {
                        continue;
                    };
                    let Some(identity) = symbol.canonical.clone() else {
                        continue;
                    };
                    let Some((owner, _)) = identity.rsplit_once("::") else {
                        continue;
                    };
                    result.insert(
                        identity.clone(),
                        Member {
                            owner: owner.to_owned(),
                            identity,
                            name: method.name.clone(),
                            inherent: false,
                            resolved_symbol: Some(symbol.id),
                            definition: Some(method.name_span),
                            own_declaration: Some(method.span),
                            deprecation: notices
                                .get(&(method.span.start, method.span.end))
                                .map(|notice| (*notice).clone()),
                            callable: None,
                        },
                    );
                }
            }
            SurfaceDecl::Impl { members, .. } => {
                for member in members {
                    let SurfaceImplMember::Method { method, .. } = member else {
                        continue;
                    };
                    let Some(TypedDecl::Fn { symbol, scheme, parameters, origin, .. }) = typed.declarations.iter().find(|declaration| matches!(declaration, TypedDecl::Fn { origin, .. } if *origin == method.span)) else { continue; };
                    let Some((owner, name)) = symbol.rsplit_once("::") else {
                        continue;
                    };
                    if name != method.name {
                        continue;
                    }
                    let callable = super::super::callable_from_parts(
                        symbol,
                        &method.name,
                        &resolved.module,
                        &scheme.type_parameters,
                        parameters,
                        &scheme.type_ref,
                        &scheme.constraints,
                    );
                    result.insert(
                        symbol.clone(),
                        Member {
                            identity: symbol.clone(),
                            owner: owner.to_owned(),
                            name: method.name.clone(),
                            inherent: true,
                            resolved_symbol: None,
                            definition: Some(method.name_span),
                            own_declaration: Some(*origin),
                            deprecation: notices
                                .get(&(method.span.start, method.span.end))
                                .map(|notice| (*notice).clone()),
                            callable: Some(callable),
                        },
                    );
                }
            }
            _ => {}
        }
    }
    for import in &resolved.imports {
        if let Some(owner) = import
            .member_owner
            .and_then(|id| resolved.symbols.get(id.0 as usize))
            .and_then(|symbol| symbol.canonical.as_ref())
        {
            if import.export.namespace == "method"
                && import.export.declaration_kind.as_deref() == Some("inherent-method")
            {
                result
                    .entry(import.export.symbol.clone())
                    .or_insert_with(|| Member {
                        identity: import.export.symbol.clone(),
                        owner: owner.clone(),
                        name: import.export.name.clone(),
                        inherent: true,
                        resolved_symbol: Some(import.symbol),
                        definition: None,
                        own_declaration: None,
                        deprecation: import.export.deprecation.clone(),
                        callable: super::super::callable_from_interface_export(&import.export),
                    });
            }
        }
        if import.export.namespace == "trait" {
            for method in &import.export.methods {
                let identity = format!("{}::{}", import.export.symbol, method.name);
                result.entry(identity.clone()).or_insert_with(|| Member {
                    identity,
                    owner: import.export.symbol.clone(),
                    name: method.name.clone(),
                    inherent: false,
                    resolved_symbol: None,
                    definition: None,
                    own_declaration: None,
                    deprecation: method.deprecation.clone(),
                    callable: None,
                });
            }
        }
    }
    result.into_values().collect()
}

pub(in crate::query) fn selected_references<'a>(
    resolved: &ResolvedModule,
    typed: &TypedModule,
    members: &'a [Member],
) -> Vec<(&'a Member, ByteSpan)> {
    if members.is_empty() {
        return Vec::new();
    }
    let members = members
        .iter()
        .map(|member| (member.identity.as_str(), member))
        .collect::<BTreeMap<_, _>>();
    let sources = super::source_calls::callee_ranges(resolved);
    let mut found = BTreeMap::new();
    let mut select = |expression: &TypedExpr| {
        let (identity, origin, dispatch) = match expression {
            TypedExpr::Call {
                callee,
                origin,
                trait_dispatch,
                ..
            } => (callee.as_str(), *origin, trait_dispatch.as_ref()),
            TypedExpr::Variable { name, origin, .. } => (name.as_str(), *origin, None),
            _ => return,
        };
        let Some(member) = members.get(identity) else {
            return;
        };
        if let Some(dispatch) = dispatch {
            if member.inherent
                || dispatch.trait_identity != member.owner
                || dispatch.method != member.name
            {
                return;
            }
        } else if !member.inherent && matches!(expression, TypedExpr::Call { .. }) {
            return;
        }
        let Some(range) = sources.get(&(origin.start, origin.end)).copied() else {
            return;
        };
        found.insert(
            (range.start, range.end, identity.to_owned()),
            (*member, range),
        );
    };
    for declaration in &typed.declarations {
        match declaration {
            TypedDecl::Fn { body, .. } | TypedDecl::EffectFn { body, .. } => {
                super::super::walk_expression(body, &mut select)
            }
            TypedDecl::Let { value, .. } => super::super::walk_expression(value, &mut select),
            _ => {}
        }
    }
    for instance in &typed.instances {
        if let crate::TypedInstanceImplementation::UserDefined { methods } =
            &instance.implementation
        {
            for method in methods {
                super::super::walk_expression(&method.body, &mut select);
            }
        }
    }
    found.into_values().collect()
}

impl Member {
    pub(in crate::query) fn analysis_symbol(&self, id: u32, module: &str) -> AnalysisSymbol {
        let type_document = self
            .callable
            .as_ref()
            .map(|callable| callable.type_document.type_ref.clone());
        AnalysisSymbol {
            id,
            identity: self.identity.clone(),
            name: self.name.clone(),
            module: super::super::module_from_identity(&self.identity, module),
            namespace: "value".to_owned(),
            kind: if self.inherent {
                "inherent-method"
            } else {
                "trait-method"
            }
            .to_owned(),
            definition: self.definition.unwrap_or(ByteSpan { start: 0, end: 0 }),
            type_name: type_document
                .as_ref()
                .map(super::super::render_document_compact),
            multiline_type_name: type_document
                .as_ref()
                .map(super::super::render_document_multiline),
            type_document,
            callable: self.callable.clone(),
            description: None,
            deprecation: self.deprecation.clone(),
            member_owner: Some(self.owner.clone()),
            scope: ScopeId(0),
        }
    }
}
