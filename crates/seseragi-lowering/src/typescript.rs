use crate::{CoreForeignMember, CoreForeignModule, CoreModule, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_syntax::{TypeParameter, Visibility};
use std::collections::BTreeMap;

mod adt;
mod decision;
mod dictionaries;
mod expr;
mod imports;
mod instances;
mod module_imports;
mod names;
mod runtime;
mod type_imports;
pub(crate) mod types;

use adt::lower_core_adt_to_typescript;
use expr::{lower_core_expr_to_typescript, typescript_expr_contains_await};
use imports::freshen_runtime_imports;
pub(crate) use instances::evidence_parameter_name;
use instances::{
    dictionary_export_name, local_instance_expression_key, lower_core_instances_to_typescript,
};
pub use instances::{
    TypeScriptDerivedShowField, TypeScriptDerivedShowPayload, TypeScriptDerivedShowVariant,
    TypeScriptInstance, TypeScriptInstanceConstraint, TypeScriptInstanceImplementation,
    TypeScriptInstanceMethod, TypeScriptShowDictionaryReference,
};
use module_imports::lower_module_imports;
use names::{local_name, module_value_name};
use runtime::{
    collect_expr_runtime_imports, collect_expr_runtime_requirements,
    collect_type_runtime_requirement,
};
use type_imports::collect_module_type_imports;
use types::{lower_core_parameter_to_typescript, type_ref_from_core_expr};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub foreign_modules: Vec<TypeScriptForeignModule>,
    pub runtime_requirements: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<TypeScriptImport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_imports: Vec<TypeScriptTypeImport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_imports: Vec<TypeScriptSourceImport>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adts: Vec<TypeScriptAdt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<TypeScriptAlias>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structs: Vec<TypeScriptStruct>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<TypeScriptInstance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<TypeScriptBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<TypeScriptFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptForeignModule {
    pub exported: bool,
    pub specifier: String,
    pub pure_load: bool,
    pub types: Vec<TypeScriptForeignOpaqueType>,
    pub members: Vec<TypeScriptForeignMember>,
    pub values: Vec<TypeScriptForeignValue>,
    pub namespaces: Vec<TypeScriptForeignNamespace>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptForeignNamespace {
    pub name: String,
    pub host_name: String,
    pub types: Vec<TypeScriptForeignOpaqueType>,
    pub members: Vec<TypeScriptForeignMember>,
    pub values: Vec<TypeScriptForeignValue>,
    pub namespaces: Vec<TypeScriptForeignNamespace>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptForeignOpaqueType {
    pub name: String,
    pub brand: String,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptForeignValue {
    pub name: String,
    pub field_name: String,
    pub host_name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeScriptType,
    pub codec: String,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptForeignMember {
    pub mode: seseragi_syntax::ForeignCallMode,
    pub call_kind: seseragi_syntax::ForeignCallKind,
    pub name: String,
    pub field_name: String,
    pub host_name: String,
    pub parameters: Vec<TypeScriptParameter>,
    pub parameter_codecs: Vec<String>,
    pub return_type: TypeScriptType,
    pub return_codec: String,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptAlias {
    pub exported: bool,
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub target: TypeScriptType,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptStruct {
    pub exported: bool,
    pub name: String,
    pub brand: String,
    pub opaque: bool,
    pub type_parameters: Vec<TypeParameter>,
    pub fields: Vec<TypeScriptRecordTypeField>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptAdt {
    pub exported: bool,
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub variants: Vec<TypeScriptAdtVariant>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptAdtVariant {
    pub exported: bool,
    pub name: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TypeScriptType>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptImport {
    pub feature: String,
    pub local: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptTypeImport {
    pub feature: String,
    pub local: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptSourceImport {
    pub module: String,
    pub specifier: String,
    /// Whether this group originated from a Seseragi source dependency edge
    /// whose module evaluation must be preserved even when every selected
    /// binding is type-only. Inferred nominal providers are type metadata only.
    #[serde(default = "default_runtime_edge", skip_serializing_if = "is_true")]
    pub runtime_edge: bool,
    pub bindings: Vec<TypeScriptSourceImportBinding>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptSourceImportBinding {
    pub imported: String,
    pub local: String,
    pub source_local: String,
    pub canonical: String,
    pub type_only: bool,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeScriptOutputPlan {
    module_specifiers: BTreeMap<String, String>,
    instance_exports: BTreeMap<(String, String), String>,
}

impl TypeScriptOutputPlan {
    pub fn new(
        module_specifiers: impl IntoIterator<Item = (String, String)>,
    ) -> TypeScriptOutputPlan {
        Self {
            module_specifiers: module_specifiers.into_iter().collect(),
            instance_exports: BTreeMap::new(),
        }
    }

    pub fn with_instance_exports(
        mut self,
        instance_exports: impl IntoIterator<Item = ((String, String), String)>,
    ) -> Self {
        self.instance_exports.extend(instance_exports);
        self
    }

    pub fn specifier_for(&self, module: &str) -> Option<&str> {
        self.module_specifiers.get(module).map(String::as_str)
    }

    pub fn instance_export_for(&self, module: &str, identity: &str) -> Option<&str> {
        self.instance_exports
            .get(&(module.to_owned(), identity.to_owned()))
            .map(String::as_str)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeScriptLoweringError {
    MissingOutputSpecifier {
        module: String,
        source_specifier: String,
    },
    MissingInstanceOutput {
        module: String,
        identity: String,
    },
    MissingInstanceOutputSpecifier {
        module: String,
        identity: String,
    },
    MissingExternalTypeBinding {
        canonical: String,
    },
    MissingSourceTypeProvider {
        canonical: String,
    },
    AmbiguousSourceTypeProvider {
        canonical: String,
    },
    MissingTypeOutputSpecifier {
        module: String,
        canonical: String,
    },
    ImportNameCollision {
        local: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptBinding {
    Const {
        exported: bool,
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptFunction {
    ConstFunction {
        exported: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        is_async: bool,
        /// Whether this function returns a cold Effect thunk.
        ///
        /// This is an emitter-only lowering flag. It is intentionally omitted
        /// from the serialized TypeScript IR so existing artifact schemas stay
        /// stable; deserialized artifacts conservatively default to `false`.
        #[serde(skip)]
        is_effect: bool,
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<TypeScriptInstanceConstraint>,
        parameters: Vec<TypeScriptParameter>,
        body: TypeScriptExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub implicit: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptRecordTypeField {
    pub name: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_ref: TypeScriptType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptRecordValueItem {
    Field { name: String, value: TypeScriptExpr },
    Spread { value: TypeScriptExpr },
}

impl TypeScriptRecordValueItem {
    pub fn value(&self) -> &TypeScriptExpr {
        match self {
            Self::Field { value, .. } | Self::Spread { value } => value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptType {
    Bigint,
    Number,
    Boolean,
    String,
    Undefined,
    Never,
    Ordering,
    Unknown,
    Reference {
        name: String,
        arguments: Vec<TypeScriptType>,
    },
    Maybe {
        element: Box<TypeScriptType>,
    },
    Either {
        error: Box<TypeScriptType>,
        value: Box<TypeScriptType>,
    },
    Tuple {
        elements: Vec<TypeScriptType>,
    },
    Record {
        fields: Vec<TypeScriptRecordTypeField>,
    },
    Array {
        element: Box<TypeScriptType>,
    },
    List {
        element: Box<TypeScriptType>,
    },
    Range,
    Function {
        parameter: Box<TypeScriptType>,
        result: Box<TypeScriptType>,
    },
    Intersection {
        operands: Vec<TypeScriptType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptExpr {
    Undefined,
    Bigint {
        value: String,
    },
    Number {
        value: String,
    },
    String {
        value: String,
    },
    Boolean {
        value: bool,
    },
    Identifier {
        name: String,
    },
    RuntimeReference {
        name: String,
    },
    CurriedRuntimeReference {
        name: String,
        arity: usize,
    },
    Tuple {
        elements: Vec<TypeScriptExpr>,
    },
    FieldAccess {
        receiver: Box<TypeScriptExpr>,
        field: String,
    },
    OptionalFieldAccess {
        receiver: Box<TypeScriptExpr>,
        field: String,
        just_constructor: String,
        nothing_constructor: String,
    },
    Record {
        items: Vec<TypeScriptRecordValueItem>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asserted_type: Option<TypeScriptType>,
    },
    Array {
        elements: Vec<TypeScriptExpr>,
        element_type: TypeScriptType,
    },
    Lambda {
        parameter: String,
        body: Box<TypeScriptExpr>,
    },
    Binary {
        operator: String,
        left: Box<TypeScriptExpr>,
        right: Box<TypeScriptExpr>,
    },
    Unary {
        operator: String,
        operand: Box<TypeScriptExpr>,
    },
    Conditional {
        condition: Box<TypeScriptExpr>,
        then_branch: Box<TypeScriptExpr>,
        else_branch: Box<TypeScriptExpr>,
    },
    Decision {
        scrutinee: Box<TypeScriptExpr>,
        scrutinee_type: TypeScriptType,
        branches: Vec<TypeScriptDecisionBranch>,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
    },
    Call {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
    },
    ForeignTaskCall {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
        function: String,
        module: String,
        origin: SourceSpan,
    },
    TypeApplicationCall {
        callee: String,
        type_arguments: Vec<TypeScriptType>,
        arguments: Vec<TypeScriptExpr>,
    },
    DictionaryCall {
        dictionary: Box<TypeScriptExpr>,
        method: String,
        arguments: Vec<TypeScriptExpr>,
    },
    RuntimeCall {
        callee: String,
        arguments: Vec<TypeScriptExpr>,
    },
    Await {
        value: Box<TypeScriptExpr>,
    },
    Sequence {
        statements: Vec<TypeScriptStatement>,
        result: Box<TypeScriptExpr>,
    },
    MonadDo {
        dictionary: Box<TypeScriptExpr>,
        statements: Vec<TypeScriptStatement>,
        result: Box<TypeScriptExpr>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptDecisionBranch {
    pub tests: Vec<TypeScriptDecisionTest>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<TypeScriptDecisionBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<TypeScriptExpr>,
    pub value: TypeScriptExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptDecisionBinding {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeScriptType,
    pub path: Vec<TypeScriptDecisionProjection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptDecisionTest {
    NumberEquals {
        path: Vec<TypeScriptDecisionProjection>,
        value: String,
    },
    StringEquals {
        path: Vec<TypeScriptDecisionProjection>,
        value: String,
    },
    BooleanEquals {
        path: Vec<TypeScriptDecisionProjection>,
        value: bool,
    },
    TagEquals {
        path: Vec<TypeScriptDecisionProjection>,
        tag: String,
    },
    ArrayLength {
        path: Vec<TypeScriptDecisionProjection>,
        length: usize,
        minimum: bool,
    },
    ListLength {
        path: Vec<TypeScriptDecisionProjection>,
        length: usize,
        minimum: bool,
    },
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptDecisionProjection {
    TupleElement { index: usize },
    ArrayElement { index: usize },
    ArrayRest { start: usize },
    ListElement { index: usize },
    ListRest { start: usize },
    RecordField { name: String },
    AdtPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeScriptStatement {
    Effect {
        value: TypeScriptExpr,
    },
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
    Const {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypeScriptType,
        initializer: TypeScriptExpr,
        origin: SourceSpan,
    },
    LocalFunction {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<TypeScriptInstanceConstraint>,
        parameters: Vec<TypeScriptParameter>,
        body: TypeScriptExpr,
        origin: SourceSpan,
    },
}

pub fn lower_core_module_to_typescript_ir(module: CoreModule) -> TypeScriptModule {
    lower_core_module_to_typescript_ir_with_plan(module, &TypeScriptOutputPlan::default())
        .expect("import-free lowering requires no linked module dependencies")
}

pub fn lower_core_module_to_typescript_ir_with_plan(
    module: CoreModule,
    plan: &TypeScriptOutputPlan,
) -> Result<TypeScriptModule, TypeScriptLoweringError> {
    let module_imports = lower_module_imports(&module, plan)?;
    let foreign_opaque_names = foreign_opaque_type_names(&module.foreign_modules);
    let mut runtime_requirements = Vec::new();
    let mut imports = Vec::new();
    let mut type_imports = Vec::new();
    collect_module_type_imports(&module, &mut runtime_requirements, &mut type_imports);
    let foreign_modules = module
        .foreign_modules
        .iter()
        .map(|foreign| TypeScriptForeignModule {
            exported: foreign.visibility == Visibility::Public,
            specifier: foreign.specifier.clone(),
            pure_load: foreign.pure_load,
            types: foreign
                .members
                .iter()
                .filter_map(|member| lower_foreign_opaque_type(member, &module.module))
                .collect(),
            members: foreign
                .members
                .iter()
                .filter_map(|member| {
                    let CoreForeignMember::Function {
                        mode,
                        call_kind,
                        symbol,
                        name,
                        host_name,
                        parameters,
                        return_type,
                        origin,
                        ..
                    } = member
                    else {
                        return None;
                    };
                    Some(TypeScriptForeignMember {
                        mode: *mode,
                        call_kind: *call_kind,
                        name: module_value_name(&module.module, symbol),
                        field_name: name.clone(),
                        host_name: host_name.clone(),
                        parameters: parameters
                            .iter()
                            .cloned()
                            .map(|parameter| {
                                lower_core_parameter_to_typescript(
                                    parameter,
                                    &module_imports.type_names,
                                    &[],
                                )
                            })
                            .collect(),
                        parameter_codecs: parameters
                            .iter()
                            .map(|parameter| {
                                foreign_codec_from_core_type(
                                    &parameter.type_ref,
                                    &foreign_opaque_names,
                                )
                            })
                            .collect(),
                        return_type: types::type_ref_from_core_type(
                            return_type,
                            &module_imports.type_names,
                        ),
                        return_codec: foreign_codec_from_core_type(
                            return_type,
                            &foreign_opaque_names,
                        ),
                        origin: origin.clone(),
                    })
                })
                .collect(),
            values: foreign
                .members
                .iter()
                .filter_map(|member| {
                    let CoreForeignMember::Value {
                        symbol,
                        name,
                        host_name,
                        type_ref,
                        origin,
                        ..
                    } = member
                    else {
                        return None;
                    };
                    Some(TypeScriptForeignValue {
                        name: module_value_name(&module.module, symbol),
                        field_name: name.clone(),
                        host_name: host_name.clone(),
                        type_ref: types::type_ref_from_core_type(
                            type_ref,
                            &module_imports.type_names,
                        ),
                        codec: foreign_codec_from_core_type(type_ref, &foreign_opaque_names),
                        origin: origin.clone(),
                    })
                })
                .collect(),
            namespaces: foreign
                .members
                .iter()
                .filter_map(|member| {
                    lower_foreign_namespace(
                        member,
                        &module.module,
                        &module_imports.type_names,
                        &foreign_opaque_names,
                        true,
                    )
                })
                .collect(),
            origin: foreign.origin.clone(),
        })
        .collect();
    let adts = module
        .adts
        .iter()
        .cloned()
        .map(|adt| {
            lower_core_adt_to_typescript(adt, &module_imports.type_names, &mut runtime_requirements)
        })
        .collect();
    let aliases = module
        .aliases
        .iter()
        .map(|alias| {
            let type_constructor_parameters = alias
                .type_parameters
                .iter()
                .filter(|parameter| parameter.is_constructor())
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>();
            TypeScriptAlias {
                exported: alias.visibility == Visibility::Public,
                name: local_name(&alias.symbol),
                type_parameters: alias.type_parameters.clone(),
                target: types::type_ref_from_core_type_with_erasure(
                    &alias.target,
                    &module_imports.type_names,
                    &type_constructor_parameters,
                ),
                origin: alias.origin.clone(),
            }
        })
        .collect();
    let structs = module
        .structs
        .iter()
        .map(|structure| TypeScriptStruct {
            exported: structure.visibility == Visibility::Public,
            name: local_name(&structure.symbol),
            brand: format!("__ssrg$brand${}", local_name(&structure.symbol)),
            opaque: structure.opaque,
            type_parameters: structure.type_parameters.clone(),
            fields: structure
                .fields
                .iter()
                .map(|field| TypeScriptRecordTypeField {
                    name: field.name.clone(),
                    optional: false,
                    type_ref: types::type_ref_from_core_type(
                        &field.type_ref,
                        &module_imports.type_names,
                    ),
                })
                .collect(),
            origin: structure.origin.clone(),
        })
        .collect();
    let mut expression_value_names = module_imports.value_names.clone();
    collect_foreign_task_markers(
        &module.module,
        &module.foreign_modules,
        &mut expression_value_names,
    );
    expression_value_names.insert(
        "__ssrg$foreign$module".to_owned(),
        diagnostic_module_name(&module.module),
    );
    for function in &module.functions {
        expression_value_names.insert(
            function.symbol.clone(),
            module_value_name(&module.module, &function.symbol),
        );
    }
    for ((_, identity), local) in &module_imports.instance_names {
        expression_value_names.insert(local_instance_expression_key(identity), local.clone());
    }
    for (index, instance) in module.instances.iter().enumerate() {
        expression_value_names.insert(
            local_instance_expression_key(&instance.identity),
            dictionary_export_name(&instance.trait_name, index),
        );
    }
    let instances = lower_core_instances_to_typescript(
        &module.instances,
        &module.adts,
        &module.structs,
        &module_imports.instance_names,
        &expression_value_names,
        &module_imports.type_names,
        &mut runtime_requirements,
        &mut imports,
        &mut type_imports,
    );
    let bindings = module
        .bindings
        .into_iter()
        .map(|binding| {
            collect_expr_runtime_requirements(&binding.value, &mut runtime_requirements);
            collect_expr_runtime_imports(&binding.value, &mut imports);
            let mut binding_value_names = expression_value_names.clone();
            binding_value_names.insert(
                "__ssrg$foreign$function".to_owned(),
                local_name(&binding.symbol),
            );
            TypeScriptBinding::Const {
                exported: binding.visibility == Visibility::Public,
                name: local_name(&binding.symbol),
                type_ref: type_ref_from_core_expr(&binding.value, &module_imports.type_names),
                initializer: lower_core_expr_to_typescript(
                    binding.value,
                    &binding_value_names,
                    &module_imports.type_names,
                ),
                origin: binding.origin,
            }
        })
        .collect();
    let functions = module
        .functions
        .into_iter()
        .map(|function| {
            let type_constructor_parameters = function
                .type_parameters
                .iter()
                .filter(|parameter| parameter.is_constructor())
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>();
            for parameter in &function.parameters {
                collect_type_runtime_requirement(&parameter.type_ref, &mut runtime_requirements);
            }
            collect_expr_runtime_requirements(&function.body, &mut runtime_requirements);
            collect_expr_runtime_imports(&function.body, &mut imports);
            let mut function_value_names = expression_value_names.clone();
            function_value_names.insert(
                "__ssrg$foreign$function".to_owned(),
                module_value_name(&module.module, &function.symbol),
            );
            let body = lower_core_expr_to_typescript(
                function.body,
                &function_value_names,
                &module_imports.type_names,
            );
            TypeScriptFunction::ConstFunction {
                exported: function.visibility == Visibility::Public,
                is_async: typescript_expr_contains_await(&body),
                is_effect: function.is_effect,
                name: module_value_name(&module.module, &function.symbol),
                type_parameters: function.type_parameters,
                constraints: function
                    .constraints
                    .iter()
                    .map(|constraint| TypeScriptInstanceConstraint {
                        name: constraint.name.clone(),
                        trait_identity: constraint.trait_identity.clone(),
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(|argument| {
                                types::type_ref_from_core_type(argument, &module_imports.type_names)
                            })
                            .collect(),
                    })
                    .collect(),
                parameters: function
                    .parameters
                    .into_iter()
                    .map(|parameter| {
                        lower_core_parameter_to_typescript(
                            parameter,
                            &module_imports.type_names,
                            &type_constructor_parameters,
                        )
                    })
                    .collect(),
                body,
                origin: function.origin,
            }
        })
        .collect();

    let mut typescript = TypeScriptModule {
        schema: module.schema,
        stage: "typescript-ir".to_owned(),
        module: module.module,
        foreign_modules,
        runtime_requirements,
        imports,
        type_imports,
        source_imports: module_imports.imports,
        adts,
        aliases,
        structs,
        instances,
        bindings,
        functions,
    };
    freshen_runtime_imports(&mut typescript);
    Ok(typescript)
}

fn lower_foreign_namespace(
    member: &CoreForeignMember,
    module: &str,
    type_names: &BTreeMap<String, String>,
    foreign_opaque_names: &std::collections::BTreeSet<String>,
    top_level: bool,
) -> Option<TypeScriptForeignNamespace> {
    let CoreForeignMember::Namespace {
        symbol,
        name,
        host_name,
        members,
        origin,
    } = member
    else {
        return None;
    };
    let local = if top_level {
        module_value_name(module, symbol)
    } else {
        local_name(name)
    };
    Some(TypeScriptForeignNamespace {
        name: local,
        host_name: host_name.clone(),
        types: members
            .iter()
            .filter_map(|member| lower_foreign_opaque_type(member, module))
            .collect(),
        members: members
            .iter()
            .filter_map(|member| {
                let CoreForeignMember::Function {
                    mode,
                    call_kind,
                    symbol,
                    name,
                    host_name,
                    parameters,
                    return_type,
                    origin,
                    ..
                } = member
                else {
                    return None;
                };
                Some(TypeScriptForeignMember {
                    mode: *mode,
                    call_kind: *call_kind,
                    name: module_value_name(module, symbol),
                    field_name: name.clone(),
                    host_name: host_name.clone(),
                    parameters: parameters
                        .iter()
                        .cloned()
                        .map(|parameter| {
                            lower_core_parameter_to_typescript(parameter, type_names, &[])
                        })
                        .collect(),
                    parameter_codecs: parameters
                        .iter()
                        .map(|parameter| {
                            foreign_codec_from_core_type(&parameter.type_ref, foreign_opaque_names)
                        })
                        .collect(),
                    return_type: types::type_ref_from_core_type(return_type, type_names),
                    return_codec: foreign_codec_from_core_type(return_type, foreign_opaque_names),
                    origin: origin.clone(),
                })
            })
            .collect(),
        values: members
            .iter()
            .filter_map(|member| {
                let CoreForeignMember::Value {
                    symbol,
                    name,
                    host_name,
                    type_ref,
                    origin,
                    ..
                } = member
                else {
                    return None;
                };
                Some(TypeScriptForeignValue {
                    name: module_value_name(module, symbol),
                    field_name: name.clone(),
                    host_name: host_name.clone(),
                    type_ref: types::type_ref_from_core_type(type_ref, type_names),
                    codec: foreign_codec_from_core_type(type_ref, foreign_opaque_names),
                    origin: origin.clone(),
                })
            })
            .collect(),
        namespaces: members
            .iter()
            .filter_map(|member| {
                lower_foreign_namespace(member, module, type_names, foreign_opaque_names, false)
            })
            .collect(),
        origin: origin.clone(),
    })
}

fn foreign_codec_from_core_type(
    type_ref: &crate::CoreType,
    foreign_opaque_names: &std::collections::BTreeSet<String>,
) -> String {
    match type_ref {
        crate::CoreType::Named { name, arguments } if arguments.is_empty() => match name.as_str() {
            "Unit" => "\"unit\"".to_owned(),
            "Bool" => "\"bool\"".to_owned(),
            "Char" => "\"char\"".to_owned(),
            "String" => "\"string\"".to_owned(),
            "Int" => "\"int\"".to_owned(),
            "Float" => "\"float\"".to_owned(),
            "BigInt" => "\"bigint\"".to_owned(),
            name if foreign_opaque_names.contains(name) => "\"opaque\"".to_owned(),
            "Js.Unknown" => "\"js-unknown\"".to_owned(),
            "Js.Object" => "\"js-object\"".to_owned(),
            "Js.Number" => "\"js-number\"".to_owned(),
            "Js.String" => "\"js-string\"".to_owned(),
            "Js.Null" => "\"js-null\"".to_owned(),
            "Js.Undefined" => "\"js-undefined\"".to_owned(),
            _ => "\"unsupported\"".to_owned(),
        },
        crate::CoreType::Named { name, arguments } if name == "Array" && arguments.len() == 1 => {
            format!(
                "{{ array: {} }}",
                foreign_codec_from_core_type(&arguments[0], foreign_opaque_names)
            )
        }
        crate::CoreType::Named { name, arguments }
            if matches!(
                name.as_str(),
                "Js.NullOr" | "Js.Nullable" | "Js.UndefinedOr" | "Js.Promise" | "Js.MutableArray"
            ) && arguments.len() == 1 =>
        {
            let key = match name.as_str() {
                "Js.NullOr" => "nullOr",
                "Js.Nullable" => "nullable",
                "Js.UndefinedOr" => "undefinedOr",
                "Js.Promise" => "promise",
                "Js.MutableArray" => "mutableArray",
                _ => unreachable!(),
            };
            format!(
                "{{ {key}: {} }}",
                foreign_codec_from_core_type(&arguments[0], foreign_opaque_names)
            )
        }
        crate::CoreType::Named { name, arguments }
            if name == "Js.Callback" && arguments.len() == 2 =>
        {
            "{ rawCallback: true }".to_owned()
        }
        crate::CoreType::ExternalNamed {
            name, canonical, ..
        } if name == "Bytes" || canonical == "std/bytes::Bytes" => "\"bytes\"".to_owned(),
        crate::CoreType::ExternalNamed {
            name, canonical, ..
        } if name == "BigInt" || canonical == "std/big-int::BigInt" => "\"bigint\"".to_owned(),
        crate::CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if canonical == "std/prelude::Js.Unknown" && arguments.is_empty() => {
            "\"js-unknown\"".to_owned()
        }
        crate::CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if arguments.is_empty() => match canonical.as_str() {
            "std/prelude::Js.Object" => "\"js-object\"".to_owned(),
            "std/prelude::Js.Number" => "\"js-number\"".to_owned(),
            "std/prelude::Js.String" => "\"js-string\"".to_owned(),
            "std/prelude::Js.Null" => "\"js-null\"".to_owned(),
            "std/prelude::Js.Undefined" => "\"js-undefined\"".to_owned(),
            _ => "\"unsupported\"".to_owned(),
        },
        crate::CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if matches!(
            canonical.as_str(),
            "std/prelude::Js.NullOr"
                | "std/prelude::Js.Nullable"
                | "std/prelude::Js.UndefinedOr"
                | "std/prelude::Js.Promise"
                | "std/prelude::Js.MutableArray"
        ) && arguments.len() == 1 =>
        {
            let key = match canonical.as_str() {
                "std/prelude::Js.NullOr" => "nullOr",
                "std/prelude::Js.Nullable" => "nullable",
                "std/prelude::Js.UndefinedOr" => "undefinedOr",
                "std/prelude::Js.Promise" => "promise",
                "std/prelude::Js.MutableArray" => "mutableArray",
                _ => unreachable!(),
            };
            format!(
                "{{ {key}: {} }}",
                foreign_codec_from_core_type(&arguments[0], foreign_opaque_names)
            )
        }
        crate::CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } if canonical == "std/prelude::Js.Callback" && arguments.len() == 2 => {
            "{ rawCallback: true }".to_owned()
        }
        crate::CoreType::Function { .. } => {
            let (parameters, result) = foreign_callback_signature(type_ref);
            format!(
                "{{ callback: {{ parameters: [{}], result: {} }} }}",
                parameters
                    .iter()
                    .map(|parameter| {
                        foreign_codec_from_core_type(parameter, foreign_opaque_names)
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                foreign_codec_from_core_type(result, foreign_opaque_names)
            )
        }
        crate::CoreType::Tuple { elements } => format!(
            "{{ tuple: [{}] }}",
            elements
                .iter()
                .map(|element| foreign_codec_from_core_type(element, foreign_opaque_names))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        crate::CoreType::Record { fields, .. } => format!(
            "{{ record: {{ {} }} }}",
            fields
                .iter()
                .map(|field| {
                    let codec = foreign_codec_from_core_type(&field.type_ref, foreign_opaque_names);
                    let codec = if field.optional {
                        format!("{{ optional: {codec} }}")
                    } else {
                        codec
                    };
                    format!("{:?}: {codec}", field.name)
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "\"unsupported\"".to_owned(),
    }
}

fn foreign_opaque_type_names(
    modules: &[crate::CoreForeignModule],
) -> std::collections::BTreeSet<String> {
    fn collect(members: &[CoreForeignMember], names: &mut std::collections::BTreeSet<String>) {
        for member in members {
            match member {
                CoreForeignMember::OpaqueType { name, .. } => {
                    names.insert(name.clone());
                }
                CoreForeignMember::Namespace { members, .. } => collect(members, names),
                CoreForeignMember::Function { .. } | CoreForeignMember::Value { .. } => {}
            }
        }
    }

    let mut names = std::collections::BTreeSet::new();
    for module in modules {
        collect(&module.members, &mut names);
    }
    names
}

fn collect_foreign_task_markers(
    module_name: &str,
    modules: &[CoreForeignModule],
    names: &mut BTreeMap<String, String>,
) {
    fn collect(
        module_name: &str,
        members: &[CoreForeignMember],
        names: &mut BTreeMap<String, String>,
    ) {
        for member in members {
            match member {
                CoreForeignMember::Function { mode, symbol, .. }
                    if *mode == seseragi_syntax::ForeignCallMode::Task =>
                {
                    names.insert(
                        format!(
                            "__ssrg$foreign$task:{}",
                            module_value_name(module_name, symbol)
                        ),
                        String::new(),
                    );
                }
                CoreForeignMember::Namespace { members, .. } => {
                    collect(module_name, members, names)
                }
                CoreForeignMember::Function { .. }
                | CoreForeignMember::Value { .. }
                | CoreForeignMember::OpaqueType { .. } => {}
            }
        }
    }

    for module in modules {
        collect(module_name, &module.members, names);
    }
}

fn diagnostic_module_name(module: &str) -> String {
    let (package, path) = module.split_once("::").unwrap_or((module, "main"));
    let package = package.split_once('@').map_or(package, |(name, _)| name);
    format!("{package}/{path}")
}

fn lower_foreign_opaque_type(
    member: &CoreForeignMember,
    module: &str,
) -> Option<TypeScriptForeignOpaqueType> {
    let CoreForeignMember::OpaqueType {
        symbol,
        name,
        origin,
    } = member
    else {
        return None;
    };
    let local = module_value_name(module, symbol);
    Some(TypeScriptForeignOpaqueType {
        brand: format!("__ssrg$foreign$brand${}", local_name(name)),
        name: local,
        origin: origin.clone(),
    })
}

fn foreign_callback_signature(
    type_ref: &crate::CoreType,
) -> (Vec<&crate::CoreType>, &crate::CoreType) {
    let mut parameters = Vec::new();
    let mut current = type_ref;
    while let crate::CoreType::Function { parameter, result } = current {
        parameters.push(parameter.as_ref());
        current = result.as_ref();
    }
    (parameters, current)
}

pub(super) fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_owned());
    }
}

pub(super) fn push_import_unique(imports: &mut Vec<TypeScriptImport>, import: TypeScriptImport) {
    if !imports
        .iter()
        .any(|existing| existing.feature == import.feature && existing.local == import.local)
    {
        imports.push(import);
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

fn default_runtime_edge() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}
