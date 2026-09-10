//! Application reachability over resolved compiler IR, before emission/bundling.
//! Names here are compiler-planned local bindings; cross-module edges use the
//! canonical source-import plan, never filesystem or bundler guesses.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

type Node = (String, String);
type Names = BTreeSet<String>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationReachability {
    pub roots: Vec<RetainedDeclaration>,
    pub retained: Vec<RetainedDeclaration>,
    pub eliminated_modules: usize,
    pub eliminated_declarations: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedDeclaration {
    pub module: String,
    pub declaration: String,
    pub reason: String,
}

/// Public visibility is not an application root. Type/constructor groups remain
/// atomic so pruning never changes a nominal representation or its dictionaries.
pub fn retain_application(
    modules: &mut BTreeMap<String, TypeScriptModule>,
    roots: &[Node],
) -> ApplicationReachability {
    let mut edges: BTreeMap<Node, BTreeSet<Node>> = BTreeMap::new();
    let mut definitions = BTreeSet::new();
    let mut foreign_roots = Vec::new();
    for (id, module) in modules.iter() {
        let mut add = |name: &str, refs: Names| {
            let key = (id.clone(), name.to_owned());
            definitions.insert(key.clone());
            edges
                .entry(key)
                .or_default()
                .extend(refs.into_iter().map(|name| (id.clone(), name)));
        };
        for TypeScriptBinding::Const {
            name,
            initializer,
            type_ref,
            ..
        } in &module.bindings
        {
            let mut refs = expression(initializer);
            types(type_ref, &mut refs);
            add(name, refs);
        }
        for TypeScriptFunction::ConstFunction {
            name,
            body,
            parameters,
            ..
        } in &module.functions
        {
            let mut refs = expression(body);
            parameters_refs(parameters, &mut refs);
            add(name, refs);
        }
        for alias in &module.aliases {
            let mut refs = Names::new();
            types(&alias.target, &mut refs);
            add(&alias.name, refs);
        }
        for structure in &module.structs {
            let mut refs = Names::new();
            for field in structure
                .fields
                .iter()
                .chain(structure.private_fields.iter().flatten())
            {
                types(&field.type_ref, &mut refs);
            }
            add(&structure.name, refs);
        }
        for adt in &module.adts {
            let mut refs = Names::from([adt.name.clone()]);
            for variant in &adt.variants {
                refs.insert(variant.name.clone());
                if let Some(payload) = &variant.payload {
                    types(payload, &mut refs);
                }
            }
            add(&adt.name, refs.clone());
            for variant in &adt.variants {
                add(&variant.name, refs.clone());
            }
        }
        for instance in &module.instances {
            let mut refs = instance_refs(instance);
            // Dictionary emitters synthesize helpers in addition to expression
            // IR. Keep their candidate runtime imports until runtime retention.
            refs.extend(module.imports.iter().map(|import| import.local.clone()));
            add(&instance.dictionary_export, refs);
        }
        // Foreign load/initialization semantics are outside this first-party
        // pass. Preserve their module's value surface rather than invent purity.
        if !module.foreign_modules.is_empty() {
            foreign_roots.extend(
                definitions
                    .iter()
                    .filter(|(module, _)| module == id)
                    .cloned(),
            );
        }
        if !module.foreign_modules.is_empty() {
            let root = (id.clone(), "$foreign-load".to_owned());
            foreign_roots.push(root.clone());
            edges.entry(root).or_default().extend(
                module
                    .source_imports
                    .iter()
                    .flat_map(|import| import.bindings.iter().chain(&import.reexports))
                    .map(|binding| (id.clone(), binding.local.clone())),
            );
        }
        for import in &module.source_imports {
            for binding in import.bindings.iter().chain(&import.reexports) {
                edges
                    .entry((id.clone(), binding.local.clone()))
                    .or_default()
                    .insert((import.module.clone(), binding.imported.clone()));
            }
        }
    }
    let all_roots = roots
        .iter()
        .cloned()
        .chain(foreign_roots)
        .collect::<BTreeSet<_>>();
    let mut pending = all_roots.clone();
    let mut live = BTreeSet::new();
    while let Some(node) = pending.pop_first() {
        if !live.insert(node.clone()) {
            continue;
        }
        if let Some(dependencies) = edges.get(&node) {
            pending.extend(dependencies.iter().cloned());
        }
    }
    let before_modules = modules.len();
    let retained = definitions
        .intersection(&live)
        .map(|(module, declaration)| RetainedDeclaration {
            module: module.clone(),
            declaration: declaration.clone(),
            reason: if all_roots.contains(&(module.clone(), declaration.clone())) {
                "product-root"
            } else {
                "referenced-by-retained-declaration"
            }
            .to_owned(),
        })
        .collect::<Vec<_>>();
    modules.retain(|id, module| {
        let keep = |name: &str| live.contains(&(id.clone(), name.to_owned()));
        let module_live = live.iter().any(|(module, _)| module == id);
        if !module_live {
            return false;
        }
        if !module.foreign_modules.is_empty() {
            return true;
        }
        module
            .bindings
            .retain(|TypeScriptBinding::Const { name, .. }| keep(name));
        module
            .functions
            .retain(|TypeScriptFunction::ConstFunction { name, .. }| keep(name));
        module.aliases.retain(|alias| keep(&alias.name));
        module.structs.retain(|structure| keep(&structure.name));
        module.adts.retain(|adt| keep(&adt.name));
        module
            .instances
            .retain(|instance| keep(&instance.dictionary_export));
        module.source_imports.retain_mut(|import| {
            import.bindings.retain(|binding| keep(&binding.local));
            import.reexports.retain(|binding| keep(&binding.local));
            !import.bindings.is_empty() || !import.reexports.is_empty()
        });
        module.imports.retain(|import| keep(&import.local));
        // Type-only runtime features do not imply executable runtime retention.
        let all_import_features = module
            .imports
            .iter()
            .map(|import| import.feature.clone())
            .chain(
                module
                    .type_imports
                    .iter()
                    .map(|import| import.feature.clone()),
            )
            .collect::<BTreeSet<_>>();
        module.runtime_requirements.retain(|feature| {
            all_import_features.contains(feature)
                || matches!(
                    feature.as_str(),
                    "core.unit" | "core.int" | "core.float64" | "core.string" | "core.bool"
                )
        });
        true
    });
    ApplicationReachability {
        roots: roots
            .iter()
            .map(|(module, declaration)| RetainedDeclaration {
                module: module.clone(),
                declaration: declaration.clone(),
                reason: "product-root".to_owned(),
            })
            .collect(),
        eliminated_declarations: definitions.len() - retained.len(),
        retained,
        eliminated_modules: before_modules - modules.len(),
    }
}

fn parameters_refs(parameters: &[TypeScriptParameter], refs: &mut Names) {
    for parameter in parameters {
        refs.remove(&parameter.name);
    }
    for parameter in parameters {
        // Parameter types are already rendered by the compiler. Tokenizing
        // identifiers only adds conservative type edges, never runtime roots.
        refs.extend(
            parameter
                .type_name
                .split(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '$'))
                .filter(|name| !name.is_empty())
                .map(str::to_owned),
        );
    }
}
fn types(value: &TypeScriptType, refs: &mut Names) {
    match value {
        TypeScriptType::Reference { name, arguments } => {
            refs.insert(name.clone());
            for value in arguments {
                types(value, refs);
            }
        }
        TypeScriptType::Maybe { element }
        | TypeScriptType::Array { element }
        | TypeScriptType::List { element } => types(element, refs),
        TypeScriptType::Either { error, value } => {
            types(error, refs);
            types(value, refs);
        }
        TypeScriptType::Tuple { elements } => {
            for value in elements {
                types(value, refs);
            }
        }
        TypeScriptType::Intersection { operands } => {
            for value in operands {
                types(value, refs);
            }
        }
        TypeScriptType::Record { fields } => {
            for field in fields {
                types(&field.type_ref, refs);
            }
        }
        TypeScriptType::Function { parameter, result } => {
            types(parameter, refs);
            types(result, refs);
        }
        TypeScriptType::Bigint
        | TypeScriptType::Number
        | TypeScriptType::Boolean
        | TypeScriptType::String
        | TypeScriptType::Undefined
        | TypeScriptType::Never
        | TypeScriptType::Ordering
        | TypeScriptType::Unknown
        | TypeScriptType::Range => {}
    }
}
fn expression(value: &TypeScriptExpr) -> Names {
    let mut refs = Names::new();
    match value {
        TypeScriptExpr::Identifier { name }
        | TypeScriptExpr::RuntimeReference { name }
        | TypeScriptExpr::CurriedRuntimeReference { name, .. } => {
            refs.insert(name.clone());
        }
        TypeScriptExpr::Tuple { elements } | TypeScriptExpr::Array { elements, .. } => {
            for value in elements {
                refs.extend(expression(value));
            }
            if let TypeScriptExpr::Array { element_type, .. } = value {
                types(element_type, &mut refs);
            }
        }
        TypeScriptExpr::FieldAccess { receiver, .. } => {
            refs.extend(expression(receiver));
        }
        TypeScriptExpr::OptionalFieldAccess {
            receiver,
            just_constructor,
            nothing_constructor,
            ..
        } => {
            refs.extend(expression(receiver));
            refs.insert(just_constructor.clone());
            refs.insert(nothing_constructor.clone());
        }
        TypeScriptExpr::Record {
            items,
            asserted_type,
        } => {
            for item in items {
                match item {
                    TypeScriptRecordValueItem::Field { value, .. }
                    | TypeScriptRecordValueItem::Spread { value, .. } => {
                        refs.extend(expression(value))
                    }
                }
            }
            if let Some(value) = asserted_type {
                types(value, &mut refs);
            }
        }
        TypeScriptExpr::Lambda { parameter, body } => {
            refs.extend(expression(body));
            refs.remove(parameter);
        }
        TypeScriptExpr::Binary { left, right, .. } => {
            refs.extend(expression(left));
            refs.extend(expression(right));
        }
        TypeScriptExpr::Unary { operand, .. } => refs.extend(expression(operand)),
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            refs.extend(expression(condition));
            refs.extend(expression(then_branch));
            refs.extend(expression(else_branch));
        }
        TypeScriptExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
        } => {
            refs.extend(expression(scrutinee));
            types(scrutinee_type, &mut refs);
            types(type_ref, &mut refs);
            for branch in branches {
                let mut branch_refs = expression(&branch.value);
                if let Some(guard) = &branch.guard {
                    branch_refs.extend(expression(guard));
                }
                for binding in &branch.bindings {
                    branch_refs.remove(&binding.name);
                    types(&binding.type_ref, &mut branch_refs);
                }
                refs.extend(branch_refs);
            }
        }
        TypeScriptExpr::Call {
            callee, arguments, ..
        }
        | TypeScriptExpr::ForeignTaskCall {
            callee, arguments, ..
        }
        | TypeScriptExpr::TypeApplicationCall {
            callee, arguments, ..
        }
        | TypeScriptExpr::RuntimeCall { callee, arguments } => {
            refs.insert(callee.clone());
            for value in arguments {
                refs.extend(expression(value));
            }
            if let TypeScriptExpr::Call {
                checked_callee_type: Some(value),
                ..
            } = value
            {
                types(value, &mut refs);
            }
            if let TypeScriptExpr::TypeApplicationCall { type_arguments, .. } = value {
                for value in type_arguments {
                    types(value, &mut refs);
                }
            }
        }
        TypeScriptExpr::CheckedResult { value, type_ref } => {
            refs.extend(expression(value));
            types(type_ref, &mut refs);
        }
        TypeScriptExpr::DictionaryCall {
            dictionary,
            arguments,
            ..
        } => {
            refs.extend(expression(dictionary));
            for value in arguments {
                refs.extend(expression(value));
            }
        }
        TypeScriptExpr::Await { value } => refs.extend(expression(value)),
        TypeScriptExpr::Sequence { statements, result }
        | TypeScriptExpr::MonadDo {
            statements, result, ..
        } => {
            refs.extend(expression(result));
            // Resolve free names backwards: a let initializer may refer to an
            // outer binding with the same spelling as the newly bound local.
            for statement in statements.iter().rev() {
                match statement {
                    TypeScriptStatement::Effect { value } => {
                        refs.insert("_ssrg_effect_flatMap".to_owned());
                        refs.extend(expression(value));
                    }
                    TypeScriptStatement::PureLet {
                        name,
                        initializer,
                        type_ref,
                        ..
                    }
                    | TypeScriptStatement::Const {
                        name,
                        initializer,
                        type_ref,
                        ..
                    } => {
                        refs.remove(name);
                        if matches!(statement, TypeScriptStatement::Const { .. }) {
                            refs.insert("_ssrg_effect_flatMap".to_owned());
                        }
                        refs.extend(expression(initializer));
                        types(type_ref, &mut refs);
                    }
                    TypeScriptStatement::LocalFunction {
                        name,
                        parameters,
                        body,
                        return_type,
                        rec_group,
                        ..
                    } => {
                        refs.remove(name);
                        let mut body_refs = expression(body);
                        parameters_refs(parameters, &mut body_refs);
                        if let Some(group) = rec_group {
                            for member in statements {
                                if let TypeScriptStatement::LocalFunction {
                                    name,
                                    rec_group: Some(other),
                                    ..
                                } = member
                                {
                                    if group == other {
                                        body_refs.remove(name);
                                    }
                                }
                            }
                        }
                        refs.extend(body_refs);
                        if let Some(value) = return_type {
                            types(value, &mut refs);
                        }
                    }
                }
            }
            if let TypeScriptExpr::MonadDo { dictionary, .. } = value {
                refs.extend(expression(dictionary));
            }
        }
        TypeScriptExpr::Undefined
        | TypeScriptExpr::Bigint { .. }
        | TypeScriptExpr::Number { .. }
        | TypeScriptExpr::String { .. }
        | TypeScriptExpr::Boolean { .. } => {}
    }
    refs
}
fn dictionary(value: &TypeScriptShowDictionaryReference, refs: &mut Names) {
    match value {
        TypeScriptShowDictionaryReference::Runtime { local, .. }
        | TypeScriptShowDictionaryReference::Imported { local, .. } => {
            refs.insert(local.clone());
        }
        TypeScriptShowDictionaryReference::Local {
            dictionary_export, ..
        } => {
            refs.insert(dictionary_export.clone());
        }
        TypeScriptShowDictionaryReference::Expression { expression: value } => {
            refs.extend(expression(value))
        }
    }
}
fn instance_refs(instance: &TypeScriptInstance) -> Names {
    let mut refs = Names::new();
    for argument in &instance.arguments {
        types(argument, &mut refs);
    }
    match &instance.implementation {
        TypeScriptInstanceImplementation::UserDefined { methods } => {
            for method in methods {
                let mut body = expression(&method.body);
                parameters_refs(&method.parameters, &mut body);
                refs.extend(body);
            }
        }
        TypeScriptInstanceImplementation::DerivedShow { adt_name, variants }
        | TypeScriptInstanceImplementation::DerivedJson {
            adt_name, variants, ..
        }
        | TypeScriptInstanceImplementation::DerivedStructural {
            adt_name, variants, ..
        } => {
            refs.insert(adt_name.clone());
            for variant in variants {
                if let Some(payload) = &variant.payload {
                    types(&payload.type_ref, &mut refs);
                    dictionary(&payload.dictionary, &mut refs);
                }
            }
        }
        TypeScriptInstanceImplementation::DerivedStructShow {
            struct_name,
            fields,
        }
        | TypeScriptInstanceImplementation::DerivedStructJson {
            struct_name,
            fields,
        }
        | TypeScriptInstanceImplementation::DerivedStructStructural {
            struct_name,
            fields,
        } => {
            refs.insert(struct_name.clone());
            for field in fields {
                types(&field.type_ref, &mut refs);
                dictionary(&field.dictionary, &mut refs);
            }
        }
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    fn module(id: &str) -> TypeScriptModule {
        lower_core_module_to_typescript_ir(lower_typed_module(seseragi_semantics::type_module(
            id,
            "pub fn main value: Int -> Int = value\nfn unused value: Int -> Int = value\n",
        )))
    }
    #[test]
    fn visits_cyclic_symbol_edges_once_and_is_order_independent() {
        let mut modules = BTreeMap::new();
        for (id, dependency) in [("a", "b"), ("b", "a")] {
            let mut module = module(id);
            module.module = id.to_owned();
            let TypeScriptFunction::ConstFunction { body, origin, .. } = &mut module.functions[0];
            *body = TypeScriptExpr::Call {
                callee: "next".to_owned(),
                arguments: vec![],
                checked_callee_type: None,
            };
            module.source_imports.push(TypeScriptSourceImport {
                module: dependency.to_owned(),
                specifier: format!("./{dependency}.js"),
                runtime_edge: true,
                bindings: vec![TypeScriptSourceImportBinding {
                    imported: "main".to_owned(),
                    local: "next".to_owned(),
                    source_local: "next".to_owned(),
                    canonical: format!("{dependency}::main"),
                    type_only: false,
                    origin: origin.clone(),
                }],
                reexports: vec![],
                origin: origin.clone(),
            });
            modules.insert(id.to_owned(), module);
        }
        let mut reversed = modules
            .iter()
            .rev()
            .map(|(id, value)| (id.clone(), value.clone()))
            .collect();
        let roots = vec![("a".to_owned(), "main".to_owned())];
        let report = retain_application(&mut modules, &roots);
        assert_eq!(report, retain_application(&mut reversed, &roots));
        assert_eq!(modules, reversed);
        assert_eq!(report.eliminated_declarations, 2);
        assert_eq!(report.retained.len(), 2);
        assert!(modules.values().all(|module| module.functions.len() == 1));
    }
    #[test]
    fn local_initializer_keeps_an_outer_binding_with_the_same_spelling() {
        let value = TypeScriptExpr::Sequence {
            statements: vec![TypeScriptStatement::PureLet {
                type_parameters: vec![],
                name: "outer".to_owned(),
                type_ref: TypeScriptType::Bigint,
                initializer: TypeScriptExpr::Identifier {
                    name: "outer".to_owned(),
                },
                origin: SourceSpan {
                    source: "test".to_owned(),
                    start: 0,
                    end: 0,
                },
            }],
            result: Box::new(TypeScriptExpr::Identifier {
                name: "outer".to_owned(),
            }),
        };
        assert!(expression(&value).contains("outer"));
        assert!(!expression(&TypeScriptExpr::Lambda {
            parameter: "outer".to_owned(),
            body: Box::new(TypeScriptExpr::Identifier {
                name: "outer".to_owned()
            })
        })
        .contains("outer"));
    }
}
