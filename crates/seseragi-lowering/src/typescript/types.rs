use crate::{CoreExpr, CoreParameter, CoreType};
use seseragi_syntax::TypeParameter;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::names::{local_name, safe_identifier};
use super::{TypeScriptParameter, TypeScriptType};
use crate::runtime_types::runtime_type_import_for_surface;

/// Import names and lexical type-constructor scope used only for TypeScript annotations.
#[derive(Clone, Debug, Default)]
pub(super) struct TypeScriptTypeContext {
    imported_names: Arc<BTreeMap<String, String>>,
    constructors: BTreeSet<String>,
    value_parameters: BTreeSet<String>,
    arities: BTreeMap<String, u32>,
    private_struct_names: BTreeSet<String>,
    private_representation: bool,
    erased_newtypes: BTreeSet<String>,
    foreign_scalar_results: BTreeMap<String, (usize, usize, usize)>,
}

impl From<BTreeMap<String, String>> for TypeScriptTypeContext {
    fn from(imported_names: BTreeMap<String, String>) -> Self {
        Self {
            imported_names: Arc::new(imported_names),
            constructors: BTreeSet::new(),
            value_parameters: BTreeSet::new(),
            arities: BTreeMap::new(),
            private_struct_names: BTreeSet::new(),
            private_representation: false,
            erased_newtypes: BTreeSet::new(),
            foreign_scalar_results: BTreeMap::new(),
        }
    }
}

impl TypeScriptTypeContext {
    pub(super) fn with_runtime_type_name(mut self, canonical: String, name: String) -> Self {
        Arc::make_mut(&mut self.imported_names).insert(canonical, name);
        self
    }

    pub(super) fn with_erased_newtypes(mut self, constructors: BTreeSet<String>) -> Self {
        self.erased_newtypes = constructors;
        self
    }

    /// Only an actual foreign declaration can bind a result to a codec slot.
    /// Matching a result display name or an unrelated argument codec is insufficient.
    pub(super) fn foreign_scalar_result(
        &self,
        callee: &str,
        argument_count: usize,
        result_type: &CoreType,
        evidence: &[crate::CoreCallEvidence],
    ) -> Option<CoreType> {
        let &(arity, slot, codec_count) = self.foreign_scalar_results.get(callee)?;
        if argument_count != arity || evidence.len() != codec_count {
            return None;
        }
        let selected = evidence.get(slot)?;
        let crate::CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return None;
        };
        let [result @ CoreType::ExternalNamed {
            name,
            canonical,
            arguments,
        }] = type_arguments.as_slice()
        else {
            return None;
        };
        (identity == "@seseragi/foreign-codec"
            && selected.constraint.name == "ForeignCodec"
            && selected.constraint.arguments.as_slice() == type_arguments.as_slice()
            && evidence_arguments.is_empty()
            && arguments.is_empty()
            && matches!(name.as_str(), "Unit" | "Bool" | "Int" | "Float" | "Char" | "String")
            && canonical == &format!("std/prelude::{name}")
            && matches!(result_type, CoreType::Named {name: actual, arguments} if actual == name && arguments.is_empty()))
            .then(|| result.clone())
    }

    pub(super) fn is_erased_newtype(&self, constructor: &str) -> bool {
        self.erased_newtypes.contains(constructor)
    }

    pub(super) fn with_module(mut self, module: &crate::CoreModule) -> Self {
        fn collect(
            members: &[crate::CoreForeignMember],
            results: &mut BTreeMap<String, (usize, usize, usize)>,
        ) {
            for member in members {
                match member {
                    crate::CoreForeignMember::Function {
                        symbol,
                        mode: seseragi_syntax::ForeignCallMode::Pure,
                        type_parameters,
                        parameters,
                        return_type: CoreType::Named { name, arguments },
                        ..
                    } if arguments.is_empty() => {
                        if let Some(index) = type_parameters
                            .iter()
                            .position(|parameter| parameter.name == *name && parameter.arity == 0)
                        {
                            results.insert(
                                symbol.clone(),
                                (parameters.len(), index, type_parameters.len()),
                            );
                        }
                    }
                    crate::CoreForeignMember::Namespace { members, .. } => {
                        collect(members, results)
                    }
                    _ => {}
                }
            }
        }
        for foreign in &module.foreign_modules {
            collect(&foreign.members, &mut self.foreign_scalar_results);
        }
        for import in module
            .module_dependencies
            .iter()
            .flat_map(|dependency| &dependency.imports)
        {
            if let Some(proof) = &import.foreign_scalar_result {
                if proof.parameter_count == 0
                    || proof.type_parameter_index >= proof.type_parameter_count
                {
                    self.foreign_scalar_results
                        .insert(import.canonical.clone(), (0, 0, 0));
                } else {
                    let incoming = (
                        proof.parameter_count,
                        proof.type_parameter_index,
                        proof.type_parameter_count,
                    );
                    match self.foreign_scalar_results.get(&import.canonical) {
                        Some(previous) if *previous != incoming => {
                            self.foreign_scalar_results
                                .insert(import.canonical.clone(), (0, 0, 0));
                        }
                        None => {
                            self.foreign_scalar_results
                                .insert(import.canonical.clone(), incoming);
                        }
                        _ => {}
                    }
                }
            }
        }
        for (symbol, arity) in module
            .structs
            .iter()
            .map(|item| (&item.symbol, item.type_parameters.len()))
            .chain(
                module
                    .adts
                    .iter()
                    .map(|item| (&item.symbol, item.type_parameters.len())),
            )
        {
            self.arities.insert(symbol.clone(), arity as u32);
        }
        if module.structs.iter().any(|structure| structure.opaque) {
            self.private_struct_names.extend(
                module
                    .structs
                    .iter()
                    .map(|structure| structure.symbol.clone()),
            );
            self.private_struct_names.extend(
                module
                    .structs
                    .iter()
                    .map(|structure| structure.name.clone()),
            );
        }
        for binding in &module.external_type_bindings {
            if let Some(arity) = binding.arity {
                self.arities.insert(binding.canonical.clone(), arity);
            }
        }
        for (name, count) in module
            .adts
            .iter()
            .map(|item| (&item.name, item.type_parameters.len()))
            .chain(
                module
                    .structs
                    .iter()
                    .map(|item| (&item.name, item.type_parameters.len())),
            )
            .chain(
                module
                    .aliases
                    .iter()
                    .map(|item| (&item.name, item.type_parameters.len())),
            )
        {
            if seseragi_semantics::prelude_type_constructor_arity(name).is_none() {
                self.arities.insert(name.clone(), count as u32);
            }
        }
        self
    }

    pub(super) fn with_private_representation(&self) -> Self {
        let mut context = self.clone();
        context.private_representation = true;
        context
    }

    pub(super) fn has_private_representations(&self) -> bool {
        !self.private_struct_names.is_empty()
    }

    pub(super) fn get(&self, canonical: &str) -> Option<&String> {
        self.imported_names.get(canonical)
    }

    pub(super) fn with_parameters(&self, parameters: &[TypeParameter]) -> Self {
        let mut scoped = self.clone();
        for parameter in parameters {
            scoped.private_struct_names.remove(&parameter.name);
            scoped.constructors.remove(&parameter.name);
            scoped.value_parameters.remove(&parameter.name);
            if parameter.is_constructor() {
                scoped.constructors.insert(parameter.name.clone());
            } else {
                scoped.value_parameters.insert(parameter.name.clone());
            }
        }
        scoped
    }
}

pub(super) fn type_ref_from_core_expr(
    expr: &CoreExpr,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptType {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptType::Undefined,
        CoreExpr::Integer { .. } => TypeScriptType::Number,
        CoreExpr::Float64 { .. } => TypeScriptType::Number,
        CoreExpr::String { .. } | CoreExpr::Char { .. } => TypeScriptType::String,
        CoreExpr::Template { .. } => TypeScriptType::String,
        CoreExpr::Boolean { .. } => TypeScriptType::Boolean,
        CoreExpr::Variable { type_ref, .. }
        | CoreExpr::Call { type_ref, .. }
        | CoreExpr::Lambda { type_ref, .. }
        | CoreExpr::Tuple { type_ref, .. }
        | CoreExpr::FieldAccess { type_ref, .. }
        | CoreExpr::OptionalFieldAccess { type_ref, .. }
        | CoreExpr::Record { type_ref, .. }
        | CoreExpr::Array { type_ref, .. }
        | CoreExpr::List { type_ref, .. }
        | CoreExpr::ArrayComprehension { type_ref, .. }
        | CoreExpr::ListComprehension { type_ref, .. }
        | CoreExpr::Binary { type_ref, .. }
        | CoreExpr::Unary { type_ref, .. }
        | CoreExpr::If { type_ref, .. }
        | CoreExpr::Decision { type_ref, .. }
        | CoreExpr::MonadDo { type_ref, .. } => type_ref_from_core_type(type_ref, imported_types),
        CoreExpr::EffectOperation { success, .. } | CoreExpr::EffectInvoke { success, .. } => {
            type_ref_from_core_type(success, imported_types)
        }
        CoreExpr::Sequence { result, .. } => type_ref_from_core_expr(result, imported_types),
    }
}

pub(super) fn lower_core_parameter_to_typescript(
    parameter: CoreParameter,
    imported_types: &TypeScriptTypeContext,
    type_constructor_parameters: &[String],
) -> TypeScriptParameter {
    TypeScriptParameter {
        name: if parameter.kind == "implicit" {
            "_unit".to_owned()
        } else {
            safe_identifier(&parameter.id)
        },
        type_name: render_typescript_type(&type_ref_from_core_type_with_erasure(
            &parameter.type_ref,
            imported_types,
            type_constructor_parameters,
        )),
        implicit: parameter.kind == "implicit",
    }
}

pub(super) fn type_ref_from_core_type(
    type_ref: &CoreType,
    imported_types: &TypeScriptTypeContext,
) -> TypeScriptType {
    type_ref_from_core_type_with_erasure(type_ref, imported_types, &[])
}

pub(super) fn type_ref_from_core_type_with_erasure(
    type_ref: &CoreType,
    imported_types: &TypeScriptTypeContext,
    type_constructor_parameters: &[String],
) -> TypeScriptType {
    match type_ref {
        CoreType::Named { name, arguments }
            if arguments.is_empty() && imported_types.value_parameters.contains(name) =>
        {
            TypeScriptType::Reference { name: name.clone(), arguments: Vec::new() }
        }
        // A source constructor is a kind, not a TypeScript value type. Erase
        // unsaturated heads wherever they occur, including nested nominal
        // arguments and dictionary instantiation, using declaration metadata.
        CoreType::Named { name, arguments }
            if imported_types.arities.get(name).copied()
                .or_else(|| seseragi_semantics::prelude_type_constructor_arity(name))
                .is_some_and(|arity| arguments.len() < arity as usize) => TypeScriptType::Unknown,
        CoreType::ExternalNamed { canonical, arguments, .. }
            if imported_types.arities.get(canonical)
                .is_some_and(|arity| arguments.len() < *arity as usize) => TypeScriptType::Unknown,
        CoreType::Named { name, arguments }
            if !arguments.is_empty()
                && (imported_types.constructors.contains(name) || type_constructor_parameters
                    .iter()
                    .any(|parameter| parameter == name)) =>
        {
            TypeScriptType::Unknown
        }
        CoreType::Named { name, arguments } if name == "Int" && arguments.is_empty() => {
            TypeScriptType::Number
        }
        CoreType::Named { name, arguments } if name == "Float" && arguments.is_empty() => {
            TypeScriptType::Number
        }
        CoreType::Named { name, arguments } if name == "String" && arguments.is_empty() => {
            TypeScriptType::String
        }
        CoreType::Named { name, arguments } if name == "Char" && arguments.is_empty() => {
            TypeScriptType::String
        }
        CoreType::Named { name, arguments } if name == "Bool" && arguments.is_empty() => {
            TypeScriptType::Boolean
        }
        CoreType::Named { name, arguments } if name == "Unit" && arguments.is_empty() => {
            TypeScriptType::Undefined
        }
        CoreType::Named { name, arguments } if name == "Never" && arguments.is_empty() => {
            TypeScriptType::Never
        }
        CoreType::Named { name, arguments } if name == "Ordering" && arguments.is_empty() => {
            TypeScriptType::Ordering
        }
        CoreType::Named { name, arguments } if name == "Maybe" && arguments.len() == 1 => {
            TypeScriptType::Maybe {
                element: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments } if name == "Either" && arguments.len() == 2 => {
            TypeScriptType::Either {
                error: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
                value: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[1],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments } if name == "Array" && arguments.len() == 1 => {
            TypeScriptType::Array {
                element: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments } if name == "List" && arguments.len() == 1 => {
            let element = type_ref_from_core_type_with_erasure(
                &arguments[0], imported_types, type_constructor_parameters,
            );
            match imported_types.imported_names.get("std/prelude::List") {
                Some(alias) if alias != "List" => TypeScriptType::Reference {
                    name: alias.clone(), arguments: vec![element],
                },
                _ => TypeScriptType::List {element: Box::new(element)},
            }
        }
        CoreType::Named { name, arguments }
            if name == "Range"
                && matches!(
                    arguments.as_slice(),
                    [CoreType::Named { name, arguments }] if name == "Int" && arguments.is_empty()
                ) =>
        {
            TypeScriptType::Range
        }
        CoreType::Named { name, arguments }
            if imported_types.private_representation
                && imported_types.private_struct_names.contains(name) =>
        {
            TypeScriptType::Reference {
                name: private_representation_name(&local_name(name)),
                arguments: arguments
                    .iter()
                    .map(|argument| {
                        type_ref_from_core_type_with_erasure(
                            argument,
                            imported_types,
                            type_constructor_parameters,
                        )
                    })
                    .collect(),
            }
        }
        CoreType::Named { name, arguments } => TypeScriptType::Reference {
            name: runtime_type_import_for_surface(name)
                .map(|type_import| safe_identifier(type_import.export_name))
                .unwrap_or_else(|| local_name(name)),
            arguments: arguments
                .iter()
                .map(|argument| {
                    type_ref_from_core_type_with_erasure(
                        argument,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::ExternalNamed {name, canonical, arguments}
            if imported_types.private_representation && imported_types.private_struct_names.contains(canonical) =>
        {
            TypeScriptType::Reference {
                name: private_representation_name(&local_name(name)),
                arguments: arguments.iter().map(|argument| type_ref_from_core_type_with_erasure(argument, imported_types, type_constructor_parameters)).collect(),
            }
        }
        // Instance selection can preserve primitive identity canonically when
        // its display name is shadowed. These builtins need no external import,
        // and must bypass lexical nominal/type-parameter name translation.
        CoreType::ExternalNamed { name, canonical, arguments }
            if arguments.is_empty()
                && canonical == &format!("std/prelude::{name}")
                && matches!(name.as_str(), "Bool" | "Int" | "Float" | "Char" | "String" | "Unit") =>
        {
            match name.as_str() {
                "Bool" => TypeScriptType::Boolean,
                "Int" | "Float" => TypeScriptType::Number,
                "Char" | "String" => TypeScriptType::String,
                "Unit" => TypeScriptType::Undefined,
                _ => unreachable!(),
            }
        }
        CoreType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => TypeScriptType::Reference {
            name: imported_types
                .get(canonical)
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "external type {canonical} ({name}) was not validated by module import planning"
                    )
                }),
            arguments: arguments
                .iter()
                .map(|argument| {
                    type_ref_from_core_type_with_erasure(
                        argument,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::Tuple { elements } => TypeScriptType::Tuple {
            elements: elements
                .iter()
                .map(|element| {
                    type_ref_from_core_type_with_erasure(
                        element,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::Record { fields, .. } => TypeScriptType::Record {
            fields: fields
                .iter()
                .map(|field| super::TypeScriptRecordTypeField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: type_ref_from_core_type_with_erasure(
                        &field.type_ref,
                        imported_types,
                        type_constructor_parameters,
                    ),
                })
                .collect(),
        },
        CoreType::Function { parameter, result } => TypeScriptType::Function {
            parameter: Box::new(type_ref_from_core_type_with_erasure(
                parameter,
                imported_types,
                type_constructor_parameters,
            )),
            result: Box::new(type_ref_from_core_type_with_erasure(
                result,
                imported_types,
                type_constructor_parameters,
            )),
        },
        CoreType::RequirementMerge { operands } => TypeScriptType::Intersection {
            operands: operands
                .iter()
                .map(|operand| {
                    type_ref_from_core_type_with_erasure(
                        operand,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::Hole => TypeScriptType::Unknown,
    }
}

pub(crate) fn render_typescript_type(type_ref: &TypeScriptType) -> String {
    match type_ref {
        TypeScriptType::Bigint => "bigint".to_owned(),
        TypeScriptType::Number => "number".to_owned(),
        TypeScriptType::Boolean => "boolean".to_owned(),
        TypeScriptType::String => "string".to_owned(),
        TypeScriptType::Undefined => "undefined".to_owned(),
        TypeScriptType::Never => "never".to_owned(),
        TypeScriptType::Ordering => {
            "{ readonly tag: \"Less\" } | { readonly tag: \"Equal\" } | { readonly tag: \"Greater\" }"
                .to_owned()
        }
        TypeScriptType::Unknown => "unknown".to_owned(),
        TypeScriptType::Reference { name, arguments } if arguments.is_empty() => name.clone(),
        TypeScriptType::Reference { name, arguments } => format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(render_typescript_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptType::Maybe { element } => {
            format!(
                "{{ readonly tag: \"Nothing\" }} | {{ readonly tag: \"Just\"; readonly value: {} }}",
                render_typescript_type(element)
            )
        }
        TypeScriptType::Either { error, value } => {
            format!(
                "{{ readonly tag: \"Left\"; readonly value: {} }} | {{ readonly tag: \"Right\"; readonly value: {} }}",
                render_typescript_type(error),
                render_typescript_type(value)
            )
        }
        TypeScriptType::Tuple { elements } => format!(
            "readonly [{}]",
            elements
                .iter()
                .map(render_typescript_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptType::Record { fields } => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|field| format!(
                    "readonly {}{}: {}",
                    format!("{:?}", field.name),
                    if field.optional { "?" } else { "" },
                    render_typescript_type(&field.type_ref)
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        TypeScriptType::Array { element } => {
            format!("ReadonlyArray<{}>", render_typescript_type(element))
        }
        TypeScriptType::List { element } => {
            format!("List<{}>", render_typescript_type(element))
        }
        TypeScriptType::Range => {
            "Readonly<{ start: number; end: number; inclusive: boolean }>".to_owned()
        }
        TypeScriptType::Function { parameter, result } => format!(
            "(argument: {}) => {}",
            render_typescript_type(parameter),
            render_typescript_type(result)
        ),
        TypeScriptType::Intersection { operands } => operands
            .iter()
            .map(render_typescript_type)
            .collect::<Vec<_>>()
            .join(" & "),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn foreign_scalar_result_uses_only_the_declared_codec_slot_and_exact_proof() {
        let context = super::TypeScriptTypeContext {
            foreign_scalar_results: [("fixture::pick".to_owned(), (2, 1, 2))]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        let scalar = |name: &str| crate::CoreType::ExternalNamed {
            name: name.into(),
            canonical: format!("std/prelude::{name}"),
            arguments: Vec::new(),
        };
        let evidence = |name: &str| crate::CoreCallEvidence {
            constraint: crate::CoreInstanceConstraint {
                name: "ForeignCodec".into(),
                trait_identity: None,
                arguments: vec![scalar(name)],
            },
            evidence: crate::CoreInstanceEvidence::Standard {
                identity: "@seseragi/foreign-codec".into(),
                type_arguments: vec![scalar(name)],
                evidence_arguments: Vec::new(),
            },
        };
        let selected = vec![evidence("String"), evidence("Bool")];
        let result = crate::CoreType::Named {
            name: "Bool".into(),
            arguments: Vec::new(),
        };
        assert_eq!(
            context.foreign_scalar_result("fixture::pick", 2, &result, &selected),
            Some(scalar("Bool"))
        );
        assert!(context
            .foreign_scalar_result("fixture::pick", 1, &result, &selected)
            .is_none());
        assert!(context
            .foreign_scalar_result("fixture::unknown", 2, &result, &selected)
            .is_none());
        assert!(context
            .foreign_scalar_result("fixture::pick", 2, &result, &selected[..1])
            .is_none());
        let wrong_result = crate::CoreType::Named {
            name: "String".into(),
            arguments: Vec::new(),
        };
        assert!(context
            .foreign_scalar_result("fixture::pick", 2, &wrong_result, &selected)
            .is_none());
        let mut forged = selected.clone();
        let crate::CoreInstanceEvidence::Standard { identity, .. } = &mut forged[1].evidence else {
            unreachable!()
        };
        *identity = "user::ForeignCodec".into();
        assert!(context
            .foreign_scalar_result("fixture::pick", 2, &result, &forged)
            .is_none());
        let mut mismatched = selected;
        mismatched[1].constraint.arguments = vec![scalar("String")];
        assert!(context
            .foreign_scalar_result("fixture::pick", 2, &result, &mismatched)
            .is_none());
    }

    use super::{
        lower_core_parameter_to_typescript, render_typescript_type, type_ref_from_core_type,
        CoreParameter, CoreType,
    };
    use crate::TypeScriptType;

    #[test]
    fn canonical_primitive_evidence_types_lower_without_external_imports() {
        for (name, expected) in [
            ("Bool", TypeScriptType::Boolean),
            ("Int", TypeScriptType::Number),
            ("Float", TypeScriptType::Number),
            ("Char", TypeScriptType::String),
            ("String", TypeScriptType::String),
            ("Unit", TypeScriptType::Undefined),
        ] {
            let ty = CoreType::ExternalNamed {
                name: name.into(),
                canonical: format!("std/prelude::{name}"),
                arguments: Vec::new(),
            };
            assert_eq!(type_ref_from_core_type(&ty, &Default::default()), expected);
        }
    }

    #[test]
    fn external_evidence_types_cannot_claim_a_mismatched_or_parameterized_primitive() {
        for (name, canonical, arguments) in [
            ("Float", "malicious/prelude::Float", Vec::new()),
            ("Bool", "std/prelude::Float", Vec::new()),
            (
                "Float",
                "std/prelude::Float",
                vec![CoreType::Named {
                    name: "Int".into(),
                    arguments: Vec::new(),
                }],
            ),
            ("Unknown", "std/prelude::Unknown", Vec::new()),
        ] {
            let ty = CoreType::ExternalNamed {
                name: name.into(),
                canonical: canonical.into(),
                arguments,
            };
            assert!(
                std::panic::catch_unwind(|| type_ref_from_core_type(&ty, &Default::default()))
                    .is_err(),
                "{ty:?}"
            );
        }
    }

    #[test]
    fn renders_qualified_nominal_types_with_local_backend_names() {
        let type_ref = CoreType::Named {
            name: "artifact/domain::Hand".to_owned(),
            arguments: Vec::new(),
        };

        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "Hand"
        );
    }

    #[test]
    fn renders_maybe_as_a_tagged_union() {
        let type_ref = CoreType::Named {
            name: "Maybe".to_owned(),
            arguments: vec![CoreType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }],
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Maybe {
                element: Box::new(TypeScriptType::String),
            }
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "{ readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: string }"
        );
    }

    #[test]
    fn renders_either_as_a_tagged_union() {
        let type_ref = CoreType::Named {
            name: "Either".to_owned(),
            arguments: vec![
                CoreType::Named {
                    name: "InputError".to_owned(),
                    arguments: Vec::new(),
                },
                CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                },
            ],
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Either {
                error: Box::new(TypeScriptType::Reference {
                    name: "InputError".to_owned(),
                    arguments: Vec::new(),
                }),
                value: Box::new(TypeScriptType::String),
            }
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "{ readonly tag: \"Left\"; readonly value: InputError } | { readonly tag: \"Right\"; readonly value: string }"
        );
    }

    #[test]
    fn renders_never_as_the_typescript_bottom_type() {
        let type_ref = CoreType::Named {
            name: "Never".to_owned(),
            arguments: Vec::new(),
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Never
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "never"
        );
    }

    #[test]
    fn renders_float_as_the_typescript_number_type() {
        let type_ref = CoreType::Named {
            name: "Float".to_owned(),
            arguments: Vec::new(),
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Number
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "number"
        );
    }

    #[test]
    fn erases_higher_kinded_parameter_applications_at_the_typescript_boundary() {
        let parameter = CoreParameter {
            id: "value".to_owned(),
            kind: "named".to_owned(),
            type_ref: CoreType::Named {
                name: "F".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                }],
            },
        };

        let lowered =
            lower_core_parameter_to_typescript(parameter, &Default::default(), &["F".to_owned()]);

        assert_eq!(lowered.type_name, "unknown");
    }
}

pub(crate) fn private_representation_name(name: &str) -> String {
    format!("__ssrg$representation${name}")
}

pub(super) fn assert_private_representation(
    value: super::TypeScriptExpr,
    type_ref: TypeScriptType,
) -> super::TypeScriptExpr {
    super::TypeScriptExpr::CheckedResult {
        value: Box::new(super::TypeScriptExpr::CheckedResult {
            value: Box::new(value),
            type_ref: TypeScriptType::Unknown,
        }),
        type_ref,
    }
}
