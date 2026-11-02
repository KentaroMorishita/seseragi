use super::*;

pub(super) fn unicode_interface() -> ModuleInterface {
    let module = "std/text/unicode";
    let form = || {
        external_type(
            "NormalizationForm",
            "std/text/unicode::NormalizationForm",
            module,
            "NormalizationForm",
            Vec::new(),
        )
    };
    let category = external_type(
        "UnicodeGeneralCategory",
        "std/text/unicode::UnicodeGeneralCategory",
        module,
        "UnicodeGeneralCategory",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "NormalizationForm", []),
        opaque_adt_type_export(module, "UnicodeGeneralCategory", []),
        function_export(
            module,
            "version",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("String"),
        ),
        function_export(
            module,
            "normalize",
            [],
            Vec::new(),
            vec![form(), named("String")],
            named("String"),
        ),
        function_export(
            module,
            "isNormalized",
            [],
            Vec::new(),
            vec![form(), named("String")],
            named("Bool"),
        ),
        function_export(
            module,
            "generalCategory",
            [],
            Vec::new(),
            vec![named("Char")],
            category,
        ),
        function_export(
            module,
            "simpleCaseFold",
            [],
            Vec::new(),
            vec![named("Char")],
            named("Char"),
        ),
        function_export(
            module,
            "fullCaseFold",
            [],
            Vec::new(),
            vec![named("String")],
            named("String"),
        ),
    ];
    for name in ["NFC", "NFD", "NFKC", "NFKD"] {
        exports.push(constructor_export(
            module,
            "NormalizationForm",
            name,
            [],
            None,
        ));
    }
    for name in [
        "UppercaseLetter",
        "LowercaseLetter",
        "TitlecaseLetter",
        "ModifierLetter",
        "OtherLetter",
        "NonspacingMark",
        "SpacingMark",
        "EnclosingMark",
        "DecimalNumber",
        "LetterNumber",
        "OtherNumber",
        "ConnectorPunctuation",
        "DashPunctuation",
        "OpenPunctuation",
        "ClosePunctuation",
        "InitialPunctuation",
        "FinalPunctuation",
        "OtherPunctuation",
        "MathSymbol",
        "CurrencySymbol",
        "ModifierSymbol",
        "OtherSymbol",
        "SpaceSeparator",
        "LineSeparator",
        "ParagraphSeparator",
        "Control",
        "Format",
        "PrivateUse",
        "Unassigned",
    ] {
        exports.push(constructor_export(
            module,
            "UnicodeGeneralCategory",
            name,
            [],
            None,
        ));
    }
    for name in ["isAlphabetic", "isWhitespace", "isDecimalDigit", "isMark"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Char")],
            named("Bool"),
        ));
    }
    standard_interface(module, exports)
}
