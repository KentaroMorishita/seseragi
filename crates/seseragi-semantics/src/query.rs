use crate::{
    typed::standard_reference_callables, ResolvedModule, ScopeId, SymbolId, SymbolKind,
    SymbolNamespace, TypedComprehensionClause, TypedConstraint, TypedDecl, TypedDoStatement,
    TypedEffect, TypedExpr, TypedInstanceMethod, TypedModule, TypedMonadDoStatement,
    TypedParameter, TypedPattern, TypedScheme, TypedType,
};
use serde::Serialize;
use seseragi_syntax::{
    ByteSpan, DiagnosticArtifact, InterfaceConstraint, InterfaceExport, InterfaceScheme,
    InterfaceType,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisCallable {
    pub identity: String,
    pub name: String,
    pub module: String,
    pub type_parameters: Vec<String>,
    pub parameters: Vec<AnalysisParameter>,
    pub result: String,
    pub constraints: Vec<String>,
    pub signature: String,
    pub remaining_parameters: Vec<AnalysisParameter>,
}

impl AnalysisCallable {
    fn with_remaining(&self, applied: usize) -> Self {
        let mut callable = self.clone();
        callable.remaining_parameters = callable
            .parameters
            .iter()
            .skip(applied.min(callable.parameters.len()))
            .cloned()
            .collect();
        callable
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSymbol {
    pub id: u32,
    pub identity: String,
    pub name: String,
    pub module: String,
    pub namespace: String,
    pub kind: String,
    pub definition: ByteSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callable: Option<AnalysisCallable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip)]
    scope: ScopeId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSymbolOccurrence {
    pub range: ByteSpan,
    pub symbol: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisTypeOccurrence {
    pub range: ByteSpan,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisCallableOccurrence {
    pub range: ByteSpan,
    pub callable: AnalysisCallable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisReferenceItem {
    pub identity: String,
    pub name: String,
    pub module: String,
    pub category: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub description: String,
    pub type_parameters: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisDocument {
    pub schema: u32,
    pub source: String,
    pub module: String,
    pub diagnostics: DiagnosticArtifact,
    pub symbols: Vec<AnalysisSymbol>,
    pub symbol_occurrences: Vec<AnalysisSymbolOccurrence>,
    pub type_occurrences: Vec<AnalysisTypeOccurrence>,
    pub callable_occurrences: Vec<AnalysisCallableOccurrence>,
    pub standard_library: Vec<AnalysisReferenceItem>,
    #[serde(skip)]
    scopes: Vec<crate::ResolvedScope>,
}

impl AnalysisDocument {
    pub fn diagnostics(&self) -> &DiagnosticArtifact {
        &self.diagnostics
    }

    pub fn symbol_at(&self, position: usize) -> Option<&AnalysisSymbol> {
        let occurrence =
            smallest_containing(&self.symbol_occurrences, position, |item| item.range)?;
        self.symbols.get(occurrence.symbol as usize)
    }

    pub fn type_at(&self, position: usize) -> Option<&AnalysisTypeOccurrence> {
        smallest_containing(&self.type_occurrences, position, |item| item.range)
    }

    pub fn callable_at(&self, position: usize) -> Option<&AnalysisCallable> {
        smallest_containing(&self.callable_occurrences, position, |item| item.range)
            .map(|item| &item.callable)
            .or_else(|| self.symbol_at(position)?.callable.as_ref())
    }

    pub fn definition_of(&self, position: usize) -> Option<ByteSpan> {
        self.symbol_at(position)
            .map(|symbol| symbol.definition)
            .filter(|range| range.end > range.start)
    }

    pub fn visible_symbols(&self, position: usize) -> Vec<&AnalysisSymbol> {
        let Some(scope) = self.innermost_scope(position) else {
            return Vec::new();
        };
        let mut visible_scopes = BTreeSet::new();
        let mut next = Some(scope);
        while let Some(scope_id) = next {
            visible_scopes.insert(scope_id);
            next = self
                .scopes
                .get(scope_id.0 as usize)
                .and_then(|scope| scope.parent);
        }
        self.symbols
            .iter()
            .filter(|symbol| {
                visible_scopes.contains(&symbol.scope)
                    && (symbol.scope == ScopeId(0) || symbol.definition.start <= position)
            })
            .collect()
    }

    pub fn standard_library_catalog(&self) -> &[AnalysisReferenceItem] {
        &self.standard_library
    }

    fn innermost_scope(&self, position: usize) -> Option<ScopeId> {
        self.scopes
            .iter()
            .filter(|scope| contains(scope.origin, position))
            .min_by_key(|scope| span_length(scope.origin))
            .map(|scope| scope.id)
            .or(Some(ScopeId(0)))
    }
}

pub fn analysis_document(
    diagnostics: DiagnosticArtifact,
    resolved: ResolvedModule,
    typed: &TypedModule,
) -> AnalysisDocument {
    let catalog = standard_library_catalog();
    let catalog_callables = catalog_callables(&catalog);
    let (symbol_types, local_callables) = collect_symbol_metadata(&resolved, typed);
    let import_callables = imported_callables(&resolved);
    let import_definitions = resolved
        .imports
        .iter()
        .map(|import| (import.symbol, import.origin))
        .collect::<BTreeMap<_, _>>();
    let symbols = resolved
        .symbols
        .iter()
        .map(|symbol| {
            let identity = symbol
                .canonical
                .clone()
                .unwrap_or_else(|| format!("{}::local({})", resolved.module, symbol.id.0));
            let callable = local_callables
                .get(&symbol.id)
                .or_else(|| import_callables.get(&symbol.id))
                .or_else(|| catalog_callables.get(&identity))
                .cloned();
            AnalysisSymbol {
                id: symbol.id.0,
                identity: identity.clone(),
                name: symbol.spelling.clone(),
                module: module_from_identity(&identity, &resolved.module),
                namespace: namespace_name(symbol.namespace).to_owned(),
                kind: kind_name(symbol.kind).to_owned(),
                definition: if symbol.origin.end > symbol.origin.start {
                    symbol.origin
                } else {
                    import_definitions
                        .get(&symbol.id)
                        .copied()
                        .unwrap_or(symbol.origin)
                },
                type_name: symbol_types
                    .get(&symbol.id)
                    .map(render_type)
                    .or_else(|| callable.as_ref().map(callable_type)),
                callable,
                description: standard_description(&identity).map(str::to_owned),
                scope: symbol.scope,
            }
        })
        .collect::<Vec<_>>();
    let mut symbol_occurrences = resolved
        .symbols
        .iter()
        .filter(|symbol| symbol.origin.end > symbol.origin.start)
        .map(|symbol| AnalysisSymbolOccurrence {
            range: symbol.origin,
            symbol: symbol.id.0,
        })
        .collect::<Vec<_>>();
    symbol_occurrences.extend(resolved.references.iter().filter_map(|reference| {
        reference.target.map(|target| AnalysisSymbolOccurrence {
            range: reference.origin,
            symbol: target.0,
        })
    }));

    let mut type_occurrences = Vec::new();
    let mut call_requests = Vec::new();
    collect_typed_occurrences(typed, &mut type_occurrences, &mut call_requests);
    type_occurrences.extend(symbol_occurrences.iter().filter_map(|occurrence| {
        let symbol = symbols.get(occurrence.symbol as usize)?;
        Some(AnalysisTypeOccurrence {
            range: occurrence.range,
            type_name: symbol.type_name.clone()?,
        })
    }));
    deduplicate_type_occurrences(&mut type_occurrences);

    let callable_by_identity = symbols
        .iter()
        .filter_map(|symbol| Some((symbol.identity.clone(), symbol.callable.as_ref()?.clone())))
        .chain(catalog_callables)
        .collect::<BTreeMap<_, _>>();
    let mut callable_occurrences = symbol_occurrences
        .iter()
        .filter_map(|occurrence| {
            let callable = symbols
                .get(occurrence.symbol as usize)?
                .callable
                .as_ref()?
                .clone();
            Some(AnalysisCallableOccurrence {
                range: occurrence.range,
                callable,
            })
        })
        .collect::<Vec<_>>();
    callable_occurrences.extend(call_requests.into_iter().filter_map(
        |(range, identity, applied)| {
            Some(AnalysisCallableOccurrence {
                range,
                callable: callable_by_identity.get(&identity)?.with_remaining(applied),
            })
        },
    ));

    AnalysisDocument {
        schema: 1,
        source: resolved.source.clone(),
        module: resolved.module.clone(),
        diagnostics,
        symbols,
        symbol_occurrences,
        type_occurrences,
        callable_occurrences,
        standard_library: catalog,
        scopes: resolved.scopes,
    }
}

pub fn diagnostics_only_analysis(
    source: impl Into<String>,
    module: impl Into<String>,
    diagnostics: DiagnosticArtifact,
) -> AnalysisDocument {
    AnalysisDocument {
        schema: 1,
        source: source.into(),
        module: module.into(),
        diagnostics,
        symbols: Vec::new(),
        symbol_occurrences: Vec::new(),
        type_occurrences: Vec::new(),
        callable_occurrences: Vec::new(),
        standard_library: standard_library_catalog(),
        scopes: Vec::new(),
    }
}

fn smallest_containing<T>(
    items: &[T],
    position: usize,
    range: impl Fn(&T) -> ByteSpan,
) -> Option<&T> {
    items
        .iter()
        .filter(|item| contains(range(item), position))
        .min_by_key(|item| span_length(range(item)))
}

fn contains(range: ByteSpan, position: usize) -> bool {
    range.start <= position && position < range.end
}

fn span_length(range: ByteSpan) -> usize {
    range.end.saturating_sub(range.start)
}

fn collect_symbol_metadata(
    resolved: &ResolvedModule,
    typed: &TypedModule,
) -> (
    BTreeMap<SymbolId, TypedType>,
    BTreeMap<SymbolId, AnalysisCallable>,
) {
    let mut types = BTreeMap::new();
    let mut callables = BTreeMap::new();
    for declaration in &typed.declarations {
        match declaration {
            TypedDecl::Adt {
                symbol,
                name,
                type_parameters,
                variants,
                ..
            } => {
                if let Some(id) = symbol_by_canonical(resolved, symbol) {
                    types.insert(
                        id,
                        TypedType::Named {
                            name: name.clone(),
                            arguments: type_parameters.iter().map(|name| named(name)).collect(),
                        },
                    );
                }
                for variant in variants {
                    let Some(id) = symbol_by_canonical(resolved, &variant.symbol) else {
                        continue;
                    };
                    types.insert(id, variant.scheme.type_ref.clone());
                    if let Some(callable) = callable_from_scheme(
                        &variant.symbol,
                        &variant.name,
                        &typed.module,
                        &variant.scheme,
                    ) {
                        callables.insert(id, callable);
                    }
                }
            }
            TypedDecl::Struct {
                symbol,
                name,
                type_parameters,
                ..
            } => {
                if let Some(id) = symbol_by_canonical(resolved, symbol) {
                    types.insert(
                        id,
                        TypedType::Named {
                            name: name.clone(),
                            arguments: type_parameters.iter().map(|name| named(name)).collect(),
                        },
                    );
                }
            }
            TypedDecl::Let { symbol, scheme, .. } => {
                if let Some(id) = symbol_by_canonical(resolved, symbol) {
                    types.insert(id, scheme.type_ref.clone());
                    if let Some(callable) =
                        callable_from_scheme(symbol, &symbol_name(symbol), &typed.module, scheme)
                    {
                        callables.insert(id, callable);
                    }
                }
            }
            TypedDecl::Fn {
                symbol,
                scheme,
                parameters,
                body,
                ..
            } => {
                if let Some(id) = symbol_by_canonical(resolved, symbol) {
                    let callable = callable_from_parts(
                        symbol,
                        &symbol_name(symbol),
                        &typed.module,
                        &scheme.type_parameters,
                        parameters,
                        &scheme.type_ref,
                        &scheme.constraints,
                    );
                    types.insert(id, callable_typed_type(parameters, &scheme.type_ref));
                    callables.insert(id, callable);
                }
                collect_parameter_types(resolved, parameters, &mut types);
                collect_local_symbol_types(resolved, body, &mut types);
            }
            TypedDecl::EffectFn {
                symbol,
                parameters,
                effect,
                body,
                ..
            } => {
                if let Some(id) = symbol_by_canonical(resolved, symbol) {
                    let result = effect_type(effect);
                    let callable = callable_from_parts(
                        symbol,
                        &symbol_name(symbol),
                        &typed.module,
                        &[],
                        parameters,
                        &result,
                        &[],
                    );
                    types.insert(id, callable_typed_type(parameters, &result));
                    callables.insert(id, callable);
                }
                collect_parameter_types(resolved, parameters, &mut types);
                collect_local_symbol_types(resolved, body, &mut types);
            }
        }
    }
    for instance in &typed.instances {
        let crate::TypedInstanceImplementation::UserDefined { methods } = &instance.implementation
        else {
            continue;
        };
        for method in methods {
            collect_instance_method(resolved, typed, method, &mut types, &mut callables);
        }
    }
    (types, callables)
}

fn collect_instance_method(
    resolved: &ResolvedModule,
    typed: &TypedModule,
    method: &TypedInstanceMethod,
    types: &mut BTreeMap<SymbolId, TypedType>,
    callables: &mut BTreeMap<SymbolId, AnalysisCallable>,
) {
    let Some(id) = local_symbol_in_range(
        resolved,
        &method.name,
        method.origin,
        SymbolKind::TraitMethod,
    ) else {
        return;
    };
    let identity = resolved
        .symbols
        .get(id.0 as usize)
        .and_then(|symbol| symbol.canonical.clone())
        .unwrap_or_else(|| format!("{}::local({})", typed.module, id.0));
    let callable = callable_from_parts(
        &identity,
        &method.name,
        &typed.module,
        &method.scheme.type_parameters,
        &method.parameters,
        &method.scheme.type_ref,
        &method.scheme.constraints,
    );
    types.insert(
        id,
        callable_typed_type(&method.parameters, &method.scheme.type_ref),
    );
    callables.insert(id, callable);
    collect_parameter_types(resolved, &method.parameters, types);
    collect_local_symbol_types(resolved, &method.body, types);
}

fn collect_parameter_types(
    resolved: &ResolvedModule,
    parameters: &[TypedParameter],
    types: &mut BTreeMap<SymbolId, TypedType>,
) {
    for parameter in parameters {
        let TypedParameter::Named {
            name,
            type_ref,
            origin,
        } = parameter
        else {
            continue;
        };
        if let Some(id) = local_symbol_at(resolved, name, *origin) {
            types.insert(id, type_ref.clone());
        }
    }
}

fn collect_local_symbol_types(
    resolved: &ResolvedModule,
    expression: &TypedExpr,
    types: &mut BTreeMap<SymbolId, TypedType>,
) {
    walk_expression(expression, &mut |expression| match expression {
        TypedExpr::Lambda { parameter, .. } => {
            collect_parameter_types(resolved, std::slice::from_ref(parameter), types);
        }
        TypedExpr::DoBlock { statements, .. } => {
            for statement in statements {
                match statement {
                    TypedDoStatement::PureLet {
                        name,
                        type_ref,
                        origin,
                        ..
                    }
                    | TypedDoStatement::Bind {
                        name,
                        type_ref,
                        origin,
                        ..
                    } => {
                        if let Some(id) = local_symbol_in_span(resolved, name, *origin) {
                            types.insert(id, type_ref.clone());
                        }
                    }
                    TypedDoStatement::Effect { .. } => {}
                }
            }
        }
        TypedExpr::MonadDo { statements, .. } => {
            for statement in statements {
                match statement {
                    TypedMonadDoStatement::PureLet {
                        name,
                        type_ref,
                        origin,
                        ..
                    }
                    | TypedMonadDoStatement::Bind {
                        name,
                        type_ref,
                        origin,
                        ..
                    } => {
                        if let Some(id) = local_symbol_in_span(resolved, name, *origin) {
                            types.insert(id, type_ref.clone());
                        }
                    }
                    TypedMonadDoStatement::Expression { .. } => {}
                }
            }
        }
        _ => {}
    });
    walk_patterns(expression, &mut |pattern| {
        if let TypedPattern::Binding {
            symbol, type_ref, ..
        } = pattern
        {
            types.insert(*symbol, type_ref.clone());
        }
    });
}

fn imported_callables(resolved: &ResolvedModule) -> BTreeMap<SymbolId, AnalysisCallable> {
    resolved
        .imports
        .iter()
        .filter_map(|import| {
            let callable = callable_from_interface_export(&import.export)?;
            Some((import.symbol, callable))
        })
        .collect()
}

fn callable_from_interface_export(export: &InterfaceExport) -> Option<AnalysisCallable> {
    let (parameters, result) = split_interface_function(&export.scheme.type_ref);
    if parameters.is_empty() {
        return None;
    }
    let module = module_from_identity(&export.symbol, "");
    Some(finish_callable(
        export.symbol.clone(),
        export.name.clone(),
        module,
        export
            .scheme
            .type_parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect(),
        parameters
            .into_iter()
            .enumerate()
            .map(|(index, type_ref)| AnalysisParameter {
                name: Some(format!("arg{}", index + 1)),
                type_name: render_interface_type(type_ref),
            })
            .collect(),
        render_interface_type(result),
        render_interface_constraints(&export.scheme.constraints),
    ))
}

fn callable_from_scheme(
    identity: &str,
    name: &str,
    module: &str,
    scheme: &TypedScheme,
) -> Option<AnalysisCallable> {
    let (parameters, result) = split_function(&scheme.type_ref);
    if parameters.is_empty() {
        return None;
    }
    Some(finish_callable(
        identity.to_owned(),
        name.to_owned(),
        module.to_owned(),
        scheme.type_parameters.clone(),
        parameters
            .into_iter()
            .enumerate()
            .map(|(index, type_ref)| AnalysisParameter {
                name: Some(format!("arg{}", index + 1)),
                type_name: render_type(type_ref),
            })
            .collect(),
        render_type(result),
        render_constraints(&scheme.constraints),
    ))
}

fn callable_from_parts(
    identity: &str,
    name: &str,
    module: &str,
    type_parameters: &[String],
    parameters: &[TypedParameter],
    result: &TypedType,
    constraints: &[TypedConstraint],
) -> AnalysisCallable {
    finish_callable(
        identity.to_owned(),
        name.to_owned(),
        module.to_owned(),
        type_parameters.to_vec(),
        parameters
            .iter()
            .map(|parameter| match parameter {
                TypedParameter::ImplicitUnit { type_ref } => AnalysisParameter {
                    name: None,
                    type_name: render_type(type_ref),
                },
                TypedParameter::Named { name, type_ref, .. } => AnalysisParameter {
                    name: Some(name.clone()),
                    type_name: render_type(type_ref),
                },
            })
            .collect(),
        render_type(result),
        render_constraints(constraints),
    )
}

fn finish_callable(
    identity: String,
    name: String,
    module: String,
    type_parameters: Vec<String>,
    parameters: Vec<AnalysisParameter>,
    result: String,
    constraints: Vec<String>,
) -> AnalysisCallable {
    let mut signature = name.clone();
    if !type_parameters.is_empty() {
        signature.push('<');
        signature.push_str(&type_parameters.join(", "));
        signature.push('>');
    }
    for (index, parameter) in parameters.iter().enumerate() {
        signature.push_str(if index == 0 { " " } else { " -> " });
        if let Some(name) = &parameter.name {
            signature.push_str(name);
            signature.push_str(": ");
        }
        if parameter.type_name.contains(" -> ") {
            signature.push('(');
            signature.push_str(&parameter.type_name);
            signature.push(')');
        } else {
            signature.push_str(&parameter.type_name);
        }
    }
    signature.push_str(" -> ");
    signature.push_str(&result);
    if !constraints.is_empty() {
        signature.push_str(" where ");
        signature.push_str(&constraints.join(", "));
    }
    AnalysisCallable {
        identity,
        name,
        module,
        type_parameters,
        parameters,
        result,
        constraints,
        signature,
        remaining_parameters: Vec::new(),
    }
}

fn callable_typed_type(parameters: &[TypedParameter], result: &TypedType) -> TypedType {
    parameters
        .iter()
        .rev()
        .fold(result.clone(), |result, parameter| {
            let parameter = match parameter {
                TypedParameter::ImplicitUnit { type_ref }
                | TypedParameter::Named { type_ref, .. } => type_ref.clone(),
            };
            TypedType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            }
        })
}

fn symbol_by_canonical(resolved: &ResolvedModule, canonical: &str) -> Option<SymbolId> {
    resolved
        .symbols
        .iter()
        .find(|symbol| symbol.canonical.as_deref() == Some(canonical))
        .map(|symbol| symbol.id)
}

fn local_symbol_at(resolved: &ResolvedModule, name: &str, origin: ByteSpan) -> Option<SymbolId> {
    resolved
        .symbols
        .iter()
        .find(|symbol| symbol.spelling == name && symbol.origin == origin)
        .map(|symbol| symbol.id)
}

fn local_symbol_in_span(
    resolved: &ResolvedModule,
    name: &str,
    origin: ByteSpan,
) -> Option<SymbolId> {
    resolved
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.spelling == name
                && symbol.origin.start >= origin.start
                && symbol.origin.end <= origin.end
        })
        .min_by_key(|symbol| span_length(symbol.origin))
        .map(|symbol| symbol.id)
}

fn local_symbol_in_range(
    resolved: &ResolvedModule,
    name: &str,
    origin: ByteSpan,
    kind: SymbolKind,
) -> Option<SymbolId> {
    resolved
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.spelling == name
                && symbol.kind == kind
                && symbol.origin.start >= origin.start
                && symbol.origin.end <= origin.end
        })
        .min_by_key(|symbol| span_length(symbol.origin))
        .map(|symbol| symbol.id)
}

pub fn standard_library_catalog() -> Vec<AnalysisReferenceItem> {
    let mut items = Vec::new();

    for (name, callable) in standard_reference_callables() {
        let callable = finish_callable(
            callable.symbol.clone(),
            name.to_owned(),
            module_from_identity(&callable.symbol, "std/prelude"),
            callable
                .type_parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect(),
            callable
                .parameters
                .iter()
                .enumerate()
                .map(|(index, type_ref)| AnalysisParameter {
                    name: Some(format!("arg{}", index + 1)),
                    type_name: render_type(type_ref),
                })
                .collect(),
            render_type(&callable.result),
            render_constraints(&callable.constraints),
        );
        items.push(reference_from_callable(
            callable,
            standard_category(name, "std/prelude"),
        ));
    }

    for operation in crate::effect_ops::known_effect_operations() {
        let callable = effect_operation_callable(operation);
        items.push(reference_from_callable(callable, "Effect"));
    }

    for sum_type in crate::prelude::SUM_TYPES {
        items.push(AnalysisReferenceItem {
            identity: sum_type.canonical.to_owned(),
            name: sum_type.name.to_owned(),
            module: "std/prelude".to_owned(),
            category: "Maybe / Either".to_owned(),
            kind: "type".to_owned(),
            signature: Some(format!(
                "{}<{}>",
                sum_type.name,
                sum_type.type_parameters.join(", ")
            )),
            description: standard_description(sum_type.canonical)
                .unwrap_or("Standard algebraic data type.")
                .to_owned(),
            type_parameters: sum_type
                .type_parameters
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
            constraints: Vec::new(),
        });
        for variant in sum_type.variants {
            let signature = match variant.payload_parameter {
                Some(index) => format!(
                    "{} -> {}<{}>",
                    sum_type.type_parameters[index],
                    sum_type.name,
                    sum_type.type_parameters.join(", ")
                ),
                None => format!("{}<{}>", sum_type.name, sum_type.type_parameters.join(", ")),
            };
            items.push(AnalysisReferenceItem {
                identity: variant.canonical.to_owned(),
                name: variant.name.to_owned(),
                module: "std/prelude".to_owned(),
                category: "Maybe / Either".to_owned(),
                kind: "constructor".to_owned(),
                signature: Some(signature),
                description: standard_description(variant.canonical)
                    .unwrap_or("Standard data constructor.")
                    .to_owned(),
                type_parameters: sum_type
                    .type_parameters
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect(),
                constraints: Vec::new(),
            });
        }
    }

    for interface in seseragi_project::standard_module_interfaces() {
        for export in &interface.exports {
            let callable = callable_from_interface_export(export);
            let signature = callable
                .as_ref()
                .map(|callable| callable.signature.clone())
                .or_else(|| Some(render_interface_scheme(&export.scheme)));
            items.push(AnalysisReferenceItem {
                identity: export.symbol.clone(),
                name: export.name.clone(),
                module: interface.module.clone(),
                category: standard_category(&export.name, &interface.module).to_owned(),
                kind: export
                    .declaration_kind
                    .clone()
                    .unwrap_or_else(|| export.namespace.clone()),
                signature,
                description: standard_description(&export.symbol)
                    .unwrap_or_else(|| module_description(&interface.module, export))
                    .to_owned(),
                type_parameters: export
                    .scheme
                    .type_parameters
                    .iter()
                    .map(|parameter| parameter.name.clone())
                    .collect(),
                constraints: render_interface_constraints(&export.scheme.constraints),
            });
        }
    }

    for operator in seseragi_syntax::standard_operators() {
        items.push(AnalysisReferenceItem {
            identity: format!("std/prelude::{}", operator.spelling),
            name: operator.spelling.to_owned(),
            module: "std/prelude".to_owned(),
            category: "Operators".to_owned(),
            kind: "operator".to_owned(),
            signature: Some(format!(
                "{} via {}.{}",
                operator.spelling, operator.trait_name, operator.method_name
            )),
            description:
                "Dispatches through the standard trait instance selected by the operand types."
                    .to_owned(),
            type_parameters: Vec::new(),
            constraints: vec![operator.trait_name.to_owned()],
        });
    }
    for operator in seseragi_syntax::standard_trait_operators() {
        items.push(AnalysisReferenceItem {
            identity: format!("std/prelude::{}", operator.spelling),
            name: operator.spelling.to_owned(),
            module: "std/prelude".to_owned(),
            category: "Operators".to_owned(),
            kind: "operator".to_owned(),
            signature: Some(format!(
                "{} via {}.{}",
                operator.spelling, operator.trait_name, operator.method_name
            )),
            description: "Operator spelling for a standard trait method.".to_owned(),
            type_parameters: Vec::new(),
            constraints: vec![operator.trait_name.to_owned()],
        });
    }

    let trait_items = items
        .iter()
        .filter(|item| item.category == "Traits")
        .flat_map(|item| item.constraints.iter())
        .filter_map(|constraint| constraint.split('<').next())
        .filter(|name| !name.is_empty())
        .collect::<BTreeSet<_>>();
    let mut traits = trait_items
        .into_iter()
        .map(|name| AnalysisReferenceItem {
            identity: format!("std/prelude::{name}"),
            name: name.to_owned(),
            module: "std/prelude".to_owned(),
            category: "Traits".to_owned(),
            kind: "trait".to_owned(),
            signature: Some(format!("trait {name}")),
            description: "Standard type class used for generic dispatch.".to_owned(),
            type_parameters: Vec::new(),
            constraints: Vec::new(),
        })
        .collect::<Vec<_>>();
    items.append(&mut traits);

    items.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.identity.cmp(&right.identity))
    });
    items.dedup_by(|left, right| left.identity == right.identity);
    items
}

fn reference_from_callable(callable: AnalysisCallable, category: &str) -> AnalysisReferenceItem {
    AnalysisReferenceItem {
        identity: callable.identity.clone(),
        name: callable.name.clone(),
        module: callable.module.clone(),
        category: category.to_owned(),
        kind: "function".to_owned(),
        signature: Some(callable.signature.clone()),
        description: standard_description(&callable.identity)
            .unwrap_or("Standard function provided by the compiler-owned library surface.")
            .to_owned(),
        type_parameters: callable.type_parameters.clone(),
        constraints: callable.constraints.clone(),
    }
}

fn catalog_callables(catalog: &[AnalysisReferenceItem]) -> BTreeMap<String, AnalysisCallable> {
    let mut callables = standard_reference_callables()
        .into_iter()
        .map(|(name, callable)| {
            let metadata = finish_callable(
                callable.symbol.clone(),
                name.to_owned(),
                module_from_identity(&callable.symbol, "std/prelude"),
                callable
                    .type_parameters
                    .iter()
                    .map(|parameter| parameter.name.clone())
                    .collect(),
                callable
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, type_ref)| AnalysisParameter {
                        name: Some(format!("arg{}", index + 1)),
                        type_name: render_type(type_ref),
                    })
                    .collect(),
                render_type(&callable.result),
                render_constraints(&callable.constraints),
            );
            (metadata.identity.clone(), metadata)
        })
        .collect::<BTreeMap<_, _>>();
    for operation in crate::effect_ops::known_effect_operations() {
        let callable = effect_operation_callable(operation);
        callables.insert(callable.identity.clone(), callable);
    }
    for interface in seseragi_project::standard_module_interfaces() {
        for export in &interface.exports {
            if let Some(callable) = callable_from_interface_export(export) {
                callables.insert(callable.identity.clone(), callable);
            }
        }
    }
    debug_assert!(catalog.iter().all(|item| item.module.starts_with("std/")));
    callables
}

fn effect_operation_callable(operation: crate::KnownEffectOperation) -> AnalysisCallable {
    let named_parameter = |name: &str, type_name: &str| AnalysisParameter {
        name: Some(name.to_owned()),
        type_name: type_name.to_owned(),
    };
    let (type_parameters, parameters, result) = match operation.surface_name {
        "readLine" => (
            Vec::new(),
            vec![named_parameter("unit", "Unit")],
            "Effect<{stdin: Stdin}, StdinError, Maybe<String>>".to_owned(),
        ),
        "print" | "println" => (
            Vec::new(),
            vec![named_parameter("text", "String")],
            "Effect<{console: Console}, ConsoleError, Unit>".to_owned(),
        ),
        "succeed" => (
            vec!["A".to_owned()],
            vec![named_parameter("value", "A")],
            "Effect<{}, Never, A>".to_owned(),
        ),
        "fail" => (
            vec!["E".to_owned()],
            vec![named_parameter("error", "E")],
            "Effect<{}, E, Never>".to_owned(),
        ),
        "mapError" => (
            vec![
                "R".to_owned(),
                "E".to_owned(),
                "F".to_owned(),
                "A".to_owned(),
            ],
            vec![
                named_parameter("mapFailure", "E -> F"),
                named_parameter("effect", "Effect<R, E, A>"),
            ],
            "Effect<R, F, A>".to_owned(),
        ),
        "fromEither" => (
            vec!["E".to_owned(), "A".to_owned()],
            vec![named_parameter("value", "Either<E, A>")],
            "Effect<{}, E, A>".to_owned(),
        ),
        _ => (Vec::new(), Vec::new(), "Effect<{}, Never, Unit>".to_owned()),
    };
    finish_callable(
        operation.semantic_name.to_owned(),
        operation.surface_name.to_owned(),
        if operation.semantic_name.starts_with("std/effect") {
            "std/effect".to_owned()
        } else {
            "std/prelude".to_owned()
        },
        type_parameters,
        parameters,
        result,
        Vec::new(),
    )
}

fn standard_category(name: &str, module: &str) -> &'static str {
    match module {
        "std/array" | "std/list" => "Collection",
        "std/signal" => "Signal",
        "std/web/html" => "HTML",
        "std/web/dom" => "DOM",
        "std/effect" => "Effect",
        "std/prelude"
            if matches!(
                name,
                "map" | "pure" | "apply" | "flatMap" | "append" | "empty"
            ) =>
        {
            "Traits"
        }
        "std/prelude"
            if matches!(
                name,
                "reduce"
                    | "join"
                    | "sum"
                    | "product"
                    | "combine"
                    | "any"
                    | "all"
                    | "forEach"
                    | "unfold"
                    | "next"
            ) =>
        {
            "Collection"
        }
        _ => "Prelude",
    }
}

fn standard_description(identity: &str) -> Option<&'static str> {
    Some(match identity {
        identity if identity.ends_with("::filter") => {
            "Keeps collection elements whose predicate returns True."
        }
        identity if identity.ends_with("::filterMap") => {
            "Transforms collection elements and drops Nothing results."
        }
        identity if identity.ends_with("::flatMap") => {
            "Transforms each element to a collection and flattens the results."
        }
        identity if identity.ends_with("::length") => {
            "Returns the number of elements in the collection."
        }
        identity if identity.ends_with("::isEmpty") => {
            "Returns whether the collection contains no elements."
        }
        identity if identity.ends_with("::get") => {
            "Returns the indexed element, or Nothing when the index is invalid."
        }
        identity if identity.ends_with("::head") => {
            "Returns the first element, or Nothing for an empty collection."
        }
        identity if identity.ends_with("::tail") => {
            "Returns all elements after the first, or Nothing for an empty collection."
        }
        "std/prelude::join" => "Joins a reducible collection of strings with a separator.",
        "std/prelude::sum" => "Adds every element of a reducible collection from zero.",
        "std/prelude::product" => "Multiplies every element of a reducible collection from one.",
        "std/prelude::combine" => "Combines a reducible collection using its Monoid instance.",
        "std/prelude::any" => "Returns true at the first element accepted by the predicate.",
        "std/prelude::all" => "Returns false at the first element rejected by the predicate.",
        "std/prelude::reduce" => "Folds a reducible collection into one accumulated value.",
        "std/prelude::forEach" => {
            "Runs one Effect for every value exposed by an Iterable instance."
        }
        "std/prelude::unfold" => "Builds a lazy Iterator from an initial state and step function.",
        "std/prelude::next" => "Reads the next Iterator value and its remaining Iterator.",
        "std/prelude::Functor::map" => {
            "Transforms the value inside a Functor without changing its shape."
        }
        "std/prelude::Applicative::pure" => "Lifts a value into an Applicative context.",
        "std/prelude::Applicative::apply" => "Applies a function inside an Applicative context.",
        "std/prelude::Monad::flatMap" => {
            "Sequences a Monad with a value-dependent next computation."
        }
        "std/prelude::Semigroup::append" => "Associatively combines two values.",
        "std/prelude::Monoid::empty" => "Returns the identity value for a Monoid.",
        "std/prelude::Maybe" => "Represents an optional value as Nothing or Just.",
        "std/prelude::Either" => "Represents either a typed failure or a success value.",
        "std/prelude::Nothing" => "Constructs an empty Maybe value.",
        "std/prelude::Just" => "Constructs a present Maybe value.",
        "std/prelude::Left" => "Constructs the failure side of Either.",
        "std/prelude::Right" => "Constructs the success side of Either.",
        "std/prelude::println" => "Writes text followed by a newline through Console.",
        "std/prelude::print" => "Writes text through Console without adding a newline.",
        "std/prelude::readLine" => "Reads one optional line through Stdin.",
        "std/effect::succeed" => "Creates an Effect that succeeds with a value.",
        "std/effect::fail" => "Creates an Effect that fails with a typed error.",
        "std/effect::mapError" => {
            "Transforms an Effect failure while preserving its success value."
        }
        "std/effect::fromEither" => {
            "Lifts Either into Effect without performing an external operation."
        }
        "std/web/dom::app" => "Mounts a typed state-update-view application into a DOM target.",
        "std/web/html::renderToString" => "Renders typed HTML to an escaped fragment string.",
        "std/web/html::renderDocument" => "Renders typed HTML as a complete document string.",
        "std/web/html::style" => "Validates and converts a style record into inline Style.",
        "std/web/html::InputEvent" => {
            "Immutable text-input snapshot containing only the current String value."
        }
        "std/web/html::ChangeEvent" => {
            "Immutable form-control snapshot containing value and checked state."
        }
        "std/web/html::form" => {
            "Creates a typed form whose onSubmit message prevents native page reload."
        }
        "std/web/html::label" => "Creates a label connected through the htmlFor prop.",
        "std/web/html::input" => {
            "Creates a controlled input with typed input and change event snapshots."
        }
        "std/web/html::textarea" => {
            "Creates a controlled textarea with typed input and change snapshots."
        }
        "std/signal::map" => "Derives a Signal by transforming each current value.",
        "std/signal::make" => "Creates a mutable Signal inside Effect.",
        _ => return None,
    })
}

fn module_description(module: &str, export: &InterfaceExport) -> &'static str {
    match (module, export.namespace.as_str()) {
        ("std/web/html", "value") => {
            "Creates or renders typed HTML through the standard HTML surface."
        }
        ("std/web/html", _) => "Type or trait from the standard HTML surface.",
        ("std/web/dom", "value") => {
            "Runs typed browser DOM behavior through the standard DOM surface."
        }
        ("std/web/dom", _) => "Type from the standard DOM capability surface.",
        ("std/signal", "value") => {
            "Creates or transforms reactive values through the standard Signal surface."
        }
        ("std/signal", _) => "Type from the standard Signal surface.",
        _ => "Compiler-owned standard library symbol.",
    }
}

fn collect_typed_occurrences(
    typed: &TypedModule,
    types: &mut Vec<AnalysisTypeOccurrence>,
    calls: &mut Vec<(ByteSpan, String, usize)>,
) {
    let mut collect = |expression: &TypedExpr| {
        if let Some(type_ref) = expression_type(expression) {
            let range = expression_origin(expression);
            if range.end > range.start {
                types.push(AnalysisTypeOccurrence {
                    range,
                    type_name: render_type(&type_ref),
                });
            }
        }
        match expression {
            TypedExpr::Call {
                callee,
                arguments,
                origin,
                ..
            }
            | TypedExpr::EffectInvoke {
                callee,
                arguments,
                origin,
                ..
            } => calls.push((*origin, callee.clone(), arguments.len())),
            TypedExpr::EffectCall {
                operation,
                arguments,
                origin,
                ..
            } => calls.push((*origin, operation.clone(), arguments.len())),
            _ => {}
        }
    };
    for declaration in &typed.declarations {
        match declaration {
            TypedDecl::Let { value, .. } => walk_expression(value, &mut collect),
            TypedDecl::Fn { body, .. } | TypedDecl::EffectFn { body, .. } => {
                walk_expression(body, &mut collect)
            }
            TypedDecl::Adt { .. } | TypedDecl::Struct { .. } => {}
        }
    }
    for instance in &typed.instances {
        let crate::TypedInstanceImplementation::UserDefined { methods } = &instance.implementation
        else {
            continue;
        };
        for method in methods {
            walk_expression(&method.body, &mut collect);
        }
    }
    for declaration in &typed.declarations {
        let expression = match declaration {
            TypedDecl::Let { value, .. } => Some(value),
            TypedDecl::Fn { body, .. } | TypedDecl::EffectFn { body, .. } => Some(body),
            TypedDecl::Adt { .. } | TypedDecl::Struct { .. } => None,
        };
        if let Some(expression) = expression {
            walk_patterns(expression, &mut |pattern| {
                let Some((range, type_ref)) = pattern_type(pattern) else {
                    return;
                };
                if range.end > range.start {
                    types.push(AnalysisTypeOccurrence {
                        range,
                        type_name: render_type(type_ref),
                    });
                }
            });
        }
    }
}

fn walk_expression(expression: &TypedExpr, visit: &mut impl FnMut(&TypedExpr)) {
    visit(expression);
    match expression {
        TypedExpr::Template { parts, .. } => {
            for part in parts {
                if let crate::TypedTemplatePart::Interpolation { value, .. } = part {
                    walk_expression(value, visit);
                }
            }
        }
        TypedExpr::FieldAccess { receiver, .. }
        | TypedExpr::OptionalFieldAccess { receiver, .. }
        | TypedExpr::Lambda { body: receiver, .. } => walk_expression(receiver, visit),
        TypedExpr::Call { arguments, .. }
        | TypedExpr::EffectCall { arguments, .. }
        | TypedExpr::EffectInvoke { arguments, .. }
        | TypedExpr::Tuple {
            elements: arguments,
            ..
        }
        | TypedExpr::Array {
            elements: arguments,
            ..
        }
        | TypedExpr::List {
            elements: arguments,
            ..
        } => {
            for argument in arguments {
                walk_expression(argument, visit);
            }
        }
        TypedExpr::Record { items, .. } => {
            for item in items {
                walk_expression(item.value(), visit);
            }
        }
        TypedExpr::ArrayComprehension {
            element, clauses, ..
        }
        | TypedExpr::ListComprehension {
            element, clauses, ..
        } => {
            walk_expression(element, visit);
            for clause in clauses {
                match clause {
                    TypedComprehensionClause::Generator { source, .. } => {
                        walk_expression(source, visit)
                    }
                    TypedComprehensionClause::Guard { condition, .. } => {
                        walk_expression(condition, visit)
                    }
                }
            }
        }
        TypedExpr::Binary { left, right, .. } => {
            walk_expression(left, visit);
            walk_expression(right, visit);
        }
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            walk_expression(condition, visit);
            walk_expression(then_branch, visit);
            walk_expression(else_branch, visit);
        }
        TypedExpr::Match {
            scrutinee, arms, ..
        } => {
            walk_expression(scrutinee, visit);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    walk_expression(guard, visit);
                }
                walk_expression(&arm.body, visit);
            }
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                let value = match statement {
                    TypedDoStatement::Effect { value }
                    | TypedDoStatement::PureLet { value, .. }
                    | TypedDoStatement::Bind { value, .. } => value,
                };
                walk_expression(value, visit);
            }
            walk_expression(result, visit);
        }
        TypedExpr::MonadDo {
            statements, result, ..
        } => {
            for statement in statements {
                let value = match statement {
                    TypedMonadDoStatement::Expression { value }
                    | TypedMonadDoStatement::PureLet { value, .. }
                    | TypedMonadDoStatement::Bind { value, .. } => value,
                };
                walk_expression(value, visit);
            }
            walk_expression(result, visit);
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. } => {}
    }
}

fn walk_patterns(expression: &TypedExpr, visit: &mut impl FnMut(&TypedPattern)) {
    walk_expression(expression, &mut |expression| match expression {
        TypedExpr::Match { arms, .. } => {
            for arm in arms {
                walk_pattern(&arm.pattern, visit);
            }
        }
        TypedExpr::ArrayComprehension { clauses, .. }
        | TypedExpr::ListComprehension { clauses, .. } => {
            for clause in clauses {
                if let TypedComprehensionClause::Generator { pattern, .. } = clause {
                    walk_pattern(pattern, visit);
                }
            }
        }
        _ => {}
    });
}

fn walk_pattern(pattern: &TypedPattern, visit: &mut impl FnMut(&TypedPattern)) {
    visit(pattern);
    match pattern {
        TypedPattern::Constructor {
            argument: Some(argument),
            ..
        } => walk_pattern(argument, visit),
        TypedPattern::Tuple { elements, .. } => {
            for element in elements {
                walk_pattern(element, visit);
            }
        }
        TypedPattern::Record { fields, .. } => {
            for field in fields {
                walk_pattern(&field.pattern, visit);
            }
        }
        TypedPattern::Integer { .. }
        | TypedPattern::String { .. }
        | TypedPattern::Boolean { .. }
        | TypedPattern::Wildcard { .. }
        | TypedPattern::Binding { .. }
        | TypedPattern::Constructor { argument: None, .. }
        | TypedPattern::Invalid { .. } => {}
    }
}

fn expression_origin(expression: &TypedExpr) -> ByteSpan {
    match expression {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
        | TypedExpr::Template { origin, .. }
        | TypedExpr::Boolean { origin, .. }
        | TypedExpr::Variable { origin, .. }
        | TypedExpr::FieldAccess { origin, .. }
        | TypedExpr::OptionalFieldAccess { origin, .. }
        | TypedExpr::Call { origin, .. }
        | TypedExpr::Lambda { origin, .. }
        | TypedExpr::Tuple { origin, .. }
        | TypedExpr::Array { origin, .. }
        | TypedExpr::List { origin, .. }
        | TypedExpr::Record { origin, .. }
        | TypedExpr::ArrayComprehension { origin, .. }
        | TypedExpr::ListComprehension { origin, .. }
        | TypedExpr::Binary { origin, .. }
        | TypedExpr::If { origin, .. }
        | TypedExpr::Match { origin, .. }
        | TypedExpr::EffectCall { origin, .. }
        | TypedExpr::EffectInvoke { origin, .. }
        | TypedExpr::DoBlock { origin, .. }
        | TypedExpr::MonadDo { origin, .. } => *origin,
    }
}

fn expression_type(expression: &TypedExpr) -> Option<TypedType> {
    Some(match expression {
        TypedExpr::Unit { type_ref, .. }
        | TypedExpr::Integer { type_ref, .. }
        | TypedExpr::String { type_ref, .. }
        | TypedExpr::Template { type_ref, .. }
        | TypedExpr::Boolean { type_ref, .. }
        | TypedExpr::Variable { type_ref, .. }
        | TypedExpr::FieldAccess { type_ref, .. }
        | TypedExpr::OptionalFieldAccess { type_ref, .. }
        | TypedExpr::Call { type_ref, .. }
        | TypedExpr::Lambda { type_ref, .. }
        | TypedExpr::Tuple { type_ref, .. }
        | TypedExpr::Array { type_ref, .. }
        | TypedExpr::List { type_ref, .. }
        | TypedExpr::Record { type_ref, .. }
        | TypedExpr::ArrayComprehension { type_ref, .. }
        | TypedExpr::ListComprehension { type_ref, .. }
        | TypedExpr::Binary { type_ref, .. }
        | TypedExpr::If { type_ref, .. }
        | TypedExpr::Match { type_ref, .. }
        | TypedExpr::MonadDo { type_ref, .. } => type_ref.clone(),
        TypedExpr::EffectCall { effect, .. } | TypedExpr::EffectInvoke { effect, .. } => {
            effect_type(effect)
        }
        TypedExpr::DoBlock { result, .. } => expression_type(result)?,
    })
}

fn pattern_type(pattern: &TypedPattern) -> Option<(ByteSpan, &TypedType)> {
    match pattern {
        TypedPattern::Integer {
            type_ref, origin, ..
        }
        | TypedPattern::String {
            type_ref, origin, ..
        }
        | TypedPattern::Boolean {
            type_ref, origin, ..
        }
        | TypedPattern::Wildcard { type_ref, origin }
        | TypedPattern::Binding {
            type_ref, origin, ..
        }
        | TypedPattern::Constructor {
            type_ref, origin, ..
        }
        | TypedPattern::Tuple {
            type_ref, origin, ..
        }
        | TypedPattern::Record {
            type_ref, origin, ..
        } => Some((*origin, type_ref)),
        TypedPattern::Invalid { .. } => None,
    }
}

fn deduplicate_type_occurrences(types: &mut Vec<AnalysisTypeOccurrence>) {
    types.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then_with(|| left.range.end.cmp(&right.range.end))
            .then_with(|| left.type_name.cmp(&right.type_name))
    });
    types.dedup();
}

fn render_type(type_ref: &TypedType) -> String {
    match type_ref {
        TypedType::Named { name, arguments }
        | TypedType::ExternalNamed {
            name, arguments, ..
        } => render_application(name, arguments.iter().map(render_type)),
        TypedType::Hole => "_".to_owned(),
        TypedType::Record { fields, .. } => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|field| format!(
                    "{}{}: {}",
                    field.name,
                    if field.optional { "?" } else { "" },
                    render_type(&field.type_ref)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypedType::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(render_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypedType::Function { parameter, result } => format!(
            "{} -> {}",
            parenthesized_parameter(parameter),
            render_type(result)
        ),
    }
}

fn render_interface_type(type_ref: &InterfaceType) -> String {
    match type_ref {
        InterfaceType::Named { name, arguments }
        | InterfaceType::ExternalNamed {
            name, arguments, ..
        } => render_application(name, arguments.iter().map(render_interface_type)),
        InterfaceType::Hole => "_".to_owned(),
        InterfaceType::TypeConstructor { name, arity } => {
            if *arity == 0 {
                name.clone()
            } else {
                format!("{name}<{}>", vec!["_"; *arity as usize].join(", "))
            }
        }
        InterfaceType::Function { parameter, result } => format!(
            "{} -> {}",
            parenthesized_interface_parameter(parameter),
            render_interface_type(result)
        ),
        InterfaceType::Apply {
            constructor,
            arguments,
        } => render_application(constructor, arguments.iter().map(render_interface_type)),
        InterfaceType::Record { fields, .. } => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|field| format!(
                    "{}{}: {}",
                    field.name,
                    if field.optional { "?" } else { "" },
                    render_interface_type(&field.type_ref)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        InterfaceType::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(render_interface_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_application(name: &str, arguments: impl Iterator<Item = String>) -> String {
    let arguments = arguments.collect::<Vec<_>>();
    if arguments.is_empty() {
        name.to_owned()
    } else {
        format!("{name}<{}>", arguments.join(", "))
    }
}

fn parenthesized_parameter(type_ref: &TypedType) -> String {
    match type_ref {
        TypedType::Function { .. } => format!("({})", render_type(type_ref)),
        _ => render_type(type_ref),
    }
}

fn parenthesized_interface_parameter(type_ref: &InterfaceType) -> String {
    match type_ref {
        InterfaceType::Function { .. } => format!("({})", render_interface_type(type_ref)),
        _ => render_interface_type(type_ref),
    }
}

fn render_constraints(constraints: &[TypedConstraint]) -> Vec<String> {
    constraints
        .iter()
        .map(|constraint| {
            render_application(
                &constraint.name,
                constraint.arguments.iter().map(render_type),
            )
        })
        .collect()
}

fn render_interface_constraints(constraints: &[InterfaceConstraint]) -> Vec<String> {
    constraints
        .iter()
        .map(|constraint| {
            render_application(
                &constraint.name,
                constraint.arguments.iter().map(render_interface_type),
            )
        })
        .collect()
}

fn render_interface_scheme(scheme: &InterfaceScheme) -> String {
    let mut signature = render_interface_type(&scheme.type_ref);
    let constraints = render_interface_constraints(&scheme.constraints);
    if !constraints.is_empty() {
        signature.push_str(" where ");
        signature.push_str(&constraints.join(", "));
    }
    signature
}

fn split_function(type_ref: &TypedType) -> (Vec<&TypedType>, &TypedType) {
    let mut parameters = Vec::new();
    let mut current = type_ref;
    while let TypedType::Function { parameter, result } = current {
        parameters.push(parameter.as_ref());
        current = result;
    }
    (parameters, current)
}

fn split_interface_function(type_ref: &InterfaceType) -> (Vec<&InterfaceType>, &InterfaceType) {
    let mut parameters = Vec::new();
    let mut current = type_ref;
    while let InterfaceType::Function { parameter, result } = current {
        parameters.push(parameter.as_ref());
        current = result;
    }
    (parameters, current)
}

fn effect_type(effect: &TypedEffect) -> TypedType {
    TypedType::Named {
        name: "Effect".to_owned(),
        arguments: vec![
            effect.environment.clone(),
            effect.failure.clone(),
            effect.success.clone(),
        ],
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn callable_type(callable: &AnalysisCallable) -> String {
    callable
        .parameters
        .iter()
        .rev()
        .fold(callable.result.clone(), |result, parameter| {
            let parameter = if parameter.type_name.contains(" -> ") {
                format!("({})", parameter.type_name)
            } else {
                parameter.type_name.clone()
            };
            format!("{parameter} -> {result}")
        })
}

fn module_from_identity(identity: &str, fallback: &str) -> String {
    identity
        .split("::")
        .next()
        .filter(|module| module.contains('/'))
        .unwrap_or(fallback)
        .to_owned()
}

fn symbol_name(identity: &str) -> String {
    identity
        .rsplit_once("::")
        .map(|(_, name)| name)
        .unwrap_or(identity)
        .trim_start_matches("trait(")
        .trim_end_matches(')')
        .to_owned()
}

fn namespace_name(namespace: SymbolNamespace) -> &'static str {
    match namespace {
        SymbolNamespace::Type => "type",
        SymbolNamespace::Value => "value",
        SymbolNamespace::Trait => "trait",
        SymbolNamespace::Field => "field",
        SymbolNamespace::Module => "module",
        SymbolNamespace::Operator => "operator",
    }
}

fn kind_name(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Let => "let",
        SymbolKind::Function => "function",
        SymbolKind::EffectFunction => "effect-function",
        SymbolKind::TypeParameter => "type-parameter",
        SymbolKind::Parameter => "parameter",
        SymbolKind::PatternBinding => "pattern-binding",
        SymbolKind::Type => "type",
        SymbolKind::Constructor => "constructor",
        SymbolKind::Trait => "trait",
        SymbolKind::TraitMethod => "trait-method",
        SymbolKind::ModuleImport => "module-import",
        SymbolKind::Imported => "imported",
        SymbolKind::Prelude => "prelude",
        SymbolKind::Operator => "operator",
    }
}
