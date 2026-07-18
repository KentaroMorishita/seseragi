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

    assert!(
        artifact.diagnostics.is_empty(),
        "{:?}",
        artifact.diagnostics
    );
}

#[test]
fn accepts_a_local_instance_that_matches_its_trait_contract() {
    let artifact = semantic_diagnostics(
        "artifact/instance-contract/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score> { fn render value: Score -> String = \"score\" }\n",
    );

    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn accepts_alpha_renamed_higher_kinded_method_contracts() {
    let artifact = semantic_diagnostics(
        "artifact/instance-hkt-contract/main.ssrg",
        "trait MapLike<F<_>> { fn map<A, B> f: (A -> B) -> value: F<A> -> F<B> where Eq<A>, Show<B> }\n\
         type Box<A> = | Box A\n\
         instance MapLike<Box> { fn map<X, Y> f: (X -> Y) -> value: Box<X> -> Box<Y> where Show<Y>, Eq<X> = match value { Box item -> Box (f item) } }\n",
    );

    assert!(
        artifact.diagnostics.is_empty(),
        "{:?}",
        artifact.diagnostics
    );
}

#[test]
fn reports_a_higher_kinded_instance_argument_mismatch() {
    let artifact = semantic_diagnostics(
        "artifact/instance-kind-mismatch/main.ssrg",
        "trait Functor<F<_>> { fn map<A, B> f: (A -> B) -> value: F<A> -> F<B> }\n\
         instance Functor<Int> { fn map<A, B> f: (A -> B) -> value: Int -> Int = value }\n",
    );

    assert!(artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0101"
            && diagnostic.message_key == "trait.instance-kind-mismatch"
            && diagnostic.related[0]
                .message
                .contains("expects kind Type -> Type")
    }));
}

#[test]
fn rejects_an_instance_without_its_supertrait_instance() {
    let artifact = semantic_diagnostics(
        "artifact/instance-supertrait-missing/main.ssrg",
        "trait Functor<F<_>> { fn map<A, B> f: (A -> B) -> value: F<A> -> F<B> }\n\
         trait Applicative<F<_>> where Functor<F> { fn pure<A> value: A -> F<A> }\n\
         instance Applicative<Maybe> { fn pure<A> value: A -> Maybe<A> = Just value }\n",
    );

    assert!(artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0201"
            && diagnostic.message_key == "trait.supertrait-instance-missing"
            && diagnostic.related[0]
                .message
                .contains("required supertrait instance Functor<Maybe>")
    }));
}

#[test]
fn accepts_an_instance_when_its_supertrait_instance_is_available() {
    let artifact = semantic_diagnostics(
        "artifact/instance-supertrait/main.ssrg",
        "trait Functor<F<_>> { fn map<A, B> f: (A -> B) -> value: F<A> -> F<B> }\n\
         instance Functor<Maybe> { fn map<A, B> f: (A -> B) -> value: Maybe<A> -> Maybe<B> = match value {\n\
           Nothing -> Nothing\n\
           Just item -> Just (f item)\n\
         } }\n\
         trait Applicative<F<_>> where Functor<F> { fn pure<A> value: A -> F<A> }\n\
         instance Applicative<Maybe> { fn pure<A> value: A -> Maybe<A> = Just value }\n",
    );

    assert!(
        artifact.diagnostics.is_empty(),
        "{:?}",
        artifact.diagnostics
    );
}

#[test]
fn reports_an_instance_method_signature_mismatch() {
    let artifact = semantic_diagnostics(
        "artifact/instance-contract-mismatch/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score> { fn render value: Score -> Int = 0 }\n",
    );

    assert_eq!(artifact.diagnostics.len(), 1);
    assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        artifact.diagnostics[0].message_key,
        "trait.instance-method-signature-mismatch"
    );
    assert_eq!(
        artifact.diagnostics[0].related[0].message,
        "instance method render must match this trait contract"
    );
}

#[test]
fn reports_missing_and_unexpected_instance_methods() {
    let artifact = semantic_diagnostics(
        "artifact/instance-method-set/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score> { fn describe value: Score -> String = \"score\" }\n",
    );

    assert_eq!(artifact.diagnostics.len(), 2);
    assert_eq!(
        artifact.diagnostics[0].message_key,
        "trait.instance-method-missing"
    );
    assert_eq!(
        artifact.diagnostics[1].message_key,
        "trait.instance-method-unexpected"
    );
}

#[test]
fn reports_instance_trait_arity_and_missing_method_implementation() {
    let arity = semantic_diagnostics(
        "artifact/instance-arity/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score, String> { fn render value: Score -> String = \"score\" }\n",
    );
    assert_eq!(arity.diagnostics.len(), 1);
    assert_eq!(
        arity.diagnostics[0].message_key,
        "trait.instance-arity-mismatch"
    );

    let body = semantic_diagnostics(
        "artifact/instance-method-body/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score> { fn render value: Score -> String }\n",
    );
    assert_eq!(body.diagnostics.len(), 1);
    assert_eq!(
        body.diagnostics[0].message_key,
        "trait.instance-method-missing"
    );
}

#[test]
fn reports_duplicate_instance_method_definitions() {
    let artifact = semantic_diagnostics(
        "artifact/instance-method-duplicate/main.ssrg",
        "trait Render<A> { fn render value: A -> String }\n\
         newtype Score = Int\n\
         instance Render<Score> {\n\
           fn render value: Score -> String = \"first\"\n\
           fn render value: Score -> String = \"second\"\n\
         }\n",
    );

    assert_eq!(artifact.diagnostics.len(), 1);
    assert_eq!(artifact.diagnostics[0].code, "SES-N0002");
    assert_eq!(
        artifact.diagnostics[0].message_key,
        "name.duplicate-definition"
    );
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

#[test]
fn rejects_a_user_instance_that_overlaps_a_standard_prelude_head() {
    let source = "instance Functor<Maybe> {\n\
                  fn map<A, B> f: (A -> B) -> value: Maybe<A> -> Maybe<B> =\n\
                    match value {\n\
                      Nothing -> Nothing\n\
                      Just item -> Just $ f item\n\
                    }\n\
                  }\n";

    let diagnostics = crate::semantic_diagnostics("main.ssrg", source);

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0202");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "trait.instance-duplicate"
    );
    assert!(diagnostics.diagnostics[0].related[0]
        .message
        .contains("std/maybe::Functor"));
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
            trait_identity: Some("Show".to_owned()),
            argument_identities: vec![type_identity.to_owned()],
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
