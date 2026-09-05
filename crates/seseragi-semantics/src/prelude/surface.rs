use serde::Serialize;
use seseragi_syntax::TypeParameter;

#[cfg(test)]
use crate::TypedType;

use super::{
    standard_instance_constraint_specs, trait_by_name, trait_method_signature,
    PreludeTraitMethodSignature, SPECIAL_STANDARD_INSTANCES, STANDARD_INSTANCES, TRAITS,
    TRAIT_METHODS,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleSurface {
    schema: u32,
    kind: &'static str,
    language_version: &'static str,
    module: &'static str,
    traits: Vec<StandardTraitSurface>,
    instances: Vec<StandardInstanceSurface>,
    builtin_instances: Vec<StandardBuiltinInstanceSurface>,
    instance_audit: StandardInstanceAuditSurface,
    coherence: StandardCoherenceSurface,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardInstanceAuditSurface {
    normative_sources: &'static [&'static str],
    matrix: Vec<StandardInstanceAuditRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardInstanceAuditRow {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    head: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity: Option<&'static str>,
    status: StandardInstanceAuditStatus,
    classification: StandardInstanceAuditClassification,
    spec: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tracking_issue: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StandardInstanceAuditStatus {
    SpecifiedAndImplemented,
    SpecifiedButImplementationMissing,
    IntentionallyUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StandardInstanceAuditClassification {
    Dictionary,
    Conditional,
    Structural,
    OperatorAbiWithDictionary,
    Missing,
    Unavailable,
}

#[derive(Clone, Copy)]
struct StandardInstanceAuditSpec {
    trait_name: &'static str,
    head: &'static str,
    spec: &'static str,
    tracking_issue: Option<u32>,
}

const STRUCTURAL_INSTANCES: &[StandardInstanceAuditSpec] = &[
    audit_spec("Eq", "Tuple<...>", "9.3", None),
    audit_spec("Eq", "closed record", "9.3", None),
    audit_spec("Ord", "Tuple<...>", "9.4", None),
    audit_spec("Hash", "Tuple<...>", "9.4", None),
    audit_spec("Show", "Tuple<...>", "9.11", None),
    audit_spec("Show", "closed record", "9.11", None),
    audit_spec("Debug", "Tuple<...>", "9.11", None),
    audit_spec("Debug", "closed record", "9.11", None),
    audit_spec("JsonEncode", "Tuple<...>", "10.9", None),
    audit_spec("JsonEncode", "closed record", "10.9", None),
    audit_spec("JsonDecode", "Tuple<...>", "10.9", None),
    audit_spec("JsonDecode", "closed record", "10.9", None),
];

// This is the specification-side half of the audit. Implemented rows are
// projected directly from the canonical registries above, while missing rows
// stay explicit until their queue issue connects the real instance and removes
// the corresponding entry here. Every missing row must name a positive local
// tracking issue; implemented/structural/unavailable rows carry no stale issue.
// This contract is checked during projection, without contacting GitHub.
const MISSING_INSTANCES: &[StandardInstanceAuditSpec] = &[];

const UNAVAILABLE_INSTANCES: &[StandardInstanceAuditSpec] = &[
    audit_spec("Functor", "Set", "10.5", None),
    audit_spec("Eq", "Float", "9.4", None),
    audit_spec("Ord", "Float", "9.4", None),
    audit_spec("Hash", "Float", "9.4", None),
    audit_spec("Monoid", "Float", "9.5", None),
    audit_spec("Monoid", "Int", "9.5", None),
    audit_spec("Monad", "Signal", "9.7", None),
    audit_spec("Monad", "Validation<E, _>", "9.7 / 10.4", None),
    audit_spec("JsonEncode", "Float", "10.9", None),
    audit_spec("JsonDecode", "Float", "10.9", None),
    audit_spec("JsonEncode", "BigInt", "10.9", None),
    audit_spec("JsonDecode", "BigInt", "10.9", None),
    audit_spec("Eq", "open record", "9.3", None),
    audit_spec("Show", "open record", "9.11", None),
    audit_spec("Debug", "open record", "9.11", None),
];

const fn audit_spec(
    trait_name: &'static str,
    head: &'static str,
    spec: &'static str,
    tracking_issue: Option<u32>,
) -> StandardInstanceAuditSpec {
    StandardInstanceAuditSpec {
        trait_name,
        head,
        spec,
        tracking_issue,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardTraitSurface {
    name: &'static str,
    canonical: &'static str,
    type_parameters: Vec<TypeParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supertrait: Option<&'static str>,
    #[serde(default, skip_serializing_if = "is_false")]
    deriving: bool,
    methods: Vec<StandardTraitMethodSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardTraitMethodSurface {
    name: &'static str,
    canonical: &'static str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    operators: Vec<&'static str>,
    signature: PreludeTraitMethodSignature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardInstanceSurface {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    trait_canonical: &'static str,
    type_constructor: &'static str,
    type_constructor_canonical: String,
    type_constructor_arity: u32,
    identity: &'static str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    constraints: Vec<StandardInstanceConstraintSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardInstanceConstraintSurface {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    trait_canonical: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_argument_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_argument_indices: Option<&'static [usize]>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardBuiltinInstanceSurface {
    #[serde(rename = "trait")]
    trait_name: &'static str,
    trait_canonical: &'static str,
    arguments: Vec<&'static str>,
    identity: &'static str,
    dispatch: super::PreludeSpecialInstanceDispatch,
    #[serde(default, skip_serializing_if = "is_false")]
    strict_equality_compatible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandardCoherenceSurface {
    standard_heads: &'static str,
    user_overlap: &'static str,
    diagnostic: &'static str,
}

fn is_false(value: &bool) -> bool {
    !value
}

fn implemented_instance_spec(trait_name: &str) -> &'static str {
    match trait_name {
        "Eq" | "Ord" | "Hash" => "9.3 / 9.4",
        "Semigroup" | "Monoid" | "Zero" | "One" => "9.5",
        "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Pow" => "9.6",
        "Functor" | "Applicative" | "Monad" => "9.7",
        "Show" | "Debug" => "9.11",
        "Iterable" | "Reducible" | "Traversable" => "10.5",
        "JsonEncode" | "JsonDecode" => "10.9",
        _ => "9.1",
    }
}

fn standard_instance_audit() -> StandardInstanceAuditSurface {
    standard_instance_audit_with_missing(MISSING_INSTANCES)
}

fn standard_instance_audit_with_missing(
    missing: &[StandardInstanceAuditSpec],
) -> StandardInstanceAuditSurface {
    let mut matrix = Vec::new();
    matrix.extend(STANDARD_INSTANCES.iter().map(|instance| {
        let conditional = !standard_instance_constraint_specs(instance.identity).is_empty();
        StandardInstanceAuditRow {
            trait_name: instance.trait_name,
            head: instance.type_name.to_owned(),
            identity: Some(instance.identity),
            status: StandardInstanceAuditStatus::SpecifiedAndImplemented,
            classification: if conditional {
                StandardInstanceAuditClassification::Conditional
            } else {
                StandardInstanceAuditClassification::Dictionary
            },
            spec: implemented_instance_spec(instance.trait_name),
            tracking_issue: None,
        }
    }));
    matrix.extend(
        SPECIAL_STANDARD_INSTANCES
            .iter()
            .map(|instance| StandardInstanceAuditRow {
                trait_name: instance.trait_name,
                head: instance.arguments.join(", "),
                identity: Some(instance.identity),
                status: StandardInstanceAuditStatus::SpecifiedAndImplemented,
                classification: match instance.dispatch {
                    super::PreludeSpecialInstanceDispatch::Dictionary => {
                        StandardInstanceAuditClassification::Dictionary
                    }
                    super::PreludeSpecialInstanceDispatch::OperatorAbi => {
                        StandardInstanceAuditClassification::OperatorAbiWithDictionary
                    }
                },
                spec: implemented_instance_spec(instance.trait_name),
                tracking_issue: None,
            }),
    );
    let structural_identities = [
        "std/tuple::Eq",
        "std/record::Eq",
        "std/tuple::Ord",
        "std/tuple::Hash",
        "std/tuple::Show",
        "std/record::Show",
        "std/tuple::Debug",
        "std/record::Debug",
        "std/tuple::JsonEncode",
        "std/record::JsonEncode",
        "std/tuple::JsonDecode",
        "std/record::JsonDecode",
    ];
    matrix.extend(STRUCTURAL_INSTANCES.iter().zip(structural_identities).map(
        |(spec, identity)| StandardInstanceAuditRow {
            trait_name: spec.trait_name,
            head: spec.head.to_owned(),
            identity: Some(identity),
            status: StandardInstanceAuditStatus::SpecifiedAndImplemented,
            classification: StandardInstanceAuditClassification::Structural,
            spec: spec.spec,
            tracking_issue: None,
        },
    ));
    matrix.extend(missing.iter().map(|spec| StandardInstanceAuditRow {
        trait_name: spec.trait_name,
        head: spec.head.to_owned(),
        identity: None,
        status: StandardInstanceAuditStatus::SpecifiedButImplementationMissing,
        classification: StandardInstanceAuditClassification::Missing,
        spec: spec.spec,
        tracking_issue: spec.tracking_issue,
    }));
    matrix.extend(
        UNAVAILABLE_INSTANCES
            .iter()
            .map(|spec| StandardInstanceAuditRow {
                trait_name: spec.trait_name,
                head: spec.head.to_owned(),
                identity: None,
                status: StandardInstanceAuditStatus::IntentionallyUnavailable,
                classification: StandardInstanceAuditClassification::Unavailable,
                spec: spec.spec,
                tracking_issue: None,
            }),
    );
    validate_audit_tracking(&matrix).expect("invalid canonical standard instance tracking");
    matrix.sort_by(|left, right| {
        left.trait_name
            .cmp(right.trait_name)
            .then_with(|| left.head.cmp(&right.head))
            .then_with(|| left.status.cmp(&right.status))
    });
    StandardInstanceAuditSurface {
        normative_sources: &[
            "docs/spec/04-type-classes.md",
            "docs/spec/09-standard-library.md",
            "docs/spec/10-library-surface.md",
        ],
        matrix,
    }
}

fn validate_audit_tracking(matrix: &[StandardInstanceAuditRow]) -> Result<(), String> {
    for row in matrix {
        let valid = match row.status {
            StandardInstanceAuditStatus::SpecifiedButImplementationMissing => {
                row.tracking_issue.is_some_and(|issue| issue > 0)
            }
            StandardInstanceAuditStatus::SpecifiedAndImplemented
            | StandardInstanceAuditStatus::IntentionallyUnavailable => row.tracking_issue.is_none(),
        };
        if !valid {
            return Err(format!(
                "{}<{}> ({:?}): missing instances require a positive tracking issue; other rows must not retain tracking metadata",
                row.trait_name, row.head, row.status
            ));
        }
    }
    Ok(())
}

pub fn standard_prelude_surface() -> StandardModuleSurface {
    StandardModuleSurface {
        schema: 1,
        kind: "standard-module-surface",
        language_version: seseragi_project::IMPLEMENTED_LANGUAGE_VERSION,
        module: "std/prelude",
        traits: TRAITS
            .iter()
            .map(|trait_spec| StandardTraitSurface {
                name: trait_spec.name,
                canonical: trait_spec.canonical,
                type_parameters: trait_spec
                    .type_parameters
                    .iter()
                    .map(|parameter| {
                        if parameter.arity == 0 {
                            TypeParameter::value(parameter.name)
                        } else {
                            TypeParameter::constructor(parameter.name, parameter.arity)
                        }
                    })
                    .collect(),
                supertrait: trait_spec.supertrait,
                deriving: trait_spec.deriving,
                methods: TRAIT_METHODS
                    .iter()
                    .filter(|method| method.trait_name == trait_spec.name)
                    .map(|method| StandardTraitMethodSurface {
                        name: method.name,
                        canonical: method.canonical,
                        operators: method.operators.to_vec(),
                        signature: trait_method_signature(method),
                    })
                    .collect(),
            })
            .collect(),
        instances: STANDARD_INSTANCES
            .iter()
            .map(|instance| {
                let trait_spec = trait_by_name(instance.trait_name)
                    .expect("standard instance trait must exist in the Prelude registry");
                StandardInstanceSurface {
                    trait_name: instance.trait_name,
                    trait_canonical: trait_spec.canonical,
                    type_constructor: instance.type_name,
                    type_constructor_canonical: instance
                        .type_canonical
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("std/prelude::{}", instance.type_name)),
                    type_constructor_arity: instance.type_arity,
                    identity: instance.identity,
                    constraints: standard_instance_constraint_specs(instance.identity)
                        .iter()
                        .map(|constraint| {
                            let required = trait_by_name(constraint.trait_name)
                                .expect("standard instance constraint trait must exist");
                            StandardInstanceConstraintSurface {
                                trait_name: constraint.trait_name,
                                trait_canonical: required.canonical,
                                type_argument_index: (constraint.type_argument_indices.len() == 1)
                                    .then(|| constraint.type_argument_indices[0]),
                                type_argument_indices: (constraint.type_argument_indices.len()
                                    != 1)
                                    .then_some(constraint.type_argument_indices),
                            }
                        })
                        .collect(),
                }
            })
            .collect(),
        builtin_instances: SPECIAL_STANDARD_INSTANCES
            .iter()
            .map(|instance| {
                let trait_spec = trait_by_name(instance.trait_name)
                    .expect("builtin instance trait must exist in the Prelude registry");
                StandardBuiltinInstanceSurface {
                    trait_name: instance.trait_name,
                    trait_canonical: trait_spec.canonical,
                    arguments: instance.arguments.to_vec(),
                    identity: instance.identity,
                    dispatch: instance.dispatch,
                    strict_equality_compatible: instance.strict_equality_compatible,
                }
            })
            .collect(),
        instance_audit: standard_instance_audit(),
        coherence: StandardCoherenceSurface {
            standard_heads: "sealed",
            user_overlap: "compile-error",
            diagnostic: "trait.instance-duplicate",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untracked_missing_instances_fail_the_canonical_projection() {
        for issue in [None, Some(0)] {
            let missing = [audit_spec("Functor", "SyntheticFuture", "9.7", issue)];
            assert!(
                std::panic::catch_unwind(|| standard_instance_audit_with_missing(&missing))
                    .is_err()
            );
        }
    }

    #[test]
    fn tracked_missing_instances_are_projected_and_removed_without_stale_metadata() {
        let missing = [audit_spec("Functor", "SyntheticFuture", "9.7", Some(507))];
        let audit = standard_instance_audit_with_missing(&missing);
        let json = serde_json::to_value(&audit).unwrap();
        let row = json["matrix"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["head"] == "SyntheticFuture")
            .unwrap();
        assert_eq!(row["trackingIssue"], 507);
        assert_eq!(row["status"], "specified-but-implementation-missing");
        let completed = standard_instance_audit_with_missing(&[]);
        assert!(completed
            .matrix
            .iter()
            .all(|row| row.tracking_issue.is_none()));
        assert!(!serde_json::to_string(&completed)
            .unwrap()
            .contains("trackingIssue"));
    }

    #[test]
    fn implemented_structural_and_unavailable_rows_need_no_tracking_issue() {
        let audit = standard_instance_audit();
        assert_eq!(validate_audit_tracking(&audit.matrix), Ok(()));
        for classification in [
            StandardInstanceAuditClassification::Dictionary,
            StandardInstanceAuditClassification::Structural,
            StandardInstanceAuditClassification::Unavailable,
        ] {
            let row = audit
                .matrix
                .iter()
                .find(|row| row.classification == classification)
                .unwrap();
            assert_eq!(row.tracking_issue, None);
            let mut stale = row.clone();
            stale.tracking_issue = Some(507);
            assert!(validate_audit_tracking(&[stale]).is_err());
        }
    }

    #[test]
    fn exposes_registered_traits_methods_instances_and_coherence() {
        let surface = standard_prelude_surface();

        assert_eq!(surface.language_version, "0.1.0");
        assert_eq!(surface.traits.len(), 23);
        assert_eq!(
            surface
                .traits
                .iter()
                .flat_map(|trait_spec| &trait_spec.methods)
                .count(),
            24
        );
        assert_eq!(surface.instances.len(), 299);
        assert_eq!(surface.builtin_instances.len(), 44);
        for identity in [
            "std/maybe::Eq",
            "std/either::Eq",
            "std/int::Hash",
            "std/bool::Hash",
            "std/char::Hash",
            "std/string::Hash",
            "std/unit::Hash",
            "std/array::Traversable",
            "std/list::Traversable",
            "std/non-empty-list::Traversable",
        ] {
            assert!(surface
                .instances
                .iter()
                .any(|instance| instance.identity == identity));
        }
        for identity in [
            "std/int::Eq",
            "std/int::Zero",
            "std/string::Add",
            "std/float::Pow",
            "std/array::Iterable",
            "std/iterator::Iterable",
            "std/list::Reducible",
            "std/range::Reducible",
        ] {
            assert!(surface
                .builtin_instances
                .iter()
                .any(|instance| instance.identity == identity));
        }

        let implemented = surface
            .instance_audit
            .matrix
            .iter()
            .filter(|row| row.status == StandardInstanceAuditStatus::SpecifiedAndImplemented)
            .collect::<Vec<_>>();
        assert_eq!(implemented.len(), 299 + 44 + 12);
        for instance in SPECIAL_STANDARD_INSTANCES {
            assert!(implemented
                .iter()
                .any(|row| row.identity == Some(instance.identity)));
        }
        for (trait_name, head, status, issue) in [
            (
                "Eq",
                "Int",
                StandardInstanceAuditStatus::SpecifiedAndImplemented,
                None,
            ),
            (
                "Eq",
                "Float",
                StandardInstanceAuditStatus::IntentionallyUnavailable,
                None,
            ),
            (
                "Hash",
                "Int",
                StandardInstanceAuditStatus::SpecifiedAndImplemented,
                None,
            ),
            (
                "Traversable",
                "Array",
                StandardInstanceAuditStatus::SpecifiedAndImplemented,
                None,
            ),
        ] {
            assert!(surface.instance_audit.matrix.iter().any(|row| {
                row.trait_name == trait_name
                    && row.head == head
                    && row.status == status
                    && row.tracking_issue == issue
            }));
        }

        let mut keys = std::collections::BTreeSet::new();
        for row in &surface.instance_audit.matrix {
            assert!(
                keys.insert((row.trait_name, row.head.as_str())),
                "duplicate canonical instance audit row: {}<{}>",
                row.trait_name,
                row.head
            );
        }
        for identity in [
            "std/decimal::Eq",
            "std/decimal::Ord",
            "Show<std/decimal::Decimal>",
            "Eq<std/decimal::DecimalParseError>",
            "std/decimal::JsonEncode",
            "std/decimal::JsonDecode",
            "std/big-int::Eq",
            "std/big-int::Ord",
            "Show<std/big-int::BigInt>",
            "Eq<std/big-int::BigIntParseError>",
            "Eq<std/bytes/hex::HexDecodeError>",
            "Show<std/bytes/hex::HexDecodeError>",
            "Debug<std/bytes/base64::Base64DecodeError>",
            "Show<std/bytes::ByteError>",
            "Debug<std/bytes::BytesSliceError>",
            "Show<std/text::Utf8DecodeError>",
            "Show<std/effect::ScheduleError>",
            "Show<std/effect::ParallelismError>",
            "Debug<std/effect::ParallelismError>",
            "Show<std/stream::BufferCapacityError>",
            "Debug<std/stream::BufferCapacityError>",
            "Show<std/queue::QueueCreateError>",
            "Debug<std/queue::QueueCreateError>",
            "Show<std/queue::QueueClosed>",
            "Debug<std/queue::QueueClosed>",
            "Show<std/semaphore::SemaphoreCreateError>",
            "Debug<std/semaphore::SemaphoreCreateError>",
            "Show<std/path::PathError>",
            "Show<std/process::ProcessSignal>",
            "Debug<std/process::ProcessSignal>",
            "Show<std/process::ProcessError>",
            "Debug<std/process::ProcessError>",
            "Show<std/child-process::ChildProcessConfigError>",
            "Debug<std/child-process::ChildProcessError>",
            "Show<std/child-process::ChildExitStatus>",
            "Show<std/random::RandomRangeError>",
            "Debug<std/random::RandomRangeError>",
            "Show<std/random::RandomConfigError>",
            "Debug<std/random::RandomConfigError>",
            "Show<std/entropy::EntropyConfigError>",
            "Debug<std/entropy::EntropyConfigError>",
            "Show<std/entropy::EntropyError>",
            "Debug<std/entropy::EntropyError>",
            "Debug<std/fs::FileSystemError>",
            "Show<std/fs::FileTextError>",
            "Debug<std/time::DurationError>",
            "Show<std/time::DateTimeError>",
            "Debug<std/time::TimeZoneError>",
            "Show<std/stdin::StdinConfigError>",
            "Debug<std/log::LogError>",
            "Show<std/http::HttpBuildError>",
            "Debug<std/http::HttpError>",
            "Show<std/web/navigation::UrlBuildError>",
            "Debug<std/web/navigation::NavigationError>",
            "Show<std/web/storage::StorageArea>",
            "Debug<std/web/storage::StorageError>",
            "std/int::JsonEncode",
            "std/either::JsonDecode",
            "std/json::JsonEncode",
            "std/stream::Functor",
            "std/stream::Applicative",
            "std/stream::Monad",
        ] {
            assert!(surface
                .instances
                .iter()
                .any(|instance| instance.identity == identity));
        }
        assert_eq!(surface.coherence.standard_heads, "sealed");

        let monoid = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Monoid")
            .expect("Monoid must be part of the standard Prelude surface");
        assert_eq!(monoid.type_parameters, vec![TypeParameter::value("A")]);
        assert_eq!(monoid.supertrait, Some("Semigroup"));

        let debug = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Debug")
            .expect("Debug must be part of the standard Prelude surface");
        assert_eq!(debug.type_parameters, vec![TypeParameter::value("A")]);
        assert_eq!(debug.methods[0].name, "debug");
        assert_eq!(
            debug.methods[0].signature.result,
            TypedType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }
        );

        for (identity, operation) in [("std/sum::Monoid", "Add"), ("std/product::Monoid", "Mul")] {
            let wrapper = surface
                .instances
                .iter()
                .find(|instance| instance.identity == identity)
                .unwrap();
            assert_eq!(wrapper.constraints.len(), 2);
            assert_eq!(wrapper.constraints[0].type_argument_index, Some(0));
            assert_eq!(wrapper.constraints[1].trait_name, operation);
            assert_eq!(wrapper.constraints[1].type_argument_index, None);
            assert_eq!(
                wrapper.constraints[1].type_argument_indices,
                Some([0, 0, 0].as_slice())
            );
            assert!(surface
                .instance_audit
                .matrix
                .iter()
                .any(|row| row.identity == Some(identity)
                    && row.status == StandardInstanceAuditStatus::SpecifiedAndImplemented));
        }

        let array_show = surface
            .instances
            .iter()
            .find(|instance| instance.identity == "std/array::Show")
            .expect("Array Show must be part of the standard Prelude surface");
        assert_eq!(array_show.constraints.len(), 1);
        assert_eq!(array_show.constraints[0].trait_name, "Show");
        assert_eq!(array_show.constraints[0].type_argument_index, Some(0));

        for name in [
            "Eq",
            "Ord",
            "Hash",
            "Show",
            "Debug",
            "Zero",
            "One",
            "Semigroup",
            "Monoid",
            "JsonEncode",
            "JsonDecode",
            "Functor",
            "Applicative",
            "Monad",
            "Iterable",
            "Reducible",
            "Traversable",
            "Add",
            "Sub",
            "Mul",
            "Div",
            "Rem",
            "Pow",
        ] {
            assert!(surface
                .traits
                .iter()
                .any(|trait_spec| trait_spec.name == name));
        }

        let reducible = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Reducible")
            .expect("Reducible must be part of the standard Prelude surface");
        assert_eq!(
            reducible.type_parameters,
            vec![TypeParameter::value("C"), TypeParameter::value("A")]
        );
        assert_eq!(reducible.supertrait, Some("Iterable"));

        let traversable = surface
            .traits
            .iter()
            .find(|trait_spec| trait_spec.name == "Traversable")
            .expect("Traversable must be part of the standard Prelude surface");
        assert_eq!(traversable.supertrait, Some("Functor"));
        assert_eq!(traversable.methods[0].signature.constraints.len(), 1);
        assert_eq!(
            traversable.methods[0].signature.constraints[0].name,
            "Applicative"
        );

        let deriving = surface
            .traits
            .iter()
            .filter(|trait_spec| trait_spec.deriving)
            .map(|trait_spec| trait_spec.name)
            .collect::<Vec<_>>();
        assert_eq!(
            deriving,
            vec![
                "Eq",
                "Ord",
                "Hash",
                "Show",
                "Debug",
                "JsonEncode",
                "JsonDecode"
            ]
        );
    }
}
