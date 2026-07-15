use crate::{
    lower_typed_module, CoreInstanceEvidence, CoreInstanceImplementation, CoreShowPayloadEvidence,
    CoreType,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_semantics::{analyze_linked_module, analyze_module_interface, type_module};
use seseragi_syntax::{parse_diagnostics, parse_unlinked_module_interface};
use std::collections::BTreeMap;

#[test]
fn carries_selected_derived_show_evidence_into_core_ir() {
    let source = "\
pub type AppError deriving Show =
  | UnknownHand String
  | EndOfInput
";

    let core = lower_typed_module(type_module("artifact/derived-show/main.ssrg", source));

    assert_eq!(core.instances.len(), 1);
    let instance = &core.instances[0];
    assert_eq!(instance.identity, "Show<artifact/derived-show::AppError>");
    assert_eq!(instance.trait_name, "Show");
    assert_eq!(
        instance.type_identity.as_deref(),
        Some("artifact/derived-show::AppError")
    );
    assert_eq!(
        instance.arguments,
        vec![CoreType::Named {
            name: "AppError".to_owned(),
            arguments: Vec::new(),
        }]
    );
    assert!(instance.constraints.is_empty());
    assert_eq!(
        instance.implementation,
        CoreInstanceImplementation::DerivedShow {
            adt_symbol: "artifact/derived-show::AppError".to_owned(),
            payload_evidence: vec![CoreShowPayloadEvidence {
                variant_symbol: "artifact/derived-show::UnknownHand".to_owned(),
                type_identity: "std/prelude::String".to_owned(),
                evidence: CoreInstanceEvidence::Standard {
                    identity: "Show<std/prelude::String>".to_owned(),
                },
            }],
        }
    );
    assert_eq!(
        &source[instance.origin.start..instance.origin.end],
        "pub type AppError deriving Show =\n  | UnknownHand String\n  | EndOfInput"
    );
}

#[test]
fn carries_local_show_payload_evidence_into_core_ir() {
    let source = "type Detail deriving Show = | Message String\ntype AppError deriving Show = | Wrapped Detail\n";

    let core = lower_typed_module(type_module("artifact/local-show/main.ssrg", source));
    let app_error = core
        .instances
        .iter()
        .find(|instance| instance.type_identity.as_deref() == Some("artifact/local-show::AppError"))
        .unwrap();

    assert!(matches!(
        &app_error.implementation,
        CoreInstanceImplementation::DerivedShow {
            payload_evidence,
            ..
        } if matches!(
            payload_evidence.as_slice(),
            [CoreShowPayloadEvidence {
                variant_symbol,
                type_identity,
                evidence: CoreInstanceEvidence::Local { identity, .. },
            }] if variant_symbol == "artifact/local-show::Wrapped"
                && type_identity == "artifact/local-show::Detail"
                && identity == "Show<artifact/local-show::Detail>"
        )
    ));
}

#[test]
fn carries_imported_show_payload_evidence_into_core_ir() {
    let domain_source = "pub type ImportedError deriving Show =\n  | Message String\n";
    let main_source = "import { ImportedError } from \"./domain\"\n\npub type AppError deriving Show =\n  | Invalid ImportedError\n";
    let domain =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let domain_interface = analyze_module_interface(
        parse_diagnostics("domain.ssrg", domain_source),
        domain.interface.clone(),
        domain_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let targets = BTreeMap::from([(
        "./domain".to_owned(),
        ModuleLinkTarget::same_package(domain.header, domain_interface).unwrap(),
    )]);
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    let linked = link_module(main, &targets).unwrap();
    let typed = analyze_linked_module(
        parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap()
    .typed_hir;

    let core = lower_typed_module(typed);

    assert_eq!(core.instances.len(), 1);
    let instance = &core.instances[0];
    assert_eq!(instance.identity, "Show<fixture/game::main::AppError>");
    assert_eq!(
        instance.implementation,
        CoreInstanceImplementation::DerivedShow {
            adt_symbol: "fixture/game::main::AppError".to_owned(),
            payload_evidence: vec![CoreShowPayloadEvidence {
                variant_symbol: "fixture/game::main::Invalid".to_owned(),
                type_identity: "fixture/game::domain::ImportedError".to_owned(),
                evidence: CoreInstanceEvidence::Imported {
                    identity: "Show<fixture/game::domain::ImportedError>".to_owned(),
                    provider_module: "fixture/game::domain".to_owned(),
                },
            }],
        }
    );
}

#[test]
fn leaves_core_instance_evidence_empty_without_selected_instances() {
    let source = "\
pub type AppError =
  | EndOfInput
";

    let core = lower_typed_module(type_module("artifact/no-show/main.ssrg", source));

    assert!(core.instances.is_empty());
}
