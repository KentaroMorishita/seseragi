use crate::semantic_diagnostics;

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
