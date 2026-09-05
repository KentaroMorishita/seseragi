use super::linked_program;
use crate::{
    analyze_linked_module, resolve_linked_module, TypedComprehensionClause, TypedDecl, TypedExpr,
    TypedInstanceEvidence, TypedInstanceImplementation, TypedPattern, TypedType,
};

mod transitive;

#[test]
fn infers_an_imported_iterable_element_and_selects_its_provider_dictionary() {
    let domain_source = "pub type Countdown = | Countdown Int\n\
         fn advance limit: Int -> current: Int -> Maybe<(Int, Int)> =\n\
           if current <= limit then Just (current, current + 1) else Nothing\n\
         instance Iterable<Countdown, Int> {\n\
           fn iterate values: Countdown -> Iterator<Int> =\n\
             match values { Countdown limit -> unfold (advance limit) 1 }\n\
         }\n";
    let main_source = "import { Countdown } from \"./domain\"\n\n\
         pub fn values source: Countdown -> Array<Int> =\n\
           [value | value <- source]\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert_eq!(resolved.dependency_instances.len(), 1);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    let TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected imported Iterable consumer");
    };
    let TypedExpr::ArrayComprehension { clauses, .. } = body else {
        panic!("expected imported Iterable comprehension: {body:#?}");
    };
    assert!(matches!(
        &clauses[0],
        TypedComprehensionClause::Generator {
            pattern: TypedPattern::Binding { type_ref, .. },
            evidence: crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Imported {
                    provider_module,
                    ..
                },
                ..
            },
            ..
        } if type_ref == &TypedType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        } && provider_module == "fixture/game::domain"
    ));
}

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
            ..
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
                                ..
                            } if identity == "Show<fixture/game::domain::ImportedError>"
                                && provider_module == "fixture/game::domain"
                        )
            )
    ));
}

#[test]
fn selects_imported_derived_evidence_for_a_generic_nominal() {
    let domain_source = "pub newtype Code deriving Show, Debug = String\n\
         \n\
         pub type Remote<A> deriving Show, Debug =\n\
           | Missing\n\
           | Remote Maybe<A>\n";
    let main_source = "import { Code, Remote } from \"./domain\"\n\n\
         pub fn render value: Remote<Code> -> String = show value\n\
         pub fn inspect value: Remote<Code> -> String = debug value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/remote::domain", domain_source)],
    );

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn selects_imported_json_evidence_for_a_nested_nominal() {
    let domain_source = "pub struct User deriving JsonEncode, JsonDecode {\n\
           id: Int,\n\
           name: String,\n\
         }\n\
         pub struct Envelope deriving JsonEncode, JsonDecode { owner: User }\n";
    let main_source = "import { Envelope } from \"./domain\"\n\n\
         fn requireEncode<T> value: T -> T where JsonEncode<T> = value\n\
         fn requireDecode<T> value: T -> T where JsonDecode<T> = value\n\
         pub fn encodeEvidence value: Envelope -> Envelope = requireEncode value\n\
         pub fn decodeEvidence value: Envelope -> Envelope = requireDecode value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/json::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert!(resolved
        .dependency_instances
        .iter()
        .any(|instance| { instance.identity == "JsonEncode<fixture/json::domain::Envelope>" }));
    assert!(resolved
        .dependency_instances
        .iter()
        .any(|instance| { instance.identity == "JsonDecode<fixture/json::domain::Envelope>" }));

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn selects_an_imported_binary_instance_for_the_same_nominal() {
    let domain_source = "pub type Score =\n\
           | Points Int\n\n\
         instance Add<Score, Int, Score> {\n\
           fn add left: Score -> right: Int -> Score =\n\
             match left { Points value -> Points (value + right) }\n\
         }\n";
    let main_source = "import { Points, Score } from \"./domain\"\n\n\
         pub fn addBonus bonus: Int -> score: Score -> Score = score + bonus\n\n\
         pub fn total values: Array<Int> -> Score =\n\
           values |> reduce (Points 0) (+) |> addBonus 0\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/score::domain", domain_source)],
    );

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
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
            ..
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

#[test]
fn preserves_imported_phantom_constraints_in_structural_deriving() {
    let domain = "pub type Phantom<A> deriving Eq = | Phantom\n";
    let source = "import { Phantom as Remote } from \"./domain\"\npub struct Outer<A> deriving Eq { value: Remote<A> }\npub fn same left: Outer<Float> -> right: Outer<Float> -> Bool = left == right\n";
    let linked = linked_program(source, [("./domain", "fixture/phantom::domain", domain)]);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap();
    assert!(analyzed.typed_hir.instances[0].constraints.is_empty());
}

#[test]
fn substitutes_local_struct_parameters_inside_imported_nominal_fields() {
    let domain = "pub struct Box<A> { value: A }\n";
    let source = "import { Box } from \"./domain\"\nstruct Outer<A> { nested: Box<A> }\npub fn wrap value: Box<Int> -> Outer<Int> = Outer { nested: value }\nlet boxed: Box<Int> = Box { value: 1 }\nlet wrapped: Outer<Int> = Outer { nested: boxed }\n";
    let linked = linked_program(
        source,
        [("./domain", "fixture/nested-fields::domain", domain)],
    );
    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap();
}
