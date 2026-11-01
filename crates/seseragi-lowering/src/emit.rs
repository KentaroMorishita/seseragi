use crate::typescript::types::{render_typescript_kind_body, render_typescript_type};
use crate::{
    TypeScriptAdt, TypeScriptAdtVariant, TypeScriptAlias, TypeScriptBinding, TypeScriptExpr,
    TypeScriptForeignMember, TypeScriptForeignNamespace, TypeScriptForeignOpaqueType,
    TypeScriptFunction, TypeScriptModule, TypeScriptRecordValueItem, TypeScriptStatement,
    TypeScriptStruct, TypeScriptType,
};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod comprehension_tests;
#[cfg(test)]
mod constrained_function_tests;
mod decision;
mod imports;
mod instances;
mod metadata;
#[cfg(test)]
mod precedence_tests;
mod scalar_tail;
mod source_map;

use imports::render_import_lines;
use instances::render_typescript_instances;
use metadata::generated_module_for;
pub use metadata::{
    GeneratedDeprecation, GeneratedInstance, GeneratedModule, GeneratedOutputs, GeneratedRuntime,
};
use source_map::source_map_for_module;
pub use source_map::SourceMap;

/// Locations chosen by the project for a generated TypeScript module and its
/// source map.
///
/// The lowering crate preserves these paths in generated metadata and uses the
/// TypeScript path as the `file` field of the source map. It does not interpret
/// or normalize paths: project output planning owns that policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedOutputPaths {
    pub typescript: String,
    pub source_map: String,
}

impl GeneratedOutputPaths {
    pub fn new(typescript: impl Into<String>, source_map: impl Into<String>) -> Self {
        Self {
            typescript: typescript.into(),
            source_map: source_map.into(),
        }
    }
}

impl Default for GeneratedOutputPaths {
    fn default() -> Self {
        Self::new("main.ts", "main.ts.map")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedBundle {
    pub metadata: GeneratedModule,
    pub typescript: String,
    pub source_map: SourceMap,
}

pub fn emit_typescript_module(module: TypeScriptModule, source_text: &str) -> GeneratedBundle {
    emit_typescript_module_with_output_paths(module, source_text, GeneratedOutputPaths::default())
}

/// Emits a module using caller-selected generated artifact paths.
///
/// This keeps TypeScript path selection at the project boundary while making
/// the selected paths observable through both generated module metadata and
/// the source map's `file` field.
pub fn emit_typescript_module_with_output_paths(
    module: TypeScriptModule,
    source_text: &str,
    output_paths: GeneratedOutputPaths,
) -> GeneratedBundle {
    let typescript = render_typescript(&module);
    let source_map = source_map_for_module(&module, source_text, &output_paths.typescript);
    let metadata = generated_module_for(module, output_paths);

    GeneratedBundle {
        metadata,
        typescript,
        source_map,
    }
}

fn render_typescript(module: &TypeScriptModule) -> String {
    let mut output = String::new();
    let aliases = module
        .aliases
        .iter()
        .filter_map(render_type_alias)
        .collect::<Vec<_>>();
    let mut import_lines = render_import_lines(module);
    if !module.foreign_modules.is_empty() {
        import_lines.push(
            concat!(
                "import { createForeignTaskModule as _ssrg_foreign_createTaskModule, ",
                "annotateForeignTask as _ssrg_foreign_annotateTask, ",
                "invokeForeignPure as _ssrg_foreign_invokePure, ",
                "invokeForeignTask as _ssrg_foreign_invokeTask, ",
                "readForeignPureValue as _ssrg_foreign_readPureValue } ",
                "from \"@seseragi/runtime/foreign\""
            )
            .to_owned(),
        );
        for (index, foreign) in module.foreign_modules.iter().enumerate() {
            if foreign.pure_load {
                import_lines.push(format!(
                    "import * as _ssrg_foreign_module_{index} from {:?}",
                    foreign.specifier
                ));
            }
        }
    }
    for line in &import_lines {
        output.push_str(line);
        output.push('\n');
    }
    output.push_str(&format!(
        "$ssrg$assertUnicodeVersion({:?})\n",
        seseragi_syntax::unicode::UNICODE_VERSION
    ));
    if !import_lines.is_empty()
        && (!module.adts.is_empty()
            || !aliases.is_empty()
            || !module.structs.is_empty()
            || !module.instances.is_empty()
            || !module.foreign_modules.is_empty()
            || !module.bindings.is_empty()
            || !module.functions.is_empty())
    {
        output.push('\n');
    }
    render_foreign_modules(&mut output, module);
    output.push_str(&render_kind_declarations(module));
    for dictionary in &module.closed_display_dictionaries {
        // Only the getter is created at module initialization. Its complete
        // proved factory stays at the original first evidence read. Storage
        // belongs to this finite module, with no process-global cache or input
        // values retained by the standard dictionary.
        output.push_str(&format!(
            "const {} = (() => {{ const create = () => {}; let value: ReturnType<typeof create> | undefined; return (): ReturnType<typeof create> => (value ??= create()); }})()\n",
            dictionary.name, render_typescript_expr(&dictionary.factory)
        ));
    }
    for adt in &module.adts {
        render_adt(&mut output, adt);
    }
    for structure in &module.structs {
        render_struct(&mut output, structure);
    }
    for alias in aliases {
        output.push_str(&alias);
    }
    render_typescript_instances(
        &mut output,
        &module.instances,
        &module.type_imports,
        &module.structs,
        &module.adts,
    );
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction {
                exported,
                is_async,
                is_effect,
                scalar_tail_loop,
                name,
                value_type,
                type_parameters,
                constraints,
                parameters,
                body,
                ..
            } => {
                if *exported {
                    output.push_str("export ");
                }
                let parameters = evidence_parameters(parameters, 0, constraints);
                let rendered_body = render_function_body(
                    type_parameters,
                    &parameters,
                    body,
                    *is_async,
                    *is_effect,
                    Some(name),
                    *scalar_tail_loop,
                );
                let annotation = value_type
                    .as_ref()
                    .map(|ty| format!(": {}", render_typescript_type(ty)))
                    .unwrap_or_default();
                output.push_str(&format!("const {name}{annotation} = {rendered_body}\n",));
            }
        }
    }
    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const {
                type_parameters,
                exported,
                name,
                type_ref,
                initializer,
                ..
            } => {
                if *exported {
                    output.push_str("export ");
                }
                if !type_parameters.is_empty()
                    && !matches!(type_ref, crate::TypeScriptType::Function { .. })
                {
                    // TS quantifies call signatures, not data types. Check the
                    // initializer in a generic scope, then evaluate it once in
                    // source order. Reads retain their checked source monotype.
                    output.push_str(&format!(
                        "const {name} = ({}(): {} => ({}))();\n",
                        render_arrow_type_parameters(type_parameters),
                        render_typescript_type(type_ref),
                        render_typescript_expr(initializer),
                    ));
                } else {
                    output.push_str(&format!(
                        "const {name}: {}{} = {};\n",
                        render_type_parameters(type_parameters),
                        render_typescript_type(type_ref),
                        render_generic_initializer(initializer, type_parameters)
                    ));
                }
            }
        }
    }
    output
}

/// A transparent public alias needs an actual type export for facade reexports.
/// Only exact targets are exported: kind applications carry checked witnesses,
/// while unresolved holes/unsupported erasure remain metadata-only.
fn render_type_alias(alias: &TypeScriptAlias) -> Option<String> {
    if !alias.exported || !exact_alias_target(&alias.target) {
        return None;
    }
    Some(format!(
        "export type {}{} = {}\n",
        alias.name,
        render_type_parameters(&alias.type_parameters),
        render_typescript_type(&alias.target)
    ))
}

pub(crate) fn exact_alias_target(target: &TypeScriptType) -> bool {
    match target {
        TypeScriptType::Unknown => false,
        TypeScriptType::KindConstructor { arguments, .. } => {
            arguments.iter().flatten().all(exact_alias_target)
        }
        TypeScriptType::KindApplication { arguments, .. } => {
            arguments.iter().all(exact_alias_target)
        }
        TypeScriptType::KindArgument { .. } => true,
        TypeScriptType::Reference { arguments, .. } => arguments.iter().all(exact_alias_target),
        TypeScriptType::Maybe { element }
        | TypeScriptType::Array { element }
        | TypeScriptType::List { element } => exact_alias_target(element),
        TypeScriptType::Either { error, value } => {
            exact_alias_target(error) && exact_alias_target(value)
        }
        TypeScriptType::Tuple { elements } => elements.iter().all(exact_alias_target),
        TypeScriptType::Record { fields } => fields
            .iter()
            .all(|field| exact_alias_target(&field.type_ref)),
        TypeScriptType::Function { parameter, result } => {
            exact_alias_target(parameter) && exact_alias_target(result)
        }
        TypeScriptType::Intersection { operands } => operands.iter().all(exact_alias_target),
        TypeScriptType::Bigint
        | TypeScriptType::Number
        | TypeScriptType::Boolean
        | TypeScriptType::String
        | TypeScriptType::Undefined
        | TypeScriptType::Never
        | TypeScriptType::Ordering
        | TypeScriptType::Range => true,
    }
}

fn render_foreign_modules(output: &mut String, module: &TypeScriptModule) {
    for (module_index, foreign) in module.foreign_modules.iter().enumerate() {
        for foreign_type in &foreign.types {
            render_foreign_opaque_type(output, foreign_type, foreign.exported);
        }
        for namespace in &foreign.namespaces {
            render_foreign_namespace_types(output, namespace, foreign.exported);
        }
        let loader = format!("_ssrg_foreign_loader_{module_index}");
        if foreign.pure_load {
            output.push_str(&format!(
                "const {loader} = _ssrg_foreign_createTaskModule(() => Promise.resolve(_ssrg_foreign_module_{module_index}), import.meta.resolve({:?}));\n",
                foreign.specifier,
            ));
        } else {
            output.push_str(&format!(
                "const {loader} = _ssrg_foreign_createTaskModule(() => import({:?}), import.meta.resolve({:?}));\n",
                foreign.specifier,
                foreign.specifier
            ));
        }
        for value in &foreign.values {
            if foreign.exported {
                output.push_str("export ");
            }
            output.push_str(&format!(
                "const {}: {} = _ssrg_foreign_readPureValue(_ssrg_foreign_module_{module_index}, {}, {}) as {};\n",
                value.name,
                render_typescript_type(&value.type_ref),
                foreign_path(&[value.host_name.clone()]),
                value.codec,
                render_typescript_type(&value.type_ref),
            ));
        }
        for member in &foreign.members {
            render_foreign_function(
                output,
                member,
                foreign.exported,
                module_index,
                &loader,
                &[member.host_name.clone()],
            );
        }
        for namespace in &foreign.namespaces {
            render_foreign_namespace_bindings(
                output,
                namespace,
                foreign.exported,
                module_index,
                &loader,
                &[],
            );
            render_foreign_namespace(
                output,
                namespace,
                foreign.exported,
                module_index,
                &loader,
                &[],
            );
        }
    }
}

fn render_foreign_opaque_type(
    output: &mut String,
    foreign_type: &TypeScriptForeignOpaqueType,
    exported: bool,
) {
    output.push_str(&format!(
        "declare const {}: unique symbol;\n",
        foreign_type.brand
    ));
    if exported {
        output.push_str("export ");
    }
    output.push_str(&format!(
        "type {}{} = object & {{ readonly [{}]: {} }};\n",
        foreign_type.name,
        render_type_parameters(&foreign_type.type_parameters),
        foreign_type.brand,
        if foreign_type.type_parameters.is_empty() {
            "true".to_owned()
        } else {
            format!(
                "readonly [{}]",
                foreign_type
                    .type_parameters
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    ));
}

fn render_foreign_namespace_types(
    output: &mut String,
    namespace: &TypeScriptForeignNamespace,
    exported: bool,
) {
    for foreign_type in &namespace.types {
        render_foreign_opaque_type(output, foreign_type, exported);
    }
    for child in &namespace.namespaces {
        render_foreign_namespace_types(output, child, exported);
    }
}

fn render_foreign_namespace_bindings(
    output: &mut String,
    namespace: &TypeScriptForeignNamespace,
    exported: bool,
    module_index: usize,
    loader: &str,
    parent_path: &[String],
) {
    let mut path = parent_path.to_vec();
    path.push(namespace.host_name.clone());
    for value in &namespace.values {
        let mut member_path = path.clone();
        member_path.push(value.host_name.clone());
        output.push_str(&format!(
            "{}const {}: {} = _ssrg_foreign_readPureValue(_ssrg_foreign_module_{module_index}, {}, {}) as {};\n",
            if exported { "export " } else { "" },
            value.name,
            render_typescript_type(&value.type_ref),
            foreign_path(&member_path),
            value.codec,
            render_typescript_type(&value.type_ref),
        ));
    }
    for member in &namespace.members {
        let mut member_path = path.clone();
        member_path.push(member.host_name.clone());
        render_foreign_function(output, member, exported, module_index, loader, &member_path);
    }
    for child in &namespace.namespaces {
        render_foreign_namespace_bindings(output, child, exported, module_index, loader, &path);
    }
}

fn render_foreign_function(
    output: &mut String,
    member: &TypeScriptForeignMember,
    exported: bool,
    module_index: usize,
    loader: &str,
    host_path: &[String],
) {
    let body = foreign_function_body(member, module_index, loader, host_path);
    if exported {
        output.push_str("export ");
    }
    output.push_str(&format!("const {} = {body};\n", member.name));
}

fn foreign_function_body(
    member: &TypeScriptForeignMember,
    module_index: usize,
    loader: &str,
    host_path: &[String],
) -> String {
    let actual = member
        .parameters
        .iter()
        .zip(&member.parameter_codecs)
        .filter(|(parameter, _)| !parameter.implicit)
        .collect::<Vec<_>>();
    let arguments = actual
        .iter()
        .map(|(parameter, _)| parameter.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let codecs = actual
        .iter()
        .map(|(_, codec)| codec.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let invocation = match member.mode {
        seseragi_syntax::ForeignCallMode::Pure => format!(
            "_ssrg_foreign_invokePure<{}>(_ssrg_foreign_module_{module_index}, {}, {:?}, [{arguments}], [{codecs}], {})",
            render_typescript_type(&member.return_type),
            foreign_path(host_path),
            foreign_call_kind(member.call_kind),
            member.return_codec,
        ),
        seseragi_syntax::ForeignCallMode::Task => format!(
            "_ssrg_foreign_invokeTask<{}>({loader}, {}, {:?}, [{arguments}], [{codecs}], {})",
            render_typescript_type(&member.return_type),
            foreign_path(host_path),
            foreign_call_kind(member.call_kind),
            member.return_codec,
        ),
    };
    let body =
        member
            .parameters
            .iter()
            .enumerate()
            .rev()
            .fold(invocation, |body, (index, parameter)| {
                let generic = if index == 0 {
                    render_arrow_type_parameters(&member.type_parameters)
                } else {
                    String::new()
                };
                format!(
                    "{generic}({}: {}) => {body}",
                    parameter.name, parameter.type_name
                )
            });
    body
}

fn render_foreign_namespace(
    output: &mut String,
    namespace: &TypeScriptForeignNamespace,
    exported: bool,
    module_index: usize,
    loader: &str,
    parent_path: &[String],
) {
    let mut path = parent_path.to_vec();
    path.push(namespace.host_name.clone());
    let expression = foreign_namespace_expression(namespace, module_index, loader, &path);
    if exported {
        output.push_str("export ");
    }
    output.push_str(&format!(
        "const {} = Object.freeze({expression});\n",
        namespace.name
    ));
}

// Quoting __proto__ still invokes JavaScript's object-literal prototype setter.
// Computed property syntax always creates an ordinary own data property.
fn object_property_key(name: &str, quoted: bool) -> String {
    if name == "__proto__" {
        format!("[{name:?}]")
    } else if quoted {
        format!("{name:?}")
    } else {
        name.to_owned()
    }
}

fn foreign_namespace_expression(
    namespace: &TypeScriptForeignNamespace,
    module_index: usize,
    loader: &str,
    path: &[String],
) -> String {
    let mut fields = Vec::new();
    for value in &namespace.values {
        fields.push(format!(
            "{}: {}",
            object_property_key(&value.field_name, false),
            value.name
        ));
    }
    for member in &namespace.members {
        fields.push(format!(
            "{}: {}",
            object_property_key(&member.field_name, false),
            member.name
        ));
    }
    for child in &namespace.namespaces {
        let mut child_path = path.to_vec();
        child_path.push(child.host_name.clone());
        fields.push(format!(
            "{}: Object.freeze({})",
            object_property_key(&child.name, false),
            foreign_namespace_expression(child, module_index, loader, &child_path)
        ));
    }
    format!("{{ {} }}", fields.join(", "))
}

fn foreign_path(path: &[String]) -> String {
    format!(
        "[{}]",
        path.iter()
            .map(|segment| format!("{segment:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn foreign_call_kind(kind: seseragi_syntax::ForeignCallKind) -> &'static str {
    match kind {
        seseragi_syntax::ForeignCallKind::Function => "function",
        seseragi_syntax::ForeignCallKind::Constructor => "constructor",
        seseragi_syntax::ForeignCallKind::Method => "method",
        seseragi_syntax::ForeignCallKind::Property => "property",
    }
}

pub(super) fn evidence_parameters(
    parameters: &[crate::TypeScriptParameter],
    start_index: usize,
    constraints: &[crate::TypeScriptInstanceConstraint],
) -> Vec<crate::TypeScriptParameter> {
    let mut rendered = parameters.to_vec();
    rendered.extend(constraints.iter().enumerate().map(|(offset, constraint)| {
        crate::TypeScriptParameter {
            name: crate::typescript::evidence_parameter_name(start_index + offset),
            type_name: evidence_type(constraint),
            implicit: true,
        }
    }));
    rendered
}

pub(super) fn evidence_type(constraint: &crate::TypeScriptInstanceConstraint) -> String {
    let dictionary = match constraint.trait_identity.as_deref() {
        Some("std/prelude::JsonEncode") if constraint.name == "JsonEncode" => "JsonEncode",
        Some("std/prelude::JsonDecode") if constraint.name == "JsonDecode" => "JsonDecode",
        _ => return ERASED_EVIDENCE_TYPE.to_owned(),
    };
    let [argument] = constraint.arguments.as_slice() else {
        return ERASED_EVIDENCE_TYPE.to_owned();
    };
    // Keep the selected value type at every evidence stage. An index signature
    // does not promise the required method to the strict JSON runtime API.
    // A type-only import also works in local functions and instance factories
    // whose enclosing source signature does not otherwise mention Json.
    format!(
        "import(\"@seseragi/runtime/json\").{dictionary}<{}>",
        render_typescript_type(argument)
    )
}

fn render_adt(output: &mut String, adt: &TypeScriptAdt) {
    if adt.erased_newtype {
        let variant = adt.variants.first().expect("newtype constructor");
        let payload = variant.payload.as_ref().expect("newtype payload");
        let parameters = render_type_parameters(&adt.type_parameters);
        let arguments = render_type_arguments(&adt.type_parameters);
        let export = if adt.exported { "export " } else { "" };
        output.push_str(&format!("declare const __ssrg$newtype${}: unique symbol;\n{export}type {}{parameters} = {} & {{ readonly [__ssrg$newtype${}]: true }};\n", adt.name, adt.name, render_typescript_type(payload), adt.name));
        let export = if variant.exported { "export " } else { "" };
        output.push_str(&format!("{export}const {} = {parameters}(value: {}): {}{arguments} => value as {}{arguments};\n", variant.name, render_typescript_type(payload), adt.name, adt.name));
        return;
    }
    if adt.exported {
        output.push_str("export ");
    }
    let type_parameters = render_type_parameters(&adt.type_parameters);
    if adt.variants.is_empty() {
        output.push_str(&format!("type {}{type_parameters} = never;\n", adt.name));
    } else {
        output.push_str(&format!("type {}{type_parameters} =\n", adt.name));
        for (index, variant) in adt.variants.iter().enumerate() {
            let suffix = if index + 1 == adt.variants.len() {
                ";"
            } else {
                ""
            };
            output.push_str(&format!(
                "  | {}{suffix}\n",
                render_adt_variant_type(variant)
            ));
        }
    }

    let result_type = format!(
        "{}{}",
        adt.name,
        render_type_arguments(&adt.type_parameters)
    );
    for variant in &adt.variants {
        if variant.exported {
            output.push_str("export ");
        }
        let tag = format!("{:?}", variant.tag);
        match &variant.payload {
            None if adt.type_parameters.is_empty() => output.push_str(&format!(
                "const {}: {result_type} = {{ tag: {tag} }} as const;\n",
                variant.name
            )),
            None => output.push_str(&format!(
                "const {} = {{ tag: {tag} }} as const;\n",
                variant.name
            )),
            Some(payload) => {
                let generic_parameters = render_type_parameters(&adt.type_parameters);
                output.push_str(&format!(
                    "const {} = {generic_parameters}(value: {}): {result_type} => ({{ tag: {tag}, value }} as const);\n",
                    variant.name,
                    render_typescript_type(payload),
                ));
            }
        }
    }
}

fn render_struct(output: &mut String, structure: &TypeScriptStruct) {
    output.push_str(&format!(
        "declare const {}: unique symbol;\n",
        structure.brand
    ));
    if structure.exported {
        output.push_str("export ");
    }
    let type_parameters = render_type_parameters(&structure.type_parameters);
    output.push_str(&format!("type {}{type_parameters} = {{\n", structure.name));
    if !structure.opaque {
        render_struct_fields(output, &structure.fields);
    }
    let brand_type = if structure.opaque && !structure.type_parameters.is_empty() {
        format!(
            "readonly [{}]",
            structure
                .type_parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        "true".to_owned()
    };
    output.push_str(&format!(
        "  readonly [{}]: {brand_type};\n",
        structure.brand
    ));
    output.push_str("};\n");
    if let Some(fields) = &structure.private_fields {
        output.push_str(&format!(
            "type {}{type_parameters} = {{\n",
            crate::typescript::types::private_representation_name(&structure.name)
        ));
        render_struct_fields(output, fields);
        output.push_str(&format!(
            "  readonly [{}]: {brand_type};\n}};\n",
            structure.brand
        ));
    }
}

fn render_struct_fields(output: &mut String, fields: &[crate::TypeScriptRecordTypeField]) {
    for field in fields {
        output.push_str(&format!(
            "  readonly {:?}: {};\n",
            field.name,
            render_typescript_type(&field.type_ref)
        ));
    }
}

fn render_adt_variant_type(variant: &TypeScriptAdtVariant) -> String {
    let tag = format!("{:?}", variant.tag);
    match &variant.payload {
        Some(payload) => format!(
            "{{ readonly tag: {tag}; readonly value: {} }}",
            render_typescript_type(payload)
        ),
        None => format!("{{ readonly tag: {tag} }}"),
    }
}

fn render_generic_initializer(
    value: &TypeScriptExpr,
    parameters: &[seseragi_syntax::TypeParameter],
) -> String {
    let rendered = render_typescript_expr(value);
    if !parameters.is_empty() && matches!(value, TypeScriptExpr::Lambda { .. }) {
        format!("{}{}", render_arrow_type_parameters(parameters), rendered)
    } else {
        rendered
    }
}

fn render_type_parameters(parameters: &[seseragi_syntax::TypeParameter]) -> String {
    if parameters.is_empty() {
        String::new()
    } else {
        format!(
            "<{}>",
            parameters
                .iter()
                .map(render_type_parameter)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn render_type_arguments(parameters: &[seseragi_syntax::TypeParameter]) -> String {
    if parameters.is_empty() {
        String::new()
    } else {
        format!(
            "<{}>",
            parameters
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn render_type_parameter(parameter: &seseragi_syntax::TypeParameter) -> String {
    if parameter.is_constructor() {
        format!(
            "{} extends {}",
            parameter.name,
            crate::typescript::kinds::kind_constraint(parameter.arity)
        )
    } else {
        parameter.name.clone()
    }
}

fn render_kind_declarations(module: &TypeScriptModule) -> String {
    let Some(kinds) = &module.higher_kinds else {
        return String::new();
    };
    let mut output = concat!(
        "interface $ssrg$Kind<Args extends readonly unknown[] = readonly unknown[]> { readonly args: Args; readonly type: unknown }\n",
        "type $ssrg$Invoke<F extends $ssrg$Kind, Args extends readonly unknown[]> = (F & { readonly args: Args })[\"type\"];\n",
        "type $ssrg$Apply<F extends $ssrg$Kind, Args extends F[\"args\"]> = $ssrg$Invoke<F, Args>;\n",
    ).to_owned();
    for shape in &kinds.partials {
        let arity = shape.len() as u32;
        assert!(
            shape.iter().any(|fixed| *fixed) && shape.iter().any(|fixed| !fixed),
            "checked partial kind application"
        );
        let mut fixed_index = 0;
        let mut hole_index = 0;
        let mut parameters = Vec::new();
        let mut holes = Vec::new();
        let arguments = shape
            .iter()
            .enumerate()
            .map(|(slot, &fixed)| {
                if fixed {
                    let name = format!("P{fixed_index}");
                    fixed_index += 1;
                    parameters.push(format!("{name} extends F[\"args\"][{slot}]"));
                    name
                } else {
                    holes.push(format!("F[\"args\"][{slot}]"));
                    let argument = format!("this[\"args\"][{hole_index}]");
                    hole_index += 1;
                    argument
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "interface {}<F extends {}, {}> extends $ssrg$Kind<readonly [{}]> {{ readonly type: $ssrg$Invoke<F, readonly [{arguments}]> }}\n",
            crate::typescript::kinds::partial_name(shape), crate::typescript::kinds::kind_constraint(arity), parameters.join(", "), holes.join(", "),
        ));
    }
    for constructor in &kinds.constructors {
        if module.aliases.iter().any(|alias| {
            constructor.name == crate::typescript::kinds::constructor_name(&alias.name)
                && !exact_alias_target(&alias.target)
        }) {
            continue;
        }
        let parameter_kinds = constructor
            .parameters
            .iter()
            .map(|parameter| {
                if parameter.is_constructor() {
                    crate::typescript::kinds::kind_constraint(parameter.arity)
                } else {
                    "unknown".to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let target = if constructor.identity.starts_with("type-function:")
            && kind_body_has_object(&constructor.target)
        {
            // Inline object types introduce a scope where polymorphic `this`
            // is unavailable. Pass each checked slot to a type-only helper;
            // keep `this` at the enclosing witness interface's member.
            let helper = format!("{}$body", constructor.name);
            let parameters = constructor
                .parameters
                .iter()
                .enumerate()
                .map(|(index, parameter)| {
                    if parameter.is_constructor() {
                        format!(
                            "$ssrg$T{index} extends {}",
                            crate::typescript::kinds::kind_constraint(parameter.arity)
                        )
                    } else {
                        format!("$ssrg$T{index}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!(
                "type {helper}<{parameters}> = {};\n",
                render_typescript_kind_body(&constructor.target)
            ));
            let arguments = (0..constructor.parameters.len())
                .map(|index| format!("this[\"args\"][{index}]"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{helper}<{arguments}>")
        } else {
            render_typescript_type(&constructor.target)
        };
        output.push_str(&format!(
            "{}interface {} extends $ssrg$Kind<readonly [{parameter_kinds}]> {{ readonly type: {} }}\n",
            if constructor.exported { "export " } else { "" }, constructor.name,
            target,
        ));
    }
    output
}

fn kind_body_has_object(value: &TypeScriptType) -> bool {
    match value {
        TypeScriptType::Record { .. }
        | TypeScriptType::Maybe { .. }
        | TypeScriptType::Either { .. } => true,
        TypeScriptType::Function { parameter, result } => {
            kind_body_has_object(parameter) || kind_body_has_object(result)
        }
        TypeScriptType::Array { element } | TypeScriptType::List { element } => {
            kind_body_has_object(element)
        }
        TypeScriptType::Tuple { elements } => elements.iter().any(kind_body_has_object),
        TypeScriptType::Reference { arguments, .. }
        | TypeScriptType::KindApplication { arguments, .. } => {
            arguments.iter().any(kind_body_has_object)
        }
        TypeScriptType::KindConstructor { arguments, .. } => {
            arguments.iter().flatten().any(kind_body_has_object)
        }
        TypeScriptType::Intersection { operands } => operands.iter().any(kind_body_has_object),
        _ => false,
    }
}

fn render_function_body(
    type_parameters: &[seseragi_syntax::TypeParameter],
    parameters: &[crate::TypeScriptParameter],
    body: &TypeScriptExpr,
    is_async: bool,
    is_effect: bool,
    self_name: Option<&str>,
    scalar_tail_loop: bool,
) -> String {
    render_function_body_with_return(
        type_parameters,
        parameters,
        body,
        is_async,
        is_effect,
        self_name,
        None,
        scalar_tail_loop,
    )
}

fn render_function_body_with_return(
    type_parameters: &[seseragi_syntax::TypeParameter],
    parameters: &[crate::TypeScriptParameter],
    body: &TypeScriptExpr,
    is_async: bool,
    is_effect: bool,
    self_name: Option<&str>,
    return_type: Option<&crate::TypeScriptType>,
    scalar_tail_loop: bool,
) -> String {
    let rendered_body = self_name
        .filter(|_| !is_effect)
        .filter(|name| !parameters.iter().any(|parameter| parameter.name == *name))
        .filter(|name| contains_direct_self_tail_call(body, name, parameters.len()))
        .map_or_else(
            || render_typescript_expr(body),
            |name| {
                if scalar_tail_loop {
                    scalar_tail::render(body, name, parameters)
                } else {
                    render_tail_recursive_body(body, name, parameters)
                }
            },
        );
    let Some((last, leading)) = parameters.split_last() else {
        return rendered_body;
    };
    let async_prefix = if is_async { "async " } else { "" };
    let generic_prefix = render_arrow_type_parameters(type_parameters);
    let final_generic_prefix = if leading.is_empty() {
        generic_prefix.as_str()
    } else {
        ""
    };
    let return_annotation = return_type
        .map(|ty| format!(": {}", render_typescript_type(ty)))
        .unwrap_or_default();
    let final_arrow = format!(
        "{async_prefix}{final_generic_prefix}({}: {}){return_annotation} => {rendered_body}",
        last.name, last.type_name
    );
    let rendered = leading.iter().rev().fold(final_arrow, |result, parameter| {
        format!("({}: {}) => {result}", parameter.name, parameter.type_name)
    });
    if leading.is_empty() || generic_prefix.is_empty() {
        rendered
    } else {
        format!("{generic_prefix}{rendered}")
    }
}

// Only calls in the scope of the original self binder enter the trampoline.
fn statements_bind_name(statements: &[TypeScriptStatement], name: &str) -> bool {
    statements.iter().any(|statement| match statement {
        TypeScriptStatement::PureLet { name: bound, .. }
        | TypeScriptStatement::Const { name: bound, .. }
        | TypeScriptStatement::LocalFunction { name: bound, .. } => bound == name,
        TypeScriptStatement::Effect { .. } => false,
    })
}

fn contains_direct_self_tail_call(expr: &TypeScriptExpr, self_name: &str, arity: usize) -> bool {
    match expr {
        TypeScriptExpr::Call {
            callee, arguments, ..
        } => callee == self_name && arguments.len() == arity,
        TypeScriptExpr::Conditional {
            then_branch,
            else_branch,
            ..
        } => {
            contains_direct_self_tail_call(then_branch, self_name, arity)
                || contains_direct_self_tail_call(else_branch, self_name, arity)
        }
        TypeScriptExpr::Decision { branches, .. } => branches.iter().any(|branch| {
            !branch
                .bindings
                .iter()
                .any(|binding| binding.name == self_name)
                && contains_direct_self_tail_call(&branch.value, self_name, arity)
        }),
        TypeScriptExpr::Sequence { statements, result } => {
            !statements_bind_name(statements, self_name)
                && contains_direct_self_tail_call(result, self_name, arity)
        }
        _ => false,
    }
}

fn render_tail_recursive_body(
    body: &TypeScriptExpr,
    self_name: &str,
    parameters: &[crate::TypeScriptParameter],
) -> String {
    let rendered_body = render_tail_position_expr(body, self_name, parameters.len());
    let argument_types = parameters
        .iter()
        .map(|parameter| parameter.type_name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let initial_arguments = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    // Curried arguments belong to the reusable outer closures, and closures
    // created by the body must retain the values of their own iteration.
    // Keep mutable loop state separate from both sets of lexical bindings.
    let iteration_bindings = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            format!(
                "const {}: {} = $ssrg$loopArguments[{index}];",
                parameter.name, parameter.type_name
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{{ const $ssrg$tail = Symbol(); let $ssrg$loopArguments: readonly [{argument_types}] = [{initial_arguments}]; while (true) {{ {iteration_bindings} const $ssrg$result = {rendered_body}; const $ssrg$tailResult: unknown = $ssrg$result; if (typeof $ssrg$tailResult === \"object\" && $ssrg$tailResult !== null && $ssrg$tail in $ssrg$tailResult) {{ const $ssrg$arguments = ($ssrg$tailResult as {{ readonly [$ssrg$tail]: readonly [{argument_types}] }})[$ssrg$tail]; $ssrg$loopArguments = $ssrg$arguments; continue; }} return $ssrg$result; }} }}"
    )
}

fn render_tail_position_expr(expr: &TypeScriptExpr, self_name: &str, arity: usize) -> String {
    match expr {
        TypeScriptExpr::Call {
            callee, arguments, ..
        } if callee == self_name && arguments.len() == arity => {
            format!(
                "({{ [$ssrg$tail]: [{}] }} as never)",
                arguments
                    .iter()
                    .map(render_typescript_expr)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => format!(
            "{} ? {} : {}",
            render_conditional_condition(condition),
            render_tail_position_expr(then_branch, self_name, arity),
            render_tail_position_expr(else_branch, self_name, arity)
        ),
        TypeScriptExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
        } => decision::render_decision_with_branch_renderer(
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
            &|branch| {
                if branch
                    .bindings
                    .iter()
                    .any(|binding| binding.name == self_name)
                {
                    render_typescript_expr(&branch.value)
                } else {
                    render_tail_position_expr(&branch.value, self_name, arity)
                }
            },
        ),
        TypeScriptExpr::Sequence { statements, result } => {
            if statements_bind_name(statements, self_name) {
                render_typescript_expr(expr)
            } else {
                render_effect_sequence_with_result_renderer(statements, result, &|value| {
                    render_tail_position_expr(value, self_name, arity)
                })
            }
        }
        _ => render_typescript_expr(expr),
    }
}

fn render_arrow_type_parameters(parameters: &[seseragi_syntax::TypeParameter]) -> String {
    let names = parameters
        .iter()
        .map(render_type_parameter)
        .collect::<Vec<_>>();
    match names.as_slice() {
        [] => String::new(),
        [parameter] => format!("<{parameter},>"),
        _ => format!("<{},>", names.join(", ")),
    }
}

fn render_typescript_expr(expr: &TypeScriptExpr) -> String {
    match expr {
        TypeScriptExpr::Undefined => "undefined".to_owned(),
        TypeScriptExpr::Bigint { value } => format!("{value}n"),
        TypeScriptExpr::Number { value } => value.clone(),
        TypeScriptExpr::String { value } => format!("{value:?}"),
        TypeScriptExpr::Boolean { value } => value.to_string(),
        TypeScriptExpr::Identifier { name } => name.clone(),
        TypeScriptExpr::RuntimeReference { name } => name.clone(),
        TypeScriptExpr::CurriedRuntimeReference { name, arity } => {
            let parameters = (0..*arity)
                .map(|index| format!("_argument{index}"))
                .collect::<Vec<_>>();
            let call = format!("{name}({})", parameters.join(", "));
            parameters
                .into_iter()
                .rev()
                .fold(call, |body, parameter| format!("({parameter}) => {body}"))
        }
        TypeScriptExpr::Tuple { elements } => format!(
            "[{}] as const",
            elements
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptExpr::FieldAccess { receiver, field } => format!(
            "({})[{}]",
            render_typescript_expr(receiver),
            format!("{field:?}")
        ),
        TypeScriptExpr::OptionalFieldAccess {
            receiver,
            field,
            value_type,
            just_constructor,
            nothing_constructor,
        } => {
            let field = format!("{field:?}");
            format!(
                "(($ssrg_record) => Object.prototype.hasOwnProperty.call($ssrg_record, {field}) ? {just_constructor}($ssrg_record[{field}] as {}) : {nothing_constructor})({})",
                render_typescript_type(value_type),
                render_typescript_expr(receiver)
            )
        }
        TypeScriptExpr::Record {
            items,
            asserted_type,
        } => {
            let value = format!(
                "({{ {} }} as const)",
                items
                    .iter()
                    .map(|item| match item {
                        TypeScriptRecordValueItem::Field { name, value } => format!(
                            "{}: {}",
                            object_property_key(name, true),
                            render_typescript_expr(value)
                        ),
                        TypeScriptRecordValueItem::Spread { value } => {
                            format!("...{}", render_typescript_expr(value))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            match asserted_type {
                Some(type_ref) => format!(
                    "({value} as unknown as {})",
                    render_typescript_type(type_ref)
                ),
                // Preserve inferred own fields while marking absent Object members.
                None => format!(
                    "(<$ssrg_record_type,>($ssrg_record: $ssrg_record_type) => $ssrg_record as $ssrg_record_type & {{ readonly [K in keyof globalThis.Object as K extends keyof $ssrg_record_type ? never : K]?: never }})({value})"
                ),
            }
        }
        TypeScriptExpr::Array {
            elements,
            element_type,
        } => {
            let literal = format!(
                "[{}]",
                elements
                    .iter()
                    .map(render_typescript_expr)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if elements.is_empty() {
                format!(
                    "{literal} as ReadonlyArray<{}>",
                    render_typescript_type(element_type)
                )
            } else {
                literal
            }
        }
        TypeScriptExpr::Lambda { parameter, body } => {
            format!("({parameter}) => {}", render_typescript_expr(body))
        }
        TypeScriptExpr::Binary {
            operator,
            left,
            right,
        } => format!(
            "{} {operator} {}",
            render_binary_operand(left, operator, BinaryOperandSide::Left),
            render_binary_operand(right, operator, BinaryOperandSide::Right)
        ),
        TypeScriptExpr::Unary { operator, operand } => {
            format!("{operator}({})", render_typescript_expr(operand))
        }
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => format!(
            "{} ? {} : {}",
            render_conditional_condition(condition),
            render_typescript_expr(then_branch),
            render_typescript_expr(else_branch)
        ),
        TypeScriptExpr::Decision {
            scrutinee,
            scrutinee_type,
            branches,
            type_ref,
        } => decision::render_decision(scrutinee, scrutinee_type, branches, type_ref),
        TypeScriptExpr::Call {
            callee,
            arguments,
            type_arguments,
            checked_callee_type,
        } => {
            let callee = checked_callee_type.as_ref().map_or_else(
                || callee.clone(),
                |type_ref| {
                    format!(
                        "({callee} as unknown as {})",
                        render_typescript_type(type_ref)
                    )
                },
            );
            let callee = if type_arguments.is_empty() {
                callee
            } else {
                format!(
                    "{callee}<{}>",
                    type_arguments
                        .iter()
                        .map(render_typescript_type)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            if arguments.is_empty() {
                return format!("{callee}()");
            }
            arguments.iter().fold(callee, |call, argument| {
                format!("{call}({})", render_typescript_expr(argument))
            })
        }
        TypeScriptExpr::ForeignTaskCall {
            callee,
            arguments,
            function,
            module,
            origin,
        } => {
            let call = if arguments.is_empty() {
                format!("{callee}()")
            } else {
                arguments.iter().fold(callee.clone(), |call, argument| {
                    format!("{call}({})", render_typescript_expr(argument))
                })
            };
            format!(
                "_ssrg_foreign_annotateTask({call}, {{ language: \"seseragi\", function: {function:?}, uri: {:?}, range: {{ start: {}, end: {} }}, generated: false }})",
                format!("seseragi://{module}"),
                origin.start,
                origin.end,
            )
        }
        TypeScriptExpr::CheckedResult { value, type_ref } => format!(
            "(({}) as {})",
            render_typescript_expr(value),
            render_typescript_type(type_ref)
        ),
        TypeScriptExpr::TypeApplicationCall {
            callee,
            type_arguments,
            arguments,
        } => {
            let rendered_callee = if type_arguments.is_empty() {
                callee.clone()
            } else {
                let rendered_types = type_arguments
                    .iter()
                    .map(render_typescript_type)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{callee}<{rendered_types}>")
            };
            let rendered_arguments = arguments
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{rendered_callee}({rendered_arguments})")
        }
        TypeScriptExpr::DictionaryCall {
            dictionary,
            method,
            arguments,
        } => arguments.iter().fold(
            format!("{}[{method:?}]", render_typescript_expr(dictionary)),
            |call, argument| format!("{call}({})", render_typescript_expr(argument)),
        ),
        TypeScriptExpr::RuntimeCall { callee, arguments } => {
            let rendered_arguments = arguments
                .iter()
                .map(render_typescript_expr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{callee}({rendered_arguments})")
        }
        TypeScriptExpr::Await { value } => format!("await ({})", render_typescript_expr(value)),
        TypeScriptExpr::Sequence { statements, result } => {
            render_effect_sequence(statements, result)
        }
        TypeScriptExpr::MonadDo {
            dictionary,
            statements,
            result,
        } => render_monad_sequence(dictionary, statements, result),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BinaryOperandSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BinaryAssociativity {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ExpressionPrecedence(u8);

const LAMBDA_PRECEDENCE: ExpressionPrecedence = ExpressionPrecedence(0);
const CONDITIONAL_PRECEDENCE: ExpressionPrecedence = ExpressionPrecedence(1);
const PRIMARY_PRECEDENCE: ExpressionPrecedence = ExpressionPrecedence(15);

fn render_binary_operand(
    operand: &TypeScriptExpr,
    parent_operator: &str,
    side: BinaryOperandSide,
) -> String {
    let rendered = render_typescript_expr(operand);
    if binary_operand_needs_parentheses(operand, parent_operator, side) {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn binary_operand_needs_parentheses(
    operand: &TypeScriptExpr,
    parent_operator: &str,
    side: BinaryOperandSide,
) -> bool {
    let (parent_precedence, associativity) = binary_operator_properties(parent_operator);
    let operand_precedence = expression_precedence(operand);

    if nullish_logical_mix(parent_operator, operand) {
        return true;
    }
    if parent_operator == "**"
        && side == BinaryOperandSide::Left
        && matches!(
            operand,
            TypeScriptExpr::Unary { .. } | TypeScriptExpr::Await { .. }
        )
    {
        return true;
    }
    if operand_precedence < parent_precedence {
        return true;
    }
    if operand_precedence > parent_precedence {
        return false;
    }

    matches!(
        (associativity, side),
        (BinaryAssociativity::Left, BinaryOperandSide::Right)
            | (BinaryAssociativity::Right, BinaryOperandSide::Left)
    )
}

fn render_conditional_condition(condition: &TypeScriptExpr) -> String {
    let rendered = render_typescript_expr(condition);
    if expression_precedence(condition) <= CONDITIONAL_PRECEDENCE {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn expression_precedence(expr: &TypeScriptExpr) -> ExpressionPrecedence {
    match expr {
        TypeScriptExpr::Lambda { .. } | TypeScriptExpr::CurriedRuntimeReference { .. } => {
            LAMBDA_PRECEDENCE
        }
        TypeScriptExpr::Conditional { .. } => CONDITIONAL_PRECEDENCE,
        TypeScriptExpr::Binary { operator, .. } => binary_operator_properties(operator).0,
        TypeScriptExpr::Unary { .. } | TypeScriptExpr::Await { .. } => ExpressionPrecedence(14),
        _ => PRIMARY_PRECEDENCE,
    }
}

fn binary_operator_properties(operator: &str) -> (ExpressionPrecedence, BinaryAssociativity) {
    let (precedence, associativity) = match operator {
        "??" => (2, BinaryAssociativity::Left),
        "||" => (3, BinaryAssociativity::Left),
        "&&" => (4, BinaryAssociativity::Left),
        "|" => (5, BinaryAssociativity::Left),
        "^" => (6, BinaryAssociativity::Left),
        "&" => (7, BinaryAssociativity::Left),
        "==" | "!=" | "===" | "!==" => (8, BinaryAssociativity::Left),
        "<" | "<=" | ">" | ">=" | "in" | "instanceof" => (9, BinaryAssociativity::Left),
        "<<" | ">>" | ">>>" => (10, BinaryAssociativity::Left),
        "+" | "-" => (11, BinaryAssociativity::Left),
        "*" | "/" | "%" => (12, BinaryAssociativity::Left),
        "**" => (13, BinaryAssociativity::Right),
        _ => (2, BinaryAssociativity::Left),
    };
    (ExpressionPrecedence(precedence), associativity)
}

fn nullish_logical_mix(parent_operator: &str, operand: &TypeScriptExpr) -> bool {
    let TypeScriptExpr::Binary {
        operator: child_operator,
        ..
    } = operand
    else {
        return false;
    };
    (parent_operator == "??" && matches!(child_operator.as_str(), "&&" | "||"))
        || (matches!(parent_operator, "&&" | "||") && child_operator == "??")
}

fn recursive_group_prefix(statements: &[TypeScriptStatement]) -> Option<(String, usize)> {
    let TypeScriptStatement::LocalFunction {
        rec_group: Some(group),
        ..
    } = statements.first()?
    else {
        return None;
    };
    let mut declarations = Vec::new();
    for statement in statements {
        let TypeScriptStatement::LocalFunction {
            rec_group: Some(member_group),
            return_type,
            name,
            type_parameters,
            constraints,
            parameters,
            body,
            ..
        } = statement
        else {
            break;
        };
        if member_group != group {
            break;
        }
        let parameters = evidence_parameters(parameters, 0, constraints);
        declarations.push(format!(
            "const {name} = {};",
            render_function_body_with_return(
                type_parameters,
                &parameters,
                body,
                false,
                false,
                Some(name),
                return_type.as_ref(),
                false,
            )
        ));
    }
    Some((declarations.join(" "), declarations.len()))
}

fn render_monad_sequence(
    dictionary: &TypeScriptExpr,
    statements: &[TypeScriptStatement],
    result: &TypeScriptExpr,
) -> String {
    let Some((statement, rest)) = statements.split_first() else {
        return render_typescript_expr(result);
    };
    if let Some((declarations, count)) = recursive_group_prefix(statements) {
        let continuation = render_monad_sequence(dictionary, &statements[count..], result);
        return format!("(() => {{ {declarations} return {continuation}; }})()");
    }
    let continuation = render_monad_sequence(dictionary, rest, result);
    let flat_map = format!("{}[\"flatMap\"]", render_typescript_expr(dictionary));
    match statement {
        TypeScriptStatement::Effect { value } => format!(
            "{flat_map}((_ignored: unknown) => {continuation})({})",
            render_typescript_expr(value)
        ),
        TypeScriptStatement::PureLet {
            type_parameters,
            name,
            type_ref,
            initializer,
            initializer_in_outer_scope,
            ..
        } => render_pure_let(
            name,
            type_parameters,
            type_ref,
            initializer,
            *initializer_in_outer_scope,
            &continuation,
        ),
        TypeScriptStatement::Const {
            name,
            type_ref,
            initializer,
            ..
        } => format!(
            "{flat_map}(({name}: {}) => {continuation})({})",
            render_typescript_type(type_ref),
            render_typescript_expr(initializer)
        ),
        TypeScriptStatement::LocalFunction {
            initializer_in_outer_scope,
            name,
            type_parameters,
            constraints,
            parameters,
            body,
            ..
        } => {
            let parameters = evidence_parameters(parameters, 0, constraints);
            let value = render_function_body(
                type_parameters,
                &parameters,
                body,
                false,
                false,
                (!initializer_in_outer_scope).then_some(name.as_str()),
                false,
            );
            if *initializer_in_outer_scope {
                // An IIFE argument is evaluated in the preceding scope. Its
                // inferred parameter type retains the generic callable ABI.
                format!("(({name}) => {{ return {continuation}; }})({value})")
            } else {
                format!("(() => {{ const {name} = {value}; return {continuation}; }})()")
            }
        }
    }
}

fn render_effect_sequence(statements: &[TypeScriptStatement], result: &TypeScriptExpr) -> String {
    render_effect_sequence_with_result_renderer(statements, result, &render_typescript_expr)
}

fn render_effect_sequence_with_result_renderer(
    statements: &[TypeScriptStatement],
    result: &TypeScriptExpr,
    render_result: &dyn Fn(&TypeScriptExpr) -> String,
) -> String {
    let Some((statement, rest)) = statements.split_first() else {
        return render_result(result);
    };
    if let Some((declarations, count)) = recursive_group_prefix(statements) {
        let continuation = render_effect_sequence_with_result_renderer(
            &statements[count..],
            result,
            render_result,
        );
        return format!("(() => {{ {declarations} return {continuation}; }})()");
    }
    let continuation = render_effect_sequence_with_result_renderer(rest, result, render_result);
    match statement {
        TypeScriptStatement::Effect { value } => format!(
            "_ssrg_effect_flatMap({}, () => {continuation})",
            render_typescript_expr(value)
        ),
        TypeScriptStatement::PureLet {
            type_parameters,
            name,
            type_ref,
            initializer,
            initializer_in_outer_scope,
            ..
        } => render_pure_let(
            name,
            type_parameters,
            type_ref,
            initializer,
            *initializer_in_outer_scope,
            &continuation,
        ),
        TypeScriptStatement::Const {
            name,
            type_ref,
            initializer,
            ..
        } => format!(
            "_ssrg_effect_flatMap({}, ({name}: {}) => {continuation})",
            render_typescript_expr(initializer),
            render_typescript_type(type_ref)
        ),
        TypeScriptStatement::LocalFunction {
            initializer_in_outer_scope,
            name,
            type_parameters,
            constraints,
            parameters,
            body,
            ..
        } => {
            let parameters = evidence_parameters(parameters, 0, constraints);
            let value = render_function_body(
                type_parameters,
                &parameters,
                body,
                false,
                false,
                (!initializer_in_outer_scope).then_some(name.as_str()),
                false,
            );
            if *initializer_in_outer_scope {
                // An IIFE argument is evaluated in the preceding scope. Its
                // inferred parameter type retains the generic callable ABI.
                format!("(({name}) => {{ return {continuation}; }})({value})")
            } else {
                format!("(() => {{ const {name} = {value}; return {continuation}; }})()")
            }
        }
    }
}

fn render_pure_let(
    name: &str,
    type_parameters: &[seseragi_syntax::TypeParameter],
    type_ref: &crate::TypeScriptType,
    initializer: &TypeScriptExpr,
    initializer_in_outer_scope: bool,
    continuation: &str,
) -> String {
    let annotation = format!(
        "{}{}",
        render_type_parameters(type_parameters),
        render_typescript_type(type_ref)
    );
    let value = render_generic_initializer(initializer, type_parameters);
    if initializer_in_outer_scope {
        // Arguments are evaluated in the outer lexical environment. The new
        // parameter is immutable and exists only in the continuation's scope.
        format!("(({name}: {annotation}) => {{ return {continuation}; }})({value})")
    } else {
        format!("(() => {{ const {name}: {annotation} = {value}; return {continuation}; }})()")
    }
}
const ERASED_EVIDENCE_TYPE: &str = "Readonly<Record<string, (...args: any[]) => any>>";
