use super::linked_program;
use crate::{
    analyze_linked_module, resolve_linked_module, TypedInstanceEvidence,
    TypedInstanceImplementation,
};

mod transitive;

#[test]
fn selects_direct_dependency_show_evidence_for_a_derived_payload() {
    let domain_source = "pub type ImportedError deriving Show =\n  | Message String\n";
    let main_source = "import { ImportedError } from \"./domain\"\n\npub type AppError deriving Show =\n  | Invalid ImportedError\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert_eq!(resolved.dependency_instances.len(), 1);
    assert_eq!(
        resolved.dependency_instances[0].identity,
        "Show<fixture/game::domain::ImportedError>"
    );
    assert_eq!(
        resolved.dependency_instances[0].type_identity.as_deref(),
        Some("fixture/game::domain::ImportedError")
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    let instance = &analyzed.typed_hir.instances[0];
    assert_eq!(instance.identity, "Show<fixture/game::main::AppError>");
    assert!(matches!(
        &instance.implementation,
        TypedInstanceImplementation::DerivedShow {
            adt_symbol,
            payload_evidence,
        } if adt_symbol == "fixture/game::main::AppError"
            && matches!(
                payload_evidence.as_slice(),
                [evidence]
                    if evidence.variant_symbol == "fixture/game::main::Invalid"
                        && evidence.type_identity == "fixture/game::domain::ImportedError"
                        && matches!(
                            &evidence.evidence,
                            TypedInstanceEvidence::Imported {
                                identity,
                                provider_module,
                            } if identity == "Show<fixture/game::domain::ImportedError>"
                                && provider_module == "fixture/game::domain"
                        )
            )
    ));
}

#[test]
fn selects_by_canonical_payload_identity_when_modules_share_a_type_spelling() {
    let left_source = "pub type ImportedError deriving Show =\n  | LeftMessage String\n";
    let right_source = "pub type ImportedError deriving Show =\n  | RightMessage String\n";
    let main_source = "import { ImportedError as LeftError } from \"./left\"\nimport { ImportedError as RightError } from \"./right\"\n\npub type AppError deriving Show =\n  | Invalid RightError\n";
    let linked = linked_program(
        main_source,
        [
            ("./left", "fixture/game::left", left_source),
            ("./right", "fixture/game::right", right_source),
        ],
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    let TypedInstanceImplementation::DerivedShow {
        payload_evidence, ..
    } = &analyzed.typed_hir.instances[0].implementation
    else {
        panic!("expected derived Show instance");
    };
    let [evidence] = payload_evidence.as_slice() else {
        panic!("expected one selected payload evidence");
    };
    assert_eq!(evidence.type_identity, "fixture/game::right::ImportedError");
    assert!(matches!(
        &evidence.evidence,
        TypedInstanceEvidence::Imported {
            identity,
            provider_module,
        } if identity == "Show<fixture/game::right::ImportedError>"
            && provider_module == "fixture/game::right"
    ));
}

#[test]
fn keeps_missing_imported_show_as_an_instance_diagnostic() {
    let domain_source = "pub type ImportedError =\n  | Message String\n";
    let main_source = "import { ImportedError } from \"./domain\"\n\npub type AppError deriving Show =\n  | Invalid ImportedError\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();
    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0201");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "trait.instance-missing"
    );
}

#[test]
fn deduplicates_instance_evidence_reached_through_repeated_dependency_edges() {
    let domain_source = "pub type ImportedError deriving Show =\n  | Message String\n";
    let main_source = "import { ImportedError } from \"./domain\"\nimport * as domain from \"./domain\"\n\npub type AppError deriving Show =\n  | Invalid ImportedError\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked, main_source);
    assert_eq!(resolved.dependency_instances.len(), 1);
}
