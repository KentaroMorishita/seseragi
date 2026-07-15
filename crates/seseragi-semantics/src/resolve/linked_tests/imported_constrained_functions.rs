use super::linked_program;
use crate::{
    analyze_linked_module, resolve_linked_module, TypedDecl, TypedExpr, TypedInstanceEvidence,
};

const DOMAIN: &str = "\
pub trait Ready<A> { fn ready value: A -> String }
pub fn describe<T> value: T -> String
where Ready<T> =
  ready value
";

const MAIN: &str = "\
import { Ready, describe } from \"./domain\"
pub type Badge = | Active
instance Ready<Badge> { fn ready value: Badge -> String = \"imported ready\" }
pub fn status value: Badge -> String = describe value
";

#[test]
fn resolves_callable_scheme_constraints_to_provider_trait_identities() {
    let linked = linked_program(MAIN, [("./domain", "fixture/game::domain", DOMAIN)]);
    let resolved = resolve_linked_module(linked, MAIN);
    let describe = resolved
        .imports
        .iter()
        .find(|import| import.export.name == "describe")
        .unwrap();

    assert!(matches!(
        describe.scheme_trait_bindings.as_deref(),
        Some([binding])
            if binding.spelling == "Ready"
                && binding.canonical == "fixture/game::domain::trait(Ready)"
    ));
}

#[test]
fn passes_consumer_local_evidence_to_an_imported_constrained_function() {
    let linked = linked_program(MAIN, [("./domain", "fixture/game::domain", DOMAIN)]);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", MAIN),
        linked,
        MAIN,
    )
    .unwrap();
    let status = analyzed
        .typed_hir
        .declarations
        .iter()
        .find_map(|declaration| {
            let TypedDecl::Fn { symbol, body, .. } = declaration else {
                return None;
            };
            symbol.ends_with("::status").then_some(body)
        });

    assert!(matches!(
        status,
        Some(TypedExpr::Call { callee, evidence, .. })
            if callee == "fixture/game::domain::describe"
                && matches!(evidence.as_slice(), [call]
                    if matches!(&call.evidence, TypedInstanceEvidence::Local { identity, .. }
                        if identity == "fixture/game::domain::trait(Ready)<fixture/game::main::Badge>"))
    ));
}

#[test]
fn reports_missing_evidence_instead_of_an_unresolved_imported_name() {
    let source = "\
import { describe } from \"./domain\"
pub type Badge = | Active
pub fn status value: Badge -> String = describe value
";
    let linked = linked_program(source, [("./domain", "fixture/game::domain", DOMAIN)]);
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0101" && diagnostic.message_key == "instance.missing"
    }));
    assert!(diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code != "SES-N0001"));
}

#[test]
fn preserves_constraint_identity_for_a_namespace_selected_callable() {
    let source = "\
import { Ready } from \"./domain\"
import * as domain from \"./domain\"
pub type Badge = | Active
instance Ready<Badge> { fn ready value: Badge -> String = \"namespace ready\" }
pub fn status value: Badge -> String = domain.describe value
";
    let linked = linked_program(source, [("./domain", "fixture/game::domain", DOMAIN)]);

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap();
}
