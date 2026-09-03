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
    html_tag!("meta", VoidElement, void),
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
    ($specifier:literal, $interface:ident, $targets:expr) => {
        StandardModuleDefinition {
            specifier: $specifier,
            status: StandardModuleStatus::Available,
            targets: $targets,
            capability_services: &[],
            interface: Some($interface),
        }
    };
    ($specifier:literal, $interface:ident, $targets:expr, $services:expr) => {
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
    available_module!("std/number", number_interface, PORTABLE_TARGETS),
    available_module!("std/int", int_interface, PORTABLE_TARGETS),
    available_module!("std/float", float_interface, PORTABLE_TARGETS),
    available_module!("std/array", array_interface, PORTABLE_TARGETS),
    available_module!("std/list", list_interface, PORTABLE_TARGETS),
    available_module!("std/web/html", web_html_interface, PORTABLE_TARGETS),
    available_module!("std/web/file", web_file_interface, BROWSER_TARGET),
    available_module!(
        "std/web/navigation",
        web_navigation_interface,
        BROWSER_TARGET,
        &["std/web/navigation::Navigation"]
    ),
    available_module!(
        "std/web/storage",
        web_storage_interface,
        BROWSER_TARGET,
        &["std/web/storage::Storage"]
    ),
    available_module!(
        "std/web/dom",
        web_dom_interface,
        BROWSER_TARGET,
        &["std/web/dom::Dom"]
    ),
    available_module!("std/signal", signal_interface, PORTABLE_TARGETS),
    available_module!(
        "std/clock",
        clock_interface,
        PORTABLE_TARGETS,
        &["std/clock::Clock"]
    ),
    available_module!(
        "std/time",
        time_interface,
        PORTABLE_TARGETS,
        &["std/time::TimeZones"]
    ),
    available_module!(
        "std/http",
        http_client_interface,
        PORTABLE_TARGETS,
        &["std/http::HttpClient"]
    ),
    available_module!(
        "std/http/server",
        http_server_interface,
        PROCESS_TARGET,
        &["std/http/server::HttpServer"]
    ),
    available_module!(
        "std/http/multipart",
        http_multipart_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/sse", sse_interface, PORTABLE_TARGETS),
    available_module!(
        "std/websocket",
        websocket_client_interface,
        PORTABLE_TARGETS,
        &["std/websocket::WebSocketClient"]
    ),
    available_module!(
        "std/websocket/server",
        websocket_server_interface,
        PROCESS_TARGET,
        &["std/websocket/server::WebSocketServer"]
    ),
    contract_module!("std/benchmark", PORTABLE_TARGETS),
    contract_module!("std/big-int", PORTABLE_TARGETS),
    available_module!("std/bytes", bytes_interface, PORTABLE_TARGETS),
    contract_module!("std/bytes/base64", PORTABLE_TARGETS),
    contract_module!("std/bytes/hex", PORTABLE_TARGETS),
    available_module!("std/char", char_interface, PORTABLE_TARGETS),
    available_module!(
        "std/child-process",
        child_process_interface,
        PROCESS_TARGET,
        &["std/child-process::ChildProcesses"]
    ),
    available_module!(
        "std/collection",
        collection_core_interface,
        PORTABLE_TARGETS
    ),
    available_module!(
        "std/console",
        console_interface,
        PORTABLE_TARGETS,
        &["std/prelude::Console"]
    ),
    contract_module!("std/decimal", PORTABLE_TARGETS),
    available_module!("std/deferred", deferred_interface, PORTABLE_TARGETS),
    available_module!("std/effect", effect_interface, PORTABLE_TARGETS),
    available_module!("std/either", either_interface, PORTABLE_TARGETS),
    available_module!(
        "std/entropy",
        entropy_interface,
        PORTABLE_TARGETS,
        &["std/entropy::Entropy"]
    ),
    available_module!(
        "std/fs",
        filesystem_interface,
        PORTABLE_TARGETS,
        &["std/fs::FileSystem"]
    ),
    contract_module!(
        "std/http/bun",
        PROCESS_TARGET,
        &["std/http/bun::BunHttpServer"]
    ),
    contract_module!("std/iterator", PORTABLE_TARGETS),
    available_module!("std/json", json_interface, PORTABLE_TARGETS),
    available_module!(
        "std/log",
        logger_interface,
        PORTABLE_TARGETS,
        &["std/log::Logger"]
    ),
    available_module!("std/map", map_interface, PORTABLE_TARGETS),
    available_module!("std/maybe", maybe_interface, PORTABLE_TARGETS),
    available_module!(
        "std/non-empty-list",
        non_empty_list_interface,
        PORTABLE_TARGETS
    ),
    available_module!("std/path", path_interface, PORTABLE_TARGETS),
    available_module!(
        "std/process",
        process_interface,
        PROCESS_TARGET,
        &["std/process::Process"]
    ),
    available_module!("std/queue", queue_interface, PORTABLE_TARGETS),
    available_module!(
        "std/random",
        random_interface,
        PORTABLE_TARGETS,
        &["std/random::Random"]
    ),
    available_module!("std/ref", ref_interface, PORTABLE_TARGETS),
    contract_module!("std/regex", PORTABLE_TARGETS),
    available_module!("std/semaphore", semaphore_interface, PORTABLE_TARGETS),
    available_module!("std/set", set_interface, PORTABLE_TARGETS),
    available_module!(
        "std/stdin",
        stdin_interface,
        PROCESS_TARGET,
        &["std/prelude::Stdin"]
    ),
    available_module!("std/stream", stream_interface, PORTABLE_TARGETS),
    available_module!("std/test", test_interface, PORTABLE_TARGETS),
    available_module!("std/text", text_interface, PORTABLE_TARGETS),
    available_module!("std/text/grapheme", grapheme_interface, PORTABLE_TARGETS),
    available_module!("std/text/unicode", unicode_interface, PORTABLE_TARGETS),
    contract_module!("std/transformer/either", PORTABLE_TARGETS),
    contract_module!("std/transformer/maybe", PORTABLE_TARGETS),
    contract_module!("std/transformer/reader", PORTABLE_TARGETS),
    contract_module!("std/transformer/state", PORTABLE_TARGETS),
    contract_module!("std/transformer/writer", PORTABLE_TARGETS),
    available_module!("std/validation", validation_interface, PORTABLE_TARGETS),
];

fn console_interface() -> ModuleInterface {
    let module = "std/console";
    let console = || prelude_type("Console");
    let console_error = || prelude_type("ConsoleError");
    let environment = || record([required("console", console())]);
    let output = || effect(environment(), console_error(), named("Unit"));
    let mut exports = vec![
        canonical_type_export(module, "Console", "std/prelude::Console", 0, "opaque-type"),
        canonical_type_export(
            module,
            "ConsoleError",
            "std/prelude::ConsoleError",
            0,
            "opaque-type",
        ),
    ];
    for name in ["print", "println", "error", "errorLine"] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            output(),
        ));
    }
    exports.push(effect_function_export(
        module,
        "printValue",
        ["A"],
        vec![InterfaceConstraint {
            name: "Show".to_owned(),
            trait_identity: Some("std/prelude::Show".to_owned()),
            arguments: vec![named("A")],
        }],
        vec![named("A")],
        output(),
    ));
    exports.push(effect_function_export(
        module,
        "flush",
        [],
        Vec::new(),
        vec![named("Unit")],
        output(),
    ));
    standard_interface(module, exports)
}

fn logger_interface() -> ModuleInterface {
    let module = "std/log";
    let mut exports = vec![
        type_export(module, "Logger", 0, "opaque-type"),
        opaque_adt_type_export(module, "LogLevel", []),
    ];
    for constructor in ["LogTrace", "LogDebug", "LogInfo", "LogWarn", "LogFailure"] {
        exports.push(constructor_export(
            module,
            "LogLevel",
            constructor,
            [],
            None,
        ));
    }
    exports.push(opaque_adt_type_export(module, "LogValue", []));
    for (constructor, payload) in [
        ("LogString", named("String")),
        ("LogInt", named("Int")),
        ("LogFloat", named("Float")),
        ("LogBool", named("Bool")),
    ] {
        exports.push(constructor_export(
            module,
            "LogValue",
            constructor,
            [],
            Some(payload),
        ));
    }
    exports.extend([
        public_record_type_export(
            module,
            "LogEvent",
            [
                required("level", named("LogLevel")),
                required("message", named("String")),
                required(
                    "fields",
                    named_with(
                        "List",
                        vec![InterfaceType::Tuple {
                            elements: vec![named("String"), named("LogValue")],
                        }],
                    ),
                ),
            ],
        ),
        type_export(module, "LogError", 0, "opaque-type"),
        effect_function_export(
            module,
            "log",
            [],
            Vec::new(),
            vec![named("LogEvent")],
            effect(
                record([required("logger", named("Logger"))]),
                named("LogError"),
                named("Unit"),
            ),
        ),
    ]);
    standard_interface(module, exports)
}

fn stdin_interface() -> ModuleInterface {
    let module = "std/stdin";
    let stdin = || prelude_type("Stdin");
    let stdin_error = || prelude_type("StdinError");
    let environment = || record([required("stdin", stdin())]);
    let bytes = || {
        external_type(
            "Bytes",
            "std/bytes::Bytes",
            "std/bytes",
            "Bytes",
            Vec::new(),
        )
    };
    let stream = |value| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment(), stdin_error(), value],
        )
    };
    let mut exports = vec![
        canonical_type_export(module, "Stdin", "std/prelude::Stdin", 0, "opaque-type"),
        opaque_adt_type_export(module, "StdinConfigError", []),
    ];
    for (constructor, payload) in [
        ("NonPositiveReadSize", named("Int")),
        ("ReadSizeTooLarge", named("Int")),
        ("NonPositiveLineLimit", named("Int")),
        ("LineLimitTooLarge", named("Int")),
    ] {
        exports.push(constructor_export(
            module,
            "StdinConfigError",
            constructor,
            [],
            Some(payload),
        ));
    }
    exports.extend([
        type_export(module, "ReadSize", 0, "opaque-type"),
        type_export(module, "LineLimit", 0, "opaque-type"),
        function_export(
            module,
            "readSize",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with("Either", vec![named("StdinConfigError"), named("ReadSize")]),
        ),
        function_export(
            module,
            "lineLimit",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("StdinConfigError"), named("LineLimit")],
            ),
        ),
        function_export(
            module,
            "defaultReadSize",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("ReadSize"),
        ),
        function_export(
            module,
            "defaultLineLimit",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("LineLimit"),
        ),
        canonical_type_export(
            module,
            "StdinError",
            "std/prelude::StdinError",
            0,
            "opaque-type",
        ),
    ]);
    for (constructor, payload) in [
        ("StdinUnavailable", None),
        ("StdinReadFailure", None),
        ("ConcurrentStdinRead", None),
        (
            "InvalidStdinUtf8",
            Some(record([required("offset", named("Int"))])),
        ),
        (
            "StdinLineTooLong",
            Some(record([required("limitBytes", named("Int"))])),
        ),
        ("StdinPositionOverflow", None),
    ] {
        exports.push(canonical_constructor_export(
            module,
            "StdinError",
            "std/prelude::StdinError",
            constructor,
            payload,
        ));
    }
    exports.extend([
        effect_function_export(
            module,
            "readChunk",
            [],
            Vec::new(),
            vec![named("ReadSize")],
            effect(
                environment(),
                stdin_error(),
                named_with("Maybe", vec![bytes()]),
            ),
        ),
        effect_function_export(
            module,
            "readLine",
            [],
            Vec::new(),
            vec![named("Unit")],
            effect(
                environment(),
                stdin_error(),
                named_with("Maybe", vec![named("String")]),
            ),
        ),
        effect_function_export(
            module,
            "readLineWith",
            [],
            Vec::new(),
            vec![named("LineLimit")],
            effect(
                environment(),
                stdin_error(),
                named_with("Maybe", vec![named("String")]),
            ),
        ),
        function_export(
            module,
            "lines",
            [],
            Vec::new(),
            vec![named("LineLimit")],
            stream(named("String")),
        ),
    ]);
    standard_interface(module, exports)
}

fn prelude_type(name: &str) -> InterfaceType {
    external_type(
        name,
        &format!("std/prelude::{name}"),
        "std/prelude",
        name,
        Vec::new(),
    )
}

fn path_interface() -> ModuleInterface {
    let module = "std/path";
    let path = named("Path");
    let path_result = named_with("Either", vec![named("PathError"), path.clone()]);
    standard_interface(
        module,
        vec![
            type_export(module, "Path", 0, "opaque-type"),
            opaque_adt_type_export(module, "PathError", []),
            constructor_export(module, "PathError", "EmptyPath", [], None),
            constructor_export(
                module,
                "PathError",
                "PathContainsNul",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(
                module,
                "PathError",
                "PathContainsBackslash",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(module, "PathError", "InvalidDriveRoot", [], None),
            constructor_export(module, "PathError", "InvalidUncRoot", [], None),
            constructor_export(
                module,
                "PathError",
                "InvalidPathSegment",
                [],
                Some(named("String")),
            ),
            constructor_export(module, "PathError", "AbsoluteChildPath", [], None),
            function_export(
                module,
                "parse",
                [],
                Vec::new(),
                vec![named("String")],
                path_result.clone(),
            ),
            function_export(
                module,
                "render",
                [],
                Vec::new(),
                vec![path.clone()],
                named("String"),
            ),
            function_export(
                module,
                "current",
                [],
                Vec::new(),
                vec![named("Unit")],
                path.clone(),
            ),
            function_export(
                module,
                "isAbsolute",
                [],
                Vec::new(),
                vec![path.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "normalize",
                [],
                Vec::new(),
                vec![path.clone()],
                path.clone(),
            ),
            function_export(
                module,
                "join",
                [],
                Vec::new(),
                vec![path.clone(), path.clone()],
                path_result.clone(),
            ),
            function_export(
                module,
                "child",
                [],
                Vec::new(),
                vec![named("String"), path.clone()],
                path_result,
            ),
            function_export(
                module,
                "parent",
                [],
                Vec::new(),
                vec![path.clone()],
                named_with("Maybe", vec![path.clone()]),
            ),
            function_export(
                module,
                "fileName",
                [],
                Vec::new(),
                vec![path.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "extension",
                [],
                Vec::new(),
                vec![path],
                named_with("Maybe", vec![named("String")]),
            ),
        ],
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

fn map_interface() -> ModuleInterface {
    let module = "std/map";
    let map = |key: &str, value: &str| named_with("Map", vec![named(key), named(value)]);
    let pair = || InterfaceType::Tuple {
        elements: vec![named("K"), named("V")],
    };
    let mut exports = vec![
        type_export(module, "Map", 2, "opaque-type"),
        function_export(
            module,
            "empty",
            ["K", "V"],
            vec![],
            vec![named("Unit")],
            map("K", "V"),
        ),
        function_export(
            module,
            "singleton",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), named("V")],
            map("K", "V"),
        ),
        function_export(
            module,
            "fromEntries",
            ["C", "K", "V"],
            {
                let mut constraints =
                    vec![collection_constraint("Iterable", vec![named("C"), pair()])];
                constraints.extend(key_constraints("K"));
                constraints
            },
            vec![named("C")],
            map("K", "V"),
        ),
        function_export(
            module,
            "get",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), map("K", "V")],
            named_with("Maybe", vec![named("V")]),
        ),
        function_export(
            module,
            "containsKey",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), map("K", "V")],
            named("Bool"),
        ),
        function_export(
            module,
            "insert",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), named("V"), map("K", "V")],
            map("K", "V"),
        ),
        function_export(
            module,
            "upsert",
            ["K", "V"],
            key_constraints("K"),
            vec![
                named("K"),
                function_type(vec![named_with("Maybe", vec![named("V")])], named("V")),
                map("K", "V"),
            ],
            map("K", "V"),
        ),
        function_export(
            module,
            "remove",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), map("K", "V")],
            map("K", "V"),
        ),
        function_export(
            module,
            "filter",
            ["K", "V"],
            vec![],
            vec![
                function_type(vec![named("K"), named("V")], named("Bool")),
                map("K", "V"),
            ],
            map("K", "V"),
        ),
        function_export(
            module,
            "mapValues",
            ["K", "A", "B"],
            vec![],
            vec![function_type(vec![named("A")], named("B")), map("K", "A")],
            map("K", "B"),
        ),
        function_export(
            module,
            "mapKeysWith",
            ["K1", "K2", "V"],
            key_constraints("K2"),
            vec![
                function_type(vec![named("V"), named("V")], named("V")),
                function_type(vec![named("K1")], named("K2")),
                map("K1", "V"),
            ],
            map("K2", "V"),
        ),
        function_export(
            module,
            "mergeWith",
            ["K", "V"],
            key_constraints("K"),
            vec![
                function_type(vec![named("V"), named("V")], named("V")),
                map("K", "V"),
                map("K", "V"),
            ],
            map("K", "V"),
        ),
        function_export(
            module,
            "keys",
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named_with("Array", vec![named("K")]),
        ),
        function_export(
            module,
            "values",
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named_with("Array", vec![named("V")]),
        ),
        function_export(
            module,
            "entries",
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named_with("Array", vec![pair()]),
        ),
    ];
    for (name, result) in [("size", "Int"), ("isEmpty", "Bool")] {
        exports.push(function_export(
            module,
            name,
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named(result),
        ));
    }
    standard_interface(module, exports)
}

fn set_interface() -> ModuleInterface {
    let module = "std/set";
    let set = |element: &str| named_with("Set", vec![named(element)]);
    let mut exports = vec![
        type_export(module, "Set", 1, "opaque-type"),
        function_export(
            module,
            "empty",
            ["A"],
            vec![],
            vec![named("Unit")],
            set("A"),
        ),
        function_export(
            module,
            "singleton",
            ["A"],
            key_constraints("A"),
            vec![named("A")],
            set("A"),
        ),
        function_export(
            module,
            "fromIterable",
            ["C", "A"],
            {
                let mut constraints = vec![collection_constraint(
                    "Iterable",
                    vec![named("C"), named("A")],
                )];
                constraints.extend(key_constraints("A"));
                constraints
            },
            vec![named("C")],
            set("A"),
        ),
        function_export(
            module,
            "contains",
            ["A"],
            key_constraints("A"),
            vec![named("A"), set("A")],
            named("Bool"),
        ),
        function_export(
            module,
            "filter",
            ["A"],
            vec![],
            vec![function_type(vec![named("A")], named("Bool")), set("A")],
            set("A"),
        ),
        function_export(
            module,
            "map",
            ["A", "B"],
            key_constraints("B"),
            vec![function_type(vec![named("A")], named("B")), set("A")],
            set("B"),
        ),
        function_export(
            module,
            "isSubsetOf",
            ["A"],
            key_constraints("A"),
            vec![set("A"), set("A")],
            named("Bool"),
        ),
    ];
    for name in ["insert", "remove"] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            key_constraints("A"),
            vec![named("A"), set("A")],
            set("A"),
        ));
    }
    for name in ["union", "intersection", "difference"] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            key_constraints("A"),
            vec![set("A"), set("A")],
            set("A"),
        ));
    }
    for (name, result) in [("toArray", "Array"), ("toList", "List")] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            vec![],
            vec![set("A")],
            named_with(result, vec![named("A")]),
        ));
    }
    for (name, result) in [("size", "Int"), ("isEmpty", "Bool")] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            vec![],
            vec![set("A")],
            named(result),
        ));
    }
    standard_interface(module, exports)
}

fn non_empty_list_interface() -> ModuleInterface {
    let module = "std/non-empty-list";
    let list = || named_with("List", vec![named("A")]);
    let non_empty = || named_with("NonEmptyList", vec![named("A")]);
    let maybe_non_empty = || named_with("Maybe", vec![non_empty()]);
    standard_interface(
        module,
        vec![
            type_export(module, "NonEmptyList", 1, "opaque-type"),
            function_export(
                module,
                "singleton",
                ["A"],
                Vec::new(),
                vec![named("A")],
                non_empty(),
            ),
            function_export(
                module,
                "cons",
                ["A"],
                Vec::new(),
                vec![named("A"), list()],
                non_empty(),
            ),
            function_export(
                module,
                "fromList",
                ["A"],
                Vec::new(),
                vec![list()],
                maybe_non_empty(),
            ),
            function_export(
                module,
                "toList",
                ["A"],
                Vec::new(),
                vec![non_empty()],
                list(),
            ),
            function_export(
                module,
                "head",
                ["A"],
                Vec::new(),
                vec![non_empty()],
                named("A"),
            ),
            function_export(module, "tail", ["A"], Vec::new(), vec![non_empty()], list()),
            function_export(
                module,
                "reduce1",
                ["A"],
                Vec::new(),
                vec![
                    function_type(vec![named("A"), named("A")], named("A")),
                    non_empty(),
                ],
                named("A"),
            ),
        ],
    )
}

fn process_interface() -> ModuleInterface {
    let module = "std/process";
    let process = || named("Process");
    let process_error = || named("ProcessError");
    let signal = || named("ProcessSignal");
    let environment = || record([required("process", process())]);
    let path = || external_type("Path", "std/path::Path", "std/path", "Path", vec![]);
    let non_empty_signals = || {
        external_type(
            "NonEmptyList",
            "std/non-empty-list::NonEmptyList",
            "std/non-empty-list",
            "NonEmptyList",
            vec![signal()],
        )
    };
    let stream = || {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment(), process_error(), signal()],
        )
    };
    let effect = |success| effect(environment(), process_error(), success);
    let mut exports = vec![
        type_export(module, "Process", 0, "opaque-type"),
        opaque_adt_type_export(module, "ProcessSignal", []),
    ];
    for constructor in ["Interrupt", "Terminate", "Hangup", "Quit", "User1", "User2"] {
        exports.push(constructor_export(
            module,
            "ProcessSignal",
            constructor,
            [],
            None,
        ));
    }
    exports.push(opaque_adt_type_export(module, "ProcessError", []));
    for (constructor, payload) in [
        ("UnsupportedProcessSignal", Some(signal())),
        ("ReservedProcessSignal", Some(signal())),
        ("InvalidArgumentEncoding", Some(named("Int"))),
        ("InvalidEnvironmentName", Some(named("String"))),
        ("InvalidEnvironmentEncoding", Some(named("String"))),
        ("CurrentDirectoryUnavailable", None),
    ] {
        exports.push(constructor_export(
            module,
            "ProcessError",
            constructor,
            [],
            payload,
        ));
    }
    exports.extend([
        effect_function_export(
            module,
            "arguments",
            [],
            Vec::new(),
            vec![named("Unit")],
            effect(named_with("Array", vec![named("String")])),
        ),
        effect_function_export(
            module,
            "environment",
            [],
            Vec::new(),
            vec![named("String")],
            effect(named_with("Maybe", vec![named("String")])),
        ),
        effect_function_export(
            module,
            "currentDirectory",
            [],
            Vec::new(),
            vec![named("Unit")],
            effect(path()),
        ),
        function_export(
            module,
            "signals",
            [],
            Vec::new(),
            vec![non_empty_signals()],
            stream(),
        ),
    ]);
    standard_interface(module, exports)
}

fn filesystem_interface() -> ModuleInterface {
    let module = "std/fs";
    let path = || external_type("Path", "std/path::Path", "std/path", "Path", Vec::new());
    let instant = || {
        external_type(
            "Instant",
            "std/time::Instant",
            "std/time",
            "Instant",
            Vec::new(),
        )
    };
    let bytes = || {
        external_type(
            "Bytes",
            "std/bytes::Bytes",
            "std/bytes",
            "Bytes",
            Vec::new(),
        )
    };
    let utf8_error = || {
        external_type(
            "Utf8DecodeError",
            "std/text::Utf8DecodeError",
            "std/text",
            "Utf8DecodeError",
            Vec::new(),
        )
    };
    let buffer_capacity = || {
        external_type(
            "BufferCapacity",
            "std/stream::BufferCapacity",
            "std/stream",
            "BufferCapacity",
            Vec::new(),
        )
    };
    let stream = |environment, failure, value| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment, failure, value],
        )
    };
    let environment = || record([required("fileSystem", named("FileSystem"))]);
    let fs_effect = |success| effect(environment(), named("FileSystemError"), success);
    let mut exports = vec![
        type_export(module, "FileSystem", 0, "opaque-type"),
        opaque_adt_type_export(module, "FileType", []),
    ];
    for constructor in ["RegularFile", "Directory", "SymbolicLink", "OtherFileType"] {
        exports.push(constructor_export(
            module,
            "FileType",
            constructor,
            [],
            None,
        ));
    }
    exports.push(opaque_adt_type_export(module, "FileSystemOperation", []));
    for constructor in [
        "ReadFile",
        "WriteFile",
        "OpenDirectory",
        "ReadMetadata",
        "CreateDirectory",
        "RemovePath",
        "MovePath",
        "CanonicalizePath",
        "CreateTemporary",
    ] {
        exports.push(constructor_export(
            module,
            "FileSystemOperation",
            constructor,
            [],
            None,
        ));
    }
    exports.push(opaque_adt_type_export(module, "FileSystemErrorKind", []));
    for constructor in [
        "FileNotFound",
        "FileAlreadyExists",
        "PermissionDenied",
        "NotADirectory",
        "IsADirectory",
        "DirectoryNotEmpty",
        "SymbolicLinkLoop",
        "CrossDeviceMove",
        "PathNotSupported",
        "FileSystemUnavailable",
    ] {
        exports.push(constructor_export(
            module,
            "FileSystemErrorKind",
            constructor,
            [],
            None,
        ));
    }
    exports.push(constructor_export(
        module,
        "FileSystemErrorKind",
        "OtherFileSystemError",
        [],
        Some(named("String")),
    ));
    exports.extend([
        public_record_type_export(
            module,
            "FileSystemError",
            [
                required("operation", named("FileSystemOperation")),
                required("path", path()),
                required("otherPath", named_with("Maybe", vec![path()])),
                required("kind", named("FileSystemErrorKind")),
            ],
        ),
        public_record_type_export(
            module,
            "FileMetadata",
            [
                required("fileType", named("FileType")),
                required("sizeBytes", named("Int")),
                required("modified", named_with("Maybe", vec![instant()])),
                required("created", named_with("Maybe", vec![instant()])),
            ],
        ),
        public_record_type_export(
            module,
            "DirectoryEntry",
            [
                required("name", named("String")),
                required("path", path()),
                required("fileType", named_with("Maybe", vec![named("FileType")])),
            ],
        ),
        opaque_adt_type_export(module, "WriteMode", []),
    ]);
    for constructor in ["Replace", "CreateNew", "Append"] {
        exports.push(constructor_export(
            module,
            "WriteMode",
            constructor,
            [],
            None,
        ));
    }
    exports.extend([
        opaque_adt_type_export(module, "FileTextError", []),
        constructor_export(
            module,
            "FileTextError",
            "FileAccessFailure",
            [],
            Some(named("FileSystemError")),
        ),
        constructor_export(
            module,
            "FileTextError",
            "FileUtf8Failure",
            [],
            Some(utf8_error()),
        ),
    ]);
    for (name, result) in [
        ("exists", named("Bool")),
        ("metadata", named("FileMetadata")),
        ("symlinkMetadata", named("FileMetadata")),
        ("canonicalize", path()),
        ("readBytes", bytes()),
    ] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![path()],
            fs_effect(result),
        ));
    }
    exports.push(effect_function_export(
        module,
        "readTextUtf8",
        [],
        Vec::new(),
        vec![path()],
        effect(environment(), named("FileTextError"), named("String")),
    ));
    exports.push(function_export(
        module,
        "readChunks",
        [],
        Vec::new(),
        vec![buffer_capacity(), path()],
        stream(environment(), named("FileSystemError"), bytes()),
    ));
    for name in ["writeBytes", "writeTextUtf8"] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![
                named("WriteMode"),
                if name == "writeBytes" {
                    bytes()
                } else {
                    named("String")
                },
                path(),
            ],
            fs_effect(named("Unit")),
        ));
    }
    exports.push(effect_function_export(
        module,
        "writeChunks",
        ["R", "E"],
        Vec::new(),
        vec![
            named("WriteMode"),
            stream(named("R"), named("E"), bytes()),
            path(),
        ],
        effect(
            requirement_merge(vec![named("R"), environment()]),
            named_with("Either", vec![named("E"), named("FileSystemError")]),
            named("Unit"),
        ),
    ));
    exports.push(effect_function_export(
        module,
        "writeAtomic",
        [],
        Vec::new(),
        vec![bytes(), path()],
        fs_effect(named("Unit")),
    ));
    exports.push(function_export(
        module,
        "list",
        [],
        Vec::new(),
        vec![path()],
        stream(
            environment(),
            named("FileSystemError"),
            named("DirectoryEntry"),
        ),
    ));
    for name in [
        "createDirectory",
        "createDirectories",
        "removeFile",
        "removeDirectory",
    ] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![path()],
            fs_effect(named("Unit")),
        ));
    }
    exports.push(effect_function_export(
        module,
        "move",
        [],
        Vec::new(),
        vec![path(), path()],
        fs_effect(named("Unit")),
    ));
    for name in ["withTemporaryDirectory", "withTemporaryFile"] {
        exports.push(effect_function_export(
            module,
            name,
            ["R", "E", "A"],
            Vec::new(),
            vec![
                named("String"),
                function_type(vec![path()], effect(named("R"), named("E"), named("A"))),
            ],
            effect(
                requirement_merge(vec![named("R"), environment()]),
                named_with("Either", vec![named("FileSystemError"), named("E")]),
                named("A"),
            ),
        ));
    }
    standard_interface(module, exports)
}

fn child_process_interface() -> ModuleInterface {
    let module = "std/child-process";
    let path = || external_type("Path", "std/path::Path", "std/path", "Path", Vec::new());
    let bytes = || {
        external_type(
            "Bytes",
            "std/bytes::Bytes",
            "std/bytes",
            "Bytes",
            Vec::new(),
        )
    };
    let duration = || {
        external_type(
            "Duration",
            "std/time::Duration",
            "std/time",
            "Duration",
            Vec::new(),
        )
    };
    let buffer_capacity = || {
        external_type(
            "BufferCapacity",
            "std/stream::BufferCapacity",
            "std/stream",
            "BufferCapacity",
            Vec::new(),
        )
    };
    let stream = |environment, failure, value| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment, failure, value],
        )
    };
    let process_signal = || {
        external_type(
            "ProcessSignal",
            "std/process::ProcessSignal",
            "std/process",
            "ProcessSignal",
            Vec::new(),
        )
    };
    let either = |left, right| named_with("Either", vec![left, right]);
    let environment = || record([required("childProcesses", named("ChildProcesses"))]);
    let mut exports = vec![
        type_export(module, "ChildProcesses", 0, "opaque-type"),
        opaque_adt_type_export(module, "Executable", []),
        constructor_export(
            module,
            "Executable",
            "SearchPath",
            [],
            Some(named("String")),
        ),
        constructor_export(module, "Executable", "ExecutablePath", [], Some(path())),
        type_export(module, "Command", 0, "opaque-type"),
        type_export(module, "CaptureLimit", 0, "opaque-type"),
        opaque_adt_type_export(module, "ChildProcessConfigError", []),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "EmptyExecutableName",
            [],
            None,
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "ExecutableNameContainsSeparator",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "ArgumentContainsNul",
            [],
            Some(record([
                required("index", named("Int")),
                required("offset", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "EnvironmentNameContainsNul",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "EnvironmentValueContainsNul",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "InvalidCaptureLimit",
            [],
            Some(named("Int")),
        ),
        opaque_adt_type_export(module, "ChildOutputChannel", []),
        constructor_export(module, "ChildOutputChannel", "ChildStdout", [], None),
        constructor_export(module, "ChildOutputChannel", "ChildStderr", [], None),
        opaque_adt_type_export(module, "ChildProcessError", []),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildSpawnFailed",
            [],
            Some(record([
                required("executable", named("Executable")),
                required("detail", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildInputAfterClose",
            [],
            None,
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildOutputReadFailed",
            [],
            Some(record([
                required("channel", named("ChildOutputChannel")),
                required("detail", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "UnsupportedChildSignal",
            [],
            Some(process_signal()),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildInputFailed",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildOutputLimitExceeded",
            [],
            Some(record([
                required("channel", named("ChildOutputChannel")),
                required("limitBytes", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildWaitFailed",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildTerminationFailed",
            [],
            Some(named("String")),
        ),
        opaque_adt_type_export(module, "ChildExitStatus", []),
        constructor_export(
            module,
            "ChildExitStatus",
            "ChildExited",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "ChildExitStatus",
            "ChildSignaled",
            [],
            Some(process_signal()),
        ),
        constructor_export(
            module,
            "ChildExitStatus",
            "ChildHostTerminated",
            [],
            Some(named("String")),
        ),
        opaque_adt_type_export(module, "ChildInput", []),
        constructor_export(module, "ChildInput", "WriteChildStdin", [], Some(bytes())),
        constructor_export(module, "ChildInput", "CloseChildStdin", [], None),
        constructor_export(
            module,
            "ChildInput",
            "SignalChild",
            [],
            Some(process_signal()),
        ),
        constructor_export(module, "ChildInput", "KillChild", [], None),
        opaque_adt_type_export(module, "ChildEvent", []),
        constructor_export(module, "ChildEvent", "ChildStdoutChunk", [], Some(bytes())),
        constructor_export(module, "ChildEvent", "ChildStderrChunk", [], Some(bytes())),
        constructor_export(
            module,
            "ChildEvent",
            "ChildExitedWith",
            [],
            Some(named("ChildExitStatus")),
        ),
        public_record_type_export(
            module,
            "CapturedProcess",
            [
                required("status", named("ChildExitStatus")),
                required("stdout", bytes()),
                required("stderr", bytes()),
            ],
        ),
        function_export(
            module,
            "command",
            [],
            Vec::new(),
            vec![named("Executable")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "addArgument",
            [],
            Vec::new(),
            vec![named("String"), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "addArguments",
            [],
            Vec::new(),
            vec![named_with("Array", vec![named("String")]), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "inDirectory",
            [],
            Vec::new(),
            vec![path(), named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "setEnvironment",
            [],
            Vec::new(),
            vec![named("String"), named("String"), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "unsetEnvironment",
            [],
            Vec::new(),
            vec![named("String"), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "clearEnvironment",
            [],
            Vec::new(),
            vec![named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "terminationGrace",
            [],
            Vec::new(),
            vec![duration(), named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "outputBuffer",
            [],
            Vec::new(),
            vec![buffer_capacity(), named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "captureLimit",
            [],
            Vec::new(),
            vec![named("Int")],
            either(named("ChildProcessConfigError"), named("CaptureLimit")),
        ),
        function_export(
            module,
            "defaultCaptureLimit",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("CaptureLimit"),
        ),
        function_export(
            module,
            "runStreaming",
            ["R", "E"],
            Vec::new(),
            vec![
                stream(named("R"), named("E"), named("ChildInput")),
                named("Command"),
            ],
            stream(
                requirement_merge(vec![named("R"), environment()]),
                either(named("E"), named("ChildProcessError")),
                named("ChildEvent"),
            ),
        ),
        effect_function_export(
            module,
            "runCaptured",
            [],
            Vec::new(),
            vec![named("CaptureLimit"), bytes(), named("Command")],
            effect(
                environment(),
                named("ChildProcessError"),
                named("CapturedProcess"),
            ),
        ),
        effect_function_export(
            module,
            "runInherited",
            [],
            Vec::new(),
            vec![named("Command")],
            effect(
                environment(),
                named("ChildProcessError"),
                named("ChildExitStatus"),
            ),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}

fn clock_interface() -> ModuleInterface {
    let module = "std/clock";
    standard_interface(
        module,
        vec![
            type_export(module, "Clock", 0, "opaque-type"),
            function_export(
                module,
                "now",
                [],
                Vec::new(),
                vec![named("Unit")],
                effect(
                    record([required("clock", named("Clock"))]),
                    named("Never"),
                    external_type(
                        "Instant",
                        "std/time::Instant",
                        "std/time",
                        "Instant",
                        Vec::new(),
                    ),
                ),
            ),
            effect_function_export(
                module,
                "sleep",
                [],
                Vec::new(),
                vec![external_type(
                    "Duration",
                    "std/time::Duration",
                    "std/time",
                    "Duration",
                    Vec::new(),
                )],
                effect(
                    record([required("clock", named("Clock"))]),
                    named("Never"),
                    named("Unit"),
                ),
            ),
        ],
    )
}

fn random_interface() -> ModuleInterface {
    let module = "std/random";
    let environment = || record([required("random", named("Random"))]);
    let infallible = |success| effect(environment(), named("Never"), success);
    let fallible = |success| effect(environment(), named("RandomRangeError"), success);
    let mut exports = vec![
        type_export(module, "Random", 0, "opaque-type"),
        opaque_adt_type_export(module, "RandomRangeError", []),
        constructor_export(
            module,
            "RandomRangeError",
            "EmptyRandomIntRange",
            [],
            Some(record([
                required("lower", named("Int")),
                required("upperExclusive", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "RandomRangeError",
            "InvalidProbability",
            [],
            Some(named("Float")),
        ),
        opaque_adt_type_export(module, "RandomConfigError", []),
        constructor_export(
            module,
            "RandomConfigError",
            "NonPositiveRandomSize",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "RandomConfigError",
            "RandomSizeTooLarge",
            [],
            Some(named("Int")),
        ),
        type_export(module, "RandomSize", 0, "opaque-type"),
        function_export(
            module,
            "randomSize",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("RandomConfigError"), named("RandomSize")],
            ),
        ),
        function_export(
            module,
            "algorithmId",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("String")),
        ),
        function_export(
            module,
            "nextBool",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("Bool")),
        ),
        function_export(
            module,
            "nextInt",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("Int")),
        ),
        function_export(
            module,
            "intBetween",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            fallible(named("Int")),
        ),
        function_export(
            module,
            "unitFloat",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("Float")),
        ),
        function_export(
            module,
            "chance",
            [],
            Vec::new(),
            vec![named("Float")],
            fallible(named("Bool")),
        ),
        function_export(
            module,
            "randomBytes",
            [],
            Vec::new(),
            vec![named("RandomSize")],
            infallible(external_type(
                "Bytes",
                "std/bytes::Bytes",
                "std/bytes",
                "Bytes",
                Vec::new(),
            )),
        ),
        function_export(
            module,
            "choose",
            ["A"],
            Vec::new(),
            vec![external_type(
                "NonEmptyList",
                "std/non-empty-list::NonEmptyList",
                "std/non-empty-list",
                "NonEmptyList",
                vec![named("A")],
            )],
            infallible(named("A")),
        ),
        function_export(
            module,
            "shuffle",
            ["A"],
            Vec::new(),
            vec![named_with("Array", vec![named("A")])],
            infallible(named_with("Array", vec![named("A")])),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}

fn entropy_interface() -> ModuleInterface {
    let module = "std/entropy";
    let mut exports = vec![
        type_export(module, "Entropy", 0, "opaque-type"),
        opaque_adt_type_export(module, "EntropyConfigError", []),
        constructor_export(
            module,
            "EntropyConfigError",
            "NonPositiveEntropySize",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "EntropyConfigError",
            "EntropySizeTooLarge",
            [],
            Some(named("Int")),
        ),
        opaque_adt_type_export(module, "EntropyError", []),
        constructor_export(module, "EntropyError", "EntropyUnavailable", [], None),
        constructor_export(module, "EntropyError", "EntropyReadFailure", [], None),
        type_export(module, "EntropySize", 0, "opaque-type"),
        function_export(
            module,
            "entropySize",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("EntropyConfigError"), named("EntropySize")],
            ),
        ),
        function_export(
            module,
            "secureBytes",
            [],
            Vec::new(),
            vec![named("EntropySize")],
            effect(
                record([required("entropy", named("Entropy"))]),
                named("EntropyError"),
                external_type(
                    "Bytes",
                    "std/bytes::Bytes",
                    "std/bytes",
                    "Bytes",
                    Vec::new(),
                ),
            ),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}

fn time_interface() -> ModuleInterface {
    let module = "std/time";
    let duration = named("Duration");
    let duration_result = named_with("Either", vec![named("DurationError"), duration.clone()]);
    let date = named("LocalDate");
    let time = named("LocalTime");
    let local = named("LocalDateTime");
    let offset = named("UtcOffset");
    let offset_date_time = named("OffsetDateTime");
    let zone = named("TimeZone");
    let zoned = named("ZonedDateTime");
    let date_result = |success| named_with("Either", vec![named("DateTimeError"), success]);
    let timezone_effect = |failure, success| {
        effect(
            record([required("timeZones", named("TimeZones"))]),
            failure,
            success,
        )
    };
    let mut exports = vec![
        type_export(module, "Instant", 0, "opaque-type"),
        type_export(module, "Duration", 0, "opaque-type"),
        opaque_adt_type_export(module, "DurationError", []),
        constructor_export(
            module,
            "DurationError",
            "NegativeDuration",
            [],
            Some(named("Int")),
        ),
        constructor_export(module, "DurationError", "DurationOutsideRange", [], None),
        function_export(
            module,
            "zeroDuration",
            [],
            Vec::new(),
            vec![named("Unit")],
            duration.clone(),
        ),
        function_export(
            module,
            "nanoseconds",
            [],
            Vec::new(),
            vec![named("Int")],
            duration_result.clone(),
        ),
        function_export(
            module,
            "milliseconds",
            [],
            Vec::new(),
            vec![named("Int")],
            duration_result.clone(),
        ),
        function_export(
            module,
            "seconds",
            [],
            Vec::new(),
            vec![named("Int")],
            duration_result.clone(),
        ),
        function_export(
            module,
            "minutes",
            [],
            Vec::new(),
            vec![named("Int")],
            duration_result.clone(),
        ),
        function_export(
            module,
            "hours",
            [],
            Vec::new(),
            vec![named("Int")],
            duration_result,
        ),
        function_export(
            module,
            "toNanoseconds",
            [],
            Vec::new(),
            vec![duration.clone()],
            named("Int"),
        ),
        function_export(
            module,
            "addDuration",
            [],
            Vec::new(),
            vec![duration.clone(), duration],
            named_with("Either", vec![named("DurationError"), named("Duration")]),
        ),
        type_export(module, "LocalDate", 0, "opaque-type"),
        type_export(module, "LocalTime", 0, "opaque-type"),
        type_export(module, "LocalDateTime", 0, "opaque-type"),
        type_export(module, "UtcOffset", 0, "opaque-type"),
        type_export(module, "OffsetDateTime", 0, "opaque-type"),
        type_export(module, "TimeZone", 0, "opaque-type"),
        type_export(module, "ZonedDateTime", 0, "opaque-type"),
        type_export(module, "TimeZones", 0, "opaque-type"),
        opaque_adt_type_export(module, "DateTimeError", []),
        constructor_export(
            module,
            "DateTimeError",
            "InvalidDate",
            [],
            Some(record([
                required("year", named("Int")),
                required("month", named("Int")),
                required("day", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "DateTimeError",
            "InvalidTime",
            [],
            Some(record([
                required("hour", named("Int")),
                required("minute", named("Int")),
                required("second", named("Int")),
                required("nanosecond", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "DateTimeError",
            "InvalidUtcOffsetSeconds",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "DateTimeError",
            "InvalidDateTimeText",
            [],
            Some(record([required("offset", named("Int"))])),
        ),
        opaque_adt_type_export(module, "TimeZoneError", []),
        constructor_export(
            module,
            "TimeZoneError",
            "UnknownTimeZone",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "TimeZoneError",
            "TimeZoneDatabaseUnavailable",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "TimeZoneError",
            "TimeZoneDatabaseVersionMismatch",
            [],
            Some(record([
                required("required", named("String")),
                required("actual", named("String")),
            ])),
        ),
        opaque_adt_type_export(module, "LocalResolution", []),
        constructor_export(module, "LocalResolution", "Unique", [], Some(zoned.clone())),
        constructor_export(
            module,
            "LocalResolution",
            "Ambiguous",
            [],
            Some(record([
                required("earlier", zoned.clone()),
                required("later", zoned.clone()),
            ])),
        ),
        constructor_export(
            module,
            "LocalResolution",
            "Gap",
            [],
            Some(record([
                required("transition", named("Instant")),
                required("offsetBefore", offset.clone()),
                required("offsetAfter", offset.clone()),
            ])),
        ),
        function_export(
            module,
            "localDate",
            [],
            Vec::new(),
            vec![named("Int"), named("Int"), named("Int")],
            date_result(date.clone()),
        ),
        function_export(
            module,
            "localTime",
            [],
            Vec::new(),
            vec![named("Int"), named("Int"), named("Int"), named("Int")],
            date_result(time.clone()),
        ),
        function_export(
            module,
            "localDateTime",
            [],
            Vec::new(),
            vec![date.clone(), time.clone()],
            local.clone(),
        ),
        function_export(
            module,
            "utcOffset",
            [],
            Vec::new(),
            vec![named("Int")],
            date_result(offset.clone()),
        ),
        function_export(
            module,
            "parseLocalDate",
            [],
            Vec::new(),
            vec![named("String")],
            date_result(date.clone()),
        ),
        function_export(
            module,
            "parseLocalTime",
            [],
            Vec::new(),
            vec![named("String")],
            date_result(time.clone()),
        ),
        function_export(
            module,
            "parseLocalDateTime",
            [],
            Vec::new(),
            vec![named("String")],
            date_result(local.clone()),
        ),
        function_export(
            module,
            "parseOffsetDateTime",
            [],
            Vec::new(),
            vec![named("String")],
            date_result(offset_date_time.clone()),
        ),
        function_export(
            module,
            "formatLocalDate",
            [],
            Vec::new(),
            vec![date],
            named("String"),
        ),
        function_export(
            module,
            "formatLocalTime",
            [],
            Vec::new(),
            vec![time],
            named("String"),
        ),
        function_export(
            module,
            "formatLocalDateTime",
            [],
            Vec::new(),
            vec![local.clone()],
            named("String"),
        ),
        function_export(
            module,
            "formatOffsetDateTime",
            [],
            Vec::new(),
            vec![offset_date_time.clone()],
            named("String"),
        ),
        function_export(
            module,
            "atOffset",
            [],
            Vec::new(),
            vec![offset, named("Instant")],
            offset_date_time.clone(),
        ),
        function_export(
            module,
            "offsetInstant",
            [],
            Vec::new(),
            vec![offset_date_time.clone()],
            named("Instant"),
        ),
        function_export(
            module,
            "offsetLocalDateTime",
            [],
            Vec::new(),
            vec![offset_date_time],
            local.clone(),
        ),
        function_export(
            module,
            "databaseVersion",
            [],
            Vec::new(),
            vec![named("Unit")],
            timezone_effect(named("Never"), named("String")),
        ),
        function_export(
            module,
            "loadTimeZone",
            [],
            Vec::new(),
            vec![named("String")],
            timezone_effect(named("TimeZoneError"), zone.clone()),
        ),
        function_export(
            module,
            "timeZoneId",
            [],
            Vec::new(),
            vec![zone.clone()],
            named("String"),
        ),
        function_export(
            module,
            "timeZoneVersion",
            [],
            Vec::new(),
            vec![zone.clone()],
            named("String"),
        ),
        function_export(
            module,
            "atTimeZone",
            [],
            Vec::new(),
            vec![named("Instant"), zone.clone()],
            zoned.clone(),
        ),
        function_export(
            module,
            "resolveLocal",
            [],
            Vec::new(),
            vec![local, zone.clone()],
            named("LocalResolution"),
        ),
        function_export(
            module,
            "zonedInstant",
            [],
            Vec::new(),
            vec![zoned.clone()],
            named("Instant"),
        ),
        function_export(
            module,
            "zonedLocalDateTime",
            [],
            Vec::new(),
            vec![zoned.clone()],
            named("LocalDateTime"),
        ),
        function_export(
            module,
            "zonedOffset",
            [],
            Vec::new(),
            vec![zoned.clone()],
            named("UtcOffset"),
        ),
        function_export(module, "zonedTimeZone", [], Vec::new(), vec![zoned], zone),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}

fn effect_interface() -> ModuleInterface {
    let module = "std/effect";
    let clock = external_type(
        "Clock",
        "std/clock::Clock",
        "std/clock",
        "Clock",
        Vec::new(),
    );
    let duration = external_type(
        "Duration",
        "std/time::Duration",
        "std/time",
        "Duration",
        Vec::new(),
    );
    let source = |success: InterfaceType| effect(named("R"), named("E"), success);
    let temporal = |success: InterfaceType| {
        effect(
            record([required("clock", clock.clone())]),
            named("E"),
            success,
        )
    };
    let schedule = |input: InterfaceType| named_with("Schedule", vec![input]);
    let decision = |input: InterfaceType| named_with("ScheduleDecision", vec![input]);
    let fiber = |failure: InterfaceType, success: InterfaceType| {
        named_with("Fiber", vec![failure, success])
    };
    let fiber_exit = |failure: InterfaceType, success: InterfaceType| {
        named_with("FiberExit", vec![failure, success])
    };
    let mut exports = vec![
        type_export(module, "Fiber", 2, "opaque-type"),
        opaque_adt_type_export(module, "FiberExit", ["E", "A"]),
        constructor_export(
            module,
            "FiberExit",
            "FiberSucceeded",
            ["E", "A"],
            Some(named("A")),
        ),
        constructor_export(
            module,
            "FiberExit",
            "FiberFailed",
            ["E", "A"],
            Some(named("E")),
        ),
        constructor_export(module, "FiberExit", "FiberCancelled", ["E", "A"], None),
        type_export(module, "Parallelism", 0, "opaque-type"),
        opaque_adt_type_export(module, "ParallelismError", []),
        constructor_export(
            module,
            "ParallelismError",
            "NonPositiveParallelism",
            [],
            Some(named("Int")),
        ),
        type_export(module, "Schedule", 1, "opaque-type"),
        opaque_adt_type_export(module, "ScheduleDecision", ["A"]),
        constructor_export(module, "ScheduleDecision", "ScheduleStop", ["A"], None),
        constructor_export(
            module,
            "ScheduleDecision",
            "ScheduleContinue",
            ["A"],
            Some(duration.clone()),
        ),
        opaque_adt_type_export(module, "ScheduleError", []),
        constructor_export(
            module,
            "ScheduleError",
            "NegativeRecurrences",
            [],
            Some(named("Int")),
        ),
        effect_function_export(
            module,
            "succeed",
            ["A"],
            Vec::new(),
            vec![named("A")],
            effect(record([]), named("Never"), named("A")),
        ),
        effect_function_export(
            module,
            "fail",
            ["E"],
            Vec::new(),
            vec![named("E")],
            effect(record([]), named("E"), named("Never")),
        ),
        effect_function_export(
            module,
            "defer",
            ["R", "E", "A"],
            Vec::new(),
            vec![function_type(vec![named("Unit")], source(named("A")))],
            source(named("A")),
        ),
        effect_function_export(
            module,
            "mapError",
            ["R", "E", "F", "A"],
            Vec::new(),
            vec![
                function_type(vec![named("E")], named("F")),
                source(named("A")),
            ],
            effect(named("R"), named("F"), named("A")),
        ),
        effect_function_export(
            module,
            "recover",
            ["R", "E", "F", "A"],
            Vec::new(),
            vec![
                function_type(vec![named("E")], effect(named("R"), named("F"), named("A"))),
                source(named("A")),
            ],
            effect(named("R"), named("F"), named("A")),
        ),
        effect_function_export(
            module,
            "provide",
            ["R", "E", "A"],
            Vec::new(),
            vec![named("R"), source(named("A"))],
            effect(record([]), named("E"), named("A")),
        ),
        effect_function_export(
            module,
            "service",
            ["R", "S"],
            Vec::new(),
            vec![function_type(vec![named("R")], named("S"))],
            effect(named("R"), named("Never"), named("S")),
        ),
        effect_function_export(
            module,
            "provideSome",
            ["R0", "R", "E", "A"],
            Vec::new(),
            vec![
                function_type(vec![named("R0")], named("R")),
                source(named("A")),
            ],
            effect(named("R0"), named("E"), named("A")),
        ),
        effect_function_export(
            module,
            "attempt",
            ["R", "E", "A"],
            Vec::new(),
            vec![source(named("A"))],
            effect(
                named("R"),
                named("Never"),
                named_with("Either", vec![named("E"), named("A")]),
            ),
        ),
        effect_function_export(
            module,
            "fromEither",
            ["E", "A"],
            Vec::new(),
            vec![named_with("Either", vec![named("E"), named("A")])],
            effect(record([]), named("E"), named("A")),
        ),
        effect_function_export(
            module,
            "fromMaybe",
            ["E", "A"],
            Vec::new(),
            vec![named("E"), named_with("Maybe", vec![named("A")])],
            effect(record([]), named("E"), named("A")),
        ),
        effect_function_export(
            module,
            "acquireRelease",
            ["R", "E", "A"],
            Vec::new(),
            vec![
                source(named("A")),
                function_type(
                    vec![named("A")],
                    effect(named("R"), named("Never"), named("Unit")),
                ),
            ],
            source(named("A")),
        ),
        effect_function_export(
            module,
            "scoped",
            ["R", "E", "A"],
            Vec::new(),
            vec![source(named("A"))],
            source(named("A")),
        ),
        effect_function_export(
            module,
            "fork",
            ["R", "E", "A"],
            Vec::new(),
            vec![source(named("A"))],
            effect(named("R"), named("Never"), fiber(named("E"), named("A"))),
        ),
        effect_function_export(
            module,
            "await",
            ["E", "A"],
            Vec::new(),
            vec![fiber(named("E"), named("A"))],
            task(fiber_exit(named("E"), named("A"))),
        ),
        effect_function_export(
            module,
            "poll",
            ["E", "A"],
            Vec::new(),
            vec![fiber(named("E"), named("A"))],
            task(named_with(
                "Maybe",
                vec![fiber_exit(named("E"), named("A"))],
            )),
        ),
        effect_function_export(
            module,
            "join",
            ["E", "A"],
            Vec::new(),
            vec![fiber(named("E"), named("A"))],
            effect(record([]), named("E"), named("A")),
        ),
        effect_function_export(
            module,
            "interrupt",
            ["E", "A"],
            Vec::new(),
            vec![fiber(named("E"), named("A"))],
            task(named("Unit")),
        ),
        effect_function_export(
            module,
            "yieldNow",
            [],
            Vec::new(),
            vec![named("Unit")],
            task(named("Unit")),
        ),
        effect_function_export(
            module,
            "race",
            ["R", "E", "A"],
            Vec::new(),
            vec![source(named("A")), source(named("A"))],
            source(named("A")),
        ),
        effect_function_export(
            module,
            "parallel",
            ["R", "E", "A"],
            Vec::new(),
            vec![named_with("Array", vec![source(named("A"))])],
            effect(
                named("R"),
                named("E"),
                named_with("Array", vec![named("A")]),
            ),
        ),
        function_export(
            module,
            "parallelism",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("ParallelismError"), named("Parallelism")],
            ),
        ),
        function_export(
            module,
            "unboundedParallelism",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Parallelism"),
        ),
        effect_function_export(
            module,
            "forEachParallel",
            ["C", "R", "E", "A"],
            vec![InterfaceConstraint {
                name: "Reducible".to_owned(),
                trait_identity: Some("std/prelude::Reducible".to_owned()),
                arguments: vec![named("C"), named("A")],
            }],
            vec![
                named("Parallelism"),
                function_type(
                    vec![named("A")],
                    effect(named("R"), named("E"), named("Unit")),
                ),
                named("C"),
            ],
            effect(named("R"), named("E"), named("Unit")),
        ),
        effect_function_export(
            module,
            "traverseParallel",
            ["C", "R", "E", "A", "B"],
            vec![InterfaceConstraint {
                name: "Reducible".to_owned(),
                trait_identity: Some("std/prelude::Reducible".to_owned()),
                arguments: vec![named("C"), named("A")],
            }],
            vec![
                named("Parallelism"),
                function_type(vec![named("A")], effect(named("R"), named("E"), named("B"))),
                named("C"),
            ],
            effect(
                named("R"),
                named("E"),
                named_with("Array", vec![named("B")]),
            ),
        ),
        effect_function_export(
            module,
            "timeout",
            ["R", "E", "A"],
            Vec::new(),
            vec![duration.clone(), source(named("A"))],
            temporal(named_with("Maybe", vec![named("A")])),
        ),
        effect_function_export(
            module,
            "timeoutFail",
            ["R", "E", "A"],
            Vec::new(),
            vec![named("E"), duration.clone(), source(named("A"))],
            temporal(named("A")),
        ),
        function_export(
            module,
            "schedule",
            ["A"],
            Vec::new(),
            vec![function_type(
                vec![named("Int"), named("A")],
                decision(named("A")),
            )],
            schedule(named("A")),
        ),
        function_export(
            module,
            "recurs",
            ["A"],
            Vec::new(),
            vec![named("Int")],
            named_with("Either", vec![named("ScheduleError"), schedule(named("A"))]),
        ),
        function_export(
            module,
            "spaced",
            ["A"],
            Vec::new(),
            vec![named("Int"), duration],
            named_with("Either", vec![named("ScheduleError"), schedule(named("A"))]),
        ),
        function_export(
            module,
            "whileInput",
            ["A"],
            Vec::new(),
            vec![function_type(vec![named("A")], named("Bool"))],
            schedule(named("A")),
        ),
        effect_function_export(
            module,
            "retry",
            ["R", "E", "A"],
            Vec::new(),
            vec![schedule(named("E")), source(named("A"))],
            temporal(named("A")),
        ),
        effect_function_export(
            module,
            "repeat",
            ["R", "E", "A"],
            Vec::new(),
            vec![schedule(named("A")), source(named("A"))],
            temporal(named("A")),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}

fn test_interface() -> ModuleInterface {
    let module = "std/test";
    let test = || named("Test");
    let test_failure = || named("TestFailure");
    let maybe_string = || named_with("Maybe", vec![named("String")]);
    let environment = || {
        record([
            required(
                "clock",
                external_type(
                    "Clock",
                    "std/clock::Clock",
                    "std/clock",
                    "Clock",
                    Vec::new(),
                ),
            ),
            required(
                "random",
                external_type(
                    "Random",
                    "std/random::Random",
                    "std/random",
                    "Random",
                    Vec::new(),
                ),
            ),
            required("console", prelude_type("Console")),
            required(
                "logger",
                external_type("Logger", "std/log::Logger", "std/log", "Logger", Vec::new()),
            ),
        ])
    };
    let assertion = || effect(record([]), test_failure(), named("Unit"));
    let debug_constraint = |parameter: &str| InterfaceConstraint {
        name: "Debug".to_owned(),
        trait_identity: Some("std/prelude::Debug".to_owned()),
        arguments: vec![named(parameter)],
    };
    let mut exports = vec![
        alias_type_export(module, "TestEnvironment", [], environment()),
        opaque_adt_type_export(module, "TestFailure", []),
        constructor_export(
            module,
            "TestFailure",
            "AssertionFailed",
            [],
            Some(record([
                required("message", named("String")),
                required("expected", maybe_string()),
                required("actual", maybe_string()),
            ])),
        ),
        constructor_export(module, "TestFailure", "ExpectedTypedFailure", [], None),
        constructor_export(
            module,
            "TestFailure",
            "TypedFailureDidNotMatch",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "TestFailure",
            "ExplicitTestFailure",
            [],
            Some(named("String")),
        ),
        type_export(module, "Test", 0, "opaque-type"),
        function_export(
            module,
            "test",
            [],
            Vec::new(),
            vec![
                named("String"),
                effect(environment(), test_failure(), named("Unit")),
            ],
            test(),
        ),
        function_export(
            module,
            "suite",
            [],
            Vec::new(),
            vec![named("String"), named_with("Array", vec![test()])],
            test(),
        ),
        function_export(
            module,
            "skip",
            [],
            Vec::new(),
            vec![named("String"), test()],
            test(),
        ),
        function_export(
            module,
            "timeout",
            [],
            Vec::new(),
            vec![
                external_type(
                    "Duration",
                    "std/time::Duration",
                    "std/time",
                    "Duration",
                    Vec::new(),
                ),
                test(),
            ],
            test(),
        ),
    ];
    for name in ["equal", "notEqual"] {
        exports.push(effect_function_export(
            module,
            name,
            ["A"],
            vec![
                InterfaceConstraint {
                    name: "Eq".to_owned(),
                    trait_identity: Some("std/prelude::Eq".to_owned()),
                    arguments: vec![named("A")],
                },
                debug_constraint("A"),
            ],
            vec![named("A"), named("A")],
            assertion(),
        ));
    }
    for name in ["isTrue", "isFalse"] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Bool")],
            assertion(),
        ));
    }
    exports.push(effect_function_export(
        module,
        "fail",
        [],
        Vec::new(),
        vec![named("String")],
        assertion(),
    ));
    exports.push(effect_function_export(
        module,
        "expectFailure",
        ["R", "E", "A"],
        vec![debug_constraint("E")],
        vec![
            function_type(vec![named("E")], named("Bool")),
            effect(named("R"), named("E"), named("A")),
        ],
        effect(named("R"), test_failure(), named("Unit")),
    ));
    standard_interface(module, exports)
}

fn stream_interface() -> ModuleInterface {
    let module = "std/stream";
    let stream = |environment: InterfaceType, failure: InterfaceType, value: InterfaceType| {
        named_with("Stream", vec![environment, failure, value])
    };
    let source = |value: InterfaceType| stream(named("R"), named("E"), value);
    let buffer_capacity = named("BufferCapacity");
    standard_interface(
        module,
        vec![
            type_export(module, "Stream", 3, "opaque-type"),
            type_export(module, "BufferCapacity", 0, "opaque-type"),
            opaque_adt_type_export(module, "BufferCapacityError", []),
            constructor_export(
                module,
                "BufferCapacityError",
                "NonPositiveBufferCapacity",
                [],
                Some(named("Int")),
            ),
            function_export(
                module,
                "empty",
                ["R", "E", "A"],
                Vec::new(),
                Vec::new(),
                source(named("A")),
            ),
            function_export(
                module,
                "singleton",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("A")],
                source(named("A")),
            ),
            function_export(
                module,
                "fromArray",
                ["R", "E", "A"],
                Vec::new(),
                vec![named_with("Array", vec![named("A")])],
                source(named("A")),
            ),
            function_export(
                module,
                "fromIterable",
                ["C", "R", "E", "A"],
                vec![InterfaceConstraint {
                    name: "Iterable".to_owned(),
                    trait_identity: Some("std/prelude::Iterable".to_owned()),
                    arguments: vec![named("C"), named("A")],
                }],
                vec![named("C")],
                source(named("A")),
            ),
            function_export(
                module,
                "fromEffect",
                ["R", "E", "A"],
                Vec::new(),
                vec![effect(named("R"), named("E"), named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "unfold",
                ["S", "R", "E", "A"],
                Vec::new(),
                vec![
                    function_type(
                        vec![named("S")],
                        named_with(
                            "Maybe",
                            vec![InterfaceType::Tuple {
                                elements: vec![named("A"), named("S")],
                            }],
                        ),
                    ),
                    named("S"),
                ],
                source(named("A")),
            ),
            function_export(
                module,
                "map",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named("B")),
                    source(named("A")),
                ],
                source(named("B")),
            ),
            function_export(
                module,
                "filter",
                ["R", "E", "A"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named("Bool")),
                    source(named("A")),
                ],
                source(named("A")),
            ),
            function_export(
                module,
                "filterMap",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named_with("Maybe", vec![named("B")])),
                    source(named("A")),
                ],
                source(named("B")),
            ),
            function_export(
                module,
                "mapError",
                ["R", "E", "F", "A"],
                Vec::new(),
                vec![
                    function_type(vec![named("E")], named("F")),
                    source(named("A")),
                ],
                stream(named("R"), named("F"), named("A")),
            ),
            function_export(
                module,
                "flatMap",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], source(named("B"))),
                    source(named("A")),
                ],
                source(named("B")),
            ),
            function_export(
                module,
                "take",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("Int"), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "drop",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("Int"), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "concat",
                ["R", "E", "A"],
                Vec::new(),
                vec![source(named("A")), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "zip",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![source(named("B")), source(named("A"))],
                source(InterfaceType::Tuple {
                    elements: vec![named("A"), named("B")],
                }),
            ),
            function_export(
                module,
                "merge",
                ["R", "E", "A"],
                Vec::new(),
                vec![source(named("A")), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "bufferCapacity",
                [],
                Vec::new(),
                vec![named("Int")],
                named_with(
                    "Either",
                    vec![named("BufferCapacityError"), buffer_capacity.clone()],
                ),
            ),
            function_export(
                module,
                "buffer",
                ["R", "E", "A"],
                Vec::new(),
                vec![buffer_capacity, source(named("A"))],
                source(named("A")),
            ),
            effect_function_export(
                module,
                "runCollect",
                ["R", "E", "A"],
                Vec::new(),
                vec![source(named("A"))],
                effect(
                    named("R"),
                    named("E"),
                    named_with("Array", vec![named("A")]),
                ),
            ),
            effect_function_export(
                module,
                "runFold",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    named("B"),
                    function_type(vec![named("B"), named("A")], named("B")),
                    source(named("A")),
                ],
                effect(named("R"), named("E"), named("B")),
            ),
            effect_function_export(
                module,
                "runForEach",
                ["R", "E", "A"],
                Vec::new(),
                vec![
                    function_type(
                        vec![named("A")],
                        effect(named("R"), named("E"), named("Unit")),
                    ),
                    source(named("A")),
                ],
                effect(named("R"), named("E"), named("Unit")),
            ),
        ],
    )
}

fn ref_interface() -> ModuleInterface {
    let module = "std/ref";
    let reference = |value: InterfaceType| named_with("Ref", vec![value]);
    standard_interface(
        module,
        vec![
            type_export(module, "Ref", 1, "opaque-type"),
            effect_function_export(
                module,
                "make",
                ["A"],
                Vec::new(),
                vec![named("A")],
                task(reference(named("A"))),
            ),
            effect_function_export(
                module,
                "get",
                ["A"],
                Vec::new(),
                vec![reference(named("A"))],
                task(named("A")),
            ),
            effect_function_export(
                module,
                "set",
                ["A"],
                Vec::new(),
                vec![named("A"), reference(named("A"))],
                task(named("Unit")),
            ),
            effect_function_export(
                module,
                "update",
                ["A"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named("A")),
                    reference(named("A")),
                ],
                task(named("Unit")),
            ),
            effect_function_export(
                module,
                "modify",
                ["A", "B"],
                Vec::new(),
                vec![
                    function_type(
                        vec![named("A")],
                        InterfaceType::Tuple {
                            elements: vec![named("B"), named("A")],
                        },
                    ),
                    reference(named("A")),
                ],
                task(named("B")),
            ),
        ],
    )
}

fn deferred_interface() -> ModuleInterface {
    let module = "std/deferred";
    let deferred = |failure: InterfaceType, success: InterfaceType| {
        named_with("Deferred", vec![failure, success])
    };
    let handle = || deferred(named("E"), named("A"));
    standard_interface(
        module,
        vec![
            type_export(module, "Deferred", 2, "opaque-type"),
            effect_function_export(
                module,
                "make",
                ["E", "A"],
                Vec::new(),
                vec![named("Unit")],
                task(handle()),
            ),
            effect_function_export(
                module,
                "await",
                ["E", "A"],
                Vec::new(),
                vec![handle()],
                effect(record([]), named("E"), named("A")),
            ),
            effect_function_export(
                module,
                "poll",
                ["E", "A"],
                Vec::new(),
                vec![handle()],
                task(named_with(
                    "Maybe",
                    vec![named_with("Either", vec![named("E"), named("A")])],
                )),
            ),
            effect_function_export(
                module,
                "complete",
                ["E", "A"],
                Vec::new(),
                vec![named_with("Either", vec![named("E"), named("A")]), handle()],
                task(named("Bool")),
            ),
            effect_function_export(
                module,
                "succeed",
                ["E", "A"],
                Vec::new(),
                vec![named("A"), handle()],
                task(named("Bool")),
            ),
            effect_function_export(
                module,
                "fail",
                ["E", "A"],
                Vec::new(),
                vec![named("E"), handle()],
                task(named("Bool")),
            ),
        ],
    )
}

fn queue_interface() -> ModuleInterface {
    let module = "std/queue";
    let queue = |value: InterfaceType| named_with("Queue", vec![value]);
    standard_interface(
        module,
        vec![
            type_export(module, "Queue", 1, "opaque-type"),
            opaque_adt_type_export(module, "QueueCreateError", []),
            constructor_export(
                module,
                "QueueCreateError",
                "NonPositiveCapacity",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "QueueClosed", []),
            constructor_export(module, "QueueClosed", "QueueClosed", [], None),
            effect_function_export(
                module,
                "bounded",
                ["A"],
                Vec::new(),
                vec![named("Int")],
                effect(record([]), named("QueueCreateError"), queue(named("A"))),
            ),
            effect_function_export(
                module,
                "unbounded",
                ["A"],
                Vec::new(),
                vec![named("Unit")],
                task(queue(named("A"))),
            ),
            effect_function_export(
                module,
                "offer",
                ["A"],
                Vec::new(),
                vec![named("A"), queue(named("A"))],
                effect(record([]), named("QueueClosed"), named("Unit")),
            ),
            effect_function_export(
                module,
                "take",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                effect(record([]), named("QueueClosed"), named("A")),
            ),
            effect_function_export(
                module,
                "tryOffer",
                ["A"],
                Vec::new(),
                vec![named("A"), queue(named("A"))],
                task(named_with(
                    "Either",
                    vec![named("QueueClosed"), named("Bool")],
                )),
            ),
            effect_function_export(
                module,
                "tryTake",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                task(named_with(
                    "Either",
                    vec![named("QueueClosed"), named_with("Maybe", vec![named("A")])],
                )),
            ),
            effect_function_export(
                module,
                "size",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                task(named("Int")),
            ),
            effect_function_export(
                module,
                "close",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                task(named("Unit")),
            ),
        ],
    )
}

fn semaphore_interface() -> ModuleInterface {
    let module = "std/semaphore";
    standard_interface(
        module,
        vec![
            type_export(module, "Semaphore", 0, "opaque-type"),
            type_export(module, "Permit", 0, "opaque-type"),
            opaque_adt_type_export(module, "SemaphoreCreateError", []),
            constructor_export(
                module,
                "SemaphoreCreateError",
                "NonPositivePermits",
                [],
                Some(named("Int")),
            ),
            effect_function_export(
                module,
                "make",
                [],
                Vec::new(),
                vec![named("Int")],
                effect(
                    record([]),
                    named("SemaphoreCreateError"),
                    named("Semaphore"),
                ),
            ),
            effect_function_export(
                module,
                "acquire",
                [],
                Vec::new(),
                vec![named("Semaphore")],
                task(named("Permit")),
            ),
            effect_function_export(
                module,
                "release",
                [],
                Vec::new(),
                vec![named("Permit")],
                task(named("Unit")),
            ),
            effect_function_export(
                module,
                "withPermit",
                ["R", "E", "A"],
                Vec::new(),
                vec![
                    named("Semaphore"),
                    effect(named("R"), named("E"), named("A")),
                ],
                effect(named("R"), named("E"), named("A")),
            ),
            effect_function_export(
                module,
                "available",
                [],
                Vec::new(),
                vec![named("Semaphore")],
                task(named("Int")),
            ),
        ],
    )
}

fn http_client_interface() -> ModuleInterface {
    let module = "std/http";
    let build_error = named("HttpBuildError");
    let method = named("Method");
    let method_value = external_type("Method", "std/http::Method", module, "Method", Vec::new());
    let status = named("Status");
    let status_value = external_type("Status", "std/http::Status", module, "Status", Vec::new());
    let headers = named("Headers");
    let headers_value = external_type(
        "Headers",
        "std/http::Headers",
        module,
        "Headers",
        Vec::new(),
    );
    let url = named("HttpUrl");
    let request = named("Request");
    let response = named("Response");
    let body = |environment: InterfaceType, failure: InterfaceType| {
        named_with("Body", vec![environment, failure])
    };
    let event = named("HttpEvent");
    let version_value = external_type(
        "HttpVersion",
        "std/http::HttpVersion",
        module,
        "HttpVersion",
        Vec::new(),
    );
    let response_head_value = named("ResponseHead");
    let limit = named("HttpBodyLimit");
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let build_result =
        |success: InterfaceType| named_with("Either", vec![build_error.clone(), success]);
    standard_interface(
        module,
        vec![
            type_export(module, "HttpClient", 0, "opaque-type"),
            type_export(module, "Method", 0, "opaque-type"),
            type_export(module, "Status", 0, "opaque-type"),
            type_export(module, "Headers", 0, "opaque-type"),
            type_export(module, "HttpUrl", 0, "opaque-type"),
            type_export(module, "Request", 0, "opaque-type"),
            type_export(module, "Response", 0, "opaque-type"),
            type_export(module, "Body", 2, "opaque-type"),
            type_export(module, "HttpBodyLimit", 0, "opaque-type"),
            opaque_adt_type_export(module, "HttpVersion", []),
            constructor_export(module, "HttpVersion", "HttpVersionUnknown", [], None),
            constructor_export(module, "HttpVersion", "Http1_0", [], None),
            constructor_export(module, "HttpVersion", "Http1_1", [], None),
            constructor_export(module, "HttpVersion", "Http2", [], None),
            constructor_export(module, "HttpVersion", "Http3", [], None),
            public_record_type_export(
                module,
                "ResponseHead",
                [
                    required("version", version_value),
                    required("status", status_value),
                    required("headers", headers_value.clone()),
                ],
            ),
            opaque_adt_type_export(module, "HttpEvent", []),
            constructor_export(
                module,
                "HttpEvent",
                "InformationalResponse",
                [],
                Some(response_head_value.clone()),
            ),
            constructor_export(
                module,
                "HttpEvent",
                "ResponseStarted",
                [],
                Some(response_head_value),
            ),
            constructor_export(
                module,
                "HttpEvent",
                "ResponseBodyChunk",
                [],
                Some(bytes.clone()),
            ),
            constructor_export(
                module,
                "HttpEvent",
                "ResponseTrailers",
                [],
                Some(headers_value.clone()),
            ),
            opaque_adt_type_export(module, "HttpBuildError", []),
            constructor_export(
                module,
                "HttpBuildError",
                "InvalidHttpUrl",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "UnsupportedHttpScheme",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "HttpUrlContainsUserInfo",
                [],
                None,
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "HttpUrlContainsFragment",
                [],
                None,
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "InvalidHttpMethod",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "InvalidHeaderName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "InvalidHeaderValue",
                [],
                Some(record([
                    required("name", named("String")),
                    required("offset", named("Int")),
                ])),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "ManagedHttpHeader",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "InvalidHttpStatus",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "HttpBuildError",
                "InvalidHttpBodyLimit",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "HttpError", []),
            constructor_export(
                module,
                "HttpError",
                "HttpDnsFailure",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpError",
                "HttpConnectionFailure",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpError",
                "HttpTlsFailure",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpError",
                "HttpProtocolFailure",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpError",
                "HttpRequestBodyFailure",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "HttpError",
                "HttpRequestLengthMismatch",
                [],
                Some(record([
                    required("declared", named("Int")),
                    required("actual", named("Int")),
                ])),
            ),
            constructor_export(
                module,
                "HttpError",
                "HttpResponseBodyLimitExceeded",
                [],
                Some(record([required("limitBytes", named("Int"))])),
            ),
            constructor_export(module, "HttpError", "HttpClientUnavailable", [], None),
            value_export(module, "get", method_value.clone()),
            value_export(module, "head", method_value.clone()),
            value_export(module, "post", method_value.clone()),
            value_export(module, "put", method_value.clone()),
            value_export(module, "patch", method_value.clone()),
            value_export(module, "delete", method_value.clone()),
            value_export(module, "options", method_value.clone()),
            value_export(module, "connect", method_value.clone()),
            value_export(module, "trace", method_value),
            function_export(
                module,
                "customMethod",
                [],
                Vec::new(),
                vec![named("String")],
                build_result(method.clone()),
            ),
            function_export(
                module,
                "methodText",
                [],
                Vec::new(),
                vec![method],
                named("String"),
            ),
            function_export(
                module,
                "status",
                [],
                Vec::new(),
                vec![named("Int")],
                build_result(status.clone()),
            ),
            function_export(
                module,
                "statusCode",
                [],
                Vec::new(),
                vec![status.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "isInformational",
                [],
                Vec::new(),
                vec![status.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "isSuccess",
                [],
                Vec::new(),
                vec![status.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "isRedirection",
                [],
                Vec::new(),
                vec![status.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "isClientError",
                [],
                Vec::new(),
                vec![status.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "isServerError",
                [],
                Vec::new(),
                vec![status],
                named("Bool"),
            ),
            function_export(
                module,
                "parseUrl",
                [],
                Vec::new(),
                vec![named("String")],
                build_result(url.clone()),
            ),
            function_export(
                module,
                "renderUrl",
                [],
                Vec::new(),
                vec![url.clone()],
                named("String"),
            ),
            value_export(module, "emptyHeaders", headers_value),
            function_export(
                module,
                "appendHeader",
                [],
                Vec::new(),
                vec![named("String"), named("String"), headers.clone()],
                build_result(headers.clone()),
            ),
            function_export(
                module,
                "setHeader",
                [],
                Vec::new(),
                vec![named("String"), named("String"), headers.clone()],
                build_result(headers.clone()),
            ),
            function_export(
                module,
                "removeHeader",
                [],
                Vec::new(),
                vec![named("String"), headers.clone()],
                headers.clone(),
            ),
            function_export(
                module,
                "headerValues",
                [],
                Vec::new(),
                vec![named("String"), headers.clone()],
                named_with("Array", vec![named("String")]),
            ),
            function_export(
                module,
                "headerEntries",
                [],
                Vec::new(),
                vec![headers.clone()],
                named_with(
                    "Array",
                    vec![InterfaceType::Tuple {
                        elements: vec![named("String"), named("String")],
                    }],
                ),
            ),
            function_export(
                module,
                "request",
                [],
                Vec::new(),
                vec![named("Method"), url],
                request.clone(),
            ),
            function_export(
                module,
                "withRequestHeader",
                [],
                Vec::new(),
                vec![named("String"), named("String"), request.clone()],
                build_result(request.clone()),
            ),
            function_export(
                module,
                "withoutRequestHeader",
                [],
                Vec::new(),
                vec![named("String"), request.clone()],
                request.clone(),
            ),
            function_export(
                module,
                "emptyBody",
                ["R", "E"],
                Vec::new(),
                vec![named("Unit")],
                body(named("R"), named("E")),
            ),
            function_export(
                module,
                "bytesBody",
                ["R", "E"],
                Vec::new(),
                vec![bytes.clone()],
                body(named("R"), named("E")),
            ),
            function_export(
                module,
                "streamBody",
                ["R", "E"],
                Vec::new(),
                vec![external_type(
                    "Stream",
                    "std/stream::Stream",
                    "std/stream",
                    "Stream",
                    vec![named("R"), named("E"), bytes.clone()],
                )],
                body(named("R"), named("E")),
            ),
            function_export(
                module,
                "exchange",
                ["R", "E"],
                Vec::new(),
                vec![body(named("R"), named("E")), request.clone()],
                external_type(
                    "Stream",
                    "std/stream::Stream",
                    "std/stream",
                    "Stream",
                    vec![
                        InterfaceType::RequirementMerge {
                            operands: vec![
                                named("R"),
                                record([required("httpClient", named("HttpClient"))]),
                            ],
                        },
                        named_with("Either", vec![named("E"), named("HttpError")]),
                        event,
                    ],
                ),
            ),
            function_export(
                module,
                "bodyLimit",
                [],
                Vec::new(),
                vec![named("Int")],
                build_result(limit.clone()),
            ),
            function_export(
                module,
                "defaultBodyLimit",
                [],
                Vec::new(),
                vec![named("Unit")],
                limit.clone(),
            ),
            effect_function_export(
                module,
                "sendBytes",
                [],
                Vec::new(),
                vec![limit.clone(), bytes.clone(), request.clone()],
                effect(
                    record([required("httpClient", named("HttpClient"))]),
                    named("HttpError"),
                    response.clone(),
                ),
            ),
            effect_function_export(
                module,
                "sendEmpty",
                [],
                Vec::new(),
                vec![limit, request],
                effect(
                    record([required("httpClient", named("HttpClient"))]),
                    named("HttpError"),
                    response.clone(),
                ),
            ),
            function_export(
                module,
                "responseStatus",
                [],
                Vec::new(),
                vec![response.clone()],
                named("Status"),
            ),
            function_export(
                module,
                "responseHeaders",
                [],
                Vec::new(),
                vec![response.clone()],
                headers,
            ),
            function_export(
                module,
                "responseBody",
                [],
                Vec::new(),
                vec![response],
                bytes,
            ),
            function_export(
                module,
                "errorMessage",
                [],
                Vec::new(),
                vec![named("HttpError")],
                named("String"),
            ),
        ],
    )
}

fn http_server_interface() -> ModuleInterface {
    let module = "std/http/server";
    let request = named("HttpServerRequest");
    let response = named("HttpServerResponse");
    let handler_request = external_type(
        "HttpServerRequest",
        "std/http/server::HttpServerRequest",
        module,
        "HttpServerRequest",
        Vec::new(),
    );
    let handler_response = external_type(
        "HttpServerResponse",
        "std/http/server::HttpServerResponse",
        module,
        "HttpServerResponse",
        Vec::new(),
    );
    let header = named("HttpHeader");
    let headers = named_with("Array", vec![header.clone()]);
    let handler = |environment: InterfaceType, failure: InterfaceType| {
        function_type(
            vec![handler_request.clone()],
            effect(environment, failure, handler_response.clone()),
        )
    };
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let exports = vec![
        type_export(module, "HttpServer", 0, "opaque-type"),
        type_export(module, "HttpServerRequest", 0, "opaque-type"),
        type_export(module, "HttpServerResponse", 0, "opaque-type"),
        type_export(module, "HttpServerHandle", 0, "opaque-type"),
        type_export(module, "HttpServerError", 0, "opaque-type"),
        alias_type_export(
            module,
            "HttpHeader",
            [],
            record([
                required("name", named("String")),
                required("value", named("String")),
            ]),
        ),
        alias_type_export(
            module,
            "Handler",
            ["R", "E"],
            handler(named("R"), named("E")),
        ),
        function_export(
            module,
            "requestMethod",
            [],
            Vec::new(),
            vec![request.clone()],
            named("String"),
        ),
        function_export(
            module,
            "requestUrl",
            [],
            Vec::new(),
            vec![request.clone()],
            named("String"),
        ),
        function_export(
            module,
            "requestPath",
            [],
            Vec::new(),
            vec![request.clone()],
            named("String"),
        ),
        function_export(
            module,
            "requestQuery",
            [],
            Vec::new(),
            vec![request.clone()],
            named_with("Maybe", vec![named("String")]),
        ),
        function_export(
            module,
            "requestHeaders",
            [],
            Vec::new(),
            vec![request.clone()],
            headers.clone(),
        ),
        function_export(
            module,
            "requestHeaderValues",
            [],
            Vec::new(),
            vec![named("String"), request.clone()],
            named_with("Array", vec![named("String")]),
        ),
        function_export(
            module,
            "requestBody",
            [],
            Vec::new(),
            vec![request.clone()],
            bytes.clone(),
        ),
        function_export(
            module,
            "header",
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            header,
        ),
        function_export(
            module,
            "emptyResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers.clone()],
            response.clone(),
        ),
        function_export(
            module,
            "bytesResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers.clone(), bytes.clone()],
            response.clone(),
        ),
        effect_function_export(
            module,
            "streamResponse",
            ["R"],
            Vec::new(),
            vec![
                named("Int"),
                headers.clone(),
                external_type(
                    "Stream",
                    "std/stream::Stream",
                    "std/stream",
                    "Stream",
                    vec![named("R"), named("Never"), bytes],
                ),
            ],
            effect(named("R"), named("Never"), response.clone()),
        ),
        function_export(
            module,
            "textResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers.clone(), named("String")],
            response.clone(),
        ),
        function_export(
            module,
            "jsonResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers, named("String")],
            response.clone(),
        ),
        function_export(
            module,
            "pureHandler",
            [],
            Vec::new(),
            vec![function_type(vec![request.clone()], response.clone())],
            handler(record([]), named("Never")),
        ),
        function_export(
            module,
            "recoverHandler",
            ["R", "E"],
            Vec::new(),
            vec![
                function_type(vec![named("E")], response.clone()),
                handler(named("R"), named("E")),
            ],
            handler(named("R"), named("Never")),
        ),
        function_export(
            module,
            "errorMessage",
            [],
            Vec::new(),
            vec![named("HttpServerError")],
            named("String"),
        ),
        function_export(
            module,
            "listen",
            ["R"],
            Vec::new(),
            vec![record([
                optional("hostname", named("String")),
                required("port", named("Int")),
                required("handler", handler(named("R"), named("Never"))),
            ])],
            effect(
                requirement_merge(vec![
                    named("R"),
                    record([required("httpServer", named("HttpServer"))]),
                ]),
                named("HttpServerError"),
                named("HttpServerHandle"),
            ),
        ),
        function_export(
            module,
            "serveOnce",
            ["R"],
            Vec::new(),
            vec![record([
                optional("hostname", named("String")),
                required("port", named("Int")),
                required("handler", handler(named("R"), named("Never"))),
            ])],
            effect(
                requirement_merge(vec![
                    named("R"),
                    record([required("httpServer", named("HttpServer"))]),
                ]),
                named("HttpServerError"),
                named("Unit"),
            ),
        ),
        function_export(
            module,
            "close",
            [],
            Vec::new(),
            vec![named("HttpServerHandle")],
            effect(
                record([required("httpServer", named("HttpServer"))]),
                named("Never"),
                named("Unit"),
            ),
        ),
    ];
    standard_interface(module, exports)
}

fn web_file_interface() -> ModuleInterface {
    let module = "std/web/file";
    let blob = named("Blob");
    let file = named("File");
    let build_error = named("BlobBuildError");
    let read_error = named("BlobReadError");
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let body = external_type(
        "Body",
        "std/http::Body",
        "std/http",
        "Body",
        vec![record([]), read_error.clone()],
    );
    let stream = external_type(
        "Stream",
        "std/stream::Stream",
        "std/stream",
        "Stream",
        vec![record([]), read_error.clone(), bytes.clone()],
    );
    standard_interface(
        module,
        vec![
            type_export(module, "Blob", 0, "opaque-type"),
            type_export(module, "File", 0, "opaque-type"),
            opaque_adt_type_export(module, "BlobBuildError", []),
            constructor_export(
                module,
                "BlobBuildError",
                "InvalidBlobMimeType",
                [],
                Some(named("String")),
            ),
            opaque_adt_type_export(module, "BlobReadError", []),
            constructor_export(
                module,
                "BlobReadError",
                "BlobReadLimitExceeded",
                [],
                Some(record([
                    required("limitBytes", named("Int")),
                    required("sizeBytes", named("Int")),
                ])),
            ),
            constructor_export(
                module,
                "BlobReadError",
                "BlobReadFailure",
                [],
                Some(named("String")),
            ),
            function_export(
                module,
                "fromBytes",
                [],
                Vec::new(),
                vec![named_with("Maybe", vec![named("String")]), bytes.clone()],
                named_with("Either", vec![build_error, blob.clone()]),
            ),
            function_export(
                module,
                "asBlob",
                [],
                Vec::new(),
                vec![file.clone()],
                blob.clone(),
            ),
            function_export(
                module,
                "name",
                [],
                Vec::new(),
                vec![file.clone()],
                named("String"),
            ),
            function_export(
                module,
                "mimeType",
                [],
                Vec::new(),
                vec![blob.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "sizeBytes",
                [],
                Vec::new(),
                vec![blob.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "lastModifiedMillis",
                [],
                Vec::new(),
                vec![file],
                named("Int"),
            ),
            function_export(
                module,
                "readBytes",
                [],
                Vec::new(),
                vec![named("Int"), blob.clone()],
                effect(record([]), read_error.clone(), bytes),
            ),
            function_export(
                module,
                "readChunks",
                [],
                Vec::new(),
                vec![blob.clone()],
                stream,
            ),
            function_export(module, "body", [], Vec::new(), vec![blob], body),
        ],
    )
}

fn http_multipart_interface() -> ModuleInterface {
    let module = "std/http/multipart";
    let multipart = |environment: InterfaceType, failure: InterfaceType| {
        named_with("Multipart", vec![environment, failure])
    };
    let body = |environment: InterfaceType, failure: InterfaceType| {
        external_type(
            "Body",
            "std/http::Body",
            "std/http",
            "Body",
            vec![environment, failure],
        )
    };
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let maybe_string = named_with("Maybe", vec![named("String")]);
    let build_error = named("MultipartBuildError");
    let build_result =
        |success: InterfaceType| named_with("Either", vec![build_error.clone(), success]);
    standard_interface(
        module,
        vec![
            type_export(module, "Multipart", 2, "opaque-type"),
            opaque_adt_type_export(module, "MultipartBuildError", []),
            constructor_export(
                module,
                "MultipartBuildError",
                "InvalidMultipartFieldName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "MultipartBuildError",
                "InvalidMultipartFileName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "MultipartBuildError",
                "InvalidMultipartMimeType",
                [],
                Some(named("String")),
            ),
            function_export(
                module,
                "empty",
                ["R", "E"],
                Vec::new(),
                vec![named("Unit")],
                multipart(named("R"), named("E")),
            ),
            function_export(
                module,
                "appendText",
                ["R", "E"],
                Vec::new(),
                vec![
                    named("String"),
                    named("String"),
                    multipart(named("R"), named("E")),
                ],
                build_result(multipart(named("R"), named("E"))),
            ),
            function_export(
                module,
                "appendBytes",
                ["R", "E"],
                Vec::new(),
                vec![
                    named("String"),
                    maybe_string.clone(),
                    maybe_string.clone(),
                    bytes,
                    multipart(named("R"), named("E")),
                ],
                build_result(multipart(named("R"), named("E"))),
            ),
            function_export(
                module,
                "appendBody",
                ["R", "E"],
                Vec::new(),
                vec![
                    named("String"),
                    maybe_string.clone(),
                    maybe_string,
                    body(named("R"), named("E")),
                    multipart(named("R"), named("E")),
                ],
                build_result(multipart(named("R"), named("E"))),
            ),
            function_export(
                module,
                "contentType",
                ["R", "E"],
                Vec::new(),
                vec![multipart(named("R"), named("E"))],
                named("String"),
            ),
            function_export(
                module,
                "body",
                ["R", "E"],
                Vec::new(),
                vec![multipart(named("R"), named("E"))],
                body(named("R"), named("E")),
            ),
        ],
    )
}

fn sse_interface() -> ModuleInterface {
    let module = "std/sse";
    let event = named("Event");
    let build_error = named("SseBuildError");
    let parse_error = named("SseParseError");
    let limit = named("DecodeLimit");
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let request = external_type(
        "Request",
        "std/http::Request",
        "std/http",
        "Request",
        Vec::new(),
    );
    let http_event = external_type(
        "HttpEvent",
        "std/http::HttpEvent",
        "std/http",
        "HttpEvent",
        Vec::new(),
    );
    let header = external_type(
        "HttpHeader",
        "std/http/server::HttpHeader",
        "std/http/server",
        "HttpHeader",
        Vec::new(),
    );
    let response = external_type(
        "HttpServerResponse",
        "std/http/server::HttpServerResponse",
        "std/http/server",
        "HttpServerResponse",
        Vec::new(),
    );
    let stream = |environment: InterfaceType, failure: InterfaceType, value: InterfaceType| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment, failure, value],
        )
    };
    let build_result =
        |success: InterfaceType| named_with("Either", vec![build_error.clone(), success]);
    standard_interface(
        module,
        vec![
            type_export(module, "Event", 0, "opaque-type"),
            type_export(module, "DecodeLimit", 0, "opaque-type"),
            opaque_adt_type_export(module, "SseBuildError", []),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseEventName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseEventId",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseRetryMillis",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseComment",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseDecodeLimit",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "SseParseError", []),
            constructor_export(
                module,
                "SseParseError",
                "SseUnexpectedStatus",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "SseParseError",
                "SseInvalidContentType",
                [],
                Some(named("String")),
            ),
            constructor_export(module, "SseParseError", "SseInvalidUtf8", [], None),
            constructor_export(
                module,
                "SseParseError",
                "SseEventTooLarge",
                [],
                Some(named("Int")),
            ),
            constructor_export(module, "SseParseError", "SseMalformedId", [], None),
            constructor_export(
                module,
                "SseParseError",
                "SseMalformedRetry",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseParseError",
                "SseMalformedHttpEvents",
                [],
                Some(named("String")),
            ),
            function_export(
                module,
                "event",
                [],
                Vec::new(),
                vec![named("String")],
                event.clone(),
            ),
            function_export(
                module,
                "withEventName",
                [],
                Vec::new(),
                vec![named("String"), event.clone()],
                build_result(event.clone()),
            ),
            function_export(
                module,
                "withId",
                [],
                Vec::new(),
                vec![named("String"), event.clone()],
                build_result(event.clone()),
            ),
            function_export(
                module,
                "withRetryMillis",
                [],
                Vec::new(),
                vec![named("Int"), event.clone()],
                build_result(event.clone()),
            ),
            function_export(
                module,
                "eventData",
                [],
                Vec::new(),
                vec![event.clone()],
                named("String"),
            ),
            function_export(
                module,
                "eventName",
                [],
                Vec::new(),
                vec![event.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "eventId",
                [],
                Vec::new(),
                vec![event.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "eventRetryMillis",
                [],
                Vec::new(),
                vec![event.clone()],
                named_with("Maybe", vec![named("Int")]),
            ),
            function_export(
                module,
                "encode",
                [],
                Vec::new(),
                vec![event.clone()],
                bytes.clone(),
            ),
            function_export(
                module,
                "keepAlive",
                [],
                Vec::new(),
                vec![named("String")],
                build_result(bytes),
            ),
            function_export(
                module,
                "decodeLimit",
                [],
                Vec::new(),
                vec![named("Int")],
                build_result(limit.clone()),
            ),
            function_export(
                module,
                "defaultDecodeLimit",
                [],
                Vec::new(),
                vec![named("Unit")],
                limit.clone(),
            ),
            function_export(
                module,
                "withLastEventId",
                [],
                Vec::new(),
                vec![named("String"), request.clone()],
                build_result(request),
            ),
            function_export(
                module,
                "events",
                ["R", "E"],
                Vec::new(),
                vec![limit, stream(named("R"), named("E"), http_event)],
                stream(
                    named("R"),
                    named_with("Either", vec![named("E"), parse_error]),
                    event.clone(),
                ),
            ),
            effect_function_export(
                module,
                "response",
                ["R"],
                Vec::new(),
                vec![
                    named_with("Array", vec![header]),
                    stream(named("R"), named("Never"), event),
                ],
                effect(named("R"), named("Never"), response),
            ),
        ],
    )
}

fn websocket_client_interface() -> ModuleInterface {
    let module = "std/websocket";
    let connection = named("WebSocketConnection");
    let event = named("WebSocketEvent");
    let close = named("WebSocketClose");
    let error = named("WebSocketError");
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let capacity = external_type(
        "BufferCapacity",
        "std/stream::BufferCapacity",
        "std/stream",
        "BufferCapacity",
        Vec::new(),
    );
    let stream = external_type(
        "Stream",
        "std/stream::Stream",
        "std/stream",
        "Stream",
        vec![record([]), error.clone(), event.clone()],
    );
    standard_interface(
        module,
        vec![
            type_export(module, "WebSocketClient", 0, "opaque-type"),
            type_export(module, "WebSocketConnection", 0, "opaque-type"),
            opaque_adt_type_export(module, "WebSocketEvent", []),
            type_export(module, "WebSocketClose", 0, "opaque-type"),
            opaque_adt_type_export(module, "WebSocketError", []),
            effect_function_export(
                module,
                "connect",
                [],
                Vec::new(),
                vec![record([
                    required("url", named("String")),
                    required("protocols", named_with("Array", vec![named("String")])),
                    required("receiveBuffer", capacity.clone()),
                ])],
                effect(
                    record([required("webSocketClient", named("WebSocketClient"))]),
                    error.clone(),
                    connection.clone(),
                ),
            ),
            function_export(
                module,
                "messages",
                [],
                Vec::new(),
                vec![connection.clone()],
                stream,
            ),
            effect_function_export(
                module,
                "sendText",
                [],
                Vec::new(),
                vec![named("String"), connection.clone()],
                effect(record([]), error.clone(), named("Unit")),
            ),
            effect_function_export(
                module,
                "sendBytes",
                [],
                Vec::new(),
                vec![bytes.clone(), connection.clone()],
                effect(record([]), error.clone(), named("Unit")),
            ),
            effect_function_export(
                module,
                "closeConnection",
                [],
                Vec::new(),
                vec![named("Int"), named("String"), connection.clone()],
                effect(record([]), error.clone(), named("Unit")),
            ),
            function_export(
                module,
                "selectedProtocol",
                [],
                Vec::new(),
                vec![connection],
                named("String"),
            ),
            function_export(
                module,
                "foldEvent",
                ["A"],
                Vec::new(),
                vec![
                    function_type(vec![named("String")], named("A")),
                    function_type(vec![bytes], named("A")),
                    function_type(vec![close.clone()], named("A")),
                    event,
                ],
                named("A"),
            ),
            function_export(
                module,
                "closeCode",
                [],
                Vec::new(),
                vec![close.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "closeReason",
                [],
                Vec::new(),
                vec![close.clone()],
                named("String"),
            ),
            function_export(
                module,
                "closeWasClean",
                [],
                Vec::new(),
                vec![close],
                named("Bool"),
            ),
            function_export(
                module,
                "errorMessage",
                [],
                Vec::new(),
                vec![error],
                named("String"),
            ),
        ],
    )
}

fn websocket_server_interface() -> ModuleInterface {
    let module = "std/websocket/server";
    let connection = external_type(
        "WebSocketConnection",
        "std/websocket::WebSocketConnection",
        "std/websocket",
        "WebSocketConnection",
        Vec::new(),
    );
    let error = external_type(
        "WebSocketError",
        "std/websocket::WebSocketError",
        "std/websocket",
        "WebSocketError",
        Vec::new(),
    );
    let capacity = external_type(
        "BufferCapacity",
        "std/stream::BufferCapacity",
        "std/stream",
        "BufferCapacity",
        Vec::new(),
    );
    let handler = |environment: InterfaceType| {
        function_type(
            vec![connection.clone()],
            effect(environment, named("Never"), named("Unit")),
        )
    };
    standard_interface(
        module,
        vec![
            type_export(module, "WebSocketServer", 0, "opaque-type"),
            type_export(module, "WebSocketServerHandle", 0, "opaque-type"),
            alias_type_export(module, "Handler", ["R"], handler(named("R"))),
            effect_function_export(
                module,
                "listen",
                ["R"],
                Vec::new(),
                vec![record([
                    optional("hostname", named("String")),
                    required("port", named("Int")),
                    required("path", named("String")),
                    required("protocols", named_with("Array", vec![named("String")])),
                    required("receiveBuffer", capacity),
                    required("handler", handler(named("R"))),
                ])],
                effect(
                    requirement_merge(vec![
                        named("R"),
                        record([required("webSocketServer", named("WebSocketServer"))]),
                    ]),
                    error,
                    named("WebSocketServerHandle"),
                ),
            ),
            effect_function_export(
                module,
                "closeServer",
                [],
                Vec::new(),
                vec![named("WebSocketServerHandle")],
                effect(
                    record([required("webSocketServer", named("WebSocketServer"))]),
                    named("Never"),
                    named("Unit"),
                ),
            ),
        ],
    )
}

fn bytes_interface() -> ModuleInterface {
    let module = "std/bytes";
    let byte = external_type("Byte", "std/bytes::Byte", module, "Byte", Vec::new());
    let bytes = external_type("Bytes", "std/bytes::Bytes", module, "Bytes", Vec::new());
    let byte_error = external_type(
        "ByteError",
        "std/bytes::ByteError",
        module,
        "ByteError",
        Vec::new(),
    );
    let slice_error = external_type(
        "BytesSliceError",
        "std/bytes::BytesSliceError",
        module,
        "BytesSliceError",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            type_export(module, "Byte", 0, "opaque-type"),
            type_export(module, "Bytes", 0, "opaque-type"),
            opaque_adt_type_export(module, "ByteError", []),
            constructor_export(
                module,
                "ByteError",
                "ByteOutOfRange",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "BytesSliceError", []),
            constructor_export(
                module,
                "BytesSliceError",
                "InvalidByteRange",
                [],
                Some(record([
                    required("start", named("Int")),
                    required("end", named("Int")),
                    required("length", named("Int")),
                ])),
            ),
            function_export(
                module,
                "byte",
                [],
                Vec::new(),
                vec![named("Int")],
                named_with("Either", vec![byte_error.clone(), byte.clone()]),
            ),
            function_export(
                module,
                "toInt",
                [],
                Vec::new(),
                vec![byte.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "empty",
                [],
                Vec::new(),
                vec![named("Unit")],
                bytes.clone(),
            ),
            function_export(
                module,
                "singleton",
                [],
                Vec::new(),
                vec![byte.clone()],
                bytes.clone(),
            ),
            function_export(
                module,
                "fromArray",
                [],
                Vec::new(),
                vec![named_with("Array", vec![byte.clone()])],
                bytes.clone(),
            ),
            function_export(
                module,
                "fromInts",
                [],
                Vec::new(),
                vec![named_with("Array", vec![named("Int")])],
                named_with("Either", vec![byte_error, bytes.clone()]),
            ),
            function_export(
                module,
                "toArray",
                [],
                Vec::new(),
                vec![bytes.clone()],
                named_with("Array", vec![byte.clone()]),
            ),
            function_export(
                module,
                "toInts",
                [],
                Vec::new(),
                vec![bytes.clone()],
                named_with("Array", vec![named("Int")]),
            ),
            function_export(
                module,
                "length",
                [],
                Vec::new(),
                vec![bytes.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "isEmpty",
                [],
                Vec::new(),
                vec![bytes.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "get",
                [],
                Vec::new(),
                vec![named("Int"), bytes.clone()],
                named_with("Maybe", vec![byte.clone()]),
            ),
            function_export(
                module,
                "slice",
                [],
                Vec::new(),
                vec![named("Int"), named("Int"), bytes.clone()],
                named_with("Either", vec![slice_error, bytes.clone()]),
            ),
            function_export(
                module,
                "copy",
                [],
                Vec::new(),
                vec![bytes.clone()],
                bytes.clone(),
            ),
            function_export(
                module,
                "append",
                [],
                Vec::new(),
                vec![bytes.clone(), bytes.clone()],
                bytes.clone(),
            ),
            function_export(
                module,
                "concat",
                [],
                Vec::new(),
                vec![named_with("Array", vec![bytes.clone()])],
                bytes,
            ),
        ],
    )
}

fn json_interface() -> ModuleInterface {
    let module = "std/json";
    let json = external_type("Json", "std/json::Json", module, "Json", Vec::new());
    let path_segment = external_type(
        "JsonPathSegment",
        "std/json::JsonPathSegment",
        module,
        "JsonPathSegment",
        Vec::new(),
    );
    let decode_error_kind = external_type(
        "DecodeErrorKind",
        "std/json::DecodeErrorKind",
        module,
        "DecodeErrorKind",
        Vec::new(),
    );
    let decode_error = external_type(
        "DecodeError",
        "std/json::DecodeError",
        module,
        "DecodeError",
        Vec::new(),
    );
    let parse_error = external_type(
        "JsonParseError",
        "std/json::JsonParseError",
        module,
        "JsonParseError",
        Vec::new(),
    );
    let read_error = external_type(
        "JsonReadError",
        "std/json::JsonReadError",
        module,
        "JsonReadError",
        Vec::new(),
    );
    let decoder = |value: InterfaceType| {
        function_type(
            vec![json.clone()],
            named_with("Either", vec![decode_error.clone(), value]),
        )
    };
    let encoder = |value: InterfaceType| function_type(vec![value], json.clone());
    let mut exports = vec![
        opaque_adt_type_export(module, "Json", []),
        constructor_export(module, "Json", "JsonNull", [], None),
        constructor_export(module, "Json", "JsonBool", [], Some(named("Bool"))),
        constructor_export(
            module,
            "Json",
            "JsonNumber",
            [],
            Some(external_type(
                "Decimal",
                "std/decimal::Decimal",
                "std/decimal",
                "Decimal",
                Vec::new(),
            )),
        ),
        constructor_export(module, "Json", "JsonString", [], Some(named("String"))),
        constructor_export(
            module,
            "Json",
            "JsonArray",
            [],
            Some(named_with("Array", vec![json.clone()])),
        ),
        constructor_export(
            module,
            "Json",
            "JsonObject",
            [],
            Some(external_type(
                "Map",
                "std/map::Map",
                "std/map",
                "Map",
                vec![named("String"), json.clone()],
            )),
        ),
        alias_type_export(module, "Decoder", ["A"], decoder(named("A"))),
        alias_type_export(module, "Encoder", ["A"], encoder(named("A"))),
        opaque_adt_type_export(module, "JsonPathSegment", []),
        constructor_export(
            module,
            "JsonPathSegment",
            "JsonField",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "JsonPathSegment",
            "JsonIndex",
            [],
            Some(named("Int")),
        ),
        opaque_adt_type_export(module, "DecodeErrorKind", []),
    ];
    for name in [
        "ExpectedJsonType",
        "MissingJsonField",
        "UnknownJsonField",
        "UnknownJsonTag",
        "InvalidJsonValue",
    ] {
        exports.push(constructor_export(
            module,
            "DecodeErrorKind",
            name,
            [],
            Some(named("String")),
        ));
    }
    exports.extend([
        record_type_export(
            module,
            "DecodeError",
            [
                required("path", named_with("Array", vec![path_segment.clone()])),
                required("kind", decode_error_kind),
            ],
        ),
        opaque_adt_type_export(module, "JsonParseError", []),
        constructor_export(
            module,
            "JsonParseError",
            "InvalidJsonSyntax",
            [],
            Some(record([
                required("offset", named("Int")),
                required("message", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "JsonParseError",
            "DuplicateJsonField",
            [],
            Some(record([
                required("path", named_with("Array", vec![path_segment])),
                required("field", named("String")),
            ])),
        ),
        opaque_adt_type_export(module, "JsonReadError", []),
        constructor_export(
            module,
            "JsonReadError",
            "JsonSyntaxFailure",
            [],
            Some(parse_error.clone()),
        ),
        constructor_export(
            module,
            "JsonReadError",
            "JsonDecodeFailure",
            [],
            Some(decode_error.clone()),
        ),
        function_export(
            module,
            "parse",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![parse_error, json.clone()]),
        ),
        function_export(
            module,
            "stringify",
            [],
            Vec::new(),
            vec![json.clone()],
            named("String"),
        ),
        function_export(
            module,
            "encodeString",
            ["A"],
            vec![InterfaceConstraint {
                name: "JsonEncode".to_owned(),
                trait_identity: Some("std/prelude::JsonEncode".to_owned()),
                arguments: vec![named("A")],
            }],
            vec![named("A")],
            named("String"),
        ),
        function_export(
            module,
            "decodeString",
            ["A"],
            vec![InterfaceConstraint {
                name: "JsonDecode".to_owned(),
                trait_identity: Some("std/prelude::JsonDecode".to_owned()),
                arguments: vec![named("A")],
            }],
            vec![named("String")],
            named_with("Either", vec![read_error, named("A")]),
        ),
        function_export(
            module,
            "field",
            ["A"],
            Vec::new(),
            vec![named("String"), decoder(named("A"))],
            decoder(named("A")),
        ),
        function_export(
            module,
            "optionalField",
            ["A"],
            Vec::new(),
            vec![named("String"), decoder(named("A"))],
            decoder(named_with("Maybe", vec![named("A")])),
        ),
        function_export(
            module,
            "index",
            ["A"],
            Vec::new(),
            vec![named("Int"), decoder(named("A"))],
            decoder(named("A")),
        ),
        function_export(
            module,
            "array",
            ["A"],
            Vec::new(),
            vec![decoder(named("A"))],
            decoder(named_with("Array", vec![named("A")])),
        ),
        function_export(
            module,
            "record",
            ["A"],
            Vec::new(),
            vec![named_with(
                "Array",
                vec![InterfaceType::Tuple {
                    elements: vec![named("String"), decoder(named("A"))],
                }],
            )],
            decoder(named_with(
                "Array",
                vec![InterfaceType::Tuple {
                    elements: vec![named("String"), named("A")],
                }],
            )),
        ),
        function_export(
            module,
            "oneOf",
            ["A"],
            Vec::new(),
            vec![named_with("Array", vec![decoder(named("A"))])],
            decoder(named("A")),
        ),
        function_export(
            module,
            "map",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("B")),
                decoder(named("A")),
            ],
            decoder(named("B")),
        ),
        function_export(
            module,
            "flatMap",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], decoder(named("B"))),
                decoder(named("A")),
            ],
            decoder(named("B")),
        ),
    ]);
    standard_interface(module, exports)
}

fn text_interface() -> ModuleInterface {
    let module = "std/text";
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let error = external_type(
        "Utf8DecodeError",
        "std/text::Utf8DecodeError",
        module,
        "Utf8DecodeError",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "Utf8DecodeError", []),
        constructor_export(
            module,
            "Utf8DecodeError",
            "InvalidUtf8",
            [],
            Some(record([required("offset", named("Int"))])),
        ),
        function_export(
            module,
            "encodeUtf8",
            [],
            Vec::new(),
            vec![named("String")],
            bytes.clone(),
        ),
        function_export(
            module,
            "decodeUtf8",
            [],
            Vec::new(),
            vec![bytes.clone()],
            named_with("Either", vec![error, named("String")]),
        ),
        function_export(
            module,
            "decodeUtf8Lossy",
            [],
            Vec::new(),
            vec![bytes],
            named("String"),
        ),
    ];
    exports.extend([
        opaque_adt_type_export(module, "TextSliceError", []),
        constructor_export(
            module,
            "TextSliceError",
            "InvalidScalarRange",
            [],
            Some(text_range_payload()),
        ),
        function_export(
            module,
            "isEmpty",
            [],
            Vec::new(),
            vec![named("String")],
            named("Bool"),
        ),
        function_export(
            module,
            "concat",
            [],
            Vec::new(),
            vec![named_with("Array", vec![named("String")])],
            named("String"),
        ),
        function_export(
            module,
            "join",
            [],
            Vec::new(),
            vec![named("String"), named_with("Array", vec![named("String")])],
            named("String"),
        ),
        function_export(
            module,
            "split",
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            named_with("Array", vec![named("String")]),
        ),
        function_export(
            module,
            "scalarAt",
            [],
            Vec::new(),
            vec![named("Int"), named("String")],
            named_with("Maybe", vec![named("Char")]),
        ),
        function_export(
            module,
            "sliceScalars",
            [],
            Vec::new(),
            vec![named("Int"), named("Int"), named("String")],
            named_with(
                "Either",
                vec![
                    external_type(
                        "TextSliceError",
                        "std/text::TextSliceError",
                        module,
                        "TextSliceError",
                        Vec::new(),
                    ),
                    named("String"),
                ],
            ),
        ),
    ]);
    for name in ["lengthScalars", "lengthBytes"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            named("Int"),
        ));
    }
    for name in ["lines", "words"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Array", vec![named("String")]),
        ));
    }
    for name in [
        "trim",
        "trimStart",
        "trimEnd",
        "toLower",
        "toUpper",
        "caseFold",
    ] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            named("String"),
        ));
    }
    for name in ["startsWith", "endsWith", "contains"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            named("Bool"),
        ));
    }
    for name in ["replace", "replaceAll"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String"), named("String"), named("String")],
            named("String"),
        ));
    }
    standard_interface(module, exports)
}

fn text_range_payload() -> InterfaceType {
    record([
        required("start", named("Int")),
        required("end", named("Int")),
        required("length", named("Int")),
    ])
}

fn char_interface() -> ModuleInterface {
    let module = "std/char";
    standard_interface(
        module,
        vec![
            function_export(
                module,
                "codePoint",
                [],
                Vec::new(),
                vec![named("Char")],
                named("Int"),
            ),
            function_export(
                module,
                "fromCodePoint",
                [],
                Vec::new(),
                vec![named("Int")],
                named_with("Maybe", vec![named("Char")]),
            ),
            function_export(
                module,
                "toString",
                [],
                Vec::new(),
                vec![named("Char")],
                named("String"),
            ),
        ],
    )
}

fn grapheme_interface() -> ModuleInterface {
    let module = "std/text/grapheme";
    let error = external_type(
        "GraphemeSliceError",
        "std/text/grapheme::GraphemeSliceError",
        module,
        "GraphemeSliceError",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "GraphemeSliceError", []),
            constructor_export(
                module,
                "GraphemeSliceError",
                "InvalidGraphemeRange",
                [],
                Some(text_range_payload()),
            ),
            function_export(
                module,
                "length",
                [],
                Vec::new(),
                vec![named("String")],
                named("Int"),
            ),
            function_export(
                module,
                "clusters",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Array", vec![named("String")]),
            ),
            function_export(
                module,
                "byteBoundaries",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Array", vec![named("Int")]),
            ),
            function_export(
                module,
                "at",
                [],
                Vec::new(),
                vec![named("Int"), named("String")],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "slice",
                [],
                Vec::new(),
                vec![named("Int"), named("Int"), named("String")],
                named_with("Either", vec![error, named("String")]),
            ),
        ],
    )
}

fn unicode_interface() -> ModuleInterface {
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

fn number_interface() -> ModuleInterface {
    let module = "std/number";
    let mut exports = vec![opaque_adt_type_export(module, "RoundingMode", [])];
    for name in [
        "HalfEven",
        "HalfUp",
        "TowardZero",
        "AwayFromZero",
        "Floor",
        "Ceiling",
    ] {
        exports.push(constructor_export(module, "RoundingMode", name, [], None));
    }
    standard_interface(module, exports)
}

fn int_interface() -> ModuleInterface {
    let module = "std/int";
    let parse_error = external_type(
        "IntParseError",
        "std/int::IntParseError",
        module,
        "IntParseError",
        Vec::new(),
    );
    let division_error = external_type(
        "IntDivisionError",
        "std/int::IntDivisionError",
        module,
        "IntDivisionError",
        Vec::new(),
    );
    let power_error = external_type(
        "IntPowerError",
        "std/int::IntPowerError",
        module,
        "IntPowerError",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "IntParseError", []),
        constructor_export(module, "IntParseError", "EmptyInt", [], None),
        constructor_export(
            module,
            "IntParseError",
            "InvalidIntRadix",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "IntParseError",
            "InvalidIntDigit",
            [],
            Some(record([
                required("offset", named("Int")),
                required("radix", named("Int")),
            ])),
        ),
        constructor_export(module, "IntParseError", "IntOutsideRange", [], None),
        opaque_adt_type_export(module, "IntDivisionError", []),
        constructor_export(module, "IntDivisionError", "IntDivisionByZero", [], None),
        opaque_adt_type_export(module, "IntPowerError", []),
        constructor_export(
            module,
            "IntPowerError",
            "NegativeIntExponent",
            [],
            Some(named("Int")),
        ),
        constructor_export(module, "IntPowerError", "IntPowerOverflow", [], None),
        function_export(
            module,
            "minValue",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Int"),
        ),
        function_export(
            module,
            "maxValue",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Int"),
        ),
        function_export(
            module,
            "parse",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![parse_error.clone(), named("Int")]),
        ),
        function_export(
            module,
            "parseRadix",
            [],
            Vec::new(),
            vec![named("Int"), named("String")],
            named_with("Either", vec![parse_error.clone(), named("Int")]),
        ),
        function_export(
            module,
            "format",
            [],
            Vec::new(),
            vec![named("Int")],
            named("String"),
        ),
        function_export(
            module,
            "formatRadix",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![parse_error, named("String")]),
        ),
    ];

    for name in ["checkedAdd", "checkedSubtract", "checkedMultiply"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Maybe", vec![named("Int")]),
        ));
    }
    for name in [
        "saturatingAdd",
        "saturatingSubtract",
        "saturatingMultiply",
        "minimum",
        "maximum",
    ] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named("Int"),
        ));
    }
    for name in ["checkedDivide", "checkedRemainder"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![division_error.clone(), named("Int")]),
        ));
    }
    exports.extend([
        function_export(
            module,
            "checkedPower",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![power_error.clone(), named("Int")]),
        ),
        function_export(
            module,
            "saturatingPower",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![power_error, named("Int")]),
        ),
        function_export(
            module,
            "abs",
            [],
            Vec::new(),
            vec![named("Int")],
            named("Int"),
        ),
        function_export(
            module,
            "clamp",
            [],
            Vec::new(),
            vec![named("Int"), named("Int"), named("Int")],
            named("Int"),
        ),
        function_export(
            module,
            "sign",
            [],
            Vec::new(),
            vec![named("Int")],
            named("Int"),
        ),
    ]);
    standard_interface(module, exports)
}

fn float_interface() -> ModuleInterface {
    let module = "std/float";
    let parse_error = external_type(
        "FloatParseError",
        "std/float::FloatParseError",
        module,
        "FloatParseError",
        Vec::new(),
    );
    let conversion_error = external_type(
        "FloatConversionError",
        "std/float::FloatConversionError",
        module,
        "FloatConversionError",
        Vec::new(),
    );
    let rounding_mode = external_type(
        "RoundingMode",
        "std/number::RoundingMode",
        "std/number",
        "RoundingMode",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "FloatParseError", []),
        constructor_export(module, "FloatParseError", "EmptyFloat", [], None),
        constructor_export(
            module,
            "FloatParseError",
            "InvalidFloat",
            [],
            Some(record([required("offset", named("Int"))])),
        ),
        constructor_export(module, "FloatParseError", "FloatParseOverflow", [], None),
        opaque_adt_type_export(module, "FloatConversionError", []),
        constructor_export(module, "FloatConversionError", "FloatNotFinite", [], None),
        constructor_export(
            module,
            "FloatConversionError",
            "FloatOutsideIntRange",
            [],
            None,
        ),
    ];
    for name in ["nan", "positiveInfinity", "negativeInfinity"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Float"),
        ));
    }
    exports.extend([
        function_export(
            module,
            "parse",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![parse_error, named("Float")]),
        ),
        function_export(
            module,
            "format",
            [],
            Vec::new(),
            vec![named("Float")],
            named("String"),
        ),
        function_export(
            module,
            "fromInt",
            [],
            Vec::new(),
            vec![named("Int")],
            named("Float"),
        ),
        function_export(
            module,
            "toInt",
            [],
            Vec::new(),
            vec![rounding_mode.clone(), named("Float")],
            named_with("Either", vec![conversion_error, named("Int")]),
        ),
    ]);
    for name in ["isNaN", "isFinite", "isInfinite", "isNegativeZero"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Float")],
            named("Bool"),
        ));
    }
    exports.extend([
        function_export(
            module,
            "ieeeEq",
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Bool"),
        ),
        function_export(
            module,
            "totalCompare",
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Ordering"),
        ),
    ]);
    for name in ["minimumNumber", "maximumNumber"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Float"),
        ));
    }
    exports.extend([
        function_export(
            module,
            "clampNumber",
            [],
            Vec::new(),
            vec![named("Float"), named("Float"), named("Float")],
            named_with("Maybe", vec![named("Float")]),
        ),
        function_export(
            module,
            "abs",
            [],
            Vec::new(),
            vec![named("Float")],
            named("Float"),
        ),
        function_export(
            module,
            "sign",
            [],
            Vec::new(),
            vec![named("Float")],
            named_with("Maybe", vec![named("Int")]),
        ),
        function_export(
            module,
            "power",
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Float"),
        ),
        function_export(
            module,
            "roundIntegral",
            [],
            Vec::new(),
            vec![rounding_mode, named("Float")],
            named("Float"),
        ),
    ]);
    standard_interface(module, exports)
}

fn standard_interface(module: &str, exports: Vec<InterfaceExport>) -> ModuleInterface {
    ModuleInterface {
        schema: 1,
        module: module.to_owned(),
        source: format!("{module}.ssrg"),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn array_interface() -> ModuleInterface {
    collection_interface("std/array", "Array", "toList", "List")
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

fn validation_interface() -> ModuleInterface {
    let module = "std/validation";
    let validation = external_type(
        "Validation",
        "std/validation::Validation",
        module,
        "Validation",
        vec![named("E"), named("A")],
    );
    let errors = external_type(
        "NonEmptyList",
        "std/non-empty-list::NonEmptyList",
        "std/non-empty-list",
        "NonEmptyList",
        vec![named("E")],
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "Validation", ["E", "A"]),
        constructor_export(module, "Validation", "Valid", ["E", "A"], Some(named("A"))),
        constructor_export(
            module,
            "Validation",
            "Invalid",
            ["E", "A"],
            Some(errors.clone()),
        ),
    ];
    for (name, argument, result) in [
        ("valid", named("A"), validation.clone()),
        ("invalid", named("E"), validation.clone()),
        ("invalidMany", errors.clone(), validation.clone()),
        (
            "fromEither",
            named_with("Either", vec![named("E"), named("A")]),
            validation.clone(),
        ),
        (
            "toEither",
            validation,
            named_with("Either", vec![errors, named("A")]),
        ),
    ] {
        exports.push(function_export(
            module,
            name,
            ["E", "A"],
            vec![],
            vec![argument],
            result,
        ));
    }
    standard_interface(module, exports)
}

fn maybe_interface() -> ModuleInterface {
    let module = "std/maybe";
    let maybe = named_with("Maybe", vec![named("A")]);
    standard_interface(
        module,
        vec![
            function_export(
                module,
                "withDefault",
                ["A"],
                vec![],
                vec![named("A"), maybe.clone()],
                named("A"),
            ),
            function_export(
                module,
                "orElse",
                ["A"],
                vec![],
                vec![maybe.clone(), maybe.clone()],
                maybe,
            ),
            sum_traversal_export(module, true, false),
            sum_traversal_export(module, false, false),
        ],
    )
}

fn either_interface() -> ModuleInterface {
    let module = "std/either";
    let either = |error, value| named_with("Either", vec![named(error), named(value)]);
    let callback = |from, to| function_type(vec![named(from)], named(to));
    standard_interface(
        module,
        vec![
            function_export(
                module,
                "mapLeft",
                ["E", "F", "A"],
                vec![],
                vec![callback("E", "F"), either("E", "A")],
                either("F", "A"),
            ),
            function_export(
                module,
                "mapRight",
                ["E", "A", "B"],
                vec![],
                vec![callback("A", "B"), either("E", "A")],
                either("E", "B"),
            ),
            function_export(
                module,
                "bimap",
                ["E", "F", "A", "B"],
                vec![],
                vec![callback("E", "F"), callback("A", "B"), either("E", "A")],
                either("F", "B"),
            ),
            function_export(
                module,
                "fold",
                ["E", "A", "B"],
                vec![],
                vec![callback("E", "B"), callback("A", "B"), either("E", "A")],
                named("B"),
            ),
            function_export(
                module,
                "swap",
                ["E", "A"],
                vec![],
                vec![either("E", "A")],
                either("A", "E"),
            ),
            sum_traversal_export(module, true, true),
            sum_traversal_export(module, false, true),
        ],
    )
}

fn collection_core_interface() -> ModuleInterface {
    let module = "std/collection";
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "SizeError", []),
            constructor_export(
                module,
                "SizeError",
                "NonPositiveSize",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "ReduceStep", ["A"]),
            constructor_export(module, "ReduceStep", "Next", ["A"], Some(named("A"))),
            constructor_export(module, "ReduceStep", "Done", ["A"], Some(named("A"))),
            function_export(
                module,
                "reduceUntil",
                ["C", "A", "B"],
                vec![collection_constraint(
                    "Iterable",
                    vec![named("C"), named("A")],
                )],
                vec![
                    named("B"),
                    function_type(
                        vec![named("B"), named("A")],
                        external_type(
                            "ReduceStep",
                            "std/collection::ReduceStep",
                            module,
                            "ReduceStep",
                            vec![named("B")],
                        ),
                    ),
                    named("C"),
                ],
                named("B"),
            ),
        ],
    )
}

fn list_interface() -> ModuleInterface {
    collection_interface("std/list", "List", "toArray", "Array")
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

fn web_dom_interface() -> ModuleInterface {
    let module = "std/web/dom";
    let dom_environment = record([required("dom", named("Dom"))]);
    let with_dom = |requirements: InterfaceType| InterfaceType::RequirementMerge {
        operands: vec![requirements, dom_environment.clone()],
    };
    let html = |action: &str| {
        external_type(
            "Html",
            "std/web/html::Html",
            "std/web/html",
            "Html",
            vec![named(action)],
        )
    };
    let signal = |value: InterfaceType| {
        external_type(
            "Signal",
            "std/signal::Signal",
            "std/signal",
            "Signal",
            vec![value],
        )
    };
    let signal_html = |action: &str| signal(html(action));
    let exports = vec![
        type_export(module, "Dom", 0, "opaque-type"),
        opaque_adt_type_export(module, "HydrationMode", []),
        constructor_export(module, "HydrationMode", "FreshMount", [], None),
        constructor_export(module, "HydrationMode", "HydrateStrict", [], None),
        constructor_export(module, "HydrationMode", "HydrateOrReplace", [], None),
        opaque_adt_type_export(module, "CleanupMode", []),
        constructor_export(module, "CleanupMode", "ClearRenderedDom", [], None),
        constructor_export(module, "CleanupMode", "PreserveRenderedDom", [], None),
        public_record_type_export(
            module,
            "DomOptions",
            [
                required("eventCapacity", named("Int")),
                required("hydration", named("HydrationMode")),
                required("cleanup", named("CleanupMode")),
            ],
        ),
        type_export(module, "DomTarget", 0, "opaque-type"),
        type_export(module, "DomMount", 1, "opaque-type"),
        type_export(module, "DomContent", 1, "opaque-type"),
        type_export(module, "DomBinding", 1, "opaque-type"),
        opaque_adt_type_export(module, "DomError", []),
        constructor_export(
            module,
            "DomError",
            "InvalidSelector",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "DomError",
            "DomTargetNotFound",
            [],
            Some(named("String")),
        ),
        constructor_export(module, "DomError", "DomTargetAlreadyMounted", [], None),
        constructor_export(
            module,
            "DomError",
            "HydrationMismatch",
            [],
            Some(record([
                required("path", named_with("Array", vec![named("Int")])),
                required("expected", named("String")),
                required("actual", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "DomError",
            "DomEventQueueOverflow",
            [],
            Some(named("Int")),
        ),
        constructor_export(module, "DomError", "DomTargetRemoved", [], None),
        constructor_export(
            module,
            "DomError",
            "DomOperationFailed",
            [],
            Some(named("String")),
        ),
        opaque_adt_type_export(module, "DomRuntimeError", ["Failure"]),
        constructor_export(
            module,
            "DomRuntimeError",
            "DomFailure",
            ["Failure"],
            Some(named("DomError")),
        ),
        constructor_export(
            module,
            "DomRuntimeError",
            "DispatchFailure",
            ["Failure"],
            Some(named("Failure")),
        ),
        function_export(
            module,
            "defaultOptions",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("DomOptions"),
        ),
        function_export(
            module,
            "query",
            [],
            Vec::new(),
            vec![named("String")],
            effect(
                dom_environment.clone(),
                named("DomError"),
                named("DomTarget"),
            ),
        ),
        function_export(
            module,
            "mount",
            ["R", "Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(named("R"), named("Failure"), named("Unit")),
                ),
                signal_html("Action"),
            ],
            effect(
                with_dom(named("R")),
                named("DomError"),
                named_with("DomMount", vec![named("Failure")]),
            ),
        ),
        function_export(
            module,
            "awaitMount",
            ["Failure"],
            Vec::new(),
            vec![named_with("DomMount", vec![named("Failure")])],
            effect(
                record([]),
                named_with("DomRuntimeError", vec![named("Failure")]),
                named("Unit"),
            ),
        ),
        function_export(
            module,
            "unmount",
            ["Failure"],
            Vec::new(),
            vec![named_with("DomMount", vec![named("Failure")])],
            effect(record([]), named("Never"), named("Unit")),
        ),
        function_export(
            module,
            "content",
            ["Action"],
            Vec::new(),
            vec![
                html("Action"),
                named_with(
                    "Array",
                    vec![named_with("DomBinding", vec![named("Action")])],
                ),
            ],
            named_with("DomContent", vec![named("Action")]),
        ),
        function_export(
            module,
            "initialHtml",
            ["Action"],
            Vec::new(),
            vec![named_with("DomContent", vec![named("Action")])],
            html("Action"),
        ),
        function_export(
            module,
            "bindText",
            ["Action"],
            Vec::new(),
            vec![named("String"), signal(named("String"))],
            named_with("DomBinding", vec![named("Action")]),
        ),
        function_export(
            module,
            "bindAttribute",
            ["Action"],
            Vec::new(),
            vec![
                named("String"),
                named("String"),
                signal(named_with("Maybe", vec![named("String")])),
            ],
            named_with("DomBinding", vec![named("Action")]),
        ),
        function_export(
            module,
            "bindValue",
            ["Action"],
            Vec::new(),
            vec![named("String"), signal(named("String"))],
            named_with("DomBinding", vec![named("Action")]),
        ),
        function_export(
            module,
            "bindChecked",
            ["Action"],
            Vec::new(),
            vec![named("String"), signal(named("Bool"))],
            named_with("DomBinding", vec![named("Action")]),
        ),
        function_export(
            module,
            "bindStyle",
            ["Action"],
            Vec::new(),
            vec![
                named("String"),
                named("String"),
                signal(named_with("Maybe", vec![named("String")])),
            ],
            named_with("DomBinding", vec![named("Action")]),
        ),
        function_export(
            module,
            "bindRegion",
            ["Action"],
            Vec::new(),
            vec![
                named("String"),
                signal(named_with("DomContent", vec![named("Action")])),
            ],
            named_with("DomBinding", vec![named("Action")]),
        ),
        function_export(
            module,
            "mountContent",
            ["R", "Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(named("R"), named("Failure"), named("Unit")),
                ),
                named_with("DomContent", vec![named("Action")]),
            ],
            effect(
                with_dom(named("R")),
                named("DomError"),
                named_with("DomMount", vec![named("Failure")]),
            ),
        ),
        function_export(
            module,
            "runContent",
            ["R", "Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(named("R"), named("Failure"), named("Unit")),
                ),
                named_with("DomContent", vec![named("Action")]),
            ],
            effect(
                with_dom(named("R")),
                named_with("DomRuntimeError", vec![named("Failure")]),
                named("Unit"),
            ),
        ),
        function_export(
            module,
            "run",
            ["R", "Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(named("R"), named("Failure"), named("Unit")),
                ),
                signal_html("Action"),
            ],
            effect(
                with_dom(named("R")),
                named_with("DomRuntimeError", vec![named("Failure")]),
                named("Unit"),
            ),
        ),
        function_export(
            module,
            "app",
            ["State", "Action"],
            Vec::new(),
            vec![record([
                required("target", named("String")),
                required("initial", named("State")),
                required(
                    "update",
                    function_type(vec![named("Action"), named("State")], named("State")),
                ),
                required(
                    "view",
                    function_type(
                        vec![named("State")],
                        external_type(
                            "Html",
                            "std/web/html::Html",
                            "std/web/html",
                            "Html",
                            vec![named("Action")],
                        ),
                    ),
                ),
            ])],
            effect(
                record([required("dom", named("Dom"))]),
                named("String"),
                named("Unit"),
            ),
        ),
    ];
    ModuleInterface {
        schema: 1,
        module: "std/web/dom".to_owned(),
        source: "std/web/dom.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn web_navigation_interface() -> ModuleInterface {
    let module = "std/web/navigation";
    let url = named("Url");
    let query = named("Query");
    let location = named("Location");
    let build_result =
        |success: InterfaceType| named_with("Either", vec![named("UrlBuildError"), success]);
    let navigation_effect = |success: InterfaceType| {
        effect(
            record([required("navigation", named("Navigation"))]),
            named("NavigationError"),
            success,
        )
    };
    standard_interface(
        module,
        vec![
            type_export(module, "Navigation", 0, "opaque-type"),
            type_export(module, "Url", 0, "opaque-type"),
            type_export(module, "Query", 0, "opaque-type"),
            type_export(module, "Location", 0, "opaque-type"),
            opaque_adt_type_export(module, "UrlBuildError", []),
            constructor_export(
                module,
                "UrlBuildError",
                "InvalidUrl",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(
                module,
                "UrlBuildError",
                "UnsupportedUrlScheme",
                [],
                Some(named("String")),
            ),
            constructor_export(module, "UrlBuildError", "UrlContainsUserInfo", [], None),
            constructor_export(
                module,
                "UrlBuildError",
                "InvalidPercentEncoding",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            opaque_adt_type_export(module, "NavigationError", []),
            constructor_export(
                module,
                "NavigationError",
                "CrossOriginNavigation",
                [],
                Some(record([
                    required("expected", named("String")),
                    required("actual", named("String")),
                ])),
            ),
            constructor_export(
                module,
                "NavigationError",
                "NavigationUnavailable",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "NavigationError",
                "NavigationSecurityFailure",
                [],
                Some(named("String")),
            ),
            function_export(
                module,
                "parseUrl",
                [],
                Vec::new(),
                vec![named("String")],
                build_result(url.clone()),
            ),
            function_export(
                module,
                "resolveUrl",
                [],
                Vec::new(),
                vec![named("String"), url.clone()],
                build_result(url.clone()),
            ),
            function_export(
                module,
                "renderUrl",
                [],
                Vec::new(),
                vec![url.clone()],
                named("String"),
            ),
            function_export(
                module,
                "urlOrigin",
                [],
                Vec::new(),
                vec![url.clone()],
                named("String"),
            ),
            function_export(
                module,
                "pathSegments",
                [],
                Vec::new(),
                vec![url.clone()],
                named_with("Array", vec![named("String")]),
            ),
            function_export(
                module,
                "withPathSegments",
                [],
                Vec::new(),
                vec![named_with("Array", vec![named("String")]), url.clone()],
                url.clone(),
            ),
            function_export(
                module,
                "urlQuery",
                [],
                Vec::new(),
                vec![url.clone()],
                query.clone(),
            ),
            function_export(
                module,
                "withQuery",
                [],
                Vec::new(),
                vec![query.clone(), url.clone()],
                url.clone(),
            ),
            function_export(
                module,
                "urlFragment",
                [],
                Vec::new(),
                vec![url.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "withFragment",
                [],
                Vec::new(),
                vec![named("String"), url.clone()],
                url.clone(),
            ),
            function_export(
                module,
                "withoutFragment",
                [],
                Vec::new(),
                vec![url.clone()],
                url.clone(),
            ),
            value_export(
                module,
                "emptyQuery",
                external_type(
                    "Query",
                    "std/web/navigation::Query",
                    module,
                    "Query",
                    Vec::new(),
                ),
            ),
            function_export(
                module,
                "parseQuery",
                [],
                Vec::new(),
                vec![named("String")],
                build_result(query.clone()),
            ),
            function_export(
                module,
                "appendQuery",
                [],
                Vec::new(),
                vec![named("String"), named("String"), query.clone()],
                query.clone(),
            ),
            function_export(
                module,
                "setQuery",
                [],
                Vec::new(),
                vec![named("String"), named("String"), query.clone()],
                query.clone(),
            ),
            function_export(
                module,
                "removeQuery",
                [],
                Vec::new(),
                vec![named("String"), query.clone()],
                query.clone(),
            ),
            function_export(
                module,
                "queryValues",
                [],
                Vec::new(),
                vec![named("String"), query.clone()],
                named_with("Array", vec![named("String")]),
            ),
            function_export(
                module,
                "queryEntries",
                [],
                Vec::new(),
                vec![query.clone()],
                named_with(
                    "Array",
                    vec![InterfaceType::Tuple {
                        elements: vec![named("String"), named("String")],
                    }],
                ),
            ),
            function_export(
                module,
                "renderQuery",
                [],
                Vec::new(),
                vec![query],
                named("String"),
            ),
            function_export(
                module,
                "toWebUrl",
                [],
                Vec::new(),
                vec![url.clone()],
                external_type(
                    "WebUrl",
                    "std/web/html::WebUrl",
                    "std/web/html",
                    "WebUrl",
                    Vec::new(),
                ),
            ),
            function_export(
                module,
                "locationUrl",
                [],
                Vec::new(),
                vec![location.clone()],
                url.clone(),
            ),
            effect_function_export(
                module,
                "current",
                [],
                Vec::new(),
                vec![named("Unit")],
                navigation_effect(location.clone()),
            ),
            effect_function_export(
                module,
                "push",
                [],
                Vec::new(),
                vec![url.clone()],
                navigation_effect(location.clone()),
            ),
            effect_function_export(
                module,
                "replace",
                [],
                Vec::new(),
                vec![url],
                navigation_effect(location.clone()),
            ),
            effect_function_export(
                module,
                "back",
                [],
                Vec::new(),
                vec![named("Unit")],
                navigation_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "forward",
                [],
                Vec::new(),
                vec![named("Unit")],
                navigation_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "locationSignal",
                [],
                Vec::new(),
                vec![named("Unit")],
                navigation_effect(external_type(
                    "Signal",
                    "std/signal::Signal",
                    "std/signal",
                    "Signal",
                    vec![location],
                )),
            ),
            function_export(
                module,
                "errorMessage",
                [],
                Vec::new(),
                vec![named("NavigationError")],
                named("String"),
            ),
        ],
    )
}

fn web_storage_interface() -> ModuleInterface {
    let module = "std/web/storage";
    let area = named("StorageArea");
    let storage_effect = |success: InterfaceType| {
        effect(
            record([required("storage", named("Storage"))]),
            named("StorageError"),
            success,
        )
    };
    standard_interface(
        module,
        vec![
            type_export(module, "Storage", 0, "opaque-type"),
            opaque_adt_type_export(module, "StorageArea", []),
            constructor_export(module, "StorageArea", "Local", [], None),
            constructor_export(module, "StorageArea", "Session", [], None),
            opaque_adt_type_export(module, "StorageError", []),
            constructor_export(
                module,
                "StorageError",
                "StorageQuotaExceeded",
                [],
                Some(record([
                    required("area", area.clone()),
                    required("key", named("String")),
                    required("message", named("String")),
                ])),
            ),
            constructor_export(
                module,
                "StorageError",
                "StorageSecurityFailure",
                [],
                Some(record([
                    required("area", area.clone()),
                    required("message", named("String")),
                ])),
            ),
            constructor_export(
                module,
                "StorageError",
                "StorageUnavailable",
                [],
                Some(record([
                    required("area", area.clone()),
                    required("message", named("String")),
                ])),
            ),
            effect_function_export(
                module,
                "get",
                [],
                Vec::new(),
                vec![area.clone(), named("String")],
                storage_effect(named_with("Maybe", vec![named("String")])),
            ),
            effect_function_export(
                module,
                "set",
                [],
                Vec::new(),
                vec![area.clone(), named("String"), named("String")],
                storage_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "remove",
                [],
                Vec::new(),
                vec![area.clone(), named("String")],
                storage_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "clear",
                [],
                Vec::new(),
                vec![area.clone()],
                storage_effect(named("Unit")),
            ),
            effect_function_export(
                module,
                "keys",
                [],
                Vec::new(),
                vec![area],
                storage_effect(named_with("Array", vec![named("String")])),
            ),
            function_export(
                module,
                "errorMessage",
                [],
                Vec::new(),
                vec![named("StorageError")],
                named("String"),
            ),
        ],
    )
}

pub fn is_standard_module(specifier: &str) -> bool {
    STANDARD_MODULES
        .iter()
        .any(|module| module.specifier == specifier)
}

fn web_html_interface() -> ModuleInterface {
    let mut exports = vec![
        type_export("std/web/html", "Html", 1, "opaque-type"),
        type_export("std/web/html", "Style", 0, "opaque-type"),
        type_export("std/web/html", "Tag", 0, "opaque-type"),
        type_export("std/web/html", "Attribute", 0, "opaque-type"),
        type_export("std/web/html", "WebUrl", 0, "opaque-type"),
        record_type_export(
            "std/web/html",
            "InputEvent",
            [required("value", named("String"))],
        ),
        record_type_export(
            "std/web/html",
            "ChangeEvent",
            [
                required("value", named("String")),
                required("checked", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "FileChangeEvent",
            [required(
                "files",
                named_with(
                    "Array",
                    vec![external_type(
                        "File",
                        "std/web/file::File",
                        "std/web/file",
                        "File",
                        Vec::new(),
                    )],
                ),
            )],
        ),
        record_type_export(
            "std/web/html",
            "KeyboardEvent",
            [
                required("key", named("String")),
                required("code", named("String")),
                required("repeat", named("Bool")),
                required("altKey", named("Bool")),
                required("controlKey", named("Bool")),
                required("metaKey", named("Bool")),
                required("shiftKey", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "MouseEvent",
            [
                required("button", named("Int")),
                required("clientX", named("Float")),
                required("clientY", named("Float")),
                required("altKey", named("Bool")),
                required("controlKey", named("Bool")),
                required("metaKey", named("Bool")),
                required("shiftKey", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "PointerEvent",
            [
                required("pointerId", named("Int")),
                required("pointerType", named("String")),
                required("isPrimary", named("Bool")),
                required("button", named("Int")),
                required("clientX", named("Float")),
                required("clientY", named("Float")),
                required("pressure", named("Float")),
                required("altKey", named("Bool")),
                required("controlKey", named("Bool")),
                required("metaKey", named("Bool")),
                required("shiftKey", named("Bool")),
            ],
        ),
        record_type_export(
            "std/web/html",
            "ScrollEvent",
            [
                required("scrollLeft", named("Float")),
                required("scrollTop", named("Float")),
            ],
        ),
        opaque_adt_type_export("std/web/html", "EventAction", ["Action"]),
        constructor_export(
            "std/web/html",
            "EventAction",
            "IgnoreEvent",
            ["Action"],
            None,
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "Dispatch",
            ["Action"],
            Some(named("Action")),
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "DispatchPreventDefault",
            ["Action"],
            Some(named("Action")),
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "DispatchStopPropagation",
            ["Action"],
            Some(named("Action")),
        ),
        constructor_export(
            "std/web/html",
            "EventAction",
            "DispatchPreventDefaultAndStop",
            ["Action"],
            Some(named("Action")),
        ),
        opaque_adt_type_export("std/web/html", "HtmlBuildError", []),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "InvalidTagName",
            [],
            Some(named("String")),
        ),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "InvalidAttributeName",
            [],
            Some(named("String")),
        ),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "ReservedAttributeName",
            [],
            Some(named("String")),
        ),
        constructor_export(
            "std/web/html",
            "HtmlBuildError",
            "UnsafeWebUrlScheme",
            [],
            Some(named("String")),
        ),
        trait_export("std/web/html", "IntoChildren", ["C", "Action"]),
        trait_export("std/web/html", "StyleRecord", ["R"]),
        function_export(
            "std/web/html",
            "style",
            ["R"],
            vec![InterfaceConstraint {
                name: "StyleRecord".to_owned(),
                trait_identity: Some("std/web/html::trait(StyleRecord)".to_owned()),
                arguments: vec![named("R")],
            }],
            vec![named("R")],
            named("Style"),
        ),
        function_export(
            "std/web/html",
            "customTag",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![named("HtmlBuildError"), named("Tag")]),
        ),
        function_export(
            "std/web/html",
            "attribute",
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            named_with("Either", vec![named("HtmlBuildError"), named("Attribute")]),
        ),
        function_export(
            "std/web/html",
            "parseWebUrl",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![named("HtmlBuildError"), named("WebUrl")]),
        ),
        function_export(
            "std/web/html",
            "text",
            ["Action"],
            Vec::new(),
            vec![named("String")],
            html(named("Action")),
        ),
        constrained_html_function("fragment", fragment_parameter()),
    ];
    exports.extend(html_props_aliases());
    exports.extend(STANDARD_HTML_TAGS.iter().copied().map(html_tag_export));
    exports.push(function_export(
        "std/web/html",
        "custom",
        ["Action", "C"],
        vec![InterfaceConstraint {
            name: "IntoChildren".to_owned(),
            trait_identity: Some("std/web/html::trait(IntoChildren)".to_owned()),
            arguments: vec![named("C"), named("Action")],
        }],
        vec![named("Tag"), element_props()],
        html(named("Action")),
    ));
    for renderer in ["renderToString", "renderDocument"] {
        exports.push(function_export(
            "std/web/html",
            renderer,
            ["Action"],
            Vec::new(),
            vec![html(named("Action"))],
            named("String"),
        ));
    }

    ModuleInterface {
        schema: 1,
        module: "std/web/html".to_owned(),
        source: "std/web/html.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
    }
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

fn signal_interface() -> ModuleInterface {
    let exports = vec![
        type_export("std/signal", "Signal", 1, "opaque-type"),
        type_export("std/signal", "MutableSignal", 1, "opaque-type"),
        type_export("std/signal", "SignalChange", 0, "opaque-type"),
        type_export("std/signal", "Subscription", 0, "opaque-type"),
        signal_function(
            "make",
            ["A"],
            vec![named("A")],
            task(signal_type("MutableSignal", named("A"))),
        ),
        signal_function(
            "read",
            ["A"],
            vec![signal_type("Signal", named("A"))],
            task(named("A")),
        ),
        signal_function(
            "set",
            ["A"],
            vec![named("A"), signal_type("MutableSignal", named("A"))],
            task(named("Unit")),
        ),
        signal_function(
            "update",
            ["A"],
            vec![
                function_type(vec![named("A")], named("A")),
                signal_type("MutableSignal", named("A")),
            ],
            task(named("Unit")),
        ),
        signal_function(
            "planSet",
            ["A"],
            vec![named("A"), signal_type("MutableSignal", named("A"))],
            named("SignalChange"),
        ),
        signal_function(
            "planUpdate",
            ["A"],
            vec![
                function_type(vec![named("A")], named("A")),
                signal_type("MutableSignal", named("A")),
            ],
            named("SignalChange"),
        ),
        signal_function(
            "transaction",
            [],
            vec![named_with("Array", vec![named("SignalChange")])],
            task(named("Unit")),
        ),
        signal_function(
            "map",
            ["A", "B"],
            vec![
                function_type(vec![named("A")], named("B")),
                signal_type("Signal", named("A")),
            ],
            signal_type("Signal", named("B")),
        ),
        signal_function(
            "combine",
            ["A", "B", "C"],
            vec![
                function_type(vec![named("A"), named("B")], named("C")),
                signal_type("Signal", named("A")),
                signal_type("Signal", named("B")),
            ],
            signal_type("Signal", named("C")),
        ),
        signal_function(
            "constant",
            ["A"],
            vec![named("A")],
            signal_type("Signal", named("A")),
        ),
        function_export(
            "std/signal",
            "distinct",
            ["A"],
            vec![InterfaceConstraint {
                name: "Eq".to_owned(),
                trait_identity: Some("std/prelude::Eq".to_owned()),
                arguments: vec![named("A")],
            }],
            vec![signal_type("Signal", named("A"))],
            signal_type("Signal", named("A")),
        ),
        signal_function(
            "switchMap",
            ["A", "B"],
            vec![
                function_type(vec![named("A")], signal_type("Signal", named("B"))),
                signal_type("Signal", named("A")),
            ],
            signal_type("Signal", named("B")),
        ),
        signal_function(
            "subscribe",
            ["R", "A"],
            vec![
                function_type(
                    vec![named("A")],
                    effect(named("R"), named("Never"), named("Unit")),
                ),
                signal_type("Signal", named("A")),
            ],
            effect(named("R"), named("Never"), named("Subscription")),
        ),
        signal_function(
            "unsubscribe",
            [],
            vec![named("Subscription")],
            task(named("Unit")),
        ),
    ];
    ModuleInterface {
        schema: 1,
        module: "std/signal".to_owned(),
        source: "std/signal.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances: Vec::new(),
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
            "onPointerUp",
            function_type(vec![html_event_type("PointerEvent")], event_action_type()),
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
        constructor_of: None,
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
        constructor_of: None,
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
        constructor_of: Some(format!("{module}::{owner}")),
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
        constructor_of: Some(owner_canonical.to_owned()),
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
        constructor_of: None,
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
        constructor_of: None,
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
        constructor_of: None,
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
        assert!(standard_module_target("std/bytes/hex").is_none());
        assert!(standard_module_target("std/bytes/base64").is_none());
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
}
