use crate::{analyze_linked_module, semantic_diagnostics};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::{
    parse_diagnostics, parse_unlinked_module_interface, ByteSpan, InterfaceInstance, InterfaceType,
    ModuleInterface,
};
use std::collections::BTreeMap;

#[test]
fn reports_unknown_deriving_trait_with_registered_name_code() {
    let artifact = semantic_diagnostics(
        "artifact/unknown-deriving/main.ssrg",
        "type Hand deriving Shwo = | Rock\n",
    );

    assert_eq!(artifact.diagnostics.len(), 1);
    assert_eq!(artifact.diagnostics[0].code, "SES-N0001");
    assert_eq!(artifact.diagnostics[0].message_key, "name.unresolved");
    assert_eq!(
        artifact.diagnostics[0].related[0].message,
        "deriving trait Shwo is not defined"
    );
}

#[test]
fn reports_missing_show_instance_for_unsupported_payload() {
    let artifact = semantic_diagnostics(
        "artifact/unsupported-derived-show/main.ssrg",
        "type Labels deriving Show = | Labels Array<String>\n",
    );

    assert_eq!(artifact.diagnostics.len(), 1);
    assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
    assert_eq!(
        artifact.diagnostics[0].message_key,
        "trait.instance-missing"
    );
    assert_eq!(
        artifact.diagnostics[0].related[0].message,
        "required Show<Array<String>> instance is not available"
    );
}

#[test]
fn reports_generic_derived_show_as_unsupported_without_inventing_constraints() {
    let artifact = semantic_diagnostics(
        "artifact/generic-derived-show/main.ssrg",
        "type Box<A> deriving Show = | Box A\n",
    );

    assert_eq!(artifact.diagnostics.len(), 1);
    assert_eq!(artifact.diagnostics[0].code, "SES-T0201");
    assert_eq!(
        artifact.diagnostics[0].related[0].message,
        "derived Show<Box> for generic ADTs is not implemented yet"
    );
}

#[test]
fn does_not_misclassify_other_standard_deriving_traits_as_unknown() {
    let artifact = semantic_diagnostics(
        "artifact/known-deriving/main.ssrg",
        "type Hand deriving Eq = | Rock\n",
    );

    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn reports_distinct_dependency_providers_as_an_ambiguous_instance() {
    let source = "import * as left from \"./left\"\nimport * as right from \"./right\"\npub effect fn main = succeed ()\n";
    let linked = linked_with_instance_targets(
        source,
        [
            (
                "./left",
                instance_target(
                    "fixture/coherence::left",
                    "fixture/coherence::provider-left",
                    "fixture/coherence::types::InputError",
                ),
            ),
            (
                "./right",
                instance_target(
                    "fixture/coherence::right",
                    "fixture/coherence::provider-right",
                    "fixture/coherence::types::InputError",
                ),
            ),
        ],
    );

    let diagnostics =
        analyze_linked_module(parse_diagnostics("main.ssrg", source), linked, source).unwrap_err();

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0202");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "trait.instance-ambiguous"
    );
}

#[test]
fn reports_a_local_and_imported_instance_conflict_as_ambiguous() {
    let source =
        "import * as evidence from \"./evidence\"\npub type Local deriving Show = | Local\n";
    let linked = linked_with_instance_targets(
        source,
        [(
            "./evidence",
            instance_target(
                "fixture/coherence::evidence",
                "fixture/coherence::evidence",
                "fixture/coherence::main::Local",
            ),
        )],
    );

    let diagnostics =
        analyze_linked_module(parse_diagnostics("main.ssrg", source), linked, source).unwrap_err();

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0202");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "trait.instance-ambiguous"
    );
    assert!(diagnostics.diagnostics[0].related[0]
        .message
        .contains("fixture/coherence::evidence"));
}

fn linked_with_instance_targets<const N: usize>(
    source: &str,
    targets: [(&str, ModuleLinkTarget); N],
) -> seseragi_project::LinkedModule {
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/coherence::main", source);
    link_module(
        main,
        &targets
            .into_iter()
            .map(|(specifier, target)| (specifier.to_owned(), target))
            .collect::<BTreeMap<_, _>>(),
    )
    .unwrap()
}

fn instance_target(module: &str, provider_module: &str, type_identity: &str) -> ModuleLinkTarget {
    ModuleLinkTarget::external(ModuleInterface {
        schema: 1,
        module: module.to_owned(),
        source: "evidence.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports: Vec::new(),
        operators: Vec::new(),
        instances: vec![InterfaceInstance {
            identity: Some(format!("Show<{type_identity}>")),
            provider_module: Some(provider_module.to_owned()),
            type_identity: Some(type_identity.to_owned()),
            trait_name: "Show".to_owned(),
            type_parameters: Vec::new(),
            head: InterfaceType::Apply {
                constructor: "Show".to_owned(),
                arguments: vec![InterfaceType::Named {
                    name: "Local".to_owned(),
                    arguments: Vec::new(),
                }],
            },
            constraints: Vec::new(),
            origin: ByteSpan { start: 0, end: 1 },
        }],
    })
}
