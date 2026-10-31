use crate::{RustArtifact, RustBackendError, RustMetadata};
use seseragi_lowering::{
    core_expr_origin, core_free_variables, is_closed_local_evidence, known_pure_call_shape,
    CoreCallShape,
};
use seseragi_lowering::{
    CoreCallEvidence, CoreDecisionProjection, CoreDecisionTest, CoreExpr, CoreInstanceEvidence,
    CoreInstanceImplementation, CoreModule, CoreParameter, CoreRecordValueItem, CoreStatement,
    CoreTemplatePart, CoreType,
};
use std::collections::{BTreeMap, BTreeSet};

mod array_operations;
mod arrays;
mod big_int;
mod big_int_traits;
mod booleans;
mod bytes;
mod calls;
mod canonical;
mod characters;
mod clock;
mod collection_control;
mod collections;
mod comparisons;
mod concurrency;
mod console;
mod decimal;
mod derived;
mod display;
mod effect_conversions;
mod effect_defects;
mod either;
mod equality;
mod filesystem;
mod floats;
mod hashing;
mod iterable;
mod iterator;
mod json;
mod json_codecs;
mod json_container_codecs;
mod json_decoders;
mod json_derived_adts;
mod json_derived_newtypes;
mod json_derived_structs;
mod lists;
mod local;
mod math;
mod maybe;
mod methods;
mod monads;
mod number_parse;
mod operators;
mod prelude_collections;
mod prelude_effects;
mod range;
mod returned_calls;
mod scalar;
mod sequence_operations;
mod sequence_order;
mod standard_errors;
mod tail;
mod text;
mod variables;

use local::{LocalCallable, Scope};
type Result<T> = std::result::Result<T, RustBackendError>;

fn id(name: &str) -> String {
    format!(
        "v_{}",
        name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    )
}
fn quoted(value: &str) -> String {
    format!("{value:?}")
}

pub fn emit(
    module: &CoreModule,
    entry: &str,
    owners: &BTreeMap<String, String>,
    scalar_globals: &seseragi_lowering::CoreScalarGlobals,
    type_names: &BTreeMap<(String, String), String>,
    trait_names: &BTreeMap<(String, String), String>,
    options: &crate::RustTargetOptions,
) -> Result<RustArtifact> {
    emit_body(
        module,
        Some(entry),
        owners,
        scalar_globals,
        type_names,
        trait_names,
        options,
    )
}

pub(crate) fn emit_library(
    module: &CoreModule,
    owners: &BTreeMap<String, String>,
    scalar_globals: &seseragi_lowering::CoreScalarGlobals,
    type_names: &BTreeMap<(String, String), String>,
    trait_names: &BTreeMap<(String, String), String>,
) -> Result<RustArtifact> {
    emit_body(
        module,
        None,
        owners,
        scalar_globals,
        type_names,
        trait_names,
        &crate::RustTargetOptions::default(),
    )
}

pub(crate) fn canonical_boundary_type(module: &CoreModule, ty: &CoreType) -> Option<CoreType> {
    let names = crate::type_names::for_boundary(module).ok()?;
    display::canonical_type(ty, &module.module, &BTreeMap::new(), &names, true)
}

pub(crate) fn checked_boundary_body_type(module: &CoreModule, body: &CoreExpr) -> Option<CoreType> {
    canonical_boundary_type(module, &local::method_argument_type(body)?)
}

fn emit_body(
    module: &CoreModule,
    entry: Option<&str>,
    owners: &BTreeMap<String, String>,
    scalar_globals: &seseragi_lowering::CoreScalarGlobals,
    type_names: &BTreeMap<(String, String), String>,
    trait_names: &BTreeMap<(String, String), String>,
    options: &crate::RustTargetOptions,
) -> Result<RustArtifact> {
    if !module.foreign_modules.is_empty() {
        return Err("Rust backend: foreign modules are unsupported".into());
    }
    let mut backend = Backend {
        module,
        owners,
        scalar_globals,
        type_names,
        trait_names,
        options,
        clock_origins: Vec::new(),
        program_wasm_library: entry.is_none(),
        owner: &module.module,
        functions: module.functions.iter().map(|f| f.symbol.clone()).collect(),
        globals: module
            .bindings
            .iter()
            .enumerate()
            .map(|(i, b)| (b.symbol.clone(), i))
            .collect(),
        constructors: module
            .adts
            .iter()
            .flat_map(|a| {
                a.variants
                    .iter()
                    .map(|v| (v.symbol.clone(), v.payload.is_some()))
            })
            .collect(),
        serial: 0,
        direct_calls: 0,
        self_tail_calls: 0,
        closed_dictionary_sites: 0,
        capabilities: BTreeSet::new(),
    };
    let entry = entry
        .map(|entry| {
            module
                .functions
                .iter()
                .find(|f| f.symbol == format!("{}::{entry}", module.module))
                .ok_or("Rust backend: entry function not found")
        })
        .transpose()?;
    if let Some(entry) = entry {
        if entry.parameters.len() != 1
            || !canonical::primitive(&entry.parameters[0].type_ref, "Unit")
            || !entry.constraints.is_empty()
        {
            return Err(crate::project::at(
                "Rust backend: entry must be an unconstrained Unit function",
                &entry.origin,
                &module.module,
            ));
        }
    }
    let mut output = String::from("// Generated by Seseragi experimental Rust backend.\n#![recursion_limit = \"1024\"]\n#![allow(dead_code, unused_variables)]\n");
    output.push_str(include_str!("runtime.rs"));
    output.push_str(include_str!("runtime_char.rs"));
    output.push_str(include_str!("runtime_float_display.rs"));
    output.push_str(include_str!("runtime_float.rs"));
    output.push_str(include_str!("runtime_math.rs"));
    output.push_str(include_str!("runtime_list.rs"));
    output.push_str(include_str!("runtime_tuple.rs"));
    output.push_str(include_str!("runtime_derived.rs"));
    output.push_str(include_str!("runtime_standard_errors.rs"));
    output.push_str(include_str!("runtime_array.rs"));
    output.push_str(include_str!("runtime_array_operations.rs"));
    output.push_str(include_str!("runtime_sequence.rs"));
    output.push_str(include_str!("runtime_sequence_order.rs"));
    output.push_str(include_str!("runtime_iterator.rs"));
    output.push_str(include_str!("runtime_range.rs"));
    output.push_str(include_str!("runtime_iterable.rs"));
    output.push_str(include_str!("runtime_collection_control.rs"));
    output.push_str(include_str!("runtime_prelude_collections.rs"));
    output.push_str(include_str!("runtime_traversable.rs"));
    output.push_str(include_str!("runtime_monad.rs"));
    output.push_str(include_str!("runtime_transformer.rs"));
    output.push_str(include_str!("runtime_maybe.rs"));
    output.push_str(include_str!("runtime_either.rs"));
    output.push_str(include_str!("runtime_text.rs"));
    output.push_str(include_str!("runtime_bytes.rs"));
    output.push_str(include_str!("runtime_big_int_value.rs"));
    output.push_str(include_str!("runtime_big_int_arithmetic.rs"));
    output.push_str(include_str!("runtime_big_int.rs"));
    output.push_str(include_str!("runtime_big_int_traits.rs"));
    output.push_str(include_str!("runtime_decimal.rs"));
    output.push_str(include_str!("runtime_decimal_context.rs"));
    output.push_str(include_str!("runtime_decimal_conversion.rs"));
    output.push_str(include_str!("runtime_decimal_arithmetic.rs"));
    output.push_str(include_str!("runtime_decimal_division.rs"));
    output.push_str(include_str!("runtime_hashing.rs"));
    output.push_str(include_str!("runtime_map.rs"));
    output.push_str(include_str!("runtime_set.rs"));
    output.push_str(include_str!("runtime_json.rs"));
    output.push_str(include_str!("runtime_json_codecs.rs"));
    output.push_str(include_str!("runtime_json_decoders.rs"));
    output.push_str(include_str!("runtime_json_container_codecs.rs"));
    output.push_str(include_str!("runtime_json_derived_adts.rs"));
    output.push_str(include_str!("runtime_int_parse.rs"));
    output.push_str(include_str!("runtime_int_checked.rs"));
    output.push_str(include_str!("runtime_float_parse.rs"));
    output.push_str(include_str!("runtime_float_conversion.rs"));
    output.push_str(include_str!("runtime_trace.rs"));
    output.push_str(include_str!("runtime_effect.rs"));
    output.push_str(include_str!("runtime_stdin.rs"));
    output.push_str(include_str!("runtime_scheduler.rs"));
    output.push_str(include_str!("runtime_path.rs"));
    output.push_str(include_str!("runtime_filesystem.rs"));
    output.push_str(include_str!("runtime_host.rs"));
    output.push_str(include_str!("runtime_clock.rs"));
    let mut initialization = String::new();
    for (index, binding) in module.bindings.iter().enumerate() {
        backend.owner = &owners[&binding.symbol];
        let mut scope = Scope::new();
        scope.type_parameters(
            binding
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &binding.symbol,
        );
        let value = backend.expr(&binding.value, &scope)?;
        initialization.push_str(&format!("set_global({index}, {value});\n"));
    }
    output.push_str(&format!(
        "\nfn initialize_globals() {{ {initialization} }}\n"
    ));
    for (symbol, payload) in &backend.constructors {
        let value = if *payload {
            format!(
                "V::fun(vec![], |_, payload| V::adt({}, payload))",
                quoted(symbol)
            )
        } else {
            format!("V::adt({}, V::Unit)", quoted(symbol))
        };
        output.push_str(&format!("\nfn {}() -> V {{ {value} }}\n", id(symbol)));
    }
    for instance in &module.instances {
        backend.owner = &owners[&instance.identity];
        if backend.program_wasm_library
            && instance
                .arguments
                .iter()
                .any(CoreType::contains_type_function)
        {
            return Err(crate::project::at(
                "program WASM: checked type-function dictionaries are unsupported",
                &instance.origin,
                backend.owner,
            ));
        }
        let mut scope = Scope::new();
        scope.type_parameters(
            instance
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &instance.identity,
        );
        let setup = dictionary_parameters(&instance.constraints, &mut scope);
        if let Some(dictionary) = backend
            .derived_dictionary(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            output.push_str(&format!(
                "\nfn {}(dicts: Vec<V>) -> V {{ {setup} {dictionary} }}\n",
                id(&instance.identity)
            ));
            continue;
        }
        if let Some(dictionary) = backend
            .json_derived_dictionary(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            output.push_str(&format!(
                "\nfn {}(dicts: Vec<V>) -> V {{ {setup} {dictionary} }}\n",
                id(&instance.identity)
            ));
            continue;
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err(crate::project::at(
                "Rust backend: derived instances are unsupported",
                &instance.origin,
                backend.owner,
            ));
        };
        backend
            .validate_eq_instance(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?;
        backend
            .validate_hash_instance(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?;
        let mut values = Vec::new();
        if let Some(fields) = backend
            .monad_supertrait(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            for field in fields {
                values.push(format!(
                    "({field:?}, {}.field({field:?}))",
                    id("\0dictionary_0")
                ));
            }
        } else if backend
            .traversable_supertrait(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            values.push(format!(
                "(\"map\", {}.field(\"map\"))",
                id("\0dictionary_0")
            ));
        } else if backend
            .reducible_supertrait(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            values.push(format!(
                "(\"iterate\", {}.field(\"iterate\"))",
                id("\0dictionary_0")
            ));
        } else if backend
            .monoid_supertrait(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            values.push(format!(
                "(\"append\", {}.field(\"append\"))",
                id("\0dictionary_0")
            ));
        } else if backend
            .ord_supertrait(instance, &scope)
            .map_err(|error| crate::project::at(&error.message, &instance.origin, backend.owner))?
        {
            values.push(format!("(\"eq\", {}.field(\"eq\"))", id("\0dictionary_0")));
        }
        if monads::tail_bind(instance) {
            values.push("(\"\\0nativeTailBind\", V::Bool(true))".into());
        }
        for method in methods {
            let mut method_scope = scope.clone();
            method_scope.type_parameters(
                method
                    .type_parameters
                    .iter()
                    .map(|p| (p.name.clone(), p.arity)),
                &format!("{}::{}", instance.identity, method.name),
            );
            backend
                .validate_method_value_provider(instance, method, &method_scope)
                .map_err(|error| {
                    crate::project::at(&error.message, &method.origin, backend.owner)
                })?;
            let value = backend
                .method_function(method, &method_scope)
                .map_err(|mut error| {
                    if error.origin.is_none() {
                        error.origin = Some(method.origin.clone());
                        error.module = Some(backend.owner.to_owned());
                    }
                    error
                })?;
            values.push(format!("({}, {value})", quoted(&method.name)));
            if !method.constraints.is_empty() && !method.parameters.is_empty() {
                let value = backend
                    .method_value_function(method, &method_scope)
                    .map_err(|error| {
                        crate::project::at(&error.message, &method.origin, backend.owner)
                    })?;
                values.push(format!(
                    "({}, {value})",
                    quoted(&seseragi_lowering::method_value_factory_name(&method.name))
                ));
            }
        }
        output.push_str(&format!(
            "\nfn {}(dicts: Vec<V>) -> V {{ {setup} V::dict(vec![{}]) }}\n",
            id(&instance.identity),
            values.join(",")
        ));
    }
    for function in &module.functions {
        backend.owner = &owners[&function.symbol];
        if function.is_effect {
            backend.capabilities.insert("effect-sync".into());
        }
        let mut scope = Scope::new();
        scope.type_parameters(
            function
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &function.symbol,
        );
        let setup = dictionary_parameters(&function.constraints, &mut scope);
        if function.parameters.is_empty() {
            backend
                .validate_global_method_factory(function, &scope)
                .map_err(|error| {
                    crate::project::at(&error.message, &function.origin, backend.owner)
                })?;
            let body = backend.expr(&function.body, &scope)?;
            output.push_str(&format!(
                "\nfn {}(dicts: Vec<V>) -> V {{ {setup} {body} }}\n",
                id(&function.symbol)
            ));
            continue;
        }
        let body = backend.function(&function.parameters, &function.body, &scope)?;
        output.push_str(&format!(
            "\nfn {}(dicts: Vec<V>) -> V {{ {setup} {body} }}\n",
            id(&function.symbol)
        ));
        for parameter in &function.parameters {
            scope.insert(parameter.id.clone());
        }
        output.push_str(&backend.direct_function(function, &setup, &scope)?);
    }
    if backend.capabilities.contains("effect-defect") {
        output.push_str(include_str!("runtime_effect_defect.rs"));
    }
    if let Some(entry) = entry {
        let startup =
            if backend.capabilities.contains("map") || backend.capabilities.contains("set") {
                "let _ = native_process_hash_seed(); "
            } else {
                ""
            };
        if entry.is_effect {
            let runner = if backend.capabilities.contains("clock-sleep") {
                "run_native_clock_entry"
            } else {
                "run_native_entry"
            };
            output.push_str(&format!("\nfn main() {{ std::process::exit({runner}(|| {{ {startup}initialize_globals(); {}(vec![]).apply(V::Unit) }}, {})); }}\n", id(&entry.symbol), backend.capabilities.contains("ref") || backend.capabilities.contains("fiber") || backend.capabilities.contains("deferred")));
        } else {
            output.push_str(&format!("\nfn main() {{ std::process::exit(run_native_pure_entry(|| {{ {startup}initialize_globals(); {}(vec![]).apply(V::Unit) }}, {})); }}\n", id(&entry.symbol), backend.capabilities.contains("ref") || backend.capabilities.contains("fiber") || backend.capabilities.contains("deferred")));
        }
    }
    Ok(RustArtifact {
        source: output,
        metadata: RustMetadata {
            schema: 1,
            backend: "rust".into(),
            stability: "experimental".into(),
            abi: if backend.capabilities.contains("clock-sleep") {
                "rust-erased-clock-sleep-v5"
            } else if backend.capabilities.contains("filesystem-async") {
                "rust-erased-async-v4"
            } else if backend.capabilities.contains("fiber")
                || backend.capabilities.contains("deferred")
            {
                "rust-erased-cooperative-v3"
            } else if backend.capabilities.contains("effect-sync") {
                "rust-erased-sync-v2"
            } else {
                "rust-erased-pure-v1"
            }
            .into(),
            module: module.module.clone(),
            entry: entry.map(|entry| entry.symbol.clone()).unwrap_or_default(),
            edition: "2021".into(),
            direct_calls: backend.direct_calls,
            self_tail_calls: backend.self_tail_calls,
            closed_dictionary_sites: backend.closed_dictionary_sites,
            entry_kind: match entry {
                Some(entry) if entry.is_effect => "effect",
                Some(_) => "pure",
                None => "library",
            }
            .into(),
            capabilities: backend.capabilities.into_iter().collect(),
            modules: Vec::new(),
            providers: if backend.clock_origins.is_empty() {
                Vec::new()
            } else {
                vec![crate::clock_provider::receipt(
                    options,
                    backend.clock_origins,
                )?]
            },
        },
    })
}

fn dictionary_parameters(
    constraints: &[seseragi_lowering::CoreInstanceConstraint],
    scope: &mut Scope,
) -> String {
    scope.dictionaries = constraints.to_vec();
    (0..constraints.len())
        .map(|n| {
            let name = format!("\0dictionary_{n}");
            scope.insert(name.clone());
            format!("let {} = dicts[{n}].clone();", id(&name))
        })
        .collect()
}

struct Backend<'a> {
    module: &'a CoreModule,
    // The explicit library emission path currently backs program WASM.
    program_wasm_library: bool,
    owners: &'a BTreeMap<String, String>,
    owner: &'a str,
    functions: BTreeSet<String>,
    globals: BTreeMap<String, usize>,
    scalar_globals: &'a seseragi_lowering::CoreScalarGlobals,
    type_names: &'a BTreeMap<(String, String), String>,
    trait_names: &'a BTreeMap<(String, String), String>,
    constructors: BTreeMap<String, bool>,
    serial: usize,
    direct_calls: usize,
    self_tail_calls: usize,
    closed_dictionary_sites: usize,
    capabilities: BTreeSet<String>,
    options: &'a crate::RustTargetOptions,
    clock_origins: Vec<seseragi_lowering::SourceSpan>,
}

impl Backend<'_> {
    fn fresh(&mut self) -> String {
        self.serial += 1;
        format!("temporary_{}", self.serial)
    }

    fn function(
        &mut self,
        parameters: &[CoreParameter],
        body: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        self.function_inner(parameters, body, scope, None)
    }

    fn function_inner(
        &mut self,
        parameters: &[CoreParameter],
        body: &CoreExpr,
        scope: &Scope,
        local: Option<&LocalCallable>,
    ) -> Result<String> {
        if let Some((parameter, rest)) = parameters.split_first() {
            let mut inner = scope.clone();
            inner.insert(parameter.id.clone());
            let mut captures = scope.clone();
            if let Some(mut free) = core_free_variables(body) {
                for parameter in parameters {
                    free.names.remove(&parameter.id);
                }
                for index in free.evidence_parameters {
                    free.names.insert(format!("\0dictionary_{index}"));
                }
                captures.retain_free(&free.names);
            }
            let body = self.function_inner(rest, body, &inner, local)?;
            Ok(Self::closure(&parameter.id, &body, &captures))
        } else if let Some(callable) = local {
            self.local_tail_body(callable, body, scope)
        } else {
            self.expr(body, scope)
        }
    }

    fn closure(parameter: &str, body: &str, scope: &Scope) -> String {
        // Captures are explicit runtime graph edges. Code is a function
        // pointer, so no hidden Rust closure can retain an untraceable value.
        // A new parameter shadows any captured term with the same source name.
        let mut captures = scope.clone();
        captures.retain(|name| name != parameter);
        let scope = &captures;
        let values = scope
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let restore = scope
            .iter()
            .enumerate()
            .map(|(index, name)| format!("let {} = captures[{index}].clone();", id(name)))
            .collect::<String>();
        format!(
            "V::fun(vec![{values}], |captures, {}| {{ {restore} {body} }})",
            id(parameter)
        )
    }

    fn evidence(&mut self, evidence: &[CoreCallEvidence], scope: &Scope) -> Result<String> {
        let mut values = Vec::new();
        for selected in evidence {
            self.validate_parameter_evidence(selected, scope)?;
            if let Some(dictionary) = self.monad_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.derived_structural_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if selected.constraint.trait_identity.as_deref() == Some("std/prelude::Hash") {
                self.validate_hash_evidence(selected, scope)?;
            }
            if let Some(dictionary) = self.decimal_trait_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.big_int_trait_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.prelude_collection_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.iterable_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.traversable_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.primitive_hash_dictionary(selected, scope)? {
                self.capabilities.insert("hash".into());
                values.push(dictionary);
                continue;
            }
            if selected.constraint.name == "Ord" {
                self.validate_ord_evidence(selected, scope)?;
            }
            if matches!(
                selected.evidence,
                CoreInstanceEvidence::Local { .. } | CoreInstanceEvidence::Imported { .. }
            ) {
                for method in ["Show", "Debug"] {
                    if selected.constraint.name == method
                        && selected
                            .constraint
                            .trait_identity
                            .as_deref()
                            .is_none_or(|identity| identity == format!("std/prelude::{method}"))
                    {
                        self.validate_display_child(selected, method, scope)?;
                    }
                }
            }
            if let Some(dictionary) = self.structural_display_dictionary(selected, scope)? {
                if let Some(key) = seseragi_lowering::closed_standard_display_key(
                    selected,
                    scope.bound_type_names(),
                ) {
                    self.closed_dictionary_sites += 1;
                    values.push(format!(
                        "cached({}, || {dictionary})",
                        quoted(&format!("closed-standard-display:{key}"))
                    ));
                } else {
                    values.push(dictionary);
                }
                continue;
            }
            if let Some(dictionary) = self.filesystem_display_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.int_parse_show_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.decimal_show_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.big_int_display_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.standard_error_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.json_codec_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = self.list_dictionary(selected, scope)? {
                values.push(dictionary);
                continue;
            }
            if let Some(dictionary) = lists::primitive_display(selected) {
                values.push(dictionary?);
                continue;
            }
            if let Some(dictionary) = floats::float_dictionary(selected) {
                if let Some(validation) = self.primitive_operator_dictionary(selected, scope) {
                    validation?;
                }
                self.capabilities.insert("float".into());
                values.push(dictionary?);
                continue;
            }
            if let Some(dictionary) = characters::character_dictionary(selected) {
                values.push(dictionary?);
                continue;
            }
            if let Some(dictionary) = comparisons::int_ord_dictionary(selected) {
                values.push(dictionary?);
                continue;
            }
            if let Some(dictionary) = self.primitive_equality_dictionary(selected, scope) {
                values.push(dictionary?);
                continue;
            }
            if let Some(dictionary) = self.primitive_operator_dictionary(selected, scope) {
                if matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. } if identity.starts_with("std/big-int::"))
                {
                    self.capabilities.insert("big-int".into());
                }
                if matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. } if identity.starts_with("std/decimal::"))
                {
                    self.capabilities.insert("decimal".into());
                }
                values.push(dictionary?);
                continue;
            }
            let value = match &selected.evidence {
                CoreInstanceEvidence::Parameter { index } => {
                    let name = format!("\0dictionary_{index}");
                    if !scope.contains(&name) {
                        return Err("Rust backend: dictionary parameter out of scope".into());
                    }
                    format!("{}.clone()", id(&name))
                }
                CoreInstanceEvidence::Local {
                    identity,
                    evidence_arguments,
                    ..
                }
                | CoreInstanceEvidence::Imported {
                    identity,
                    evidence_arguments,
                    ..
                } => {
                    let provider = match &selected.evidence {
                        CoreInstanceEvidence::Imported {
                            provider_module, ..
                        } => provider_module.as_str(),
                        _ => self.owner,
                    };
                    if !self
                        .module
                        .instances
                        .iter()
                        .any(|i| &i.identity == identity)
                        || self.owners.get(identity).map(String::as_str) != Some(provider)
                    {
                        return Err(format!(
                            "Rust backend: unknown dictionary or provider owner {identity}"
                        )
                        .into());
                    }
                    let arguments = self.evidence(evidence_arguments, scope)?;
                    let factory = format!("{}({arguments})", id(identity));
                    if is_closed_local_evidence(self.module, &selected.evidence) {
                        self.closed_dictionary_sites += 1;
                        format!(
                            "cached({}, || {factory})",
                            quoted(&format!("{:?}", selected.evidence))
                        )
                    } else {
                        factory
                    }
                }
                other => {
                    return Err(format!("Rust backend: unsupported dictionary {other:?}").into())
                }
            };
            values.push(value);
        }
        Ok(format!("vec![{}]", values.join(",")))
    }

    fn reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        if scope.is_blocked(name) {
            return Err("Rust backend: recursive local function is unsupported".into());
        }
        if scope.contains(name) {
            if let Some(callable) = scope.callables.get(name) {
                self.validate_local_evidence(callable, evidence, scope)?;
                let values = self.evidence(evidence, scope)?;
                let value = self.fresh();
                let function = scope.recursive.get(name).map_or_else(
                    || format!("{}.clone()", id(name)),
                    |recursive| recursive.reference(),
                );
                Ok(format!("{{ let mut {value} = {function}; for dictionary in {values} {{ {value} = {value}.apply(dictionary); }} {value} }}"))
            } else if evidence.is_empty() {
                Ok(format!("{}.clone()", id(name)))
            } else {
                Err("Rust backend: evidence for an unproven local callable".into())
            }
        } else if let Some(index) = self.globals.get(name).or_else(|| {
            // CoreIr also uses module-local spellings for let-bound values and
            // generated pattern temporaries. Resolve those only in their owner.
            (!name.contains("::"))
                .then(|| self.globals.get(&format!("{}::{name}", self.owner)))
                .flatten()
        }) {
            if !evidence.is_empty() {
                return Err("Rust backend: global binding evidence is unsupported".into());
            }
            Ok(format!("get_global({index})"))
        } else if self.functions.contains(name) {
            let args = self.evidence(evidence, scope)?;
            Ok(format!("{}({args})", id(name)))
        } else if self.constructors.contains_key(name) {
            if !evidence.is_empty() {
                return Err("Rust backend: constructor evidence unsupported".into());
            }
            Ok(format!("{}()", id(name)))
        } else if comparisons::is_comparison(name) {
            self.comparison_reference(name, evidence, scope)
        } else if equality::is_equality(name) {
            self.equality_reference(name, evidence, scope)
        } else if operators::primitive_reference(name, evidence).is_some()
            || floats::float_binary(evidence, name)
            || dictionary_arithmetic(evidence, name).is_some()
        {
            let method = match name {
                "+" => "add",
                "-" => "sub",
                "*" => "mul",
                "/" => "div",
                "%" => "rem",
                "**" => "pow",
                _ => unreachable!(),
            };
            let dictionary = self.evidence(evidence, scope)?;
            Ok(format!(
                "({dictionary}).remove(0).field({})",
                quoted(method)
            ))
        } else if let Some(arity) = lists::list_operation_arity(name) {
            if !evidence.is_empty() {
                return Err("Rust backend: List operation with unsupported evidence".into());
            }
            self.capabilities.insert("list".into());
            let names: Vec<_> = (0..arity).map(|i| format!("\0list_argument_{i}")).collect();
            let arguments = names
                .iter()
                .map(|n| format!("{}.clone()", id(n)))
                .collect::<Vec<_>>()
                .join(",");
            let mut body = format!("native_list_operation({}, vec![{arguments}])", quoted(name));
            for i in (0..arity).rev() {
                body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
            }
            Ok(body)
        } else if let Some(value) = characters::character_reference(name) {
            if !evidence.is_empty() {
                return Err("Rust backend: Char operation with unsupported evidence".into());
            }
            Ok(value)
        } else if let Some(arity) = floats::float_operation_arity(name) {
            if !evidence.is_empty() {
                return Err("Rust backend: Float operation with unsupported evidence".into());
            }
            self.capabilities.insert("float".into());
            let names: Vec<_> = (0..arity)
                .map(|i| format!("\0float_argument_{i}"))
                .collect();
            let arguments = names
                .iter()
                .map(|n| format!("{}.clone()", id(n)))
                .collect::<Vec<_>>()
                .join(",");
            let mut body = format!(
                "native_float_operation({}, vec![{arguments}])",
                quoted(name)
            );
            for i in (0..arity).rev() {
                body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
            }
            Ok(body)
        } else if name == "std/console::printValue" && evidence.len() == 1 {
            self.capabilities.insert("console".into());
            self.capabilities.insert("effect-sync".into());
            let dictionaries = self.evidence(evidence, scope)?;
            Ok(format!("V::fun(vec![({dictionaries}).remove(0)], |c, value| native_operation(\"console.print\", vec![c[0].field(\"show\").apply(value)]))"))
        } else if let Some(arity) = operation_arity(name) {
            if !evidence.is_empty() {
                return Err("Rust backend: native operation with unsupported evidence".into());
            }
            self.require_operation(name);
            let names: Vec<_> = (0..arity)
                .map(|i| format!("\0native_argument_{i}"))
                .collect();
            let args = names
                .iter()
                .map(|name| format!("{}.clone()", id(name)))
                .collect::<Vec<_>>()
                .join(",");
            let mut body = format!("native_operation({}, vec![{args}])", quoted(name));
            for i in (0..arity).rev() {
                body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
            }
            Ok(body)
        } else {
            Err(format!("Rust backend: unsupported reference {name}").into())
        }
    }

    fn expr(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        self.expr_inner(expr, scope).map_err(|mut error| {
            if error.origin.is_none() {
                error.origin = Some(core_expr_origin(expr).clone());
            }
            if error.module.is_none() {
                error.module = Some(self.owner.to_owned());
            }
            error
        })
    }

    fn expr_inner(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        Ok(match expr {
            CoreExpr::Unit { .. } => "V::Unit".into(),
            CoreExpr::Integer { value, .. } => {
                let value = value
                    .parse::<i64>()
                    .map_err(|_| "Rust backend: invalid integer")?;
                format!("checked_int({value}_i64)")
            }
            CoreExpr::Float64 { value, .. } => {
                self.capabilities.insert("float".into());
                floats::float_literal(value)?
            }
            CoreExpr::String { value, .. } => format!("V::text({})", quoted(value)),
            CoreExpr::Char { value, .. } => characters::character_literal(value)?,
            CoreExpr::Boolean { value, .. } => format!("V::Bool({value})"),
            CoreExpr::Variable { .. } => self.variable(expr, scope)?,
            CoreExpr::EffectOperation {
                operation,
                arguments,
                success,
                ..
            } => {
                if let Some(value) = self.clock_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.prelude_effect_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.die_operation(expr, scope)? {
                    return Ok(value);
                }
                let Some(arity) = operation_arity(operation) else {
                    return Err(format!(
                        "Rust backend: unsupported native Effect operation {operation}"
                    )
                    .into());
                };
                if arguments.len() != arity {
                    return Err(format!("Rust backend: native operation arity {operation}").into());
                }
                if matches!(operation.as_str(), "effect.succeed" | "std/effect::succeed")
                    && matches!(arguments.as_slice(), [CoreExpr::Unit { .. }])
                    && self.local_canonical_type(success, scope)?
                        != (CoreType::Named {
                            name: "std/prelude::Unit".into(),
                            arguments: vec![],
                        })
                {
                    return Err(
                        "Rust backend: literal Unit success requires its canonical result type"
                            .into(),
                    );
                }
                self.require_operation(operation);
                format!(
                    "native_operation({}, vec![{}])",
                    quoted(operation),
                    self.expressions(arguments, scope)?
                )
            }
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                evidence,
                ..
            } => {
                if let Some(value) = self.clock_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.prelude_effect_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.die_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.concurrency_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.from_either_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(value) = self.filesystem_call(expr, scope)? {
                    return Ok(value);
                }
                if let Some(callable) = scope.callables.get(callee) {
                    self.validate_local_arguments(callable, arguments, evidence, scope)?;
                }
                let function = self.reference(callee, evidence, scope)?;
                let temporary = self.fresh();
                let mut value = format!("{{ let mut {temporary} = {function};");
                for arg in arguments {
                    value.push_str(&format!(
                        "{temporary} = {temporary}.apply({});",
                        self.expr(arg, scope)?
                    ));
                }
                value.push_str(&format!("{temporary} }}"));
                value
            }
            CoreExpr::Template { parts, .. } => {
                let text = self.fresh();
                let mut output = format!("{{ let mut {text} = String::new();");
                for part in parts {
                    match part {
                        CoreTemplatePart::Text { value, .. } => {
                            output.push_str(&format!("{text}.push_str({});", quoted(value)))
                        }
                        CoreTemplatePart::Interpolation {
                            value, evidence, ..
                        } => {
                            let selected = evidence.as_ref().ok_or(
                                "Rust backend: interpolation requires checked Show evidence",
                            )?;
                            let dictionary =
                                self.evidence(std::slice::from_ref(selected), scope)?;
                            output.push_str(&format!("{text}.push_str(&({dictionary}).remove(0).field(\"show\").apply({}).render());", self.expr(value, scope)?));
                        }
                    }
                }
                output.push_str(&format!("V::text(&{text}) }}"));
                output
            }
            CoreExpr::Call { .. } => self.call(expr, scope)?,
            CoreExpr::Lambda {
                parameter, body, ..
            } => self.function(std::slice::from_ref(parameter), body, scope)?,
            CoreExpr::Tuple { elements, .. } => format!(
                "V::Tuple(Shared::new(vec![{}]))",
                self.expressions(elements, scope)?
            ),
            CoreExpr::Array { elements, .. } => format!(
                "V::Array(Shared::new(vec![{}]))",
                self.expressions(elements, scope)?
            ),
            CoreExpr::List {
                elements, type_ref, ..
            } => {
                if lists::list_element(type_ref).is_none() {
                    return Err("Rust backend: List literal requires canonical List type".into());
                }
                self.capabilities.insert("list".into());
                format!(
                    "V::list_from_values(vec![{}])",
                    self.expressions(elements, scope)?
                )
            }
            CoreExpr::FieldAccess {
                receiver, field, ..
            } => format!("({}).field({})", self.expr(receiver, scope)?, quoted(field)),
            CoreExpr::OptionalFieldAccess {
                receiver, field, ..
            } => format!(
                "({}).optional_field({})",
                self.expr(receiver, scope)?,
                quoted(field)
            ),
            CoreExpr::Record { items, .. } => self.record(items, scope)?,
            CoreExpr::Binary {
                operator,
                left,
                right,
                evidence,
                type_ref,
                ..
            } => {
                if matches!(operator.as_str(), ".." | "..=") {
                    return self.range_expression(
                        operator,
                        [left, right],
                        evidence,
                        type_ref,
                        scope,
                    );
                }
                if operator == ":" {
                    if !evidence.is_empty() || lists::list_element(type_ref).is_none() {
                        return Err(
                            "Rust backend: cons requires canonical List type and no evidence"
                                .into(),
                        );
                    }
                    self.capabilities.insert("list".into());
                    return Ok(format!(
                        "V::list_cons({}, {})",
                        self.expr(left, scope)?,
                        self.expr(right, scope)?
                    ));
                }
                if self.program_wasm_library && operator == "**" {
                    return Err(
                        "program WASM: native power requires a target-specific arithmetic proof"
                            .into(),
                    );
                }
                self.validate_opaque_numeric_operator_provider(
                    operator,
                    [left, right],
                    evidence,
                    scope,
                )?;
                if operators::big_int_binary(evidence, operator) {
                    self.validate_primitive_operator_binary(
                        operator,
                        [left, right],
                        evidence,
                        type_ref,
                        scope,
                    )?;
                    self.capabilities.insert("big-int".into());
                    return Ok(format!(
                        "native_big_int_arithmetic({}, {}, {})",
                        quoted(operator),
                        self.expr(left, scope)?,
                        self.expr(right, scope)?
                    ));
                }
                if operators::decimal_binary(evidence, operator) {
                    self.validate_primitive_operator_binary(
                        operator,
                        [left, right],
                        evidence,
                        type_ref,
                        scope,
                    )?;
                    self.capabilities.insert("decimal".into());
                    return Ok(format!(
                        "native_decimal_arithmetic({}, {}, {})",
                        quoted(operator),
                        self.expr(left, scope)?,
                        self.expr(right, scope)?
                    ));
                }
                if floats::float_binary(evidence, operator) {
                    if operator == "**" {
                        self.validate_primitive_operator_binary(
                            operator,
                            [left, right],
                            evidence,
                            type_ref,
                            scope,
                        )?;
                    }
                    self.capabilities.insert("float".into());
                    return Ok(format!(
                        "native_float_binary({}, {}, {})",
                        quoted(operator),
                        self.expr(left, scope)?,
                        self.expr(right, scope)?
                    ));
                }
                if let Some(method) = dictionary_arithmetic(evidence, operator) {
                    let dictionary = self.evidence(evidence, scope)?;
                    let function = self.fresh();
                    // A selected user method can execute after its first
                    // source argument, before the second argument is evaluated.
                    return Ok(format!("{{ let {function} = ({dictionary}).remove(0).field({}).apply({}); {function}.apply({}) }}", quoted(method), self.expr(left, scope)?, self.expr(right, scope)?));
                }
                if equality::is_equality(operator) {
                    return self.equality_binary(operator, left, right, evidence, type_ref, scope);
                }
                if let Some(value) =
                    self.descending_int_binary(operator, left, right, evidence, type_ref, scope)?
                {
                    return Ok(value);
                }
                if characters::character_comparison(evidence, operator) {
                    return Ok(format!(
                        "native_char_binary({}, {}, {})",
                        quoted(operator),
                        self.expr(left, scope)?,
                        self.expr(right, scope)?
                    ));
                }
                if let Some(value) = self
                    .decimal_comparison_binary(operator, left, right, evidence, type_ref, scope)?
                {
                    return Ok(value);
                }
                if let Some(value) = self
                    .big_int_comparison_binary(operator, left, right, evidence, type_ref, scope)?
                {
                    return Ok(value);
                }
                if characters::dictionary_comparison(evidence, operator) {
                    let dictionary = self.evidence(evidence, scope)?;
                    let a = self.fresh();
                    let b = self.fresh();
                    let d = self.fresh();
                    if comparisons::is_comparison(operator) {
                        let stage = self.fresh();
                        return Ok(format!("{{ let {stage} = ({dictionary}).remove(0).field(\"compare\").apply({}); native_ordering_comparison({}, {stage}.apply({})) }}", self.expr(left, scope)?, quoted(operator), self.expr(right, scope)?));
                    }
                    return Ok(format!(
                        "{{ let {d} = ({dictionary}).remove(0); let {a} = {}; let {b} = {}; native_dictionary_comparison({}, {d}, {a}, {b}) }}",
                        self.expr(left, scope)?,
                        self.expr(right, scope)?,
                        quoted(operator)
                    ));
                }
                if matches!(operator.as_str(), "/" | "%" | "**") {
                    self.validate_primitive_operator_binary(
                        operator,
                        [left, right],
                        evidence,
                        type_ref,
                        scope,
                    )?;
                    return Ok(format!(
                        "binary({}, {}, {})",
                        quoted(operator),
                        self.expr(left, scope)?,
                        self.expr(right, scope)?
                    ));
                }
                let standard = evidence.iter().all(|e| matches!(&e.evidence, CoreInstanceEvidence::Standard { identity, .. } if matches!(identity.as_str(), "std/int::Add" | "std/int::Sub" | "std/int::Mul" | "std/int::Eq" | "std/int::Ord" | "std/string::Add")));
                if !standard || !matches!(operator.as_str(), "+" | "-" | "*" | "==" | "<" | "<=") {
                    return Err(format!(
                        "Rust backend: unsupported operator/evidence {operator} {evidence:?}"
                    )
                    .into());
                }
                format!(
                    "binary({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                )
            }
            CoreExpr::Unary {
                operator,
                operand,
                type_ref,
                ..
            } if operator == "!" => self.boolean_not(operand, type_ref, scope)?,
            CoreExpr::Unary {
                operator, operand, ..
            } if operator == "-" => format!("negate({})", self.expr(operand, scope)?),
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => format!(
                "if ({}).boolean() {{ {} }} else {{ {} }}",
                self.expr(condition, scope)?,
                self.expr(then_branch, scope)?,
                self.expr(else_branch, scope)?
            ),
            CoreExpr::Decision {
                scrutinee,
                scrutinee_type,
                branches,
                ..
            } => {
                let temporary = self.fresh();
                let label = self.fresh();
                let mut value = format!(
                    "{{ let {temporary} = {}; '{label}: {{",
                    self.expr(scrutinee, scope)?
                );
                for branch in branches {
                    let plan = seseragi_lowering::core_list_prefix_plan(
                        scrutinee_type,
                        branch,
                        scope.bound_type_names(),
                    );
                    let prefix = plan.as_ref().map(|plan| {
                        let prefix = self.fresh();
                        let branch_label = self.fresh();
                        value.push_str(&Self::prefix_checks(
                            plan,
                            &temporary,
                            &prefix,
                            &branch_label,
                        ));
                        prefix
                    });
                    let prefix = prefix.as_deref();
                    let conditions = branch
                        .tests
                        .iter()
                        .skip(usize::from(plan.is_some()))
                        .map(|test| self.test(test, &temporary, prefix))
                        .collect::<Result<Vec<_>>>()?;
                    value.push_str(&format!(
                        "if {} {{",
                        if conditions.is_empty() {
                            "true".into()
                        } else {
                            conditions.join(" && ")
                        }
                    ));
                    let mut inner = scope.clone();
                    for binding in &branch.bindings {
                        value.push_str(&format!(
                            "let {} = {};",
                            id(&binding.name),
                            Self::project_prefix(&temporary, &binding.path, prefix)?
                        ));
                        inner.insert(binding.name.clone());
                    }
                    let guard = branch
                        .guard
                        .as_ref()
                        .map(|g| self.expr(g, &inner))
                        .transpose()?
                        .map(|g| format!("({g}).boolean()"))
                        .unwrap_or_else(|| "true".into());
                    value.push_str(&format!(
                        "if {guard} {{ break '{label} {}; }} }}",
                        self.expr(&branch.value, &inner)?
                    ));
                    if plan.is_some() {
                        value.push('}');
                    }
                }
                value.push_str("panic!(\"Rust backend: unmatched checked decision\"); } }");
                value
            }
            CoreExpr::Sequence {
                statements, result, ..
            } => self.sequence(statements, result, scope)?,
            CoreExpr::MonadDo {
                statements,
                result,
                evidence,
                type_ref,
                ..
            } => self.monad_do(statements, result, evidence, type_ref, scope)?,
            _ => {
                return Err(format!(
                    "Rust backend: unsupported {} expression",
                    match expr {
                        CoreExpr::Float64 { .. } => "Float",
                        CoreExpr::Char { .. } => "Char",
                        CoreExpr::List { .. } => "List",
                        CoreExpr::Template { .. } => "template",
                        CoreExpr::ArrayComprehension { .. }
                        | CoreExpr::ListComprehension { .. } => "comprehension",
                        CoreExpr::Unary { .. } => "unary operator",
                        CoreExpr::EffectOperation { .. } | CoreExpr::EffectInvoke { .. } =>
                            "Effect",
                        CoreExpr::MonadDo { .. } => "monad do",
                        _ => "CoreIr",
                    }
                )
                .into())
            }
        })
    }

    fn record(&mut self, items: &[CoreRecordValueItem], scope: &Scope) -> Result<String> {
        let record = self.fresh();
        let mut output = format!("{{ let mut {record} = BTreeMap::new();");
        for item in items {
            match item {
                CoreRecordValueItem::Field { name, value, .. } => output.push_str(&format!(
                    "{record}.insert({}.to_owned(), {});",
                    quoted(name),
                    self.expr(value, scope)?
                )),
                CoreRecordValueItem::Spread { value, .. } => {
                    let spread = self.fresh();
                    output.push_str(&format!(
                        "{{ let V::Record({spread}) = {} else {{ panic!(\"Rust ABI: expected record\") }}; {record}.extend({spread}.get().iter().map(|(key, value)| (key.clone(), value.clone()))); }}",
                        self.expr(value, scope)?
                    ));
                }
            }
        }
        output.push_str(&format!("V::Record(Shared::new({record})) }}"));
        Ok(output)
    }

    fn require_operation(&mut self, name: &str) {
        self.capabilities.insert("effect-sync".into());
        if name.starts_with("std/ref::") {
            self.capabilities.insert("ref".into());
        }
        if name.starts_with("console.")
            || name.starts_with("std/console::")
            || matches!(name, "std/prelude::print" | "std/prelude::println")
        {
            self.capabilities.insert("console".into());
        }
    }

    fn sequence(
        &mut self,
        statements: &[CoreStatement],
        result: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        let Some((statement, rest)) = statements.split_first() else {
            return self.expr(result, scope);
        };
        if let Some((declarations, inner, count)) =
            self.recursive_local_prefix(statements, scope)?
        {
            return Ok(format!(
                "{{ {declarations} {} }}",
                self.sequence(&statements[count..], result, &inner)?
            ));
        }
        match statement {
            CoreStatement::PureLet { name, value, .. } => {
                let value = self.expr(value, scope)?;
                let mut inner = scope.clone();
                inner.insert(name.clone());
                Ok(format!(
                    "{{ let {} = {value}; {} }}",
                    id(name),
                    self.sequence(rest, result, &inner)?
                ))
            }
            CoreStatement::LocalFunction {
                initializer_in_outer_scope,
                name,
                constraints,
                type_parameters,
                parameters,
                body,
                rec_group,
                origin,
                ..
            } if rec_group.is_none() => {
                let mut callable = LocalCallable::new(
                    type_parameters.iter().map(|p| (p.name.clone(), p.arity)),
                    constraints,
                    parameters,
                );
                if !initializer_in_outer_scope {
                    callable.self_name = Some(name.clone());
                }
                if *initializer_in_outer_scope && parameters.is_empty() {
                    callable.method_value = local::method_argument_type(body)
                        .filter(|ty| matches!(ty, CoreType::Function { .. }));
                }
                let value = self
                    .local_function(&callable, body, scope)
                    .map_err(|error| crate::project::at(&error.message, origin, self.owner))?;
                let mut inner = scope.clone();
                inner.insert_callable(name.clone(), callable);
                Ok(format!(
                    "{{ let {} = {value}; {} }}",
                    id(name),
                    self.sequence(rest, result, &inner)?
                ))
            }
            CoreStatement::Bind { name, value, .. } => {
                self.effect_step(name, value, rest, result, scope)
            }
            CoreStatement::Effect { value } => {
                let unused = self.fresh();
                self.effect_step(&unused, value, rest, result, scope)
            }
            _ => Err("Rust backend: recursive local function is unsupported".into()),
        }
    }

    fn effect_step(
        &mut self,
        name: &str,
        value: &CoreExpr,
        rest: &[CoreStatement],
        result: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        self.capabilities.insert("effect-sync".into());
        let source = self.expr(value, scope)?;
        let mut inner = scope.clone();
        inner.insert(name.into());
        let body = self.sequence(rest, result, &inner)?;
        let tail = CoreExpr::Sequence {
            statements: rest.to_vec(),
            result: Box::new(result.clone()),
            origin: core_expr_origin(result).clone(),
        };
        let mut captures = scope.clone();
        if let Some(mut free) = core_free_variables(&tail) {
            free.names.remove(name);
            for index in free.evidence_parameters {
                free.names.insert(format!("\0dictionary_{index}"));
            }
            captures.retain_free(&free.names);
        }
        let next = Self::closure(name, &body, &captures);
        let temporary = self.fresh();
        Ok(format!(
            "{{ let {temporary} = {source}; V::bind({temporary}, {next}) }}"
        ))
    }

    fn expressions(&mut self, values: &[CoreExpr], scope: &Scope) -> Result<String> {
        Ok(values
            .iter()
            .map(|v| self.expr(v, scope))
            .collect::<Result<Vec<_>>>()?
            .join(","))
    }

    fn project(root: &str, path: &[CoreDecisionProjection]) -> Result<String> {
        let mut value = format!("{root}.clone()");
        for projection in path {
            value = match projection {
                CoreDecisionProjection::AdtPayload => format!("({value}).payload()"),
                CoreDecisionProjection::TupleElement { index }
                | CoreDecisionProjection::ArrayElement { index } => {
                    format!("({value}).index({index})")
                }
                CoreDecisionProjection::RecordField { name } => {
                    format!("({value}).field({})", quoted(name))
                }
                CoreDecisionProjection::ListElement { index } => {
                    format!("({value}).list_element({index})")
                }
                CoreDecisionProjection::ListRest { start } => {
                    format!("({value}).list_rest({start})")
                }
                _ => return Err("Rust backend: unsupported decision projection".into()),
            };
        }
        Ok(value)
    }

    fn prefix_checks(
        plan: &seseragi_lowering::CoreListPrefixPlan,
        root: &str,
        prefix: &str,
        label: &str,
    ) -> String {
        let mut body = format!("'{label}: {{ let {prefix}_0 = &{root};");
        for index in 0..plan.length {
            body.push_str(&format!(
                "let Some({prefix}_cell{index}) = {prefix}_{index}.list_cell() else {{ break '{label}; }}; let {prefix}_{} = &{prefix}_cell{index}.tail;",
                index+1
            ));
        }
        if !plan.minimum {
            body.push_str(&format!(
                "if {prefix}_{}.list_cell().is_some() {{ break '{label}; }}",
                plan.length
            ));
        }
        body
    }

    fn project_prefix(
        root: &str,
        path: &[CoreDecisionProjection],
        prefix: Option<&str>,
    ) -> Result<String> {
        if let Some(prefix) = prefix {
            match path.first() {
                Some(CoreDecisionProjection::ListElement { index }) => {
                    return Self::project(&format!("{prefix}_cell{index}.head"), &path[1..]);
                }
                Some(CoreDecisionProjection::ListRest { start }) => {
                    return Self::project(&format!("{prefix}_{start}"), &path[1..]);
                }
                _ => {}
            }
        }
        Self::project(root, path)
    }

    fn test(&self, test: &CoreDecisionTest, root: &str, prefix: Option<&str>) -> Result<String> {
        let project = |path: &[CoreDecisionProjection]| Self::project_prefix(root, path, prefix);
        Ok(match test {
            CoreDecisionTest::Constructor {
                path, constructor, ..
            } => format!("({}).tagged({})", project(path)?, quoted(constructor)),
            CoreDecisionTest::Integer { path, value, .. } => format!(
                "matches!({}, V::Int(n) if n == {}_i64)",
                project(path)?,
                value
                    .parse::<i64>()
                    .map_err(|_| "Rust backend: invalid decision integer")?
            ),
            CoreDecisionTest::Boolean { path, value, .. } => {
                format!("matches!({}, V::Bool(n) if n == {value})", project(path)?)
            }
            CoreDecisionTest::String { path, value, .. } => format!(
                "native_textual_literal_matches(&({}), {})",
                project(path)?,
                quoted(value)
            ),
            CoreDecisionTest::ListLength {
                path,
                length,
                minimum,
                ..
            } => format!(
                "({}).list_length_matches({length}, {minimum})",
                project(path)?
            ),
            _ => return Err("Rust backend: unsupported decision test".into()),
        })
    }
}

fn dictionary_arithmetic<'a>(evidence: &[CoreCallEvidence], operator: &'a str) -> Option<&'a str> {
    let (name, method) = match operator {
        "+" => ("Add", "add"),
        "-" => ("Sub", "sub"),
        "*" => ("Mul", "mul"),
        "/" => ("Div", "div"),
        "%" => ("Rem", "rem"),
        "**" => ("Pow", "pow"),
        _ => return None,
    };
    let [selected] = evidence else { return None };
    (selected.constraint.name == name
        && selected.constraint.arguments.len() == 3
        && selected
            .constraint
            .trait_identity
            .as_deref()
            .is_none_or(|id| id == format!("std/prelude::{name}"))
        && matches!(
            selected.evidence,
            CoreInstanceEvidence::Parameter { .. }
                | CoreInstanceEvidence::Local { .. }
                | CoreInstanceEvidence::Imported { .. }
        ))
    .then_some(method)
}
fn operation_arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/effect::succeed"
        | "effect.succeed"
        | "std/effect::fail"
        | "effect.fail"
        | "std/effect::defer"
        | "std/effect::scoped"
        | "std/ref::make"
        | "std/ref::get"
        | "std/prelude::println"
        | "console.println"
        | "std/prelude::print"
        | "console.print"
        | "std/console::print"
        | "std/console::println"
        | "std/console::error"
        | "std/console::errorLine"
        | "std/console::flush" => 1,
        "std/effect::acquireRelease"
        | "std/effect::recover"
        | "std/effect::flatMap"
        | "std/effect::mapError"
        | "effect.mapError"
        | "std/ref::set"
        | "std/ref::update"
        | "std/ref::modify" => 2,
        _ => return None,
    })
}
