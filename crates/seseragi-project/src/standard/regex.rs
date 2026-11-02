use super::*;

pub(super) fn regex_interface() -> ModuleInterface {
    let module = "std/regex";
    let regex = external_type("Regex", "std/regex::Regex", module, "Regex", Vec::new());
    let error_kind = external_type(
        "RegexCompileErrorKind",
        "std/regex::RegexCompileErrorKind",
        module,
        "RegexCompileErrorKind",
        Vec::new(),
    );
    let error = external_type(
        "RegexCompileError",
        "std/regex::RegexCompileError",
        module,
        "RegexCompileError",
        Vec::new(),
    );
    let options = external_type(
        "RegexOptions",
        "std/regex::RegexOptions",
        module,
        "RegexOptions",
        Vec::new(),
    );
    let span = external_type(
        "RegexSpan",
        "std/regex::RegexSpan",
        module,
        "RegexSpan",
        Vec::new(),
    );
    let capture = external_type(
        "RegexCapture",
        "std/regex::RegexCapture",
        module,
        "RegexCapture",
        Vec::new(),
    );
    let matched = external_type(
        "RegexMatch",
        "std/regex::RegexMatch",
        module,
        "RegexMatch",
        Vec::new(),
    );
    let maybe_capture = named_with("Maybe", vec![capture.clone()]);
    let named_captures = external_type(
        "Map",
        "std/map::Map",
        "std/map",
        "Map",
        vec![named("String"), maybe_capture.clone()],
    );
    standard_interface(
        module,
        vec![
            type_export(module, "Regex", 0, "opaque-type"),
            opaque_adt_type_export(module, "RegexCompileErrorKind", []),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "UnexpectedRegexEnd",
                [],
                None,
            ),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "UnexpectedRegexToken",
                [],
                Some(named("Char")),
            ),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "InvalidRegexEscape",
                [],
                None,
            ),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "InvalidRegexRange",
                [],
                None,
            ),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "InvalidRegexQuantifier",
                [],
                None,
            ),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "DuplicateCaptureName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "RegexCompileErrorKind",
                "UnsupportedRegexFeature",
                [],
                Some(named("String")),
            ),
            public_record_type_export(
                module,
                "RegexCompileError",
                [
                    required("kind", error_kind),
                    required("offset", named("Int")),
                ],
            ),
            public_record_type_export(
                module,
                "RegexOptions",
                [
                    required("caseInsensitive", named("Bool")),
                    required("multiline", named("Bool")),
                    required("dotMatchesNewline", named("Bool")),
                ],
            ),
            public_record_type_export(
                module,
                "RegexSpan",
                [
                    required("start", named("Int")),
                    required("end", named("Int")),
                ],
            ),
            public_record_type_export(
                module,
                "RegexCapture",
                [
                    required("span", span.clone()),
                    required("text", named("String")),
                ],
            ),
            public_record_type_export(
                module,
                "RegexMatch",
                [
                    required("span", span),
                    required("text", named("String")),
                    required("captures", named_with("Array", vec![maybe_capture])),
                    required("named", named_captures),
                ],
            ),
            function_export(
                module,
                "defaultOptions",
                [],
                Vec::new(),
                vec![named("Unit")],
                options.clone(),
            ),
            function_export(
                module,
                "compile",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Either", vec![error.clone(), regex.clone()]),
            ),
            function_export(
                module,
                "compileWith",
                [],
                Vec::new(),
                vec![options, named("String")],
                named_with("Either", vec![error, regex.clone()]),
            ),
            function_export(
                module,
                "isMatch",
                [],
                Vec::new(),
                vec![regex.clone(), named("String")],
                named("Bool"),
            ),
            function_export(
                module,
                "find",
                [],
                Vec::new(),
                vec![regex.clone(), named("String")],
                named_with("Maybe", vec![matched.clone()]),
            ),
            function_export(
                module,
                "findAll",
                [],
                Vec::new(),
                vec![regex.clone(), named("String")],
                named_with("Array", vec![matched.clone()]),
            ),
            function_export(
                module,
                "split",
                [],
                Vec::new(),
                vec![regex.clone(), named("String")],
                named_with("Array", vec![named("String")]),
            ),
            function_export(
                module,
                "replaceAll",
                [],
                Vec::new(),
                vec![regex.clone(), named("String"), named("String")],
                named("String"),
            ),
            function_export(
                module,
                "replaceAllWith",
                [],
                Vec::new(),
                vec![
                    regex,
                    function_type(vec![matched], named("String")),
                    named("String"),
                ],
                named("String"),
            ),
            function_export(
                module,
                "escape",
                [],
                Vec::new(),
                vec![named("String")],
                named("String"),
            ),
        ],
    )
}
