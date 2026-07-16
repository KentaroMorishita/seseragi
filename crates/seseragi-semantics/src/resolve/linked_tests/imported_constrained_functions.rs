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

const DOMAIN_WITH_INSTANCE: &str = "\
pub type Badge = | Active
pub trait Ready<A> { fn ready value: A -> String }
instance Ready<Badge> { fn ready value: Badge -> String = \"provider ready\" }
pub fn describe<T> value: T -> String
where Ready<T> =
  ready value
";

const DOMAIN_WITH_GENERIC_INSTANCE: &str = "\
pub trait Inspect<A> { fn inspect value: A -> String }
instance<T> Inspect<Maybe<T>> {
  fn inspect value: Maybe<T> -> String = \"generic\"
}
pub fn report<T> value: T -> String
where Inspect<T> =
  inspect value
";

const DOMAIN_WITH_CONSTRAINED_GENERIC_INSTANCE: &str = "\
pub trait Ready<A> { fn ready value: A -> String }
pub trait Inspect<A> { fn inspect value: A -> String }
instance Ready<Int> { fn ready value: Int -> String = \"ready\" }
instance<T> Inspect<Maybe<T>>
where Ready<T> {
  fn inspect value: Maybe<T> -> String = \"inspect\"
}
pub fn report<T> value: T -> String
where Inspect<T> =
  inspect value
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
fn passes_provider_evidence_to_an_imported_constrained_function() {
    let source = "\
import { Active, describe } from \"./domain\"
pub fn status value: Unit -> String = describe Active
";
    let linked = linked_program(
        source,
        [("./domain", "fixture/game::domain", DOMAIN_WITH_INSTANCE)],
    );
    let resolved = resolve_linked_module(linked.clone(), source);
    assert!(matches!(
        resolved.dependency_instances.as_slice(),
        [instance]
            if instance.trait_identity == "fixture/game::domain::trait(Ready)"
                && instance.argument_identities == ["fixture/game::domain::Badge"]
    ));

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
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
        Some(TypedExpr::Call { evidence, .. })
            if matches!(evidence.as_slice(), [call]
                if matches!(&call.evidence, TypedInstanceEvidence::Imported {
                    identity,
                    provider_module,
                    ..
                } if identity == "fixture/game::domain::trait(Ready)<fixture/game::domain::Badge>"
                    && provider_module == "fixture/game::domain"))
    ));
}

#[test]
fn specializes_an_imported_generic_instance_factory() {
    let source = "\
import { report } from \"./domain\"
pub fn status value: Unit -> String = report (Just 42)
";
    let linked = linked_program(
        source,
        [(
            "./domain",
            "fixture/game::domain",
            DOMAIN_WITH_GENERIC_INSTANCE,
        )],
    );
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
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
        Some(TypedExpr::Call { evidence, .. })
            if matches!(evidence.as_slice(), [call]
                if matches!(&call.evidence, TypedInstanceEvidence::Imported {
                    identity,
                    provider_module,
                    type_arguments,
                    evidence_arguments,
                } if identity == "fixture/game::domain::trait(Inspect)<std/prelude::Maybe<$0>>"
                    && provider_module == "fixture/game::domain"
                    && matches!(type_arguments.as_slice(), [crate::TypedType::Named { name, arguments }]
                        if name == "Int" && arguments.is_empty())
                    && evidence_arguments.is_empty()))
    ));
}

#[test]
fn does_not_specialize_an_imported_generic_instance_for_a_different_head() {
    let source = "\
import { report } from \"./domain\"
pub fn status value: Unit -> String = report 42
";
    let linked = linked_program(
        source,
        [(
            "./domain",
            "fixture/game::domain",
            DOMAIN_WITH_GENERIC_INSTANCE,
        )],
    );
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
    }));
}

#[test]
fn recursively_materializes_imported_factory_constraints() {
    let source = "\
import { report } from \"./domain\"
pub fn status value: Unit -> String = report (Just 42)
";
    let linked = linked_program(
        source,
        [(
            "./domain",
            "fixture/game::domain",
            DOMAIN_WITH_CONSTRAINED_GENERIC_INSTANCE,
        )],
    );
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
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

    let Some(TypedExpr::Call { evidence, .. }) = status else {
        panic!("expected typed status call");
    };
    let [call] = evidence.as_slice() else {
        panic!("expected one Inspect evidence argument");
    };
    let TypedInstanceEvidence::Imported {
        evidence_arguments, ..
    } = &call.evidence
    else {
        panic!("expected imported Inspect evidence");
    };
    let [required] = evidence_arguments.as_slice() else {
        panic!("expected one Ready factory argument");
    };
    assert!(matches!(
        &required.evidence,
        TypedInstanceEvidence::Imported {
            identity,
            provider_module,
            ..
        } if identity == "fixture/game::domain::trait(Ready)<std/prelude::Int>"
            && provider_module == "fixture/game::domain"
    ));
}

#[test]
fn reports_a_missing_imported_factory_constraint() {
    let domain = DOMAIN_WITH_CONSTRAINED_GENERIC_INSTANCE.replace(
        "instance Ready<Int> { fn ready value: Int -> String = \"ready\" }\n",
        "",
    );
    let source = "\
import { report } from \"./domain\"
pub fn status value: Unit -> String = report (Just 42)
";
    let linked = linked_program(
        source,
        [("./domain", "fixture/game::domain", domain.as_str())],
    );
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
    }));
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
        diagnostic.code == "SES-T0201" && diagnostic.message_key == "instance.missing"
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
