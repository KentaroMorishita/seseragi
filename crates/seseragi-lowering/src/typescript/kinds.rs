use super::{TypeScriptKindConstructor, TypeScriptKinds, TypeScriptType};
use seseragi_syntax::TypeParameter;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn constructor_name(name: &str) -> String {
    // `$` cannot occur in a source identifier. Names have already been selected
    // by canonical source import planning, including aliases and collisions.
    format!("$ssrg$kind${name}")
}

pub(crate) fn partial_name(shape: &[bool]) -> String {
    format!(
        "$ssrg$Partial${}${}",
        shape.len(),
        shape
            .iter()
            .map(|&fixed| if fixed { '1' } else { '0' })
            .collect::<String>()
    )
}

pub(crate) fn kind_constraint(arity: u32) -> String {
    format!(
        "{{ readonly args: readonly [{}]; readonly type: unknown }}",
        vec!["unknown"; arity as usize].join(", ")
    )
}

#[derive(Clone, Debug, Default)]
pub(super) struct Requirements {
    pub(super) constructors: BTreeMap<String, TypeScriptKindConstructor>,
    pub(super) partials: BTreeSet<Vec<bool>>,
    pub(super) application: bool,
}

impl Requirements {
    pub(super) fn output(&self) -> Option<TypeScriptKinds> {
        (!self.constructors.is_empty() || !self.partials.is_empty() || self.application).then(
            || TypeScriptKinds {
                constructors: self.constructors.values().cloned().collect(),
                partials: self.partials.iter().cloned().collect(),
                application: self.application,
            },
        )
    }
}

pub(super) fn nominal_constructor(
    name: &str,
    identity: &str,
    exported: bool,
    parameters: &[TypeParameter],
) -> TypeScriptKindConstructor {
    TypeScriptKindConstructor {
        name: constructor_name(name),
        identity: identity.to_owned(),
        exported,
        parameters: parameters.to_vec(),
        target: TypeScriptType::Reference {
            name: name.to_owned(),
            arguments: (0..parameters.len())
                .map(|index| TypeScriptType::KindArgument { index })
                .collect(),
        },
    }
}

/// Closed, value-parameter constructors with a canonical intrinsic TS
/// representation. Other runtime constructors keep their existing erasure
/// unless their complete kind signature is supplied by declaration metadata.
pub(super) fn intrinsic_constructor(canonical: &str) -> Option<TypeScriptKindConstructor> {
    let arg = |index| Box::new(TypeScriptType::KindArgument { index });
    let (name, arity, target) = match canonical {
        "std/prelude::Maybe" => (
            "Maybe",
            1,
            TypeScriptType::Reference {
                name: "import(\"@seseragi/runtime/sum\").Maybe".to_owned(),
                arguments: vec![TypeScriptType::KindArgument { index: 0 }],
            },
        ),
        "std/prelude::Either" => (
            "Either",
            2,
            TypeScriptType::Reference {
                name: "import(\"@seseragi/runtime/sum\").Either".to_owned(),
                arguments: vec![
                    TypeScriptType::KindArgument { index: 0 },
                    TypeScriptType::KindArgument { index: 1 },
                ],
            },
        ),
        "std/prelude::Array" => ("Array", 1, TypeScriptType::Array { element: arg(0) }),
        "std/prelude::List" => ("List", 1, TypeScriptType::List { element: arg(0) }),
        _ => return runtime_constructor(canonical),
    };
    Some(TypeScriptKindConstructor {
        name: format!("$ssrg$builtin$kind${name}"),
        identity: canonical.to_owned(),
        exported: false,
        parameters: (0..arity)
            .map(|index| TypeParameter::value(format!("A{index}")))
            .collect(),
        target,
    })
}

// These are complete canonical declaration kinds, rather than a guess from an
// imported total arity. The parity test checks each against the standard source
// interface. The target is the runtime's existing public type contract.
fn runtime_constructor(canonical: &str) -> Option<TypeScriptKindConstructor> {
    let kinds: &[u32] = match canonical {
        "std/prelude::Effect" | "std/stream::Stream" => &[0, 0, 0],
        "std/prelude::Product"
        | "std/prelude::Sum"
        | "std/ref::Ref"
        | "std/signal::Signal"
        | "std/signal::MutableSignal"
        | "std/collection::ReduceStep"
        | "std/prelude::Iterator"
        | "std/non-empty-list::NonEmptyList"
        | "std/set::Set"
        | "std/queue::Queue"
        | "std/json::Decoder"
        | "std/json::Encoder"
        | "std/prelude::Js.NullOr"
        | "std/prelude::Js.Nullable"
        | "std/prelude::Js.UndefinedOr"
        | "std/prelude::Js.Promise"
        | "std/prelude::Js.MutableArray" => &[0],
        "std/effect::Fiber"
        | "std/effect::FiberExit"
        | "std/deferred::Deferred"
        | "std/validation::Validation"
        | "std/map::Map"
        | "std/prelude::Js.Callback" => &[0, 0],
        "std/transformer/maybe::MaybeT" => &[1, 0],
        "std/transformer/either::EitherT"
        | "std/transformer/reader::ReaderT"
        | "std/transformer/state::StateT"
        | "std/transformer/writer::WriterT" => &[0, 1, 0],
        _ => return None,
    };
    let runtime = crate::runtime_types::runtime_type_import(canonical)?;
    Some(TypeScriptKindConstructor {
        name: format!("$ssrg$builtin$kind${}", runtime.export_name),
        identity: canonical.to_owned(),
        exported: false,
        parameters: kinds
            .iter()
            .enumerate()
            .map(|(index, &arity)| TypeParameter {
                name: format!("A{index}"),
                arity,
            })
            .collect(),
        target: TypeScriptType::Reference {
            name: format!("import({:?}).{}", runtime.module, runtime.export_name),
            arguments: (0..kinds.len())
                .map(|index| TypeScriptType::KindArgument { index })
                .collect(),
        },
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_constructor_kinds_match_canonical_standard_declarations() {
        for canonical in [
            "std/prelude::Effect",
            "std/prelude::Product",
            "std/prelude::Sum",
            "std/prelude::Iterator",
            "std/prelude::Js.NullOr",
            "std/prelude::Js.Nullable",
            "std/prelude::Js.UndefinedOr",
            "std/prelude::Js.Promise",
            "std/prelude::Js.MutableArray",
            "std/prelude::Js.Callback",
            "std/non-empty-list::NonEmptyList",
            "std/set::Set",
            "std/map::Map",
            "std/queue::Queue",
            "std/json::Decoder",
            "std/json::Encoder",
            "std/ref::Ref",
            "std/signal::Signal",
            "std/signal::MutableSignal",
            "std/effect::Fiber",
            "std/effect::FiberExit",
            "std/deferred::Deferred",
            "std/stream::Stream",
            "std/validation::Validation",
            "std/collection::ReduceStep",
            "std/transformer/maybe::MaybeT",
            "std/transformer/either::EitherT",
            "std/transformer/reader::ReaderT",
            "std/transformer/state::StateT",
            "std/transformer/writer::WriterT",
        ] {
            let (module, name) = canonical.rsplit_once("::").unwrap();
            if module == "std/prelude" {
                assert_eq!(
                    seseragi_semantics::prelude_type_constructor_arity(name),
                    Some(
                        super::intrinsic_constructor(canonical)
                            .unwrap()
                            .parameters
                            .len() as u32
                    )
                );
                continue;
            }
            let interface = seseragi_project::standard_module_target(module).unwrap();
            let export = interface
                .interface()
                .exports
                .iter()
                .find(|export| export.namespace == "type" && export.name == name)
                .unwrap();
            assert_eq!(
                export
                    .scheme
                    .type_parameters
                    .iter()
                    .map(|parameter| parameter.arity)
                    .collect::<Vec<_>>(),
                super::intrinsic_constructor(canonical)
                    .unwrap()
                    .parameters
                    .iter()
                    .map(|parameter| parameter.arity)
                    .collect::<Vec<_>>(),
                "{canonical}"
            );
        }
        assert!(super::intrinsic_constructor("user::Effect").is_none());
    }
}
