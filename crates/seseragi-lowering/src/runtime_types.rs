#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeTypeImport {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
}

const RUNTIME_TYPE_IMPORTS: &[RuntimeTypeImport] = &[
    RuntimeTypeImport {
        canonical: "std/prelude::Show",
        runtime_feature: "core.show.dictionary",
        module: "@seseragi/runtime/show",
        export_name: "Show",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Console",
        runtime_feature: "effect.console.service",
        module: "@seseragi/runtime/console",
        export_name: "Console",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::ConsoleError",
        runtime_feature: "effect.console.error",
        module: "@seseragi/runtime/console",
        export_name: "ConsoleError",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Stdin",
        runtime_feature: "effect.stdin.service",
        module: "@seseragi/runtime/stdin",
        export_name: "Stdin",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::StdinError",
        runtime_feature: "effect.stdin.error",
        module: "@seseragi/runtime/stdin",
        export_name: "StdinError",
    },
];

pub(crate) fn runtime_type_import(canonical: &str) -> Option<RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS
        .iter()
        .copied()
        .find(|type_import| type_import.canonical == canonical)
}

pub(crate) fn runtime_type_import_for_feature(feature: &str) -> Option<RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS
        .iter()
        .copied()
        .find(|type_import| type_import.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_runtime_types_by_canonical_language_identity() {
        let type_import = runtime_type_import("std/prelude::StdinError").unwrap();

        assert_eq!(type_import.runtime_feature, "effect.stdin.error");
        assert_eq!(type_import.module, "@seseragi/runtime/stdin");
        assert_eq!(type_import.export_name, "StdinError");
    }

    #[test]
    fn does_not_treat_local_spelling_as_a_runtime_type_identity() {
        assert!(runtime_type_import("StdinError").is_none());
        assert!(runtime_type_import("artifact/domain::StdinError").is_none());
    }

    #[test]
    fn resolves_show_dictionary_type_by_identity_and_feature() {
        let expected = RuntimeTypeImport {
            canonical: "std/prelude::Show",
            runtime_feature: "core.show.dictionary",
            module: "@seseragi/runtime/show",
            export_name: "Show",
        };

        assert_eq!(runtime_type_import("std/prelude::Show"), Some(expected));
        assert_eq!(
            runtime_type_import_for_feature("core.show.dictionary"),
            Some(expected)
        );
    }
}
