use crate::CompileInput;
use seseragi_project::{link_module, standard_module_target};
use seseragi_semantics::{
    analysis_document, analyze_linked_module, analyze_linked_module_recovering,
    diagnostics_only_analysis, AnalysisDocument, AnalyzedModule,
};
use seseragi_syntax::{
    parse_diagnostics, parse_unlinked_module_interface, DiagnosticArtifact, DiagnosticSeverity,
};
use std::collections::BTreeMap;

/// Runs the shared compiler frontend without lowering, code generation, or
/// Effect execution and returns the reusable position-query snapshot.
pub fn analyze_module(input: CompileInput<'_>) -> AnalysisDocument {
    match analyze_module_frontend_with(input, true) {
        Ok(analyzed) => {
            analysis_document(analyzed.diagnostics, analyzed.resolved, &analyzed.typed_hir)
        }
        Err(diagnostics) => {
            diagnostics_only_analysis(input.source_name(), input.module_id(), diagnostics)
        }
    }
}

pub(crate) fn analyze_module_frontend(
    input: CompileInput<'_>,
) -> Result<AnalyzedModule, DiagnosticArtifact> {
    analyze_module_frontend_with(input, false)
}

fn analyze_module_frontend_with(
    input: CompileInput<'_>,
    retain_semantic_recovery: bool,
) -> Result<AnalyzedModule, DiagnosticArtifact> {
    let mut diagnostics = parse_diagnostics(input.source_name(), input.source());
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let unlinked =
        parse_unlinked_module_interface(input.source_name(), input.module_id(), input.source());
    let targets = unlinked
        .imports
        .iter()
        .filter_map(|import| {
            standard_module_target(&import.specifier)
                .map(|target| (import.specifier.clone(), target))
        })
        .collect::<BTreeMap<_, _>>();
    let linked = match link_module(unlinked, &targets) {
        Ok(linked) => linked,
        Err(errors) => {
            crate::dependencies::append_link_diagnostics(errors, &mut diagnostics);
            return Err(diagnostics);
        }
    };
    if retain_semantic_recovery {
        analyze_linked_module_recovering(diagnostics, linked, input.source())
    } else {
        analyze_linked_module(diagnostics, linked, input.source())
    }
}

fn has_errors(diagnostics: &DiagnosticArtifact) -> bool {
    diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_does_not_require_lowering_or_an_entry_point() {
        let source =
            "// 雫\nfn add left: Int -> right: Int -> Int = left + right\nlet addOne = add 1\n";
        let analysis = analyze_module(CompileInput::new("main.ssrg", "analysis/shared", source));

        assert!(analysis.diagnostics.diagnostics.is_empty());
        let add_reference = source.rfind("add 1").unwrap();
        let add_symbol = analysis.symbol_at(add_reference).unwrap();
        assert_eq!(add_symbol.name, "add");
        assert_eq!(
            analysis.type_at(add_reference).unwrap().type_name,
            "Int -> Int -> Int"
        );
        assert!(matches!(
            analysis.type_at(add_reference).unwrap().type_document,
            seseragi_semantics::TypeDocument::Function {
                ref parameters,
                ..
            } if parameters.len() == 2
        ));
        assert_eq!(
            analysis.definition_of(add_reference),
            Some(add_symbol.definition)
        );
        let callable = analysis.callable_at(add_reference).unwrap();
        assert_eq!(callable.parameters.len(), 2);
        assert_eq!(callable.result, "Int");
        assert_eq!(
            callable.multiline_signature,
            "add\n  left: Int\n  -> right: Int\n  -> Int"
        );

        let applied_argument = source.rfind('1').unwrap();
        let partial = analysis.callable_at(applied_argument).unwrap();
        assert_eq!(partial.remaining_parameters.len(), 1);
        assert_eq!(
            partial.remaining_parameters[0].name.as_deref(),
            Some("right")
        );
        assert_eq!(partial.remaining_parameters[0].type_name, "Int");

        let visible = analysis
            .visible_symbols(applied_argument)
            .into_iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>();
        assert!(visible.contains(&"add"));
        assert!(visible.contains(&"addOne"));

        for expected in ["join", "sum", "forEach", "map", "Task"] {
            assert!(analysis
                .standard_library_catalog()
                .iter()
                .any(|item| item.name == expected));
        }
        let task = analysis
            .standard_library_catalog()
            .iter()
            .find(|item| item.name == "Task")
            .expect("Task is available from the Prelude Reference catalog");
        assert_eq!(
            task.signature.as_deref(),
            Some("alias Task<A> = Effect<{}, Never, A>")
        );
        assert!(task
            .multiline_signature
            .as_deref()
            .is_some_and(|signature| signature.contains("Effect<\n")));
        assert_eq!(task.type_parameters, ["A"]);

        let map = analysis
            .standard_library_catalog()
            .iter()
            .find(|item| {
                item.name == "map" && item.constraints.iter().any(|item| item == "Functor<F<_>>")
            })
            .expect("map is available from the Prelude Reference catalog");
        assert!(map
            .multiline_signature
            .as_deref()
            .is_some_and(|signature| signature.contains("\n") && signature.contains("Functor<")));
    }

    #[test]
    fn analysis_exposes_concrete_unannotated_top_level_call_results() {
        let source = "fn wrap<A> value: A -> Maybe<A> = Just value\n\
                      pub let wrapped = wrap 42\n\
                      pub effect fn main = debug wrapped |> println\n";
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/top-level-call-inference",
            source,
        ));

        assert!(analysis.diagnostics.diagnostics.is_empty());
        let wrapped = source.find("wrapped =").expect("wrapped declaration");
        let symbol = analysis.symbol_at(wrapped).expect("wrapped symbol");
        assert_eq!(symbol.name, "wrapped");
        assert_eq!(symbol.type_name.as_deref(), Some("Maybe<Int>"));
        assert_eq!(
            analysis
                .type_at(wrapped)
                .map(|occurrence| occurrence.type_name.as_str()),
            Some("Maybe<Int>")
        );
    }

    #[test]
    fn analysis_exposes_instantiated_missing_record_fields_at_call_sites() {
        let source = r##"import * as dom from "std/web/dom"
import * as html from "std/web/html"

type Mode = | Ready
type Action = | Reset

let initial_mode: Mode = Ready

fn update action: Action -> mode: Mode -> Mode = mode

pub effect fn main =
  dom.app {
    target: "#app",
    initial: initial_mode,
    update
  }
"##;
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/expected-record",
            source,
        ));

        let cursor = source.rfind("  }").expect("app record close") + 2;
        let completion = analysis
            .completion_at(cursor)
            .expect("app record has an expected type");
        assert_eq!(
            completion
                .record_fields
                .iter()
                .map(|field| (field.name.as_str(), field.type_name.as_str()))
                .collect::<Vec<_>>(),
            [("view", "Mode -> Html<Action>")]
        );
    }

    #[test]
    fn analysis_selects_the_innermost_nested_record_expectation() {
        let source = r#"fn accept
  config: { profile: { name: String, score: Int }, enabled: Bool }
  -> Int = 0

let result =
  accept {
    profile: {
      name: "Aki"
    },
    enabled: true
  }
"#;
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/nested-record-completion",
            source,
        ));

        let cursor = source.find("    },").expect("nested record close") + 4;
        let completion = analysis
            .completion_at(cursor)
            .expect("nested record has an expected type");
        assert_eq!(completion.type_name, "{ name: String, score: Int }");
        assert_eq!(completion.record_fields.len(), 1);
        assert_eq!(completion.record_fields[0].name, "score");
        assert_eq!(completion.record_fields[0].type_name, "Int");
    }

    #[test]
    fn analysis_keeps_remaining_record_arguments_after_partial_application() {
        let source = r#"fn configure
  first: { host: String }
  -> second: { secure: Bool, retries: Int }
  -> Int = 0

let with_host = configure { host: "localhost" }
let result = configure { host: "localhost" } { secure: true }
"#;
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/partial-record-completion",
            source,
        ));

        let first_argument = source.find("host: \"").expect("first argument");
        let partial = analysis
            .callable_at(first_argument)
            .expect("partial call remains queryable");
        assert_eq!(partial.remaining_parameters.len(), 1);
        assert_eq!(
            partial.remaining_parameters[0].name.as_deref(),
            Some("second")
        );
        assert_eq!(
            partial.remaining_parameters[0].type_name,
            "{ secure: Bool, retries: Int }"
        );

        let cursor = source.rfind('}').expect("second record close");
        let completion = analysis
            .completion_at(cursor)
            .expect("the second curried argument retains its record expectation");
        assert_eq!(completion.record_fields.len(), 1);
        assert_eq!(completion.record_fields[0].name, "retries");
        assert_eq!(completion.record_fields[0].type_name, "Int");
    }

    #[test]
    fn exposes_document_html_tags_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/html-reference",
            "pub let value = 1\n",
        ));

        for name in [
            "html",
            "head",
            "body",
            "title",
            "meta",
            "link",
            "header",
            "footer",
            "nav",
            "article",
            "aside",
            "h3",
            "h4",
            "h5",
            "h6",
            "strong",
            "em",
            "small",
            "code",
            "pre",
            "blockquote",
            "ul",
            "ol",
            "li",
            "br",
            "hr",
            "a",
            "img",
            "picture",
            "source",
            "video",
            "audio",
            "select",
            "option",
            "fieldset",
            "legend",
            "table",
            "thead",
            "tbody",
            "tfoot",
            "tr",
            "th",
            "td",
            "caption",
            "details",
            "summary",
            "dialog",
        ] {
            let item = analysis
                .standard_library_catalog()
                .iter()
                .find(|item| item.identity == format!("std/web/html::{name}"))
                .unwrap_or_else(|| panic!("missing Reference entry for std/web/html::{name}"));
            assert_eq!(item.module, "std/web/html");
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
        let head = analysis
            .standard_library_catalog()
            .iter()
            .find(|item| item.identity == "std/web/html::head")
            .expect("std/web/html::head is available in Reference");
        assert_eq!(
            head.description,
            "Creates the metadata container for a typed document."
        );
    }

    #[test]
    fn exposes_only_the_safe_numeric_api_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/numeric-reference",
            "pub let value = 1\n",
        ));
        let catalog = analysis.standard_library_catalog();

        for identity in [
            "std/int::minValue",
            "std/int::parseRadix",
            "std/int::checkedAdd",
            "std/int::saturatingPower",
            "std/int::abs",
            "std/big-int::BigInt",
            "std/big-int::parseRadix",
            "std/big-int::toInt",
            "std/big-int::checkedPower",
            "std/decimal::Decimal",
            "std/decimal::DecimalContext",
            "std/decimal::parse",
            "std/decimal::divideExact",
            "std/decimal::quantize",
            "std/decimal::FloatNotFinite",
            "std/float::fromInt",
            "std/float::toInt",
            "std/float::totalCompare",
            "std/number::HalfEven",
            "std/prelude::Ordering",
            "std/prelude::Less",
        ] {
            let item = catalog
                .iter()
                .find(|item| item.identity == identity)
                .unwrap_or_else(|| panic!("missing Reference entry for {identity}"));
            assert_eq!(item.category, "Number");
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
        for removed in [
            "std/int::wrappingAdd",
            "std/int::checkedNegate",
            "std/int::IntDivisionOverflow",
            "std/float::fromIntExact",
        ] {
            assert!(catalog.iter().all(|item| item.identity != removed));
        }
    }

    #[test]
    fn exposes_bytes_and_utf8_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/bytes-reference",
            "pub let value = 1\n",
        ));
        let catalog = analysis.standard_library_catalog();

        for (identity, category) in [
            ("std/bytes::Byte", "Bytes"),
            ("std/bytes::Bytes", "Bytes"),
            ("std/bytes::byte", "Bytes"),
            ("std/bytes::slice", "Bytes"),
            ("std/bytes/hex::HexDecodeError", "Bytes"),
            ("std/bytes/hex::encode", "Bytes"),
            ("std/bytes/hex::decode", "Bytes"),
            ("std/bytes/base64::Base64DecodeError", "Bytes"),
            ("std/bytes/base64::encode", "Bytes"),
            ("std/bytes/base64::decode", "Bytes"),
            ("std/bytes/base64::encodeUrl", "Bytes"),
            ("std/bytes/base64::decodeUrl", "Bytes"),
            ("std/text::Utf8DecodeError", "Text"),
            ("std/text::encodeUtf8", "Text"),
            ("std/text::decodeUtf8", "Text"),
            ("std/text::decodeUtf8Lossy", "Text"),
        ] {
            let item = catalog
                .iter()
                .find(|item| item.identity == identity)
                .unwrap_or_else(|| panic!("missing Reference entry for {identity}"));
            assert_eq!(item.category, category);
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_validation_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/validation-reference",
            "pub let value = 1\n",
        ));
        let catalog = analysis.standard_library_catalog();
        for name in [
            "Validation",
            "Valid",
            "Invalid",
            "valid",
            "invalid",
            "invalidMany",
            "fromEither",
            "toEither",
        ] {
            let item = catalog
                .iter()
                .find(|item| item.identity == format!("std/validation::{name}"))
                .unwrap();
            assert_eq!(item.category, "Validation");
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_regex_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/regex-reference",
            "pub let value = 1\n",
        ));
        let catalog = analysis.standard_library_catalog();
        for name in [
            "Regex",
            "RegexCompileError",
            "RegexOptions",
            "RegexSpan",
            "RegexCapture",
            "RegexMatch",
            "compile",
            "compileWith",
            "find",
            "findAll",
            "replaceAllWith",
            "escape",
        ] {
            let item = catalog
                .iter()
                .find(|item| item.identity == format!("std/regex::{name}"))
                .unwrap_or_else(|| panic!("missing Reference entry for std/regex::{name}"));
            assert_eq!(item.category, "Regex");
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_process_and_non_empty_list_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/process-reference",
            "pub let value = 1\n",
        ));
        let catalog = analysis.standard_library_catalog();

        for identity in [
            "std/non-empty-list::NonEmptyList",
            "std/non-empty-list::singleton",
            "std/process::Process",
            "std/process::ProcessSignal",
            "std/process::ProcessError",
            "std/process::arguments",
            "std/process::environment",
            "std/process::currentDirectory",
            "std/process::signals",
        ] {
            let item = catalog
                .iter()
                .find(|item| item.identity == identity)
                .unwrap_or_else(|| panic!("missing Reference entry for {identity}"));
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_json_core_and_decoders_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/json-reference",
            "pub let value = 1\n",
        ));
        let catalog = analysis.standard_library_catalog();

        for identity in [
            "std/json::Json",
            "std/json::DecodeError",
            "std/json::JsonParseError",
            "std/json::parse",
            "std/json::stringify",
            "std/json::encodeString",
            "std/json::decodeString",
            "std/json::field",
            "std/json::optionalField",
            "std/json::array",
            "std/json::record",
        ] {
            let item = catalog
                .iter()
                .find(|item| item.identity == identity)
                .unwrap_or_else(|| panic!("missing Reference entry for {identity}"));
            assert_eq!(item.category, "JSON");
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_validated_custom_html_values_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/html-custom-reference",
            "pub let value = 1\n",
        ));

        for name in [
            "Tag",
            "Attribute",
            "HtmlBuildError",
            "customTag",
            "attribute",
            "custom",
            "InvalidTagName",
            "InvalidAttributeName",
            "ReservedAttributeName",
        ] {
            let item = analysis
                .standard_library_catalog()
                .iter()
                .find(|item| item.identity == format!("std/web/html::{name}"))
                .unwrap_or_else(|| panic!("missing Reference entry for std/web/html::{name}"));
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_keyboard_snapshot_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/html-keyboard-reference",
            "pub let value = 1\n",
        ));

        let keyboard = analysis
            .standard_library_catalog()
            .iter()
            .find(|item| item.identity == "std/web/html::KeyboardEvent")
            .expect("KeyboardEvent is available in Reference");
        assert_eq!(keyboard.signature.as_deref(), Some("KeyboardEvent"));
        assert!(keyboard.description.contains("modifier keys"));
    }

    #[test]
    fn exposes_pointer_scroll_and_event_controls_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/html-pointer-reference",
            "pub let value = 1\n",
        ));

        for name in [
            "MouseEvent",
            "PointerEvent",
            "ScrollEvent",
            "EventAction",
            "IgnoreEvent",
            "Dispatch",
            "DispatchPreventDefault",
            "DispatchStopPropagation",
            "DispatchPreventDefaultAndStop",
        ] {
            let item = analysis
                .standard_library_catalog()
                .iter()
                .find(|item| item.identity == format!("std/web/html::{name}"))
                .unwrap_or_else(|| panic!("missing Reference entry for std/web/html::{name}"));
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn exposes_html_props_aliases_in_the_reference_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/html-props-reference",
            "pub let value = 1\n",
        ));

        for name in [
            "ElementProps",
            "ButtonProps",
            "FormProps",
            "LabelProps",
            "InputProps",
            "TextareaProps",
            "AnchorProps",
            "ImageProps",
        ] {
            let item = analysis
                .standard_library_catalog()
                .iter()
                .find(|item| item.identity == format!("std/web/html::{name}"))
                .unwrap_or_else(|| panic!("missing Reference entry for std/web/html::{name}"));
            assert_eq!(item.module, "std/web/html");
            assert!(item.signature.is_some());
            assert!(!item.description.is_empty());
        }
    }

    #[test]
    fn invalid_source_still_returns_shared_diagnostics_and_catalog() {
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/invalid",
            "pub let broken: Int =\n",
        ));

        assert!(!analysis.diagnostics.diagnostics.is_empty());
        assert!(analysis.symbols.is_empty());
        assert!(analysis
            .standard_library_catalog()
            .iter()
            .any(|item| item.name == "join"));
    }

    #[test]
    fn imported_member_definition_points_to_its_source_import() {
        let source = concat!(
            "import * as html from \"std/web/html\"\n",
            "let page = html.div { children: \"Hi\" }\n",
        );
        let analysis = analyze_module(CompileInput::new("main.ssrg", "analysis/import", source));
        let member = source.rfind("div").unwrap();
        let definition = analysis.definition_of(member).unwrap();

        assert_eq!(analysis.symbol_at(member).unwrap().name, "div");
        assert!(definition.start < member);
        assert!(definition.end > definition.start);
    }

    #[test]
    fn alias_queries_preserve_the_source_name_and_definition() {
        let source = concat!(
            "alias UserId = Int\n",
            "fn reveal value: UserId -> Int = value\n",
        );
        let analysis = analyze_module(CompileInput::new("main.ssrg", "analysis/alias", source));
        assert!(analysis.diagnostics.diagnostics.is_empty());

        let usage = source.rfind("UserId").unwrap();
        let symbol = analysis.symbol_at(usage).expect("alias usage is queryable");
        assert_eq!(symbol.name, "UserId");
        assert_eq!(symbol.kind, "type");
        assert_eq!(symbol.type_name.as_deref(), Some("Int"));
        assert_eq!(analysis.definition_of(usage), Some(symbol.definition));
        assert_eq!(
            &source[symbol.definition.start..symbol.definition.end],
            "UserId"
        );

        let visible = analysis
            .visible_symbols(source.len())
            .into_iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>();
        assert!(visible.contains(&"UserId"));
    }

    #[test]
    fn alias_hover_preserves_higher_kinded_parameter_structure() {
        let source = concat!(
            "alias StateT<S, M<_>, A> = S -> M<(A, S)>\n",
            "fn use value: StateT<Int, Maybe, String> -> Int = 0\n",
        );
        let analysis = analyze_module(CompileInput::new(
            "main.ssrg",
            "analysis/higher-kinded-alias",
            source,
        ));
        assert!(analysis.diagnostics.diagnostics.is_empty());

        let usage = source.rfind("StateT").unwrap();
        let symbol = analysis.symbol_at(usage).expect("alias usage is queryable");
        let seseragi_semantics::TypeDocument::Function { parameters, result } =
            symbol.type_document.as_ref().expect("alias type document")
        else {
            panic!("expected alias function target, received {symbol:#?}");
        };
        assert!(
            matches!(parameters.as_slice(), [seseragi_semantics::TypeDocument::Variable {
            name,
            arity: 0,
            arguments,
        }] if name == "S" && arguments.is_empty())
        );
        assert!(
            matches!(result.as_ref(), seseragi_semantics::TypeDocument::Variable {
            name,
            arity: 1,
            arguments,
        } if name == "M" && arguments.len() == 1)
        );
    }
}
