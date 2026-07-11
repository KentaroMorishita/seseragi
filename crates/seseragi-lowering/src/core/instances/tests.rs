use crate::{lower_typed_module, CoreInstanceImplementation, CoreType};
use seseragi_semantics::type_module;

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
    assert_eq!(instance.trait_name, "Show");
    assert_eq!(instance.type_identity, "artifact/derived-show::AppError");
    assert_eq!(
        instance.head,
        CoreType::Named {
            name: "AppError".to_owned(),
            arguments: Vec::new(),
        }
    );
    assert!(instance.constraints.is_empty());
    assert_eq!(
        instance.implementation,
        CoreInstanceImplementation::DerivedShow {
            adt_symbol: "artifact/derived-show::AppError".to_owned(),
        }
    );
    assert_eq!(
        &source[instance.origin.start..instance.origin.end],
        "pub type AppError deriving Show =\n  | UnknownHand String\n  | EndOfInput"
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
