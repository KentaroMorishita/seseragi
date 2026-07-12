use super::TypedModuleInterface;
use seseragi_syntax::ModuleInterface;

impl TypedModuleInterface {
    /// Converts the final typed public contract into the body-free shape used
    /// by project linking. The `typed-interface` stage marker is deliberately
    /// not part of a dependency's semantic identity.
    pub fn into_link_interface(self) -> ModuleInterface {
        ModuleInterface {
            schema: self.schema,
            module: self.module,
            source: self.source,
            dependencies: self.dependencies,
            exports: self.exports,
            operators: self.operators,
            instances: self.instances,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ResolvedInterfaceDecl;
    use seseragi_syntax::InterfaceType;

    #[test]
    fn preserves_body_inferred_public_contracts_for_project_linking() {
        let typed = crate::type_module_public_interface(
            "artifact/effect-compact-public/main.ssrg",
            "pub effect fn greet name: String =\n  println \"hello\"\n",
        );

        let linked = typed.into_link_interface();
        let greet = linked
            .exports
            .iter()
            .find(|export| export.name == "greet")
            .unwrap();
        assert_eq!(greet.declaration_kind.as_deref(), Some("effect-function"));
        assert!(matches!(
            &greet.scheme.type_ref,
            InterfaceType::Function { result, .. }
                if matches!(result.as_ref(), InterfaceType::Named { name, arguments }
                    if name == "Effect" && arguments.len() == 3)
        ));
    }

    #[test]
    fn preserves_final_instance_identity_through_linking_and_resolution() {
        let typed = crate::type_module_public_interface(
            "fixture/errors/main.ssrg",
            "pub type AppError deriving Show = | Failed String\n",
        );

        let linked = typed.into_link_interface();
        assert_eq!(
            linked.instances[0].identity.as_deref(),
            Some("Show<fixture/errors::AppError>")
        );

        let resolved = crate::resolve_module_interface(linked);
        let instance = resolved
            .declarations
            .iter()
            .find(|declaration| matches!(declaration, ResolvedInterfaceDecl::Instance { .. }))
            .expect("resolved interface contains its typed instance");
        assert!(matches!(
            instance,
            ResolvedInterfaceDecl::Instance {
                identity: Some(identity),
                trait_name,
                ..
            } if identity == "Show<fixture/errors::AppError>" && trait_name == "Show"
        ));
    }
}
