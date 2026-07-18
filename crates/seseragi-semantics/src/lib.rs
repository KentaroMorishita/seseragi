mod analysis;
mod diagnostics;
mod effect_ops;
mod instance_identity;
mod model;
mod prelude;
mod resolve;
mod typed;

pub use analysis::{analyze_linked_module, analyze_module_interface, AnalyzedModule};
pub use diagnostics::semantic_diagnostics;
pub use effect_ops::{known_effect_operation_by_semantic, KnownEffectOperation};
pub use model::*;
pub use prelude::{
    standard_equality_instance_by_identity, standard_prelude_surface, StandardEqualityInstance,
    StandardModuleSurface,
};
pub use resolve::{resolve_linked_module, resolve_module, resolve_module_interface};
pub use typed::{type_module, type_module_interface, type_module_public_interface};
