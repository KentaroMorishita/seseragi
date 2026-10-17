use crate::{semantic_diagnostics, type_module, TypedInstanceEvidence};

#[test]
fn alias_head_uses_the_expanded_owner_for_registration_and_call_evidence() {
    let typed = type_module(
        "artifact/alias-head/main.ssrg",
        "type Label = | Label\n\
         alias Name = Label\n\
         instance Show<Name> { fn show value: Label -> String = \"label\" }\n\
         pub fn label value: Label -> String = show value\n",
    );
    assert_eq!(typed.instances.len(), 1);
    assert_eq!(
        typed.instances[0].identity,
        "std/prelude::Show<artifact/alias-head::Label>"
    );
    assert_eq!(
        typed.instances[0].argument_identities,
        ["artifact/alias-head::Label"]
    );
    let evidence = typed
        .declarations
        .iter()
        .find_map(|declaration| {
            let crate::TypedDecl::Fn {
                body: crate::TypedExpr::Call { evidence, .. },
                ..
            } = declaration
            else {
                return None;
            };
            evidence.first()
        })
        .expect("selected Show evidence");
    assert!(
        matches!(&evidence.evidence, TypedInstanceEvidence::Local { identity, .. }
        if identity == &typed.instances[0].identity)
    );
}

#[test]
fn alias_head_interface_keeps_the_provider_owner_and_parameter_order() {
    let interface = crate::type_module_public_interface(
        "artifact/alias-interface/main.ssrg",
        "pub type Pair<A, B> = | Pair (A, B)\nalias Flipped<A, B> = Pair<B, A>\ninstance<A, B> Show<Flipped<A, B>> where Show<A>, Show<B> { fn show value: Pair<B, A> -> String = \"pair\" }\n",
    );
    let instance = &interface.instances[0];
    assert_eq!(
        instance.identity.as_deref(),
        Some("std/prelude::Show<artifact/alias-interface::Pair<$1,$0>>")
    );
    let seseragi_syntax::InterfaceType::Apply { arguments, .. } = &instance.head else {
        panic!("instance application")
    };
    assert!(
        matches!(arguments.as_slice(), [seseragi_syntax::InterfaceType::ExternalNamed {
        canonical, provider_module, provider_export, arguments, ..
    }] if canonical == "artifact/alias-interface::Pair"
        && provider_module == "artifact/alias-interface"
        && provider_export == "Pair"
        && arguments == &[seseragi_syntax::InterfaceType::Named { name: "B".into(), arguments: vec![] }, seseragi_syntax::InterfaceType::Named { name: "A".into(), arguments: vec![] }])
    );
}

#[test]
fn alias_head_rejects_duplicates_even_when_never_called() {
    for (alias, target) in [("Name", "Label"), ("Wrapper<A>", "Box<A>")] {
        let source = if alias == "Name" {
            "type Label = | Label\nalias Name = Label\ninstance Show<Name> { fn show value: Label -> String = \"alias\" }\ninstance Show<Label> { fn show value: Label -> String = \"label\" }\n".to_owned()
        } else {
            format!("type Box<A> = | Box A\nalias {alias} = {target}\ninstance<A> Show<{alias}> {{ fn show value: {target} -> String = \"alias\" }}\ninstance<B> Show<Box<B>> {{ fn show value: Box<B> -> String = \"box\" }}\n")
        };
        let diagnostics = semantic_diagnostics("artifact/alias-duplicate/main.ssrg", &source);
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-T0202" && d.message_key == "trait.instance-duplicate"),
            "{source}\n{:#?}",
            diagnostics.diagnostics
        );
    }
}

#[test]
fn alias_head_cannot_replace_a_standard_instance() {
    for source in [
        "alias Number = Int\ninstance Show<Number> { fn show value: Int -> String = \"number\" }\n",
        "alias Numbers<A> = Array<A>\ninstance<A> Show<Numbers<A>> { fn show value: Array<A> -> String = \"numbers\" }\n",
    ] {
        let diagnostics = semantic_diagnostics("artifact/alias-standard/main.ssrg", source);
        assert!(diagnostics.diagnostics.iter().any(|d|
            d.code == "SES-T0202" && d.message_key == "trait.instance-duplicate"
        ), "{source}\n{:#?}", diagnostics.diagnostics);
    }
}

#[test]
fn alias_head_preserves_nominal_and_shadowed_primitive_owners() {
    for nominal in ["Int", "Array", "Float", "Bool", "Char"] {
        let source = format!("type {nominal} = | Value\nalias Name = {nominal}\ninstance Show<Name> {{ fn show value: {nominal} -> String = \"local\" }}\npub fn label value: {nominal} -> String = show value\n");
        let diagnostics = semantic_diagnostics("artifact/alias-owner/main.ssrg", &source);
        assert!(
            diagnostics.diagnostics.is_empty(),
            "{source}\n{:#?}",
            diagnostics.diagnostics
        );
        let typed = type_module("artifact/alias-owner/main.ssrg", &source);
        assert_eq!(
            typed.instances[0].argument_identities,
            [format!("artifact/alias-owner::{nominal}")]
        );
    }
}

#[test]
fn alias_head_rejects_an_unrepresented_constructor_before_it_is_used() {
    let source = "type Pair<A, B> = | Pair (A, B)\nalias Repeated<A> = Pair<A, A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Repeated> { fn tag<A> value: Pair<A, A> -> String = \"pair\" }\n";
    let diagnostics = semantic_diagnostics("artifact/alias-constructor/main.ssrg", source);
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|d| d.message_key == "trait.instance-head-unresolved"),
        "{diagnostics:#?}"
    );
}

#[test]
fn alias_head_cannot_erase_an_instance_parameter_or_confuse_a_same_named_nominal() {
    for source in [
        "type Label = | Label\nalias Erased<A> = Label\ninstance<A> Show<Erased<A>> { fn show value: Label -> String = \"label\" }\n",
        "type A = | Label\nalias Erased<T> = A\ninstance<A> Show<Erased<A>> { fn show value: Erased<A> -> String = \"label\" }\n",
    ] {
        let diagnostics = semantic_diagnostics("artifact/alias-erased/main.ssrg", source);
        assert!(diagnostics.diagnostics.iter().any(|d| d.message_key == "trait.instance-head-unresolved"), "{diagnostics:#?}");
    }
}

#[test]
fn alias_head_binary_inference_preserves_nominal_results_and_saved_operator_arity() {
    let source = "newtype Count = Int\nalias Number = Count\ninstance Add<Number, Number, Number> { fn add left: Count -> right: Count -> Count = match (left, right) { (Count a, Count b) -> Count (a + b) } }\nfn unbox value: Count -> Int = match value { Count n -> n }\npub fn probe -> (Int, Int) = { let plus: Count -> Count -> Count = (+); let saved = plus (Count 20); (unbox ((Count 20) + (Count 22)), unbox (saved (Count 22))) }\n";
    let diagnostics = semantic_diagnostics("artifact/alias-arithmetic/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    let typed = type_module("artifact/alias-arithmetic/main.ssrg", source);
    assert_eq!(
        typed.instances[0].argument_identities,
        ["artifact/alias-arithmetic::Count"; 3]
    );
}
