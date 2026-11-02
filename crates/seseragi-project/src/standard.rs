mod array;
mod base64;
mod benchmark;
mod big_int;
mod bytes;
mod char;
mod child_process;
mod clock;
mod collection_core;
mod console;
mod decimal;
mod deferred;
mod effect;
mod either;
mod entropy;
mod filesystem;
mod float;
mod grapheme;
mod hex;
mod http_client;
mod http_multipart;
mod http_server;
mod int;
mod iterator;
mod json;
mod list;
mod logger;
mod map;
mod math;
mod maybe;
mod non_empty_list;
mod number;
mod path;
mod process;
mod queue;
mod random;
mod r#ref;
mod regex;
mod semaphore;
mod set;
mod signal;
mod sse;
mod stdin;
mod stream;
mod test;
mod text;
mod time;
mod transformer;
mod unicode;
mod validation;
mod web_dom;
mod web_file;
mod web_html;
mod web_navigation;
mod web_storage;
mod web_svg;
mod websocket_client;
mod websocket_server;
use crate::ModuleLinkTarget;
use serde::Serialize;
use seseragi_syntax::{
    ByteSpan, InterfaceConstraint, InterfaceExport, InterfaceRecordField, InterfaceScheme,
    InterfaceType, ModuleInterface, TypeParameter, Visibility,
};

const ORIGIN: ByteSpan = ByteSpan { start: 0, end: 0 };

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardHtmlTagKind {
    Element,
    VoidElement,
    Meta,
    Link,
    Anchor,
    Image,
    Source,
    Video,
    Audio,
    Button,
    Form,
    Label,
    Input,
    Textarea,
    Select,
    Option,
    TableCell,
    OpenElement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardHtmlTag {
    pub name: &'static str,
    pub kind: StandardHtmlTagKind,
    pub void_element: bool,
}

macro_rules! html_tag {
    ($name:literal, $kind:ident) => {
        StandardHtmlTag {
            name: $name,
            kind: StandardHtmlTagKind::$kind,
            void_element: false,
        }
    };
    ($name:literal, $kind:ident, void) => {
        StandardHtmlTag {
            name: $name,
            kind: StandardHtmlTagKind::$kind,
            void_element: true,
        }
    };
}

pub const STANDARD_HTML_TAGS: &[StandardHtmlTag] = &[
    html_tag!("html", Element),
    html_tag!("head", Element),
    html_tag!("body", Element),
    html_tag!("title", Element),
    html_tag!("meta", Meta, void),
    html_tag!("link", Link, void),
    html_tag!("header", Element),
    html_tag!("footer", Element),
    html_tag!("nav", Element),
    html_tag!("article", Element),
    html_tag!("aside", Element),
    html_tag!("div", Element),
    html_tag!("span", Element),
    html_tag!("p", Element),
    html_tag!("main", Element),
    html_tag!("section", Element),
    html_tag!("h1", Element),
    html_tag!("h2", Element),
    html_tag!("h3", Element),
    html_tag!("h4", Element),
    html_tag!("h5", Element),
    html_tag!("h6", Element),
    html_tag!("strong", Element),
    html_tag!("em", Element),
    html_tag!("small", Element),
    html_tag!("code", Element),
    html_tag!("pre", Element),
    html_tag!("blockquote", Element),
    html_tag!("ul", Element),
    html_tag!("ol", Element),
    html_tag!("li", Element),
    html_tag!("br", VoidElement, void),
    html_tag!("hr", VoidElement, void),
    html_tag!("a", Anchor),
    html_tag!("img", Image, void),
    html_tag!("picture", Element),
    html_tag!("source", Source, void),
    html_tag!("video", Video),
    html_tag!("audio", Audio),
    html_tag!("button", Button),
    html_tag!("form", Form),
    html_tag!("label", Label),
    html_tag!("input", Input, void),
    html_tag!("textarea", Textarea),
    html_tag!("select", Select),
    html_tag!("option", Option),
    html_tag!("fieldset", Element),
    html_tag!("legend", Element),
    html_tag!("table", Element),
    html_tag!("thead", Element),
    html_tag!("tbody", Element),
    html_tag!("tfoot", Element),
    html_tag!("tr", Element),
    html_tag!("th", TableCell),
    html_tag!("td", TableCell),
    html_tag!("caption", Element),
    html_tag!("details", OpenElement),
    html_tag!("summary", Element),
    html_tag!("dialog", OpenElement),
];

pub fn standard_html_tag(name: &str) -> Option<StandardHtmlTag> {
    STANDARD_HTML_TAGS
        .iter()
        .copied()
        .find(|tag| tag.name == name)
}

pub fn standard_html_tag_props(name: &str) -> Option<(StandardHtmlTag, Vec<InterfaceRecordField>)> {
    let tag = standard_html_tag(name)?;
    let InterfaceType::Record { fields, .. } = props_for_html_tag(tag) else {
        unreachable!("standard HTML tag props must be a record")
    };
    Some((tag, fields))
}

pub fn is_standard_void_html_tag(name: &str) -> bool {
    standard_html_tag(name).is_some_and(|tag| tag.void_element)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StandardModuleStatus {
    Available,
    ContractOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleRegistrySurface {
    pub schema: u32,
    pub kind: &'static str,
    pub language_version: &'static str,
    pub prelude: StandardPreludeBoundary,
    pub modules: Vec<StandardModuleSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardPreludeBoundary {
    pub specifier: &'static str,
    pub availability: &'static str,
    pub registry: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleSurface {
    pub specifier: &'static str,
    pub identity: &'static str,
    pub status: StandardModuleStatus,
    pub targets: &'static [&'static str],
    #[serde(default, skip_serializing_if = "<[&str]>::is_empty")]
    pub capability_services: &'static [&'static str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_interface: Option<ModuleInterface>,
}

struct StandardModuleDefinition {
    specifier: &'static str,
    status: StandardModuleStatus,
    targets: &'static [&'static str],
    capability_services: &'static [&'static str],
    interface: Option<fn() -> ModuleInterface>,
}

const PORTABLE_TARGETS: &[&str] = &["process", "browser"];
const PROCESS_TARGET: &[&str] = &["process"];
const BROWSER_TARGET: &[&str] = &["browser"];

macro_rules! available_module {
    ($specifier:literal, $interface:path, $targets:expr) => {
        StandardModuleDefinition {
            specifier: $specifier,
            status: StandardModuleStatus::Available,
            targets: $targets,
            capability_services: &[],
            interface: Some($interface),
        }
    };
    ($specifier:literal, $interface:path, $targets:expr, $services:expr) => {
        StandardModuleDefinition {
            specifier: $specifier,
            status: StandardModuleStatus::Available,
            targets: $targets,
            capability_services: $services,
            interface: Some($interface),
        }
    };
}

macro_rules! contract_module {
    ($specifier:literal, $targets:expr) => {
        StandardModuleDefinition {
            specifier: $specifier,
            status: StandardModuleStatus::ContractOnly,
            targets: $targets,
            capability_services: &[],
            interface: None,
        }
    };
    ($specifier:literal, $targets:expr, $services:expr) => {
        StandardModuleDefinition {
            specifier: $specifier,
            status: StandardModuleStatus::ContractOnly,
            targets: $targets,
            capability_services: $services,
            interface: None,
        }
    };
}

const STANDARD_MODULES: &[StandardModuleDefinition] = &[
    available_module!("std/number", number::number_interface, PORTABLE_TARGETS),
    available_module!("std/int", int::int_interface, PORTABLE_TARGETS),
    available_module!("std/float", float::float_interface, PORTABLE_TARGETS),
    available_module!("std/math", math::math_interface, PORTABLE_TARGETS),
    available_module!("std/array", array::array_interface, PORTABLE_TARGETS),
    available_module!("std/list", list::list_interface, PORTABLE_TARGETS),
    available_module!(
        "std/web/html",
        web_html::web_html_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/web/svg", web_svg::web_svg_interface, PORTABLE_TARGETS),
    available_module!("std/web/file", web_file::web_file_interface, BROWSER_TARGET),
    available_module!(
        "std/web/navigation",
        web_navigation::web_navigation_interface,
        BROWSER_TARGET,
        &["std/web/navigation::Navigation"]
    ),
    available_module!(
        "std/web/storage",
        web_storage::web_storage_interface,
        BROWSER_TARGET,
        &["std/web/storage::Storage"]
    ),
    available_module!(
        "std/web/dom",
        web_dom::web_dom_interface,
        BROWSER_TARGET,
        &["std/web/dom::Dom"]
    ),
    available_module!("std/signal", signal::signal_interface, PORTABLE_TARGETS),
    available_module!(
        "std/clock",
        clock::clock_interface,
        PORTABLE_TARGETS,
        &["std/clock::Clock"]
    ),
    available_module!(
        "std/time",
        time::time_interface,
        PORTABLE_TARGETS,
        &["std/time::TimeZones"]
    ),
    available_module!(
        "std/http",
        http_client::http_client_interface,
        PORTABLE_TARGETS,
        &["std/http::HttpClient"]
    ),
    available_module!(
        "std/http/server",
        http_server::http_server_interface,
        PROCESS_TARGET,
        &["std/http/server::HttpServer"]
    ),
    available_module!(
        "std/http/multipart",
        http_multipart::http_multipart_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/sse", sse::sse_interface, PORTABLE_TARGETS),
    available_module!(
        "std/websocket",
        websocket_client::websocket_client_interface,
        PORTABLE_TARGETS,
        &["std/websocket::WebSocketClient"]
    ),
    available_module!(
        "std/websocket/server",
        websocket_server::websocket_server_interface,
        PROCESS_TARGET,
        &["std/websocket/server::WebSocketServer"]
    ),
    available_module!(
        "std/benchmark",
        benchmark::benchmark_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/big-int", big_int::big_int_interface, PORTABLE_TARGETS),
    available_module!("std/bytes", bytes::bytes_interface, PORTABLE_TARGETS),
    available_module!(
        "std/bytes/base64",
        base64::base64_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/bytes/hex", hex::hex_interface, PORTABLE_TARGETS),
    available_module!("std/char", char::char_interface, PORTABLE_TARGETS),
    available_module!(
        "std/child-process",
        child_process::child_process_interface,
        PROCESS_TARGET,
        &["std/child-process::ChildProcesses"]
    ),
    available_module!(
        "std/collection",
        collection_core::collection_core_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/console",
        console::console_interface,
        PORTABLE_TARGETS,
        &["std/prelude::Console"]
    ),
    available_module!("std/decimal", decimal::decimal_interface, PORTABLE_TARGETS),
    available_module!(
        "std/deferred",
        deferred::deferred_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/effect", effect::effect_interface, PORTABLE_TARGETS),
    available_module!("std/either", either::either_interface, PORTABLE_TARGETS),
    available_module!(
        "std/entropy",
        entropy::entropy_interface,
        PORTABLE_TARGETS,
        &["std/entropy::Entropy"]
    ),
    available_module!(
        "std/fs",
        filesystem::filesystem_interface,
        PORTABLE_TARGETS,
        &["std/fs::FileSystem"]
    ),
    contract_module!(
        "std/http/bun",
        PROCESS_TARGET,
        &["std/http/bun::BunHttpServer"]
    ),
    available_module!(
        "std/iterator",
        iterator::iterator_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/transformer/maybe",
        transformer::maybe_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/transformer/either",
        transformer::either_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/transformer/reader",
        transformer::reader_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/transformer/state",
        transformer::state_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/transformer/writer",
        transformer::writer_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/json", json::json_interface, PORTABLE_TARGETS),
    available_module!(
        "std/log",
        logger::logger_interface,
        PORTABLE_TARGETS,
        &["std/log::Logger"]
    ),
    available_module!("std/map", map::map_interface, PORTABLE_TARGETS),
    available_module!("std/maybe", maybe::maybe_interface, PORTABLE_TARGETS),
    available_module!(
        "std/non-empty-list",
        non_empty_list::non_empty_list_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/path", path::path_interface, PORTABLE_TARGETS),
    available_module!(
        "std/process",
        process::process_interface,
        PROCESS_TARGET,
        &["std/process::Process"]
    ),
    available_module!("std/queue", queue::queue_interface, PORTABLE_TARGETS),
    available_module!(
        "std/random",
        random::random_interface,
        PORTABLE_TARGETS,
        &["std/random::Random"]
    ),
    available_module!("std/ref", r#ref::ref_interface, PORTABLE_TARGETS),
    available_module!("std/regex", regex::regex_interface, PORTABLE_TARGETS),
    available_module!(
        "std/semaphore",
        semaphore::semaphore_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/set", set::set_interface, PORTABLE_TARGETS),
    available_module!(
        "std/stdin",
        stdin::stdin_interface,
        PROCESS_TARGET,
        &["std/prelude::Stdin"]
    ),
    available_module!("std/stream", stream::stream_interface, PORTABLE_TARGETS),
    available_module!("std/test", test::test_interface, PORTABLE_TARGETS),
    available_module!("std/text", text::text_interface, PORTABLE_TARGETS),
    available_module!(
        "std/text/grapheme",
        grapheme::grapheme_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/text/unicode",
        unicode::unicode_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/validation",
        validation::validation_interface,
        PORTABLE_TARGETS
    ),
];

fn prelude_type(name: &str) -> InterfaceType {
    external_type(
        name,
        &format!("std/prelude::{name}"),
        "std/prelude",
        name,
        Vec::new(),
    )
}

fn collection_constraint(name: &str, arguments: Vec<InterfaceType>) -> InterfaceConstraint {
    InterfaceConstraint {
        name: name.to_owned(),
        trait_identity: Some(format!("std/prelude::{name}")),
        arguments,
    }
}

fn key_constraints(parameter: &str) -> Vec<InterfaceConstraint> {
    ["Eq", "Hash"]
        .into_iter()
        .map(|name| collection_constraint(name, vec![named(parameter)]))
        .collect()
}

fn text_range_payload() -> InterfaceType {
    record([
        required("start", named("Int")),
        required("end", named("Int")),
        required("length", named("Int")),
    ])
}

fn standard_interface(module: &str, exports: Vec<InterfaceExport>) -> ModuleInterface {
    ModuleInterface {
        namespace_deprecations: Vec::new(),
        operator_deprecations: Vec::new(),
        schema: 1,
        module: module.to_owned(),
        source: format!("{module}.ssrg"),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn sum_traversal_export(module: &str, sequence: bool, either: bool) -> InterfaceExport {
    let wrap = |value: InterfaceType| {
        if either {
            named_with("Either", vec![named("E"), value])
        } else {
            named_with("Maybe", vec![value])
        }
    };
    let container = |value| named_with("F", vec![value]);
    let (name, arguments, result) = if sequence {
        (
            "sequence",
            vec![container(wrap(named("A")))],
            wrap(container(named("A"))),
        )
    } else {
        (
            "traverse",
            vec![
                function_type(vec![named("A")], wrap(named("B"))),
                container(named("A")),
            ],
            wrap(container(named("B"))),
        )
    };
    let mut export = function_export(
        module,
        name,
        [],
        vec![collection_constraint("Traversable", vec![named("F")])],
        arguments,
        result,
    );
    export.scheme.type_parameters = vec![TypeParameter::constructor("F", 1)];
    if either {
        export
            .scheme
            .type_parameters
            .push(TypeParameter::value("E"));
    }
    export
        .scheme
        .type_parameters
        .push(TypeParameter::value("A"));
    if !sequence {
        export
            .scheme
            .type_parameters
            .push(TypeParameter::value("B"));
    }
    export
}

fn collection_interface(
    module: &str,
    collection: &str,
    conversion: &str,
    conversion_target: &str,
) -> ModuleInterface {
    let values = named_with(collection, vec![named("A")]);
    let tuple = |elements| InterfaceType::Tuple { elements };
    let mapped_values = named_with(collection, vec![named("B")]);
    let maybe_value = named_with("Maybe", vec![named("A")]);
    let mut exports = vec![
        function_export(
            module,
            conversion,
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named_with(conversion_target, vec![named("A")]),
        ),
        function_export(
            module,
            "filter",
            ["A"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("Bool")),
                values.clone(),
            ],
            values.clone(),
        ),
        function_export(
            module,
            "filterMap",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named_with("Maybe", vec![named("B")])),
                values.clone(),
            ],
            mapped_values.clone(),
        ),
        function_export(
            module,
            "flatMap",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], mapped_values.clone()),
                values.clone(),
            ],
            mapped_values,
        ),
        function_export(
            module,
            "find",
            ["A"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("Bool")),
                values.clone(),
            ],
            maybe_value.clone(),
        ),
        function_export(
            module,
            "take",
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "drop",
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "append",
            ["A"],
            Vec::new(),
            vec![values.clone(), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "concat",
            ["A"],
            Vec::new(),
            vec![named_with(collection, vec![values.clone()])],
            values.clone(),
        ),
        function_export(
            module,
            "reverse",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "length",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named("Int"),
        ),
        function_export(
            module,
            "isEmpty",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named("Bool"),
        ),
        function_export(
            module,
            "get",
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            maybe_value.clone(),
        ),
        function_export(
            module,
            "head",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            maybe_value,
        ),
        function_export(
            module,
            "tail",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named_with("Maybe", vec![values.clone()]),
        ),
    ];
    exports.extend([
        function_export(
            module,
            "empty",
            ["A"],
            Vec::new(),
            vec![named("Unit")],
            values.clone(),
        ),
        function_export(
            module,
            "singleton",
            ["A"],
            Vec::new(),
            vec![named("A")],
            values.clone(),
        ),
        function_export(
            module,
            "fromIterable",
            ["C", "A"],
            vec![collection_constraint(
                "Iterable",
                vec![named("C"), named("A")],
            )],
            vec![named("C")],
            values.clone(),
        ),
        function_export(
            module,
            "reduceRight",
            ["A", "B"],
            Vec::new(),
            vec![
                named("B"),
                function_type(vec![named("A"), named("B")], named("B")),
                values.clone(),
            ],
            named("B"),
        ),
        function_export(
            module,
            "findIndex",
            ["A"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("Bool")),
                values.clone(),
            ],
            named_with("Maybe", vec![named("Int")]),
        ),
        function_export(
            module,
            "zip",
            ["A", "B"],
            Vec::new(),
            vec![named_with(collection, vec![named("B")]), values.clone()],
            named_with(collection, vec![tuple(vec![named("A"), named("B")])]),
        ),
        function_export(
            module,
            "zipWith",
            ["A", "B", "C"],
            Vec::new(),
            vec![
                function_type(vec![named("A"), named("B")], named("C")),
                named_with(collection, vec![named("B")]),
                values.clone(),
            ],
            named_with(collection, vec![named("C")]),
        ),
        function_export(
            module,
            "unzip",
            ["A", "B"],
            Vec::new(),
            vec![named_with(
                collection,
                vec![tuple(vec![named("A"), named("B")])],
            )],
            tuple(vec![
                values.clone(),
                named_with(collection, vec![named("B")]),
            ]),
        ),
        function_export(
            module,
            "sort",
            ["A"],
            vec![collection_constraint("Ord", vec![named("A")])],
            vec![values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "sortBy",
            ["A", "K"],
            vec![collection_constraint("Ord", vec![named("K")])],
            vec![function_type(vec![named("A")], named("K")), values.clone()],
            values.clone(),
        ),
        function_export(
            module,
            "groupBy",
            ["A", "K"],
            key_constraints("K"),
            vec![function_type(vec![named("A")], named("K")), values.clone()],
            external_type(
                "Map",
                "std/map::Map",
                "std/map",
                "Map",
                vec![named("K"), values.clone()],
            ),
        ),
        function_export(
            module,
            "last",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named_with("Maybe", vec![named("A")]),
        ),
        function_export(
            module,
            "init",
            ["A"],
            Vec::new(),
            vec![values.clone()],
            named_with("Maybe", vec![values.clone()]),
        ),
    ]);
    for name in ["takeWhile", "dropWhile"] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("Bool")),
                values.clone(),
            ],
            values.clone(),
        ));
    }
    for name in ["chunksOf", "windows"] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            Vec::new(),
            vec![named("Int"), values.clone()],
            named_with(
                "Either",
                vec![
                    external_type(
                        "SizeError",
                        "std/collection::SizeError",
                        "std/collection",
                        "SizeError",
                        Vec::new(),
                    ),
                    named_with(collection, vec![values.clone()]),
                ],
            ),
        ));
    }
    ModuleInterface {
        namespace_deprecations: Vec::new(),
        operator_deprecations: Vec::new(),
        schema: 1,
        module: module.to_owned(),
        source: format!("{module}.ssrg"),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

/// Returns the compiler-owned public interface for a standard module.
///
/// Standard modules participate in ordinary project linking. Their
/// implementation is selected later by the target runtime ABI, so they never
/// become filesystem graph nodes or generated source-module imports.
pub fn standard_module_target(specifier: &str) -> Option<ModuleLinkTarget> {
    STANDARD_MODULES
        .iter()
        .find(|module| module.specifier == specifier)
        .and_then(|module| module.interface)
        .map(|interface| ModuleLinkTarget::external(interface()))
}

/// Returns every compiler-owned standard module interface.
///
/// Tooling consumes the same interface registry as the linker so Reference,
/// hover, and future completion surfaces cannot drift from compilation.
pub fn standard_module_interfaces() -> Vec<ModuleInterface> {
    STANDARD_MODULES
        .iter()
        .filter_map(|module| module.interface.map(|interface| interface()))
        .collect()
}

/// Returns the canonical explicit standard-module registry.
///
/// `contract-only` entries establish stable module identity and target /
/// capability metadata, but intentionally carry no public interface and are
/// therefore not linkable. The implicit Prelude remains owned by the
/// semantics Prelude registry until its dedicated consolidation work.
pub fn standard_module_registry_surface() -> StandardModuleRegistrySurface {
    StandardModuleRegistrySurface {
        schema: 1,
        kind: "standard-module-registry",
        language_version: crate::IMPLEMENTED_LANGUAGE_VERSION,
        prelude: StandardPreludeBoundary {
            specifier: "std/prelude",
            availability: "implicit",
            registry: "standard-prelude-surface",
        },
        modules: STANDARD_MODULES
            .iter()
            .map(|module| StandardModuleSurface {
                specifier: module.specifier,
                identity: module.specifier,
                status: module.status,
                targets: module.targets,
                capability_services: module.capability_services,
                public_interface: module.interface.map(|interface| interface()),
            })
            .collect(),
    }
}

pub fn standard_module_status(specifier: &str) -> Option<StandardModuleStatus> {
    STANDARD_MODULES
        .iter()
        .find(|module| module.specifier == specifier)
        .map(|module| module.status)
}

pub fn is_available_standard_module(specifier: &str) -> bool {
    standard_module_status(specifier) == Some(StandardModuleStatus::Available)
}

pub fn is_standard_module(specifier: &str) -> bool {
    STANDARD_MODULES
        .iter()
        .any(|module| module.specifier == specifier)
}

fn html_props_aliases() -> Vec<InterfaceExport> {
    vec![
        alias_type_export(
            "std/web/html",
            "ElementProps",
            ["Action", "C"],
            element_props(),
        ),
        alias_type_export(
            "std/web/html",
            "ButtonProps",
            ["Action", "C"],
            button_props(),
        ),
        alias_type_export("std/web/html", "FormProps", ["Action", "C"], form_props()),
        alias_type_export("std/web/html", "LabelProps", ["Action", "C"], label_props()),
        alias_type_export("std/web/html", "InputProps", ["Action"], input_props()),
        alias_type_export(
            "std/web/html",
            "TextareaProps",
            ["Action"],
            textarea_props(),
        ),
        alias_type_export(
            "std/web/html",
            "AnchorProps",
            ["Action", "C"],
            anchor_props(),
        ),
        alias_type_export("std/web/html", "ImageProps", ["Action"], image_props()),
    ]
}

fn html_tag_export(tag: StandardHtmlTag) -> InterfaceExport {
    let props = props_for_html_tag(tag);
    if tag.void_element || tag.kind == StandardHtmlTagKind::Textarea {
        function_export(
            "std/web/html",
            tag.name,
            ["Action"],
            Vec::new(),
            vec![props],
            html(named("Action")),
        )
    } else {
        constrained_html_function(tag.name, props)
    }
}

fn signal_function<const N: usize>(
    name: &str,
    parameters: [&str; N],
    arguments: Vec<InterfaceType>,
    result: InterfaceType,
) -> InterfaceExport {
    function_export(
        "std/signal",
        name,
        parameters,
        Vec::new(),
        arguments,
        result,
    )
}

fn constrained_html_function(name: &str, parameter: InterfaceType) -> InterfaceExport {
    function_export(
        "std/web/html",
        name,
        ["Action", "C"],
        vec![InterfaceConstraint {
            name: "IntoChildren".to_owned(),
            trait_identity: Some("std/web/html::trait(IntoChildren)".to_owned()),
            arguments: vec![named("C"), named("Action")],
        }],
        vec![parameter],
        html(named("Action")),
    )
}

fn fragment_parameter() -> InterfaceType {
    named("C")
}

fn props_for_html_tag(tag: StandardHtmlTag) -> InterfaceType {
    use StandardHtmlTagKind as Kind;
    match tag.kind {
        Kind::Element => element_props(),
        Kind::VoidElement => void_element_props(),
        Kind::Meta => meta_props(),
        Kind::Link => link_props(),
        Kind::Anchor => anchor_props(),
        Kind::Image => image_props(),
        Kind::Source => source_props(),
        Kind::Video => video_props(),
        Kind::Audio => audio_props(),
        Kind::Button => button_props(),
        Kind::Form => form_props(),
        Kind::Label => label_props(),
        Kind::Input => input_props(),
        Kind::Textarea => textarea_props(),
        Kind::Select => select_props(),
        Kind::Option => option_props(),
        Kind::TableCell => table_cell_props(),
        Kind::OpenElement => open_element_props(),
    }
}

fn element_props() -> InterfaceType {
    with_children(common_html_props())
}

fn void_element_props() -> InterfaceType {
    record_vec(common_html_props())
}

fn meta_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            optional("charSet", named("String")),
            optional("name", named("String")),
            optional("content", named("String")),
            optional("httpEquiv", named("String")),
        ],
    ))
}

fn link_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            required("rel", named("String")),
            required("href", named("WebUrl")),
        ],
    ))
}

fn anchor_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            required("href", named("WebUrl")),
            optional("target", named("String")),
            optional("rel", named("String")),
            optional("download", named("Bool")),
        ],
    ))
}

fn image_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            required("src", named("WebUrl")),
            required("alt", named("String")),
            optional("width", named("Int")),
            optional("height", named("Int")),
            optional("loading", named("String")),
        ],
    ))
}

fn source_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            required("src", named("WebUrl")),
            optional("media", named("String")),
            optional("mimeType", named("String")),
        ],
    ))
}

fn video_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("src", named("WebUrl")),
            optional("width", named("Int")),
            optional("height", named("Int")),
        ],
    ))
}

fn audio_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [optional("src", named("WebUrl"))],
    ))
}

fn button_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("disabled", named("Bool")),
            optional("buttonType", named("String")),
            optional("name", named("String")),
            optional("value", named("String")),
            optional("autoFocus", named("Bool")),
        ],
    ))
}

fn form_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("onSubmit", named("Action")),
            optional("name", named("String")),
            optional("autoComplete", named("String")),
        ],
    ))
}

fn label_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [optional("htmlFor", named("String"))],
    ))
}

fn input_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            optional("value", named("String")),
            optional("checked", named("Bool")),
            optional("name", named("String")),
            optional("disabled", named("Bool")),
            optional("required", named("Bool")),
            optional("readOnly", named("Bool")),
            optional("multiple", named("Bool")),
            optional("placeholder", named("String")),
            optional("autoComplete", named("String")),
            optional("autoFocus", named("Bool")),
            optional("min", named("String")),
            optional("max", named("String")),
            optional("step", named("String")),
            optional("pattern", named("String")),
            optional("inputType", named("String")),
            optional(
                "onInput",
                function_type(vec![html_event_type("InputEvent")], named("Action")),
            ),
            optional(
                "onChange",
                function_type(vec![html_event_type("ChangeEvent")], named("Action")),
            ),
            optional(
                "onFileChange",
                function_type(vec![html_event_type("FileChangeEvent")], named("Action")),
            ),
        ],
    ))
}

fn textarea_props() -> InterfaceType {
    record_vec(with_fields(
        common_html_props(),
        [
            optional("value", named("String")),
            optional("name", named("String")),
            optional("disabled", named("Bool")),
            optional("required", named("Bool")),
            optional("readOnly", named("Bool")),
            optional("placeholder", named("String")),
            optional("autoComplete", named("String")),
            optional("autoFocus", named("Bool")),
            optional("rows", named("Int")),
            optional("cols", named("Int")),
            optional(
                "onInput",
                function_type(vec![html_event_type("InputEvent")], named("Action")),
            ),
            optional(
                "onChange",
                function_type(vec![html_event_type("ChangeEvent")], named("Action")),
            ),
        ],
    ))
}

fn select_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("name", named("String")),
            optional("value", named("String")),
            optional("disabled", named("Bool")),
            optional("required", named("Bool")),
            optional("multiple", named("Bool")),
            optional("autoFocus", named("Bool")),
            optional(
                "onChange",
                function_type(vec![html_event_type("ChangeEvent")], named("Action")),
            ),
        ],
    ))
}

fn option_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("value", named("String")),
            optional("selected", named("Bool")),
            optional("disabled", named("Bool")),
        ],
    ))
}

fn table_cell_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [
            optional("colSpan", named("Int")),
            optional("rowSpan", named("Int")),
        ],
    ))
}

fn open_element_props() -> InterfaceType {
    with_children(with_fields(
        common_html_props(),
        [optional("open", named("Bool"))],
    ))
}

fn common_html_props() -> Vec<InterfaceRecordField> {
    vec![
        optional("elementRef", named("ElementRef")),
        optional("id", named("String")),
        optional("class", named("String")),
        optional("title", named("String")),
        optional("hidden", named("Bool")),
        optional("key", named("String")),
        optional("style", named("Style")),
        optional("attributes", named_with("Array", vec![named("Attribute")])),
        optional("role", named("String")),
        optional("tabIndex", named("Int")),
        optional("lang", named("String")),
        optional("dir", named("String")),
        optional("draggable", named("Bool")),
        optional("contentEditable", named("Bool")),
        optional("inert", named("Bool")),
        optional("ariaLabel", named("String")),
        optional("ariaBusy", named("Bool")),
        optional("ariaExpanded", named("Bool")),
        optional("ariaHidden", named("Bool")),
        optional("ariaSelected", named("Bool")),
        optional("onClick", named("Action")),
        optional("preventClickDefault", named("Bool")),
        optional("stopClickPropagation", named("Bool")),
        optional("onFocus", named("Action")),
        optional("onBlur", named("Action")),
        optional(
            "onKeyDown",
            function_type(vec![html_event_type("KeyboardEvent")], event_action_type()),
        ),
        optional(
            "onKeyUp",
            function_type(vec![html_event_type("KeyboardEvent")], event_action_type()),
        ),
        optional(
            "onMouseDown",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onMouseUp",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onPointerDown",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
        ),
        optional(
            "onPointerMove",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
        ),
        optional(
            "onPointerUp",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
        ),
        optional(
            "onPointerCancel",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
        ),
        optional(
            "onWheel",
            function_type(vec![html_event_type("WheelEvent")], event_action_type()),
        ),
        optional(
            "onDoubleClick",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onContextMenu",
            function_type(vec![html_event_type("MouseEvent")], event_action_type()),
        ),
        optional(
            "onScroll",
            function_type(vec![html_event_type("ScrollEvent")], event_action_type()),
        ),
    ]
}

fn event_action_type() -> InterfaceType {
    external_type(
        "EventAction",
        "std/web/html::EventAction",
        "std/web/html",
        "EventAction",
        vec![named("Action")],
    )
}

fn with_fields<const N: usize>(
    mut fields: Vec<InterfaceRecordField>,
    additions: [InterfaceRecordField; N],
) -> Vec<InterfaceRecordField> {
    fields.extend(additions);
    fields
}

fn with_children(mut fields: Vec<InterfaceRecordField>) -> InterfaceType {
    fields.push(required("children", named("C")));
    record_vec(fields)
}

fn record_vec(fields: Vec<InterfaceRecordField>) -> InterfaceType {
    InterfaceType::Record {
        closed: true,
        fields,
    }
}

fn type_export(module: &str, name: &str, arity: u32, declaration_kind: &str) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: None,
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some(declaration_kind.to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: (0..arity)
                .map(|index| TypeParameter::value(format!("T{index}")))
                .collect(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity,
            },
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn canonical_type_export(
    module: &str,
    name: &str,
    canonical: &str,
    arity: u32,
    declaration_kind: &str,
) -> InterfaceExport {
    let mut export = type_export(module, name, arity, declaration_kind);
    export.symbol = canonical.to_owned();
    export
}

fn alias_type_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
    representation: InterfaceType,
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: None,
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some("alias".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity: N as u32,
            },
        },
        methods: Vec::new(),
        representation: Some(representation),
    }
}

fn opaque_adt_type_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
) -> InterfaceExport {
    let mut export = type_export(module, name, N as u32, "opaque-type");
    export.scheme.type_parameters = parameters.into_iter().map(TypeParameter::value).collect();
    export
}

fn constructor_export<const N: usize>(
    module: &str,
    owner: &str,
    name: &str,
    parameters: [&str; N],
    payload: Option<InterfaceType>,
) -> InterfaceExport {
    let arguments = parameters
        .iter()
        .map(|parameter| named(parameter))
        .collect::<Vec<_>>();
    let result = external_type(
        owner,
        &format!("{module}::{owner}"),
        module,
        owner,
        arguments,
    );
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "value".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: Some(format!("{module}::{owner}")),
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some("constructor".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints: Vec::new(),
            type_ref: payload.map_or(result.clone(), |payload| InterfaceType::Function {
                parameter: Box::new(payload),
                result: Box::new(result),
            }),
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn canonical_constructor_export(
    module: &str,
    owner: &str,
    owner_canonical: &str,
    name: &str,
    payload: Option<InterfaceType>,
) -> InterfaceExport {
    let result = external_type(owner, owner_canonical, "std/prelude", owner, Vec::new());
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "value".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: Some(owner_canonical.to_owned()),
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some("constructor".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: Vec::new(),
            constraints: Vec::new(),
            type_ref: match payload {
                Some(payload) => function_type(vec![payload], result),
                None => result,
            },
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn record_type_export<const N: usize>(
    module: &str,
    name: &str,
    fields: [InterfaceRecordField; N],
) -> InterfaceExport {
    let mut export = type_export(module, name, 0, "opaque-struct");
    export.representation = Some(record(fields));
    export
}

fn public_record_type_export<const N: usize>(
    module: &str,
    name: &str,
    fields: [InterfaceRecordField; N],
) -> InterfaceExport {
    let mut export = type_export(module, name, 0, "struct");
    export.representation = Some(record(fields));
    export
}

fn trait_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::trait({name})"),
        namespace: "trait".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: None,
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some("trait".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity: N as u32,
            },
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn function_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
    constraints: Vec<InterfaceConstraint>,
    arguments: Vec<InterfaceType>,
    result: InterfaceType,
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "value".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: None,
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some("function".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: parameters.into_iter().map(TypeParameter::value).collect(),
            constraints,
            type_ref: function_type(arguments, result),
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn value_export(module: &str, name: &str, type_ref: InterfaceType) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module}::{name}"),
        namespace: "value".to_owned(),
        name: name.to_owned(),
        deprecation: None,
        constructor_of: None,
        call_arity: None,
        visibility: Visibility::Public,
        declaration_kind: Some("value".to_owned()),
        declaration: ORIGIN,
        scheme: InterfaceScheme {
            type_parameters: Vec::new(),
            constraints: Vec::new(),
            type_ref,
        },
        methods: Vec::new(),
        representation: None,
    }
}

fn effect_function_export<const N: usize>(
    module: &str,
    name: &str,
    parameters: [&str; N],
    constraints: Vec<InterfaceConstraint>,
    arguments: Vec<InterfaceType>,
    result: InterfaceType,
) -> InterfaceExport {
    let mut export = function_export(module, name, parameters, constraints, arguments, result);
    export.declaration_kind = Some("effect-function".to_owned());
    export
}

fn html(action: InterfaceType) -> InterfaceType {
    InterfaceType::Named {
        name: "Html".to_owned(),
        arguments: vec![action],
    }
}

fn html_event_type(name: &str) -> InterfaceType {
    external_type(
        name,
        &format!("std/web/html::{name}"),
        "std/web/html",
        name,
        Vec::new(),
    )
}

fn named(name: &str) -> InterfaceType {
    named_with(name, Vec::new())
}

fn named_with(name: &str, arguments: Vec<InterfaceType>) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments,
    }
}

fn external_type(
    name: &str,
    canonical: &str,
    provider_module: &str,
    provider_export: &str,
    arguments: Vec<InterfaceType>,
) -> InterfaceType {
    InterfaceType::ExternalNamed {
        name: name.to_owned(),
        canonical: canonical.to_owned(),
        provider_module: provider_module.to_owned(),
        provider_export: provider_export.to_owned(),
        arguments,
    }
}

fn signal_type(name: &str, value: InterfaceType) -> InterfaceType {
    named_with(name, vec![value])
}

fn task(success: InterfaceType) -> InterfaceType {
    effect(record([]), named("Never"), success)
}

fn effect(
    environment: InterfaceType,
    failure: InterfaceType,
    success: InterfaceType,
) -> InterfaceType {
    named_with("Effect", vec![environment, failure, success])
}

fn function_type(parameters: Vec<InterfaceType>, result: InterfaceType) -> InterfaceType {
    parameters
        .into_iter()
        .rev()
        .fold(result, |result, parameter| InterfaceType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        })
}

fn record<const N: usize>(fields: [InterfaceRecordField; N]) -> InterfaceType {
    InterfaceType::Record {
        closed: true,
        fields: fields.into_iter().collect(),
    }
}

fn requirement_merge(operands: Vec<InterfaceType>) -> InterfaceType {
    InterfaceType::RequirementMerge { operands }
}

fn required(name: &str, type_ref: InterfaceType) -> InterfaceRecordField {
    InterfaceRecordField {
        name: name.to_owned(),
        optional: false,
        type_ref,
    }
}

fn optional(name: &str, type_ref: InterfaceType) -> InterfaceRecordField {
    InterfaceRecordField {
        name: name.to_owned(),
        optional: true,
        type_ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterator_module_exposes_the_existing_canonical_type_without_instances() {
        let target = standard_module_target("std/iterator").unwrap();
        let interface = target.interface();
        assert_eq!(interface.exports.len(), 3);
        let iterator = interface
            .exports
            .iter()
            .find(|export| export.name == "Iterator")
            .unwrap();
        assert_eq!(iterator.symbol, "std/prelude::Iterator");
        assert_eq!(iterator.scheme.type_parameters.len(), 1);
        assert!(interface.instances.is_empty());
        for name in ["unfold", "next"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.name == name && export.namespace == "value"));
        }
    }

    #[test]
    fn validation_interface_owns_its_type_constructors_and_specific_operations() {
        let target = standard_module_target("std/validation").unwrap();
        let exports = &target.interface().exports;
        assert_eq!(exports.len(), 8);
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
            assert!(exports
                .iter()
                .any(|export| export.name == name
                    && export.symbol == format!("std/validation::{name}")));
        }
        for name in ["Valid", "Invalid"] {
            let constructor = exports.iter().find(|export| export.name == name).unwrap();
            assert_eq!(
                constructor.constructor_of.as_deref(),
                Some("std/validation::Validation")
            );
            assert_eq!(
                constructor.scheme.type_parameters,
                [TypeParameter::value("E"), TypeParameter::value("A")]
            );
        }
        let invalid = exports
            .iter()
            .find(|export| export.name == "Invalid")
            .unwrap();
        assert!(
            matches!(&invalid.scheme.type_ref, InterfaceType::Function { parameter, .. }
            if matches!(parameter.as_ref(), InterfaceType::ExternalNamed { canonical, .. } if canonical == "std/non-empty-list::NonEmptyList"))
        );
        for name in ["map", "apply", "pure", "flatMap"] {
            assert!(!exports.iter().any(|export| export.name == name));
        }
    }

    #[test]
    fn maybe_either_interfaces_own_only_their_specific_operations() {
        for (module, count) in [("std/maybe", 4), ("std/either", 7)] {
            let target = standard_module_target(module).unwrap();
            let exports = &target.interface().exports;
            assert_eq!(exports.len(), count);
            for name in [
                "Maybe", "Either", "Just", "Nothing", "Left", "Right", "map", "apply", "pure",
                "flatMap",
            ] {
                assert!(!exports.iter().any(|export| export.name == name));
            }
            for name in ["sequence", "traverse"] {
                let operation = exports.iter().find(|export| export.name == name).unwrap();
                assert_eq!(
                    operation.scheme.type_parameters[0],
                    TypeParameter::constructor("F", 1)
                );
                assert_eq!(
                    operation.scheme.constraints,
                    vec![collection_constraint("Traversable", vec![named("F")])]
                );
            }
        }
    }

    #[test]
    fn registry_separates_contract_identity_from_product_availability() {
        let registry = standard_module_registry_surface();

        assert_eq!(registry.prelude.specifier, "std/prelude");
        assert_eq!(registry.prelude.registry, "standard-prelude-surface");

        let effect = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/effect")
            .expect("specified standard module is registered");
        assert_eq!(effect.status, StandardModuleStatus::Available);
        assert_eq!(
            effect
                .public_interface
                .as_ref()
                .map(|interface| interface.module.as_str()),
            Some("std/effect")
        );
        assert!(is_standard_module("std/effect"));
        assert!(is_available_standard_module("std/effect"));
        assert!(standard_module_target("std/effect").is_some());

        let http = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/http")
            .expect("available standard module is registered");
        assert_eq!(http.status, StandardModuleStatus::Available);
        assert_eq!(http.capability_services, ["std/http::HttpClient"]);
        assert_eq!(
            http.public_interface
                .as_ref()
                .map(|interface| interface.module.as_str()),
            Some("std/http")
        );
        assert!(is_available_standard_module("std/http"));
    }

    #[test]
    fn registry_specifiers_are_unique() {
        let registry = standard_module_registry_surface();
        let mut specifiers = std::collections::BTreeSet::new();

        for module in registry.modules {
            assert!(
                specifiers.insert(module.specifier),
                "duplicate standard module registry entry for `{}`",
                module.specifier
            );
        }
    }

    #[test]
    fn exposes_the_portable_regex_surface() {
        let regex = standard_module_target("std/regex").expect("std/regex is available");
        let exports = &regex.interface().exports;
        for name in [
            "Regex",
            "RegexCompileErrorKind",
            "UnexpectedRegexEnd",
            "UnexpectedRegexToken",
            "InvalidRegexEscape",
            "InvalidRegexRange",
            "InvalidRegexQuantifier",
            "DuplicateCaptureName",
            "UnsupportedRegexFeature",
            "RegexCompileError",
            "RegexOptions",
            "RegexSpan",
            "RegexCapture",
            "RegexMatch",
            "defaultOptions",
            "compile",
            "compileWith",
            "isMatch",
            "find",
            "findAll",
            "split",
            "replaceAll",
            "replaceAllWith",
            "escape",
        ] {
            assert!(
                exports.iter().any(|export| export.name == name),
                "missing std/regex::{name}"
            );
        }

        let registry = standard_module_registry_surface();
        let surface = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/regex")
            .unwrap();
        assert_eq!(surface.status, StandardModuleStatus::Available);
        assert_eq!(surface.targets, PORTABLE_TARGETS);
        assert!(surface.capability_services.is_empty());
    }

    #[test]
    fn exposes_the_small_response_http_client_surface() {
        let http = standard_module_target("std/http").unwrap();
        for name in [
            "HttpClient",
            "Method",
            "Status",
            "Headers",
            "HttpUrl",
            "Request",
            "Response",
            "HttpBodyLimit",
            "HttpBuildError",
            "HttpError",
            "customMethod",
            "parseUrl",
            "appendHeader",
            "request",
            "sendBytes",
            "sendEmpty",
            "responseStatus",
            "responseHeaders",
            "responseBody",
        ] {
            assert!(http
                .interface()
                .exports
                .iter()
                .any(|export| export.name == name));
        }
        for name in [
            "get",
            "head",
            "post",
            "put",
            "patch",
            "delete",
            "options",
            "connect",
            "trace",
            "emptyHeaders",
        ] {
            let export = http
                .interface()
                .exports
                .iter()
                .find(|export| export.name == name)
                .expect("HTTP value is exported");
            assert_eq!(export.declaration_kind.as_deref(), Some("value"));
        }
    }

    #[test]
    fn exposes_the_effectful_http_server_application_surface() {
        let server = standard_module_target("std/http/server").unwrap();
        let interface = server.interface();
        for name in [
            "HttpServer",
            "HttpServerRequest",
            "HttpServerResponse",
            "HttpServerHandle",
            "HttpServerError",
            "HttpHeader",
            "Handler",
            "requestMethod",
            "requestUrl",
            "requestPath",
            "requestQuery",
            "requestHeaders",
            "requestHeaderValues",
            "requestBody",
            "header",
            "emptyResponse",
            "bytesResponse",
            "streamResponse",
            "textResponse",
            "jsonResponse",
            "pureHandler",
            "recoverHandler",
            "listen",
            "serveOnce",
            "close",
        ] {
            assert!(
                interface.exports.iter().any(|export| export.name == name),
                "missing std/http/server::{name}"
            );
        }

        let handler = interface
            .exports
            .iter()
            .find(|export| export.name == "Handler")
            .unwrap();
        assert_eq!(handler.declaration_kind.as_deref(), Some("alias"));
        assert!(format!("{:?}", handler.representation).contains("Effect"));

        for name in ["listen", "serveOnce"] {
            let operation = interface
                .exports
                .iter()
                .find(|export| export.name == name)
                .unwrap();
            assert_eq!(
                operation.scheme.type_parameters,
                [TypeParameter::value("R")]
            );
            assert!(format!("{:?}", operation.scheme.type_ref).contains("RequirementMerge"));
        }
    }

    #[test]
    fn exposes_the_portable_sse_stream_adapter_surface() {
        let sse = standard_module_target("std/sse").unwrap();
        for name in [
            "Event",
            "DecodeLimit",
            "SseBuildError",
            "SseParseError",
            "event",
            "withEventName",
            "withId",
            "withRetryMillis",
            "eventData",
            "eventName",
            "eventId",
            "eventRetryMillis",
            "encode",
            "keepAlive",
            "decodeLimit",
            "defaultDecodeLimit",
            "withLastEventId",
            "events",
            "response",
        ] {
            assert!(
                sse.interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/sse::{name}"
            );
        }
        let registry = standard_module_registry_surface();
        let surface = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/sse")
            .unwrap();
        assert_eq!(surface.targets, PORTABLE_TARGETS);
        assert!(surface.capability_services.is_empty());
    }

    #[test]
    fn exposes_browser_file_handles_and_portable_multipart_streaming() {
        let file = standard_module_target("std/web/file").unwrap();
        for name in [
            "Blob",
            "File",
            "BlobBuildError",
            "BlobReadError",
            "fromBytes",
            "asBlob",
            "name",
            "mimeType",
            "sizeBytes",
            "lastModifiedMillis",
            "readBytes",
            "readChunks",
            "body",
        ] {
            assert!(
                file.interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/web/file::{name}"
            );
        }
        let multipart = standard_module_target("std/http/multipart").unwrap();
        for name in [
            "Multipart",
            "MultipartBuildError",
            "empty",
            "appendText",
            "appendBytes",
            "appendBody",
            "contentType",
            "body",
        ] {
            assert!(
                multipart
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/http/multipart::{name}"
            );
        }
        let registry = standard_module_registry_surface();
        assert_eq!(
            registry
                .modules
                .iter()
                .find(|module| module.specifier == "std/web/file")
                .unwrap()
                .targets,
            BROWSER_TARGET
        );
        assert_eq!(
            registry
                .modules
                .iter()
                .find(|module| module.specifier == "std/http/multipart")
                .unwrap()
                .targets,
            PORTABLE_TARGETS
        );

        let html = standard_module_target("std/web/html").unwrap();
        assert!(html
            .interface()
            .exports
            .iter()
            .any(|export| export.name == "FileChangeEvent"));
        let input = html
            .interface()
            .exports
            .iter()
            .find(|export| export.name == "InputProps")
            .unwrap();
        assert!(format!("{:?}", input.representation).contains("onFileChange"));
    }

    #[test]
    fn exposes_the_portable_websocket_client_and_process_server_surfaces() {
        let registry = standard_module_registry_surface();
        let client_surface = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/websocket")
            .unwrap();
        assert_eq!(client_surface.targets, ["process", "browser"]);
        assert_eq!(
            client_surface.capability_services,
            ["std/websocket::WebSocketClient"]
        );
        let client = standard_module_target("std/websocket").unwrap();
        for name in [
            "WebSocketClient",
            "WebSocketConnection",
            "WebSocketEvent",
            "WebSocketClose",
            "WebSocketError",
            "connect",
            "messages",
            "sendText",
            "sendBytes",
            "closeConnection",
            "selectedProtocol",
            "foldEvent",
            "closeCode",
            "closeReason",
            "closeWasClean",
            "errorMessage",
        ] {
            assert!(
                client
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/websocket::{name}"
            );
        }

        let server_surface = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/websocket/server")
            .unwrap();
        assert_eq!(server_surface.targets, ["process"]);
        assert_eq!(
            server_surface.capability_services,
            ["std/websocket/server::WebSocketServer"]
        );
        let server = standard_module_target("std/websocket/server").unwrap();
        for name in [
            "WebSocketServer",
            "WebSocketServerHandle",
            "Handler",
            "listen",
            "closeServer",
        ] {
            assert!(
                server
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/websocket/server::{name}"
            );
        }
    }

    #[test]
    fn exposes_the_browser_navigation_surface() {
        let navigation = standard_module_target("std/web/navigation").unwrap();
        let registry = standard_module_registry_surface();
        let registry_navigation = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/web/navigation")
            .unwrap();
        assert_eq!(registry_navigation.targets, ["browser"]);
        assert_eq!(
            registry_navigation.capability_services,
            ["std/web/navigation::Navigation"]
        );
        for name in [
            "Navigation",
            "Url",
            "Query",
            "Location",
            "UrlBuildError",
            "NavigationError",
            "parseUrl",
            "resolveUrl",
            "pathSegments",
            "parseQuery",
            "queryValues",
            "toWebUrl",
            "current",
            "push",
            "replace",
            "back",
            "forward",
            "locationSignal",
        ] {
            assert!(
                navigation
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/web/navigation::{name}"
            );
        }
    }

    #[test]
    fn exposes_the_browser_storage_surface() {
        let storage = standard_module_target("std/web/storage").unwrap();
        let registry = standard_module_registry_surface();
        let registry_storage = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/web/storage")
            .unwrap();
        assert_eq!(registry_storage.targets, ["browser"]);
        assert_eq!(
            registry_storage.capability_services,
            ["std/web/storage::Storage"]
        );
        for name in [
            "Storage",
            "StorageArea",
            "Local",
            "Session",
            "StorageError",
            "StorageQuotaExceeded",
            "StorageSecurityFailure",
            "StorageUnavailable",
            "get",
            "set",
            "remove",
            "clear",
            "keys",
        ] {
            assert!(
                storage
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/web/storage::{name}"
            );
        }
    }

    #[test]
    fn exposes_effect_ref_duration_and_clock_execution_surfaces() {
        let effect = standard_module_target("std/effect").unwrap();
        for name in [
            "Fiber",
            "FiberExit",
            "FiberSucceeded",
            "FiberFailed",
            "FiberCancelled",
            "Parallelism",
            "ParallelismError",
            "Schedule",
            "ScheduleDecision",
            "ScheduleError",
            "succeed",
            "fail",
            "die",
            "defer",
            "mapError",
            "recover",
            "provide",
            "service",
            "provideSome",
            "attempt",
            "fromEither",
            "fromMaybe",
            "acquireRelease",
            "scoped",
            "fork",
            "await",
            "poll",
            "join",
            "interrupt",
            "yieldNow",
            "race",
            "parallel",
            "parallelism",
            "unboundedParallelism",
            "forEachParallel",
            "traverseParallel",
            "timeout",
            "timeoutFail",
            "schedule",
            "recurs",
            "spaced",
            "whileInput",
            "retry",
            "repeat",
        ] {
            assert!(
                effect.interface().exports.iter().any(|export| {
                    export.name == name && matches!(export.namespace.as_str(), "type" | "value")
                }),
                "missing std/effect::{name}"
            );
        }

        for (module, names) in [
            (
                "std/deferred",
                &[
                    "Deferred", "make", "await", "poll", "complete", "succeed", "fail",
                ][..],
            ),
            (
                "std/queue",
                &[
                    "Queue",
                    "QueueCreateError",
                    "NonPositiveCapacity",
                    "QueueClosed",
                    "bounded",
                    "unbounded",
                    "offer",
                    "take",
                    "tryOffer",
                    "tryTake",
                    "size",
                    "close",
                ][..],
            ),
            (
                "std/semaphore",
                &[
                    "Semaphore",
                    "Permit",
                    "SemaphoreCreateError",
                    "NonPositivePermits",
                    "make",
                    "acquire",
                    "release",
                    "withPermit",
                    "available",
                ][..],
            ),
        ] {
            let target = standard_module_target(module).unwrap();
            for name in names {
                assert!(
                    target
                        .interface()
                        .exports
                        .iter()
                        .any(|export| export.name == *name),
                    "missing {module}::{name}"
                );
            }
        }

        let reference = standard_module_target("std/ref").unwrap();
        for name in ["Ref", "make", "get", "set", "update", "modify"] {
            assert!(
                reference
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/ref::{name}"
            );
        }

        let time = standard_module_target("std/time").unwrap();
        for name in [
            "Instant",
            "Duration",
            "DurationError",
            "NegativeDuration",
            "DurationOutsideRange",
            "zeroDuration",
            "nanoseconds",
            "milliseconds",
            "seconds",
            "minutes",
            "hours",
            "toNanoseconds",
            "addDuration",
            "LocalDate",
            "LocalTime",
            "LocalDateTime",
            "UtcOffset",
            "OffsetDateTime",
            "TimeZone",
            "ZonedDateTime",
            "TimeZones",
            "DateTimeError",
            "TimeZoneError",
            "LocalResolution",
            "localDate",
            "localTime",
            "localDateTime",
            "utcOffset",
            "parseLocalDate",
            "parseLocalTime",
            "parseLocalDateTime",
            "parseOffsetDateTime",
            "formatLocalDate",
            "formatLocalTime",
            "formatLocalDateTime",
            "formatOffsetDateTime",
            "atOffset",
            "offsetInstant",
            "offsetLocalDateTime",
            "databaseVersion",
            "loadTimeZone",
            "timeZoneId",
            "timeZoneVersion",
            "atTimeZone",
            "resolveLocal",
            "zonedInstant",
            "zonedLocalDateTime",
            "zonedOffset",
            "zonedTimeZone",
        ] {
            assert!(
                time.interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/time::{name}"
            );
        }

        let clock = standard_module_target("std/clock").unwrap();
        assert!(clock
            .interface()
            .exports
            .iter()
            .any(|export| export.namespace == "value" && export.name == "sleep"));
    }

    #[test]
    fn exposes_stream_core_as_an_available_portable_module() {
        let stream = standard_module_target("std/stream").unwrap();
        for name in [
            "Stream",
            "BufferCapacity",
            "BufferCapacityError",
            "NonPositiveBufferCapacity",
            "empty",
            "singleton",
            "fromArray",
            "fromIterable",
            "fromEffect",
            "unfold",
            "map",
            "filter",
            "filterMap",
            "mapError",
            "flatMap",
            "take",
            "drop",
            "concat",
            "zip",
            "merge",
            "bufferCapacity",
            "buffer",
            "runCollect",
            "runFold",
            "runForEach",
        ] {
            assert!(
                stream
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/stream::{name}"
            );
        }
        let registry = standard_module_registry_surface();
        assert_eq!(
            registry
                .modules
                .iter()
                .find(|module| module.specifier == "std/stream")
                .unwrap()
                .targets,
            PORTABLE_TARGETS
        );
    }

    #[test]
    fn exposes_process_state_and_signals_only_on_process_targets() {
        let process = standard_module_target("std/process").expect("std/process is available");
        for name in [
            "Process",
            "ProcessSignal",
            "Interrupt",
            "Terminate",
            "Hangup",
            "Quit",
            "User1",
            "User2",
            "ProcessError",
            "UnsupportedProcessSignal",
            "ReservedProcessSignal",
            "InvalidArgumentEncoding",
            "InvalidEnvironmentName",
            "InvalidEnvironmentEncoding",
            "CurrentDirectoryUnavailable",
            "arguments",
            "environment",
            "currentDirectory",
            "signals",
        ] {
            assert!(
                process
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/process::{name}"
            );
        }
        let registry = standard_module_registry_surface();
        let entry = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/process")
            .unwrap();
        assert_eq!(entry.status, StandardModuleStatus::Available);
        assert_eq!(entry.capability_services, ["std/process::Process"]);
        assert_eq!(entry.targets, ["process"]);
    }

    #[test]
    fn exposes_cold_child_process_execution_only_on_process_targets() {
        let children =
            standard_module_target("std/child-process").expect("std/child-process is available");
        for name in [
            "ChildProcesses",
            "Executable",
            "Command",
            "CaptureLimit",
            "ChildProcessConfigError",
            "ChildProcessError",
            "ChildExitStatus",
            "ChildInput",
            "ChildEvent",
            "CapturedProcess",
            "command",
            "addArgument",
            "addArguments",
            "inDirectory",
            "setEnvironment",
            "unsetEnvironment",
            "clearEnvironment",
            "terminationGrace",
            "outputBuffer",
            "captureLimit",
            "defaultCaptureLimit",
            "runStreaming",
            "runCaptured",
            "runInherited",
        ] {
            assert!(
                children
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/child-process::{name}"
            );
        }
        let registry = standard_module_registry_surface();
        let surface = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/child-process")
            .unwrap();
        assert_eq!(surface.targets, PROCESS_TARGET);
        assert_eq!(
            surface.capability_services,
            ["std/child-process::ChildProcesses"]
        );
    }

    #[test]
    fn every_available_registry_interface_is_the_linker_projection() {
        let registry = standard_module_registry_surface();
        let registered = registry
            .modules
            .iter()
            .filter_map(|module| module.public_interface.clone())
            .collect::<Vec<_>>();

        assert_eq!(registered, standard_module_interfaces());
        for interface in registered {
            assert_eq!(
                standard_module_target(&interface.module).map(|target| target.interface().clone()),
                Some(interface)
            );
        }
    }

    #[test]
    fn exposes_compiler_owned_standard_modules_as_external_link_targets() {
        let target = standard_module_target("std/web/html").unwrap();

        assert_eq!(target.interface().module, "std/web/html");
        assert!(target
            .interface()
            .exports
            .iter()
            .any(|export| { export.namespace == "type" && export.name == "Html" }));
        assert!(target
            .interface()
            .exports
            .iter()
            .any(|export| { export.namespace == "value" && export.name == "renderToString" }));
        assert!(standard_module_target("std/web/missing").is_none());
        for (module, conversion) in [("std/array", "toList"), ("std/list", "toArray")] {
            let target = standard_module_target(module).unwrap();
            for name in [
                "empty",
                "singleton",
                "fromIterable",
                "reduceRight",
                "findIndex",
                "takeWhile",
                "dropWhile",
                "zip",
                "zipWith",
                "unzip",
                "sort",
                "sortBy",
                "groupBy",
                "last",
                "init",
                "chunksOf",
                "windows",
                "filter",
                "filterMap",
                "flatMap",
                "find",
                "take",
                "drop",
                "append",
                "concat",
                "reverse",
                conversion,
                "length",
                "isEmpty",
                "get",
                "head",
                "tail",
            ] {
                assert!(target
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.namespace == "value" && export.name == name));
            }
            assert_eq!(target.interface().exports.len(), 32);
            let group = target
                .interface()
                .exports
                .iter()
                .find(|e| e.name == "groupBy")
                .unwrap();
            assert_eq!(group.scheme.constraints, key_constraints("K"));
            let collect = target
                .interface()
                .exports
                .iter()
                .find(|e| e.name == "fromIterable")
                .unwrap();
            assert_eq!(
                collect.scheme.constraints,
                vec![collection_constraint(
                    "Iterable",
                    vec![named("C"), named("A")]
                )]
            );
        }
        let collection = standard_module_target("std/collection").unwrap();
        assert_eq!(collection.interface().exports.len(), 6);
        let reduce_until = collection
            .interface()
            .exports
            .iter()
            .find(|e| e.name == "reduceUntil")
            .unwrap();
        assert_eq!(
            reduce_until.scheme.constraints,
            vec![collection_constraint(
                "Iterable",
                vec![named("C"), named("A")]
            )]
        );
        for name in ["Next", "Done"] {
            assert!(collection.interface().exports.iter().any(|e| e.name == name
                && e.constructor_of.as_deref() == Some("std/collection::ReduceStep")));
        }
        for prelude_owned in [
            "map", "reduce", "sum", "product", "combine", "any", "all", "traverse",
        ] {
            assert!(!collection
                .interface()
                .exports
                .iter()
                .any(|e| e.name == prelude_owned));
        }
        assert!(collection
            .interface()
            .exports
            .iter()
            .any(|e| e.name == "NonPositiveSize"
                && e.constructor_of.as_deref() == Some("std/collection::SizeError")));
        let signal = standard_module_target("std/signal").unwrap();
        let distinct = signal
            .interface()
            .exports
            .iter()
            .find(|export| export.name == "distinct")
            .expect("std/signal exports distinct");
        assert!(matches!(
            distinct.scheme.constraints.as_slice(),
            [InterfaceConstraint {
                name,
                trait_identity: Some(identity),
                arguments,
            }] if name == "Eq" && identity == "std/prelude::Eq" && arguments == &[named("A")]
        ));
        let dom = standard_module_target("std/web/dom").unwrap();
        let dom_exports = &dom.interface().exports;
        for name in [
            "FreshMount",
            "HydrateStrict",
            "HydrateOrReplace",
            "ClearRenderedDom",
            "PreserveRenderedDom",
            "defaultOptions",
            "query",
            "mount",
            "awaitMount",
            "unmount",
            "content",
            "initialHtml",
            "bind",
            "textTarget",
            "attributeTarget",
            "booleanAttributeTarget",
            "ariaBooleanTarget",
            "classTarget",
            "titleTarget",
            "hiddenTarget",
            "disabledTarget",
            "inertTarget",
            "ariaLabelTarget",
            "ariaBusyTarget",
            "ariaExpandedTarget",
            "ariaHiddenTarget",
            "ariaSelectedTarget",
            "valueTarget",
            "checkedTarget",
            "styleTarget",
            "regionTarget",
            "bindText",
            "bindAttribute",
            "bindValue",
            "bindChecked",
            "bindStyle",
            "bindRegion",
            "mountContent",
            "runContent",
            "run",
            "app",
        ] {
            assert!(
                dom_exports
                    .iter()
                    .any(|export| export.namespace == "value" && export.name == name),
                "missing std/web/dom::{name}"
            );
        }
        for name in [
            "DomOptions",
            "DomTarget",
            "DomMount",
            "DomContent",
            "DomBinding",
            "BindingTarget",
            "DomError",
            "DomRuntimeError",
        ] {
            assert!(
                dom_exports
                    .iter()
                    .any(|export| export.namespace == "type" && export.name == name),
                "missing std/web/dom::{name}"
            );
        }
    }

    #[test]
    fn exposes_safe_integer_and_float_conversion_surfaces() {
        let int = standard_module_target("std/int").unwrap();
        let int_names = int
            .interface()
            .exports
            .iter()
            .map(|export| export.name.as_str())
            .collect::<Vec<_>>();
        for name in [
            "minValue",
            "maxValue",
            "parse",
            "parseRadix",
            "format",
            "formatRadix",
            "checkedAdd",
            "saturatingPower",
            "abs",
            "checkedDivide",
        ] {
            assert!(int_names.contains(&name), "missing std/int::{name}");
        }
        for removed in [
            "wrappingAdd",
            "checkedNegate",
            "checkedAbs",
            "saturatingNegate",
            "saturatingAbs",
            "IntDivisionOverflow",
        ] {
            assert!(
                !int_names.contains(&removed),
                "removed API {removed} leaked"
            );
        }

        let float = standard_module_target("std/float").unwrap();
        let float_names = float
            .interface()
            .exports
            .iter()
            .map(|export| export.name.as_str())
            .collect::<Vec<_>>();
        for name in ["fromInt", "toInt", "roundIntegral", "totalCompare"] {
            assert!(float_names.contains(&name), "missing std/float::{name}");
        }
        assert!(!float_names.contains(&"fromIntExact"));

        let number = standard_module_target("std/number").unwrap();
        for name in [
            "HalfEven",
            "HalfUp",
            "TowardZero",
            "AwayFromZero",
            "Floor",
            "Ceiling",
        ] {
            assert!(number
                .interface()
                .exports
                .iter()
                .any(|export| export.name == name));
        }
    }

    #[test]
    fn exposes_the_arbitrary_precision_big_int_surface() {
        let big_int = standard_module_target("std/big-int").expect("std/big-int is available");
        let names = big_int
            .interface()
            .exports
            .iter()
            .map(|export| export.name.as_str())
            .collect::<Vec<_>>();
        for name in [
            "BigInt",
            "BigIntParseError",
            "BigIntDivisionError",
            "BigIntPowerError",
            "BigIntConversionError",
            "parse",
            "parseRadix",
            "format",
            "formatRadix",
            "fromInt",
            "toInt",
            "checkedDivide",
            "checkedRemainder",
            "checkedPower",
            "abs",
            "sign",
        ] {
            assert!(names.contains(&name), "missing std/big-int::{name}");
        }
    }

    #[test]
    fn exposes_the_exact_decimal_surface() {
        let decimal = standard_module_target("std/decimal").expect("std/decimal is available");
        let names = decimal
            .interface()
            .exports
            .iter()
            .map(|export| export.name.as_str())
            .collect::<Vec<_>>();
        for name in [
            "Decimal",
            "DecimalContext",
            "DecimalParseError",
            "DecimalContextError",
            "DecimalArithmeticError",
            "DecimalConversionError",
            "parse",
            "fromInt",
            "toIntExact",
            "fromFloat",
            "toFloat",
            "context",
            "precision",
            "rounding",
            "divideExact",
            "divide",
            "quantize",
        ] {
            assert!(names.contains(&name), "missing std/decimal::{name}");
        }
    }

    #[test]
    fn exposes_bytes_and_utf8_as_available_standard_modules() {
        let bytes = standard_module_target("std/bytes").expect("std/bytes is available");
        for name in [
            "Byte",
            "Bytes",
            "ByteError",
            "BytesSliceError",
            "byte",
            "fromInts",
            "slice",
            "concat",
        ] {
            assert!(bytes
                .interface()
                .exports
                .iter()
                .any(|export| export.name == name));
        }

        let text = standard_module_target("std/text").expect("std/text is available");
        for name in [
            "Utf8DecodeError",
            "encodeUtf8",
            "decodeUtf8",
            "decodeUtf8Lossy",
        ] {
            assert!(text
                .interface()
                .exports
                .iter()
                .any(|export| export.name == name));
        }
        let hex = standard_module_target("std/bytes/hex").expect("std/bytes/hex is available");
        for name in [
            "HexDecodeError",
            "OddHexLength",
            "InvalidHexDigit",
            "encode",
            "decode",
        ] {
            assert!(hex
                .interface()
                .exports
                .iter()
                .any(|export| export.name == name));
        }

        let base64 =
            standard_module_target("std/bytes/base64").expect("std/bytes/base64 is available");
        for name in [
            "Base64DecodeError",
            "InvalidBase64Length",
            "InvalidBase64Digit",
            "InvalidBase64Padding",
            "NonCanonicalBase64Bits",
            "encode",
            "decode",
            "encodeUrl",
            "decodeUrl",
        ] {
            assert!(base64
                .interface()
                .exports
                .iter()
                .any(|export| export.name == name));
        }
    }

    #[test]
    fn exposes_random_and_entropy_as_available_standard_modules() {
        let random = standard_module_target("std/random").expect("std/random is available");
        for name in [
            "Random",
            "RandomRangeError",
            "RandomConfigError",
            "RandomSize",
            "randomSize",
            "algorithmId",
            "nextBool",
            "nextInt",
            "intBetween",
            "unitFloat",
            "chance",
            "randomBytes",
            "choose",
            "shuffle",
        ] {
            assert!(
                random
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/random::{name}"
            );
        }

        let entropy = standard_module_target("std/entropy").expect("std/entropy is available");
        for name in [
            "Entropy",
            "EntropyConfigError",
            "EntropyError",
            "EntropySize",
            "entropySize",
            "secureBytes",
        ] {
            assert!(
                entropy
                    .interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/entropy::{name}"
            );
        }
    }

    #[test]
    fn exposes_typed_form_events_from_the_shared_html_interface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for name in ["form", "label", "input", "textarea"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value" && export.name == name));
        }
        let input_event = interface
            .exports
            .iter()
            .find(|export| export.namespace == "type" && export.name == "InputEvent")
            .unwrap();
        assert_eq!(
            input_event.declaration_kind.as_deref(),
            Some("opaque-struct")
        );
        let Some(InterfaceType::Record { fields, .. }) = &input_event.representation else {
            panic!("InputEvent must expose its immutable snapshot fields");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "value");

        let input = interface
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "input")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &input.scheme.type_ref else {
            panic!("input must be callable");
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("input must accept a props record");
        };
        assert!(fields.iter().any(|field| field.name == "onInput"));
        assert!(fields.iter().any(|field| field.name == "onChange"));
        assert!(fields.iter().any(|field| field.name == "required"));
        assert!(fields.iter().any(|field| field.name == "inputType"));
    }

    #[test]
    fn exposes_focus_and_keyboard_events_from_the_shared_html_interface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        let keyboard_event = interface
            .exports
            .iter()
            .find(|export| export.namespace == "type" && export.name == "KeyboardEvent")
            .unwrap();
        assert_eq!(
            keyboard_event.declaration_kind.as_deref(),
            Some("opaque-struct")
        );
        let Some(InterfaceType::Record { fields, .. }) = &keyboard_event.representation else {
            panic!("KeyboardEvent must expose only immutable snapshot fields");
        };
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            [
                "key",
                "code",
                "repeat",
                "altKey",
                "controlKey",
                "metaKey",
                "shiftKey",
            ]
        );

        let button = interface
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "button")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &button.scheme.type_ref else {
            panic!("button must be callable");
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("button must accept a props record");
        };
        for name in ["onFocus", "onBlur", "onKeyDown", "onKeyUp"] {
            assert!(fields.iter().any(|field| field.name == name));
        }
    }

    #[test]
    fn exposes_pointer_scroll_and_event_control_surface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for name in ["EventAction", "MouseEvent", "PointerEvent", "ScrollEvent"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "type" && export.name == name));
        }
        for name in [
            "IgnoreEvent",
            "Dispatch",
            "DispatchPreventDefault",
            "DispatchStopPropagation",
            "DispatchPreventDefaultAndStop",
        ] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value"
                    && export.declaration_kind.as_deref() == Some("constructor")
                    && export.name == name));
        }

        let div = interface
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "div")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &div.scheme.type_ref else {
            panic!("div must be callable");
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("div must accept a props record");
        };
        for name in [
            "preventClickDefault",
            "stopClickPropagation",
            "onMouseDown",
            "onMouseUp",
            "onPointerDown",
            "onPointerUp",
            "onDoubleClick",
            "onContextMenu",
            "onScroll",
        ] {
            assert!(fields.iter().any(|field| field.name == name));
        }
    }

    #[test]
    fn exposes_standard_html_tags_with_their_children_contract() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for tag in STANDARD_HTML_TAGS {
            let name = tag.name;
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            assert_eq!(
                fields.iter().any(|field| field.name == "children"),
                !tag.void_element && tag.kind != StandardHtmlTagKind::Textarea,
                "std/web/html::{name} has the wrong children contract"
            );
        }
    }

    #[test]
    fn exposes_props_aliases_and_derives_diagnostic_metadata_from_the_same_records() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

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
            let alias = interface
                .exports
                .iter()
                .find(|export| export.namespace == "type" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            assert_eq!(alias.declaration_kind.as_deref(), Some("alias"));
            assert!(matches!(
                alias.representation,
                Some(InterfaceType::Record { .. })
            ));
        }

        let (image, fields) = standard_html_tag_props("img").unwrap();
        assert!(image.void_element);
        assert!(fields
            .iter()
            .any(|field| field.name == "src" && !field.optional));
        assert!(fields
            .iter()
            .any(|field| field.name == "alt" && !field.optional));
        assert!(!fields.iter().any(|field| field.name == "children"));
        assert!(standard_html_tag_props("custom").is_none());
    }

    #[test]
    fn exposes_tag_specific_link_and_media_props() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        for (namespace, name) in [
            ("type", "WebUrl"),
            ("value", "parseWebUrl"),
            ("value", "UnsafeWebUrlScheme"),
        ] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == namespace && export.name == name));
        }

        for name in ["a", "img", "picture", "source", "video", "audio", "link"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value" && export.name == name));
        }

        let props = |name: &str| {
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap();
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
        };

        assert!(props("a").contains(&"href"));
        assert!(!props("article").contains(&"href"));
        assert!(props("img").contains(&"alt"));
        assert!(!props("img").contains(&"children"));
        assert!(!props("a").contains(&"alt"));
        assert!(!props("source").contains(&"children"));
        for (tag, prop) in [
            ("a", "href"),
            ("link", "href"),
            ("img", "src"),
            ("source", "src"),
            ("video", "src"),
            ("audio", "src"),
        ] {
            let (_, fields) = standard_html_tag_props(tag).unwrap();
            let field = fields.iter().find(|field| field.name == prop).unwrap();
            assert!(matches!(
                &field.type_ref,
                InterfaceType::Named { name, arguments }
                    if name == "WebUrl" && arguments.is_empty()
            ));
        }
        assert!(is_standard_void_html_tag("img"));
        assert!(is_standard_void_html_tag("source"));
        assert!(!is_standard_void_html_tag("picture"));
    }

    #[test]
    fn exposes_form_table_and_interactive_tag_props() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        let props = |name: &str| {
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
        };

        for name in [
            "select", "option", "fieldset", "legend", "table", "thead", "tbody", "tfoot", "tr",
            "th", "td", "caption", "details", "summary", "dialog",
        ] {
            assert!(props(name).contains(&"children"));
        }
        for name in ["input", "textarea"] {
            assert!(props(name).contains(&"readOnly"));
            assert!(props(name).contains(&"autoFocus"));
        }
        assert!(props("input").contains(&"multiple"));
        assert!(props("input").contains(&"pattern"));
        assert!(props("textarea").contains(&"rows"));
        assert!(props("textarea").contains(&"cols"));
        assert!(props("select").contains(&"onChange"));
        assert!(props("option").contains(&"selected"));
        assert!(props("th").contains(&"colSpan"));
        assert!(props("td").contains(&"rowSpan"));
        assert!(props("details").contains(&"open"));
        assert!(props("dialog").contains(&"open"));
        assert!(!props("table").contains(&"colSpan"));
    }

    #[test]
    fn exposes_global_and_validated_custom_html_surface() {
        let target = standard_module_target("std/web/html").unwrap();
        let interface = target.interface();

        let props = |name: &str| {
            let export = interface
                .exports
                .iter()
                .find(|export| export.namespace == "value" && export.name == name)
                .unwrap_or_else(|| panic!("missing std/web/html::{name}"));
            let InterfaceType::Function { parameter, .. } = &export.scheme.type_ref else {
                panic!("std/web/html::{name} must be callable");
            };
            let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
                panic!("std/web/html::{name} must accept a props record");
            };
            fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
        };

        for tag in ["div", "img", "input", "table", "dialog"] {
            let fields = props(tag);
            for field in [
                "attributes",
                "role",
                "tabIndex",
                "lang",
                "dir",
                "draggable",
                "contentEditable",
            ] {
                assert!(fields.contains(&field), "std/web/html::{tag} lacks {field}");
            }
        }
        for name in ["Tag", "Attribute", "HtmlBuildError"] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "type" && export.name == name));
        }
        for name in [
            "customTag",
            "attribute",
            "custom",
            "InvalidTagName",
            "InvalidAttributeName",
            "ReservedAttributeName",
        ] {
            assert!(interface
                .exports
                .iter()
                .any(|export| export.namespace == "value" && export.name == name));
        }
    }

    #[test]
    fn exposes_svg_scene_events_and_dom_geometry_as_separate_surfaces() {
        let svg = standard_module_target("std/web/svg").expect("std/web/svg is available");
        let svg_names = svg
            .interface()
            .exports
            .iter()
            .map(|export| export.name.as_str())
            .collect::<Vec<_>>();
        for name in [
            "Svg",
            "svg",
            "g",
            "rect",
            "path",
            "circle",
            "line",
            "polyline",
            "polygon",
            "text",
            "viewBoxTarget",
            "transformTarget",
            "pathDataTarget",
            "pointsTarget",
            "fillTarget",
            "strokeTarget",
            "strokeWidthTarget",
            "pointerEventsTarget",
            "ariaLabelTarget",
            "xTarget",
            "yTarget",
            "x1Target",
            "y1Target",
            "x2Target",
            "y2Target",
            "cxTarget",
            "cyTarget",
            "radiusTarget",
            "rxTarget",
            "ryTarget",
            "widthTarget",
            "heightTarget",
            "opacityTarget",
            "toHtml",
        ] {
            assert!(svg_names.contains(&name), "missing std/web/svg::{name}");
        }

        let html = standard_module_target("std/web/html").unwrap();
        for name in [
            "ElementRef",
            "elementRef",
            "capturePointer",
            "releasePointer",
            "suppressCompatibilityClick",
        ] {
            assert!(
                html.interface()
                    .exports
                    .iter()
                    .any(|export| export.name == name),
                "missing std/web/html::{name}"
            );
        }
        assert!(html
            .interface()
            .exports
            .iter()
            .any(|export| export.name == "WheelEvent"));
        let div = html
            .interface()
            .exports
            .iter()
            .find(|export| export.namespace == "value" && export.name == "div")
            .unwrap();
        let InterfaceType::Function { parameter, .. } = &div.scheme.type_ref else {
            panic!("std/web/html::div must be callable")
        };
        let InterfaceType::Record { fields, .. } = parameter.as_ref() else {
            panic!("std/web/html::div must accept props")
        };
        for name in ["onPointerMove", "onPointerCancel", "onWheel"] {
            assert!(fields.iter().any(|field| field.name == name));
        }

        let dom = standard_module_target("std/web/dom").unwrap();
        let dom_names = dom
            .interface()
            .exports
            .iter()
            .map(|export| export.name.as_str())
            .collect::<Vec<_>>();
        for name in [
            "ElementRect",
            "DomObservation",
            "capturePointer",
            "releasePointer",
            "measure",
            "observeResize",
            "awaitObservation",
            "disconnect",
        ] {
            assert!(dom_names.contains(&name), "missing std/web/dom::{name}");
        }
    }
}
