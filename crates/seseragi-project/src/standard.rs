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
    contract_module!("std/char", PORTABLE_TARGETS),
    contract_module!(
        "std/child-process",
        PROCESS_TARGET,
        &["std/child-process::ChildProcesses"]
    ),
    contract_module!("std/collection", PORTABLE_TARGETS),
    contract_module!("std/console", PORTABLE_TARGETS, &["std/prelude::Console"]),
    contract_module!("std/decimal", PORTABLE_TARGETS),
    available_module!("std/deferred", deferred_interface, PORTABLE_TARGETS),
    available_module!("std/effect", effect_interface, PORTABLE_TARGETS),
    contract_module!("std/either", PORTABLE_TARGETS),
    contract_module!("std/entropy", PORTABLE_TARGETS, &["std/entropy::Entropy"]),
    contract_module!("std/fs", PORTABLE_TARGETS, &["std/fs::FileSystem"]),
    contract_module!(
        "std/http/bun",
        PROCESS_TARGET,
        &["std/http/bun::BunHttpServer"]
    ),
    contract_module!("std/iterator", PORTABLE_TARGETS),
    available_module!("std/json", json_interface, PORTABLE_TARGETS),
    contract_module!("std/log", PORTABLE_TARGETS, &["std/log::Logger"]),
    contract_module!("std/map", PORTABLE_TARGETS),
    contract_module!("std/maybe", PORTABLE_TARGETS),
    contract_module!("std/non-empty-list", PORTABLE_TARGETS),
    contract_module!("std/path", PORTABLE_TARGETS),
    contract_module!("std/process", PROCESS_TARGET, &["std/process::Process"]),
    available_module!("std/queue", queue_interface, PORTABLE_TARGETS),
    contract_module!("std/random", PORTABLE_TARGETS, &["std/random::Random"]),
    available_module!("std/ref", ref_interface, PORTABLE_TARGETS),
    contract_module!("std/regex", PORTABLE_TARGETS),
    available_module!("std/semaphore", semaphore_interface, PORTABLE_TARGETS),
    contract_module!("std/set", PORTABLE_TARGETS),
    contract_module!("std/stdin", PROCESS_TARGET, &["std/prelude::Stdin"]),
    available_module!("std/stream", stream_interface, PORTABLE_TARGETS),
    contract_module!("std/test", PORTABLE_TARGETS),
    available_module!("std/text", text_interface, PORTABLE_TARGETS),
    contract_module!("std/text/grapheme", PORTABLE_TARGETS),
    contract_module!("std/text/unicode", PORTABLE_TARGETS),
    contract_module!("std/transformer/either", PORTABLE_TARGETS),
    contract_module!("std/transformer/maybe", PORTABLE_TARGETS),
    contract_module!("std/transformer/reader", PORTABLE_TARGETS),
    contract_module!("std/transformer/state", PORTABLE_TARGETS),
    contract_module!("std/transformer/writer", PORTABLE_TARGETS),
    contract_module!("std/validation", PORTABLE_TARGETS),
];

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

fn time_interface() -> ModuleInterface {
    let module = "std/time";
    let duration = named("Duration");
    let duration_result = named_with("Either", vec![named("DurationError"), duration.clone()]);
    standard_interface(
        module,
        vec![
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
        ],
    )
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
            vec![named("Int"), headers.clone(), bytes],
            response.clone(),
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
    standard_interface(
        module,
        vec![
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
        ],
    )
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
    let mapped_values = named_with(collection, vec![named("B")]);
    let maybe_value = named_with("Maybe", vec![named("A")]);
    let exports = vec![
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
            named_with("Maybe", vec![values]),
        ),
    ];
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
            ["Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(record([]), named("Failure"), named("Unit")),
                ),
                signal_html("Action"),
            ],
            effect(
                dom_environment.clone(),
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
            ["Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(record([]), named("Failure"), named("Unit")),
                ),
                named_with("DomContent", vec![named("Action")]),
            ],
            effect(
                dom_environment.clone(),
                named("DomError"),
                named_with("DomMount", vec![named("Failure")]),
            ),
        ),
        function_export(
            module,
            "runContent",
            ["Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(record([]), named("Failure"), named("Unit")),
                ),
                named_with("DomContent", vec![named("Action")]),
            ],
            effect(
                dom_environment.clone(),
                named_with("DomRuntimeError", vec![named("Failure")]),
                named("Unit"),
            ),
        ),
        function_export(
            module,
            "run",
            ["Action", "Failure"],
            Vec::new(),
            vec![
                named("DomOptions"),
                named("DomTarget"),
                function_type(
                    vec![named("Action")],
                    effect(record([]), named("Failure"), named("Unit")),
                ),
                signal_html("Action"),
            ],
            effect(
                dom_environment,
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
        }
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
