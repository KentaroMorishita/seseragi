use crate::runtime_modules::RuntimeDataOperation;

macro_rules! operation {
    ($canonical:literal, $feature:literal, $name:literal) => {
        RuntimeDataOperation {
            canonical: $canonical,
            runtime_feature: $feature,
            local_name: concat!("_ssrg_regex_", $name),
            module: "@seseragi/runtime/regex",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

const OPERATIONS: &[RuntimeDataOperation] = &[
    operation!(
        "std/regex::UnexpectedRegexEnd",
        "core.regex.unexpected-end",
        "UnexpectedRegexEnd"
    ),
    operation!(
        "std/regex::UnexpectedRegexToken",
        "core.regex.unexpected-token",
        "UnexpectedRegexToken"
    ),
    operation!(
        "std/regex::InvalidRegexEscape",
        "core.regex.invalid-escape",
        "InvalidRegexEscape"
    ),
    operation!(
        "std/regex::InvalidRegexRange",
        "core.regex.invalid-range",
        "InvalidRegexRange"
    ),
    operation!(
        "std/regex::InvalidRegexQuantifier",
        "core.regex.invalid-quantifier",
        "InvalidRegexQuantifier"
    ),
    operation!(
        "std/regex::DuplicateCaptureName",
        "core.regex.duplicate-capture-name",
        "DuplicateCaptureName"
    ),
    operation!(
        "std/regex::UnsupportedRegexFeature",
        "core.regex.unsupported-feature",
        "UnsupportedRegexFeature"
    ),
    operation!(
        "std/regex::defaultOptions",
        "core.regex.default-options",
        "defaultOptions"
    ),
    operation!("std/regex::compile", "core.regex.compile", "compile"),
    operation!(
        "std/regex::compileWith",
        "core.regex.compile-with",
        "compileWith"
    ),
    operation!("std/regex::isMatch", "core.regex.is-match", "isMatch"),
    operation!("std/regex::find", "core.regex.find", "find"),
    operation!("std/regex::findAll", "core.regex.find-all", "findAll"),
    operation!("std/regex::split", "core.regex.split", "split"),
    operation!(
        "std/regex::replaceAll",
        "core.regex.replace-all",
        "replaceAll"
    ),
    operation!(
        "std/regex::replaceAllWith",
        "core.regex.replace-all-with",
        "replaceAllWith"
    ),
    operation!("std/regex::escape", "core.regex.escape", "escape"),
];

pub(crate) fn runtime_regex_operation(canonical: &str) -> Option<RuntimeDataOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_regex_operation_for_feature(feature: &str) -> Option<RuntimeDataOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_regex_operations_only_by_canonical_identity() {
        for operation in OPERATIONS {
            assert_eq!(
                runtime_regex_operation(operation.canonical),
                Some(*operation)
            );
            assert_eq!(
                runtime_regex_operation_for_feature(operation.runtime_feature),
                Some(*operation)
            );
        }
        assert!(runtime_regex_operation("app/regex::compile").is_none());
    }
}
