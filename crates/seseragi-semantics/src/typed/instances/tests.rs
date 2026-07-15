use crate::{
    type_module, type_module_public_interface, TypedInstanceEvidence, TypedInstanceImplementation,
    TypedType,
};
use seseragi_syntax::InterfaceType;

#[test]
fn selects_non_generic_derived_show_as_typed_evidence() {
    let typed = type_module(
        "artifact/derived-show/main.ssrg",
        "type AppError deriving Show =\n  | StdinFailure StdinError\n  | UnknownHand String\n  | ConsoleFailure ConsoleError\n",
    );

    assert_eq!(typed.instances.len(), 1);
    let instance = &typed.instances[0];
    assert_eq!(instance.identity, "Show<artifact/derived-show::AppError>");
    assert_eq!(instance.trait_name, "Show");
    assert_eq!(instance.arguments, vec![named("AppError")]);
    assert_eq!(
        instance.type_identity.as_deref(),
        Some("artifact/derived-show::AppError")
    );
    assert!(instance.constraints.is_empty());
    assert!(matches!(
        &instance.implementation,
        TypedInstanceImplementation::DerivedShow {
            adt_symbol,
            payload_evidence,
        }
            if adt_symbol == "artifact/derived-show::AppError"
                && payload_evidence.len() == 3
                && payload_evidence.iter().all(|evidence| matches!(
                    evidence.evidence,
                    TypedInstanceEvidence::Standard { .. }
                ))
    ));

    let json = serde_json::to_value(&typed).expect("typed module serializes");
    assert_eq!(json["instances"][0]["trait"], "Show");
    assert_eq!(
        json["instances"][0]["implementation"]["kind"],
        "derived-show"
    );
}

#[test]
fn accepts_payload_with_a_local_derived_show_instance() {
    let typed = type_module(
        "artifact/nested-derived-show/main.ssrg",
        "type Outer deriving Show = | Nested Detail\ntype Detail deriving Show = | Message String\n",
    );

    assert_eq!(typed.instances.len(), 2);
    assert_eq!(
        typed.instances[0].type_identity.as_deref(),
        Some("artifact/nested-derived-show::Outer")
    );
    assert_eq!(
        typed.instances[1].type_identity.as_deref(),
        Some("artifact/nested-derived-show::Detail")
    );
    assert!(matches!(
        &typed.instances[0].implementation,
        TypedInstanceImplementation::DerivedShow {
            payload_evidence,
            ..
        } if matches!(
            payload_evidence.as_slice(),
            [evidence] if matches!(
                &evidence.evidence,
                TypedInstanceEvidence::Local { identity, .. }
                    if identity == "Show<artifact/nested-derived-show::Detail>"
            )
        )
    ));
}

#[test]
fn excludes_derived_show_when_a_payload_instance_is_not_supported() {
    let typed = type_module(
        "artifact/unsupported-derived-show/main.ssrg",
        "type Labels deriving Show = | Labels Array<String>\n",
    );

    assert!(typed.instances.is_empty());
}

#[test]
fn projects_selected_evidence_as_show_adt_interface_instance() {
    let interface = type_module_public_interface(
        "artifact/public-derived-show/main.ssrg",
        "pub type AppError deriving Show = | UnknownHand String\n",
    );

    assert_eq!(interface.instances.len(), 1);
    assert_eq!(
        interface.instances[0].identity.as_deref(),
        Some("Show<artifact/public-derived-show::AppError>")
    );
    assert_eq!(interface.instances[0].trait_name, "Show");
    assert_eq!(
        interface.instances[0].head,
        InterfaceType::Apply {
            constructor: "Show".to_owned(),
            arguments: vec![InterfaceType::Named {
                name: "AppError".to_owned(),
                arguments: Vec::new(),
            }],
        }
    );
    assert!(interface.instances[0].constraints.is_empty());

    let json = serde_json::to_value(&interface).expect("typed interface serializes");
    assert_eq!(
        json["instances"][0]["identity"],
        "Show<artifact/public-derived-show::AppError>"
    );
}

#[test]
fn assigns_distinct_identities_to_same_spelling_in_different_modules() {
    let source = "pub type AppError deriving Show = | Failed String\n";
    let left = type_module_public_interface("fixture/left/main.ssrg", source);
    let right = type_module_public_interface("fixture/right/main.ssrg", source);

    assert_eq!(
        left.instances[0].identity.as_deref(),
        Some("Show<fixture/left::AppError>")
    );
    assert_eq!(
        right.instances[0].identity.as_deref(),
        Some("Show<fixture/right::AppError>")
    );
    assert_ne!(left.instances[0].identity, right.instances[0].identity);
}

#[test]
fn types_user_defined_instance_methods_without_duplicating_the_interface_head() {
    let source = "\
pub type Badge = | Active
pub trait Identity<A> { fn identity value: A -> A }
instance Identity<Badge> { fn identity value: Badge -> Badge = value }
";
    let typed = type_module("artifact/user-instance/main.ssrg", source);

    assert_eq!(typed.instances.len(), 1);
    let instance = &typed.instances[0];
    assert_eq!(
        instance.identity,
        "artifact/user-instance::trait(Identity)<artifact/user-instance::Badge>"
    );
    assert_eq!(instance.arguments, vec![named("Badge")]);
    assert!(instance.type_identity.is_none());
    assert!(matches!(
        &instance.implementation,
        TypedInstanceImplementation::UserDefined { methods }
            if matches!(
                methods.as_slice(),
                [method]
                    if method.name == "identity"
                        && matches!(
                            &method.body,
                            crate::TypedExpr::Variable { name, type_ref, .. }
                                if name == "value" && type_ref == &named("Badge")
                        )
            )
    ));

    let interface = type_module_public_interface("artifact/user-instance/main.ssrg", source);
    assert_eq!(interface.instances.len(), 1);
    assert_eq!(
        interface.instances[0].identity.as_deref(),
        Some("artifact/user-instance::trait(Identity)<artifact/user-instance::Badge>")
    );
}

#[test]
fn alpha_normalizes_generic_instance_binders_in_canonical_identity() {
    let source_for = |binder: &str| {
        format!(
            "trait Identity<A> {{ fn identity value: A -> A }}\n\
             instance<{binder}> Identity<{binder}> {{ fn identity value: {binder} -> {binder} = value }}\n"
        )
    };
    let left = type_module("artifact/generic-instance/main.ssrg", &source_for("T"));
    let right = type_module("artifact/generic-instance/main.ssrg", &source_for("Value"));

    assert_eq!(
        left.instances[0].identity,
        "artifact/generic-instance::trait(Identity)<$0>"
    );
    assert_eq!(left.instances[0].identity, right.instances[0].identity);
    assert_eq!(left.instances[0].type_parameters, vec!["T"]);
    assert_eq!(right.instances[0].type_parameters, vec!["Value"]);
}

#[test]
fn selects_local_instance_evidence_for_a_trait_method_call() {
    let typed = type_module(
        "artifact/trait-method-call/main.ssrg",
        "pub type Badge = | Active\n\
         pub trait Render<A> { fn render value: A -> String }\n\
         instance Render<Badge> { fn render value: Badge -> String = \"active\" }\n\
         pub fn label value: Badge -> String = render value\n",
    );

    let call = typed.declarations.iter().find_map(|declaration| {
        let crate::TypedDecl::Fn { symbol, body, .. } = declaration else {
            return None;
        };
        symbol.ends_with("::label").then_some(body)
    });
    assert!(matches!(
        call,
        Some(crate::TypedExpr::Call {
            trait_dispatch: Some(crate::TypedTraitDispatch { trait_identity, method }),
            evidence,
            ..
        }) if trait_identity == "artifact/trait-method-call::trait(Render)"
            && method == "render"
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Local { identity, .. },
                ..
            }] if identity == "artifact/trait-method-call::trait(Render)<artifact/trait-method-call::Badge>")
    ));
}

#[test]
fn selects_one_same_named_trait_method_from_argument_instance_evidence() {
    let typed = type_module(
        "artifact/trait-method-candidates/main.ssrg",
        "pub type Badge = | Active\n\
         pub type Mode = | Automatic\n\
         pub trait Render<A> { fn present value: A -> String }\n\
         pub trait Describe<A> { fn present value: A -> String }\n\
         instance Render<Badge> { fn present value: Badge -> String = \"active\" }\n\
         instance Describe<Mode> { fn present value: Mode -> String = \"automatic\" }\n\
         pub fn badge value: Badge -> String = present value\n\
         pub fn mode value: Mode -> String = present value\n",
    );

    let calls = typed
        .declarations
        .iter()
        .filter_map(|declaration| {
            let crate::TypedDecl::Fn { symbol, body, .. } = declaration else {
                return None;
            };
            Some((symbol.as_str(), body))
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        calls.as_slice(),
        [(badge, crate::TypedExpr::Call {
            trait_dispatch: Some(crate::TypedTraitDispatch { trait_identity: badge_trait, .. }),
            evidence: badge_evidence,
            ..
        }), (mode, crate::TypedExpr::Call {
            trait_dispatch: Some(crate::TypedTraitDispatch { trait_identity: mode_trait, .. }),
            evidence: mode_evidence,
            ..
        })]
            if badge.ends_with("::badge")
                && mode.ends_with("::mode")
                && badge_trait.ends_with("::trait(Render)")
                && mode_trait.ends_with("::trait(Describe)")
                && matches!(badge_evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Local { identity, .. }, ..
                }] if identity.ends_with("::trait(Render)<artifact/trait-method-candidates::Badge>"))
                && matches!(mode_evidence.as_slice(), [crate::TypedCallEvidence {
                    evidence: TypedInstanceEvidence::Local { identity, .. }, ..
                }] if identity.ends_with("::trait(Describe)<artifact/trait-method-candidates::Mode>"))
    ));
}

#[test]
fn selects_unconstrained_generic_local_instance_with_concrete_type_arguments() {
    let typed = type_module(
        "artifact/generic-instance-dispatch/main.ssrg",
        "pub trait Tag<A> { fn tag value: A -> String }\n\
         instance<T> Tag<Maybe<T>> { fn tag value: Maybe<T> -> String = \"maybe\" }\n\
         pub fn label value: Maybe<Int> -> String = tag value\n",
    );

    let call = typed.declarations.iter().find_map(|declaration| {
        let crate::TypedDecl::Fn { symbol, body, .. } = declaration else {
            return None;
        };
        symbol.ends_with("::label").then_some(body)
    });
    assert!(matches!(
        call,
        Some(crate::TypedExpr::Call {
            trait_dispatch: Some(crate::TypedTraitDispatch { trait_identity, method }),
            evidence,
            ..
        }) if trait_identity == "artifact/generic-instance-dispatch::trait(Tag)"
            && method == "tag"
            && matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                evidence: TypedInstanceEvidence::Local {
                    identity,
                    type_arguments,
                    ..
                },
                ..
            }] if identity == "artifact/generic-instance-dispatch::trait(Tag)<std/prelude::Maybe<$0>>"
                && type_arguments == &[named("Int")])
    ));
}

#[test]
fn selects_required_local_evidence_for_a_constrained_generic_instance() {
    let typed = type_module(
        "artifact/constrained-instance-dispatch/main.ssrg",
        "pub type Badge = | Active\n\
         pub trait Ready<A> { fn ready value: A -> String }\n\
         pub trait Render<A> { fn render value: A -> String }\n\
         instance Ready<Badge> { fn ready value: Badge -> String = \"active\" }\n\
         instance<T> Render<Maybe<T>> where Ready<T> {\n\
           fn render value: Maybe<T> -> String = \"ready\"\n\
         }\n\
         pub fn label value: Maybe<Badge> -> String = render value\n",
    );

    let call = typed.declarations.iter().find_map(|declaration| {
        let crate::TypedDecl::Fn { symbol, body, .. } = declaration else {
            return None;
        };
        symbol.ends_with("::label").then_some(body)
    });
    assert!(matches!(
        call,
        Some(crate::TypedExpr::Call {
            evidence,
            ..
        }) if matches!(evidence.as_slice(), [crate::TypedCallEvidence {
            evidence: TypedInstanceEvidence::Local {
                identity,
                type_arguments,
                evidence_arguments,
            },
            ..
        }] if identity == "artifact/constrained-instance-dispatch::trait(Render)<std/prelude::Maybe<$0>>"
            && type_arguments == &[named("Badge")]
            && matches!(evidence_arguments.as_slice(), [crate::TypedCallEvidence {
                constraint: crate::TypedConstraint { name, arguments },
                evidence: TypedInstanceEvidence::Local {
                    identity: required_identity,
                    ..
                },
            }] if name == "Ready"
                && arguments == &[named("Badge")]
                && required_identity == "artifact/constrained-instance-dispatch::trait(Ready)<artifact/constrained-instance-dispatch::Badge>"))
    ));
}

#[test]
fn uses_instance_constraint_evidence_inside_a_method_body() {
    let typed = type_module(
        "artifact/scoped-instance-evidence/main.ssrg",
        "pub type Badge = | Active\n\
         pub trait Ready<A> { fn ready value: A -> String }\n\
         pub trait Inspect<A> { fn inspect value: A -> String }\n\
         instance Ready<Badge> { fn ready value: Badge -> String = \"active\" }\n\
         instance<T> Inspect<Maybe<T>> where Ready<T> {\n\
           fn inspect value: Maybe<T> -> String =\n\
             match value {\n\
               Nothing -> \"empty\"\n\
               Just item -> ready item\n\
             }\n\
         }\n",
    );

    assert!(matches!(
        &typed.instances[1].implementation,
        TypedInstanceImplementation::UserDefined { methods }
            if matches!(methods.as_slice(), [method]
                if matches!(&method.body, crate::TypedExpr::Match { arms, .. }
                    if matches!(arms.as_slice(), [_, arm]
                        if matches!(&arm.body, crate::TypedExpr::Call { evidence, .. }
                            if matches!(evidence.as_slice(), [crate::TypedCallEvidence {
                                evidence: TypedInstanceEvidence::Parameter { index: 0 },
                                ..
                            }])))))
    ));
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
