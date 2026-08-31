#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeTypeImport {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
}

const RUNTIME_TYPE_IMPORTS: &[RuntimeTypeImport] = &[
    RuntimeTypeImport {
        canonical: "std/test::Test",
        runtime_feature: "test.tree.type",
        module: "@seseragi/runtime/test",
        export_name: "Test",
    },
    RuntimeTypeImport {
        canonical: "std/test::TestFailure",
        runtime_feature: "test.failure.type",
        module: "@seseragi/runtime/test",
        export_name: "TestFailure",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Show",
        runtime_feature: "core.show.dictionary",
        module: "@seseragi/runtime/show",
        export_name: "Show",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Debug",
        runtime_feature: "core.debug.dictionary",
        module: "@seseragi/runtime/show",
        export_name: "Debug",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::JsonEncode",
        runtime_feature: "json.encode-dictionary-type",
        module: "@seseragi/runtime/json",
        export_name: "JsonEncode",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::JsonDecode",
        runtime_feature: "json.decode-dictionary-type",
        module: "@seseragi/runtime/json",
        export_name: "JsonDecode",
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
        canonical: "std/prelude::Js.Error",
        runtime_feature: "foreign.js-error.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsError",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Unknown",
        runtime_feature: "foreign.js-unknown.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsUnknown",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.NullOr",
        runtime_feature: "foreign.js-null-or.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsNullOr",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Nullable",
        runtime_feature: "foreign.js-nullable.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsNullable",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.UndefinedOr",
        runtime_feature: "foreign.js-undefined-or.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsUndefinedOr",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Promise",
        runtime_feature: "foreign.js-promise.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsPromise",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Object",
        runtime_feature: "foreign.js-object.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsObject",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Number",
        runtime_feature: "foreign.js-number.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsNumber",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.String",
        runtime_feature: "foreign.js-string.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsString",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Null",
        runtime_feature: "foreign.js-null.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsNull",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Undefined",
        runtime_feature: "foreign.js-undefined.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsUndefined",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.MutableArray",
        runtime_feature: "foreign.js-mutable-array.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsMutableArray",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Js.Callback",
        runtime_feature: "foreign.js-callback.type",
        module: "@seseragi/runtime/foreign",
        export_name: "JsCallback",
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
    RuntimeTypeImport {
        canonical: "std/prelude::Effect",
        runtime_feature: "effect.core.type",
        module: "@seseragi/runtime/effect",
        export_name: "Effect",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::Iterator",
        runtime_feature: "core.iterator",
        module: "@seseragi/runtime/iterator",
        export_name: "Iterator",
    },
    RuntimeTypeImport {
        canonical: "std/prelude::List",
        runtime_feature: "core.list",
        module: "@seseragi/runtime/list",
        export_name: "List",
    },
    RuntimeTypeImport {
        canonical: "std/non-empty-list::NonEmptyList",
        runtime_feature: "core.non-empty-list",
        module: "@seseragi/runtime/list",
        export_name: "NonEmptyList",
    },
    RuntimeTypeImport {
        canonical: "std/number::RoundingMode",
        runtime_feature: "core.number.rounding-mode",
        module: "@seseragi/runtime/number",
        export_name: "RoundingMode",
    },
    RuntimeTypeImport {
        canonical: "std/int::IntParseError",
        runtime_feature: "core.int.parse-error",
        module: "@seseragi/runtime/int",
        export_name: "IntParseError",
    },
    RuntimeTypeImport {
        canonical: "std/int::IntDivisionError",
        runtime_feature: "core.int.division-error",
        module: "@seseragi/runtime/int",
        export_name: "IntDivisionError",
    },
    RuntimeTypeImport {
        canonical: "std/int::IntPowerError",
        runtime_feature: "core.int.power-error",
        module: "@seseragi/runtime/int",
        export_name: "IntPowerError",
    },
    RuntimeTypeImport {
        canonical: "std/float::FloatParseError",
        runtime_feature: "core.float64.parse-error",
        module: "@seseragi/runtime/float",
        export_name: "FloatParseError",
    },
    RuntimeTypeImport {
        canonical: "std/float::FloatConversionError",
        runtime_feature: "core.float64.conversion-error",
        module: "@seseragi/runtime/float",
        export_name: "FloatConversionError",
    },
    RuntimeTypeImport {
        canonical: "std/bytes::Byte",
        runtime_feature: "core.bytes.byte-type",
        module: "@seseragi/runtime/bytes",
        export_name: "Byte",
    },
    RuntimeTypeImport {
        canonical: "std/bytes::Bytes",
        runtime_feature: "core.bytes.type",
        module: "@seseragi/runtime/bytes",
        export_name: "Bytes",
    },
    RuntimeTypeImport {
        canonical: "std/bytes::ByteError",
        runtime_feature: "core.bytes.byte-error-type",
        module: "@seseragi/runtime/bytes",
        export_name: "ByteError",
    },
    RuntimeTypeImport {
        canonical: "std/bytes::BytesSliceError",
        runtime_feature: "core.bytes.slice-error-type",
        module: "@seseragi/runtime/bytes",
        export_name: "BytesSliceError",
    },
    RuntimeTypeImport {
        canonical: "std/text::Utf8DecodeError",
        runtime_feature: "core.text.utf8-error-type",
        module: "@seseragi/runtime/text",
        export_name: "Utf8DecodeError",
    },
    RuntimeTypeImport {
        canonical: "std/decimal::Decimal",
        runtime_feature: "json.decimal-type",
        module: "@seseragi/runtime/json",
        export_name: "Decimal",
    },
    RuntimeTypeImport {
        canonical: "std/map::Map",
        runtime_feature: "json.map-type",
        module: "@seseragi/runtime/json",
        export_name: "JsonMap",
    },
    RuntimeTypeImport {
        canonical: "std/json::Json",
        runtime_feature: "json.value-type",
        module: "@seseragi/runtime/json",
        export_name: "Json",
    },
    RuntimeTypeImport {
        canonical: "std/json::Decoder",
        runtime_feature: "json.decoder-type",
        module: "@seseragi/runtime/json",
        export_name: "Decoder",
    },
    RuntimeTypeImport {
        canonical: "std/json::Encoder",
        runtime_feature: "json.encoder-type",
        module: "@seseragi/runtime/json",
        export_name: "Encoder",
    },
    RuntimeTypeImport {
        canonical: "std/json::JsonPathSegment",
        runtime_feature: "json.path-type",
        module: "@seseragi/runtime/json",
        export_name: "JsonPathSegment",
    },
    RuntimeTypeImport {
        canonical: "std/json::DecodeErrorKind",
        runtime_feature: "json.decode-error-kind-type",
        module: "@seseragi/runtime/json",
        export_name: "DecodeErrorKind",
    },
    RuntimeTypeImport {
        canonical: "std/json::DecodeError",
        runtime_feature: "json.decode-error-type",
        module: "@seseragi/runtime/json",
        export_name: "DecodeError",
    },
    RuntimeTypeImport {
        canonical: "std/json::JsonParseError",
        runtime_feature: "json.parse-error-type",
        module: "@seseragi/runtime/json",
        export_name: "JsonParseError",
    },
    RuntimeTypeImport {
        canonical: "std/json::JsonReadError",
        runtime_feature: "json.read-error-type",
        module: "@seseragi/runtime/json",
        export_name: "JsonReadError",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Html",
        runtime_feature: "web.html.type",
        module: "@seseragi/runtime/html",
        export_name: "Html",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Style",
        runtime_feature: "web.html.style-type",
        module: "@seseragi/runtime/html",
        export_name: "Style",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Tag",
        runtime_feature: "web.html.tag-type",
        module: "@seseragi/runtime/html",
        export_name: "Tag",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::Attribute",
        runtime_feature: "web.html.attribute-type",
        module: "@seseragi/runtime/html",
        export_name: "Attribute",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::WebUrl",
        runtime_feature: "web.html.url-type",
        module: "@seseragi/runtime/html",
        export_name: "WebUrl",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::HtmlBuildError",
        runtime_feature: "web.html.build-error-type",
        module: "@seseragi/runtime/html",
        export_name: "HtmlBuildError",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::InputEvent",
        runtime_feature: "web.html.input-event-type",
        module: "@seseragi/runtime/html",
        export_name: "InputEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::ChangeEvent",
        runtime_feature: "web.html.change-event-type",
        module: "@seseragi/runtime/html",
        export_name: "ChangeEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::FileChangeEvent",
        runtime_feature: "web.html.file-change-event-type",
        module: "@seseragi/runtime/html",
        export_name: "FileChangeEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::KeyboardEvent",
        runtime_feature: "web.html.keyboard-event-type",
        module: "@seseragi/runtime/html",
        export_name: "KeyboardEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::MouseEvent",
        runtime_feature: "web.html.mouse-event-type",
        module: "@seseragi/runtime/html",
        export_name: "MouseEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::PointerEvent",
        runtime_feature: "web.html.pointer-event-type",
        module: "@seseragi/runtime/html",
        export_name: "PointerEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::ScrollEvent",
        runtime_feature: "web.html.scroll-event-type",
        module: "@seseragi/runtime/html",
        export_name: "ScrollEvent",
    },
    RuntimeTypeImport {
        canonical: "std/web/html::EventAction",
        runtime_feature: "web.html.event-action-type",
        module: "@seseragi/runtime/html",
        export_name: "EventAction",
    },
    RuntimeTypeImport {
        canonical: "std/web/file::Blob",
        runtime_feature: "web.file.blob-type",
        module: "@seseragi/runtime/web-file",
        export_name: "Blob",
    },
    RuntimeTypeImport {
        canonical: "std/web/file::File",
        runtime_feature: "web.file.file-type",
        module: "@seseragi/runtime/web-file",
        export_name: "File",
    },
    RuntimeTypeImport {
        canonical: "std/web/file::BlobBuildError",
        runtime_feature: "web.file.build-error-type",
        module: "@seseragi/runtime/web-file",
        export_name: "BlobBuildError",
    },
    RuntimeTypeImport {
        canonical: "std/web/file::BlobReadError",
        runtime_feature: "web.file.read-error-type",
        module: "@seseragi/runtime/web-file",
        export_name: "BlobReadError",
    },
    RuntimeTypeImport {
        canonical: "std/web/navigation::Navigation",
        runtime_feature: "web.navigation.service-type",
        module: "@seseragi/runtime/navigation",
        export_name: "Navigation",
    },
    RuntimeTypeImport {
        canonical: "std/web/navigation::Url",
        runtime_feature: "web.navigation.url-type",
        module: "@seseragi/runtime/navigation",
        export_name: "Url",
    },
    RuntimeTypeImport {
        canonical: "std/web/navigation::Query",
        runtime_feature: "web.navigation.query-type",
        module: "@seseragi/runtime/navigation",
        export_name: "Query",
    },
    RuntimeTypeImport {
        canonical: "std/web/navigation::Location",
        runtime_feature: "web.navigation.location-type",
        module: "@seseragi/runtime/navigation",
        export_name: "Location",
    },
    RuntimeTypeImport {
        canonical: "std/web/navigation::UrlBuildError",
        runtime_feature: "web.navigation.url-error-type",
        module: "@seseragi/runtime/navigation",
        export_name: "UrlBuildError",
    },
    RuntimeTypeImport {
        canonical: "std/web/navigation::NavigationError",
        runtime_feature: "web.navigation.error-type",
        module: "@seseragi/runtime/navigation",
        export_name: "NavigationError",
    },
    RuntimeTypeImport {
        canonical: "std/web/storage::Storage",
        runtime_feature: "web.storage.service-type",
        module: "@seseragi/runtime/storage",
        export_name: "Storage",
    },
    RuntimeTypeImport {
        canonical: "std/web/storage::StorageArea",
        runtime_feature: "web.storage.area-type",
        module: "@seseragi/runtime/storage",
        export_name: "StorageArea",
    },
    RuntimeTypeImport {
        canonical: "std/web/storage::StorageError",
        runtime_feature: "web.storage.error-type",
        module: "@seseragi/runtime/storage",
        export_name: "StorageError",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::Dom",
        runtime_feature: "web.dom.service",
        module: "@seseragi/runtime/dom",
        export_name: "Dom",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomOptions",
        runtime_feature: "web.dom.options",
        module: "@seseragi/runtime/dom",
        export_name: "DomOptions",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::HydrationMode",
        runtime_feature: "web.dom.hydration-mode",
        module: "@seseragi/runtime/dom",
        export_name: "HydrationMode",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::CleanupMode",
        runtime_feature: "web.dom.cleanup-mode",
        module: "@seseragi/runtime/dom",
        export_name: "CleanupMode",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomTarget",
        runtime_feature: "web.dom.target",
        module: "@seseragi/runtime/dom",
        export_name: "DomTarget",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomError",
        runtime_feature: "web.dom.error",
        module: "@seseragi/runtime/dom",
        export_name: "DomError",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomRuntimeError",
        runtime_feature: "web.dom.runtime-error",
        module: "@seseragi/runtime/dom",
        export_name: "DomRuntimeError",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomMount",
        runtime_feature: "web.dom.mount-type",
        module: "@seseragi/runtime/dom",
        export_name: "DomMount",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomContent",
        runtime_feature: "web.dom.content-type",
        module: "@seseragi/runtime/dom",
        export_name: "DomContent",
    },
    RuntimeTypeImport {
        canonical: "std/web/dom::DomBinding",
        runtime_feature: "web.dom.binding-type",
        module: "@seseragi/runtime/dom",
        export_name: "DomBinding",
    },
    RuntimeTypeImport {
        canonical: "std/signal::Signal",
        runtime_feature: "signal.type",
        module: "@seseragi/runtime/signal",
        export_name: "Signal",
    },
    RuntimeTypeImport {
        canonical: "std/signal::MutableSignal",
        runtime_feature: "signal.mutable-type",
        module: "@seseragi/runtime/signal",
        export_name: "MutableSignal",
    },
    RuntimeTypeImport {
        canonical: "std/signal::SignalChange",
        runtime_feature: "signal.change-type",
        module: "@seseragi/runtime/signal",
        export_name: "SignalChange",
    },
    RuntimeTypeImport {
        canonical: "std/signal::Subscription",
        runtime_feature: "signal.subscription-type",
        module: "@seseragi/runtime/signal",
        export_name: "Subscription",
    },
    RuntimeTypeImport {
        canonical: "std/clock::Clock",
        runtime_feature: "clock.service-type",
        module: "@seseragi/runtime/clock",
        export_name: "Clock",
    },
    RuntimeTypeImport {
        canonical: "std/log::Logger",
        runtime_feature: "logger.service-type",
        module: "@seseragi/runtime/logger",
        export_name: "Logger",
    },
    RuntimeTypeImport {
        canonical: "std/log::LogLevel",
        runtime_feature: "logger.level-type",
        module: "@seseragi/runtime/logger",
        export_name: "LogLevel",
    },
    RuntimeTypeImport {
        canonical: "std/log::LogValue",
        runtime_feature: "logger.value-type",
        module: "@seseragi/runtime/logger",
        export_name: "LogValue",
    },
    RuntimeTypeImport {
        canonical: "std/log::LogEvent",
        runtime_feature: "logger.event-type",
        module: "@seseragi/runtime/logger",
        export_name: "LogEvent",
    },
    RuntimeTypeImport {
        canonical: "std/log::LogError",
        runtime_feature: "logger.error-type",
        module: "@seseragi/runtime/logger",
        export_name: "LogError",
    },
    RuntimeTypeImport {
        canonical: "std/stdin::StdinConfigError",
        runtime_feature: "stdin.config-error-type",
        module: "@seseragi/runtime/stdin",
        export_name: "StdinConfigError",
    },
    RuntimeTypeImport {
        canonical: "std/stdin::ReadSize",
        runtime_feature: "stdin.read-size-type",
        module: "@seseragi/runtime/stdin",
        export_name: "ReadSize",
    },
    RuntimeTypeImport {
        canonical: "std/stdin::LineLimit",
        runtime_feature: "stdin.line-limit-type",
        module: "@seseragi/runtime/stdin",
        export_name: "LineLimit",
    },
    RuntimeTypeImport {
        canonical: "std/path::Path",
        runtime_feature: "path.type",
        module: "@seseragi/runtime/path",
        export_name: "Path",
    },
    RuntimeTypeImport {
        canonical: "std/path::PathError",
        runtime_feature: "path.error-type",
        module: "@seseragi/runtime/path",
        export_name: "PathError",
    },
    RuntimeTypeImport {
        canonical: "std/process::Process",
        runtime_feature: "process.service-type",
        module: "@seseragi/runtime/process",
        export_name: "Process",
    },
    RuntimeTypeImport {
        canonical: "std/process::ProcessSignal",
        runtime_feature: "process.signal-type",
        module: "@seseragi/runtime/process",
        export_name: "ProcessSignal",
    },
    RuntimeTypeImport {
        canonical: "std/process::ProcessError",
        runtime_feature: "process.error-type",
        module: "@seseragi/runtime/process",
        export_name: "ProcessError",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildProcesses",
        runtime_feature: "child-process.service-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildProcesses",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::Executable",
        runtime_feature: "child-process.executable-type",
        module: "@seseragi/runtime/child-process",
        export_name: "Executable",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::Command",
        runtime_feature: "child-process.command-type",
        module: "@seseragi/runtime/child-process",
        export_name: "Command",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::CaptureLimit",
        runtime_feature: "child-process.capture-limit-type",
        module: "@seseragi/runtime/child-process",
        export_name: "CaptureLimit",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildProcessConfigError",
        runtime_feature: "child-process.config-error-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildProcessConfigError",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildOutputChannel",
        runtime_feature: "child-process.output-channel-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildOutputChannel",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildProcessError",
        runtime_feature: "child-process.error-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildProcessError",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildExitStatus",
        runtime_feature: "child-process.exit-status-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildExitStatus",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildInput",
        runtime_feature: "child-process.input-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildInput",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::ChildEvent",
        runtime_feature: "child-process.event-type",
        module: "@seseragi/runtime/child-process",
        export_name: "ChildEvent",
    },
    RuntimeTypeImport {
        canonical: "std/child-process::CapturedProcess",
        runtime_feature: "child-process.captured-type",
        module: "@seseragi/runtime/child-process",
        export_name: "CapturedProcess",
    },
    RuntimeTypeImport {
        canonical: "std/random::Random",
        runtime_feature: "random.service-type",
        module: "@seseragi/runtime/random",
        export_name: "Random",
    },
    RuntimeTypeImport {
        canonical: "std/random::RandomRangeError",
        runtime_feature: "random.range-error-type",
        module: "@seseragi/runtime/random",
        export_name: "RandomRangeError",
    },
    RuntimeTypeImport {
        canonical: "std/random::RandomConfigError",
        runtime_feature: "random.config-error-type",
        module: "@seseragi/runtime/random",
        export_name: "RandomConfigError",
    },
    RuntimeTypeImport {
        canonical: "std/random::RandomSize",
        runtime_feature: "random.size-type",
        module: "@seseragi/runtime/random",
        export_name: "RandomSize",
    },
    RuntimeTypeImport {
        canonical: "std/entropy::Entropy",
        runtime_feature: "entropy.service-type",
        module: "@seseragi/runtime/entropy",
        export_name: "Entropy",
    },
    RuntimeTypeImport {
        canonical: "std/entropy::EntropyConfigError",
        runtime_feature: "entropy.config-error-type",
        module: "@seseragi/runtime/entropy",
        export_name: "EntropyConfigError",
    },
    RuntimeTypeImport {
        canonical: "std/entropy::EntropyError",
        runtime_feature: "entropy.error-type",
        module: "@seseragi/runtime/entropy",
        export_name: "EntropyError",
    },
    RuntimeTypeImport {
        canonical: "std/entropy::EntropySize",
        runtime_feature: "entropy.size-type",
        module: "@seseragi/runtime/entropy",
        export_name: "EntropySize",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileSystem",
        runtime_feature: "filesystem.service-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileSystem",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileType",
        runtime_feature: "filesystem.file-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileType",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileSystemOperation",
        runtime_feature: "filesystem.operation-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileSystemOperation",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileSystemErrorKind",
        runtime_feature: "filesystem.error-kind-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileSystemErrorKind",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileSystemError",
        runtime_feature: "filesystem.error-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileSystemError",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileMetadata",
        runtime_feature: "filesystem.metadata-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileMetadata",
    },
    RuntimeTypeImport {
        canonical: "std/fs::DirectoryEntry",
        runtime_feature: "filesystem.directory-entry-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "DirectoryEntry",
    },
    RuntimeTypeImport {
        canonical: "std/fs::WriteMode",
        runtime_feature: "filesystem.write-mode-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "WriteMode",
    },
    RuntimeTypeImport {
        canonical: "std/fs::FileTextError",
        runtime_feature: "filesystem.text-error-type",
        module: "@seseragi/runtime/filesystem",
        export_name: "FileTextError",
    },
    RuntimeTypeImport {
        canonical: "std/time::Instant",
        runtime_feature: "clock.instant-type",
        module: "@seseragi/runtime/clock",
        export_name: "Instant",
    },
    RuntimeTypeImport {
        canonical: "std/time::Duration",
        runtime_feature: "time.duration.type",
        module: "@seseragi/runtime/clock",
        export_name: "Duration",
    },
    RuntimeTypeImport {
        canonical: "std/time::DurationError",
        runtime_feature: "time.duration.error-type",
        module: "@seseragi/runtime/clock",
        export_name: "DurationError",
    },
    RuntimeTypeImport {
        canonical: "std/time::LocalDate",
        runtime_feature: "time.local-date.type",
        module: "@seseragi/runtime/time",
        export_name: "LocalDate",
    },
    RuntimeTypeImport {
        canonical: "std/time::LocalTime",
        runtime_feature: "time.local-time.type",
        module: "@seseragi/runtime/time",
        export_name: "LocalTime",
    },
    RuntimeTypeImport {
        canonical: "std/time::LocalDateTime",
        runtime_feature: "time.local-date-time.type",
        module: "@seseragi/runtime/time",
        export_name: "LocalDateTime",
    },
    RuntimeTypeImport {
        canonical: "std/time::UtcOffset",
        runtime_feature: "time.utc-offset.type",
        module: "@seseragi/runtime/time",
        export_name: "UtcOffset",
    },
    RuntimeTypeImport {
        canonical: "std/time::OffsetDateTime",
        runtime_feature: "time.offset-date-time.type",
        module: "@seseragi/runtime/time",
        export_name: "OffsetDateTime",
    },
    RuntimeTypeImport {
        canonical: "std/time::TimeZone",
        runtime_feature: "time.zone.type",
        module: "@seseragi/runtime/time",
        export_name: "TimeZone",
    },
    RuntimeTypeImport {
        canonical: "std/time::ZonedDateTime",
        runtime_feature: "time.zoned-date-time.type",
        module: "@seseragi/runtime/time",
        export_name: "ZonedDateTime",
    },
    RuntimeTypeImport {
        canonical: "std/time::TimeZones",
        runtime_feature: "time.zones.service-type",
        module: "@seseragi/runtime/time",
        export_name: "TimeZones",
    },
    RuntimeTypeImport {
        canonical: "std/time::DateTimeError",
        runtime_feature: "time.date-time-error.type",
        module: "@seseragi/runtime/time",
        export_name: "DateTimeError",
    },
    RuntimeTypeImport {
        canonical: "std/time::TimeZoneError",
        runtime_feature: "time.zone-error.type",
        module: "@seseragi/runtime/time",
        export_name: "TimeZoneError",
    },
    RuntimeTypeImport {
        canonical: "std/time::LocalResolution",
        runtime_feature: "time.local-resolution.type",
        module: "@seseragi/runtime/time",
        export_name: "LocalResolution",
    },
    RuntimeTypeImport {
        canonical: "std/effect::Schedule",
        runtime_feature: "effect.schedule.type",
        module: "@seseragi/runtime/effect",
        export_name: "Schedule",
    },
    RuntimeTypeImport {
        canonical: "std/effect::ScheduleDecision",
        runtime_feature: "effect.schedule.decision-type",
        module: "@seseragi/runtime/effect",
        export_name: "ScheduleDecision",
    },
    RuntimeTypeImport {
        canonical: "std/effect::ScheduleError",
        runtime_feature: "effect.schedule.error-type",
        module: "@seseragi/runtime/effect",
        export_name: "ScheduleError",
    },
    RuntimeTypeImport {
        canonical: "std/effect::Fiber",
        runtime_feature: "effect.fiber.type",
        module: "@seseragi/runtime/effect",
        export_name: "Fiber",
    },
    RuntimeTypeImport {
        canonical: "std/effect::FiberExit",
        runtime_feature: "effect.fiber.exit-type",
        module: "@seseragi/runtime/effect",
        export_name: "FiberExit",
    },
    RuntimeTypeImport {
        canonical: "std/effect::Parallelism",
        runtime_feature: "effect.parallelism.type",
        module: "@seseragi/runtime/effect",
        export_name: "Parallelism",
    },
    RuntimeTypeImport {
        canonical: "std/effect::ParallelismError",
        runtime_feature: "effect.parallelism.error-type",
        module: "@seseragi/runtime/effect",
        export_name: "ParallelismError",
    },
    RuntimeTypeImport {
        canonical: "std/stream::Stream",
        runtime_feature: "stream.type",
        module: "@seseragi/runtime/stream",
        export_name: "Stream",
    },
    RuntimeTypeImport {
        canonical: "std/stream::BufferCapacity",
        runtime_feature: "stream.buffer.capacity-type",
        module: "@seseragi/runtime/stream",
        export_name: "BufferCapacity",
    },
    RuntimeTypeImport {
        canonical: "std/stream::BufferCapacityError",
        runtime_feature: "stream.buffer.capacity-error-type",
        module: "@seseragi/runtime/stream",
        export_name: "BufferCapacityError",
    },
    RuntimeTypeImport {
        canonical: "std/deferred::Deferred",
        runtime_feature: "effect.deferred.type",
        module: "@seseragi/runtime/deferred",
        export_name: "Deferred",
    },
    RuntimeTypeImport {
        canonical: "std/queue::Queue",
        runtime_feature: "effect.queue.type",
        module: "@seseragi/runtime/queue",
        export_name: "Queue",
    },
    RuntimeTypeImport {
        canonical: "std/queue::QueueCreateError",
        runtime_feature: "effect.queue.create-error-type",
        module: "@seseragi/runtime/queue",
        export_name: "QueueCreateError",
    },
    RuntimeTypeImport {
        canonical: "std/queue::QueueClosed",
        runtime_feature: "effect.queue.closed-type",
        module: "@seseragi/runtime/queue",
        export_name: "QueueClosed",
    },
    RuntimeTypeImport {
        canonical: "std/semaphore::Semaphore",
        runtime_feature: "effect.semaphore.type",
        module: "@seseragi/runtime/semaphore",
        export_name: "Semaphore",
    },
    RuntimeTypeImport {
        canonical: "std/semaphore::Permit",
        runtime_feature: "effect.semaphore.permit-type",
        module: "@seseragi/runtime/semaphore",
        export_name: "Permit",
    },
    RuntimeTypeImport {
        canonical: "std/semaphore::SemaphoreCreateError",
        runtime_feature: "effect.semaphore.create-error-type",
        module: "@seseragi/runtime/semaphore",
        export_name: "SemaphoreCreateError",
    },
    RuntimeTypeImport {
        canonical: "std/ref::Ref",
        runtime_feature: "effect.ref.type",
        module: "@seseragi/runtime/ref",
        export_name: "Ref",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpClient",
        runtime_feature: "http-client.service-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpClient",
    },
    RuntimeTypeImport {
        canonical: "std/http::Method",
        runtime_feature: "http-client.method-type",
        module: "@seseragi/runtime/http-client",
        export_name: "Method",
    },
    RuntimeTypeImport {
        canonical: "std/http::Status",
        runtime_feature: "http-client.status-type",
        module: "@seseragi/runtime/http-client",
        export_name: "Status",
    },
    RuntimeTypeImport {
        canonical: "std/http::Headers",
        runtime_feature: "http-client.headers-type",
        module: "@seseragi/runtime/http-client",
        export_name: "Headers",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpUrl",
        runtime_feature: "http-client.url-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpUrl",
    },
    RuntimeTypeImport {
        canonical: "std/http::Request",
        runtime_feature: "http-client.request-type",
        module: "@seseragi/runtime/http-client",
        export_name: "Request",
    },
    RuntimeTypeImport {
        canonical: "std/http::Response",
        runtime_feature: "http-client.response-type",
        module: "@seseragi/runtime/http-client",
        export_name: "Response",
    },
    RuntimeTypeImport {
        canonical: "std/http::Body",
        runtime_feature: "http-client.body-type",
        module: "@seseragi/runtime/http-client",
        export_name: "Body",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpVersion",
        runtime_feature: "http-client.version-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpVersion",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpEvent",
        runtime_feature: "http-client.event-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpEvent",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpBodyLimit",
        runtime_feature: "http-client.body-limit-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpBodyLimit",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpBuildError",
        runtime_feature: "http-client.build-error-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpBuildError",
    },
    RuntimeTypeImport {
        canonical: "std/http::HttpError",
        runtime_feature: "http-client.error-type",
        module: "@seseragi/runtime/http-client",
        export_name: "HttpError",
    },
    RuntimeTypeImport {
        canonical: "std/http/server::HttpServer",
        runtime_feature: "http-server.service-type",
        module: "@seseragi/runtime/http-server",
        export_name: "HttpServer",
    },
    RuntimeTypeImport {
        canonical: "std/http/server::HttpServerRequest",
        runtime_feature: "http-server.request-type",
        module: "@seseragi/runtime/http-server",
        export_name: "HttpServerRequest",
    },
    RuntimeTypeImport {
        canonical: "std/http/server::HttpServerResponse",
        runtime_feature: "http-server.response-type",
        module: "@seseragi/runtime/http-server",
        export_name: "HttpServerResponse",
    },
    RuntimeTypeImport {
        canonical: "std/http/server::HttpServerHandle",
        runtime_feature: "http-server.handle-type",
        module: "@seseragi/runtime/http-server",
        export_name: "HttpServerHandle",
    },
    RuntimeTypeImport {
        canonical: "std/http/server::HttpServerError",
        runtime_feature: "http-server.error-type",
        module: "@seseragi/runtime/http-server",
        export_name: "HttpServerError",
    },
    RuntimeTypeImport {
        canonical: "std/http/server::HttpHeader",
        runtime_feature: "http-server.header-type",
        module: "@seseragi/runtime/http-server",
        export_name: "HttpHeader",
    },
    RuntimeTypeImport {
        canonical: "std/http/multipart::Multipart",
        runtime_feature: "http.multipart.type",
        module: "@seseragi/runtime/multipart",
        export_name: "Multipart",
    },
    RuntimeTypeImport {
        canonical: "std/http/multipart::MultipartBuildError",
        runtime_feature: "http.multipart.build-error-type",
        module: "@seseragi/runtime/multipart",
        export_name: "MultipartBuildError",
    },
    RuntimeTypeImport {
        canonical: "std/sse::Event",
        runtime_feature: "sse.event-type",
        module: "@seseragi/runtime/sse",
        export_name: "Event",
    },
    RuntimeTypeImport {
        canonical: "std/sse::DecodeLimit",
        runtime_feature: "sse.decode-limit-type",
        module: "@seseragi/runtime/sse",
        export_name: "DecodeLimit",
    },
    RuntimeTypeImport {
        canonical: "std/sse::SseBuildError",
        runtime_feature: "sse.build-error-type",
        module: "@seseragi/runtime/sse",
        export_name: "SseBuildError",
    },
    RuntimeTypeImport {
        canonical: "std/sse::SseParseError",
        runtime_feature: "sse.parse-error-type",
        module: "@seseragi/runtime/sse",
        export_name: "SseParseError",
    },
    RuntimeTypeImport {
        canonical: "std/websocket::WebSocketClient",
        runtime_feature: "websocket.client-service-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketClient",
    },
    RuntimeTypeImport {
        canonical: "std/websocket::WebSocketConnection",
        runtime_feature: "websocket.connection-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketConnection",
    },
    RuntimeTypeImport {
        canonical: "std/websocket::WebSocketEvent",
        runtime_feature: "websocket.event-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketEvent",
    },
    RuntimeTypeImport {
        canonical: "std/websocket::WebSocketClose",
        runtime_feature: "websocket.close-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketClose",
    },
    RuntimeTypeImport {
        canonical: "std/websocket::WebSocketError",
        runtime_feature: "websocket.error-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketError",
    },
    RuntimeTypeImport {
        canonical: "std/websocket/server::WebSocketServer",
        runtime_feature: "websocket.server-service-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketServer",
    },
    RuntimeTypeImport {
        canonical: "std/websocket/server::WebSocketServerHandle",
        runtime_feature: "websocket.server-handle-type",
        module: "@seseragi/runtime/websocket",
        export_name: "WebSocketServerHandle",
    },
];

pub(crate) fn runtime_type_import(canonical: &str) -> Option<RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS
        .iter()
        .copied()
        .find(|type_import| type_import.canonical == canonical)
}

pub(crate) fn runtime_type_surface_name(type_import: RuntimeTypeImport) -> &'static str {
    type_import
        .canonical
        .rsplit_once("::")
        .map_or(type_import.canonical, |(_, name)| name)
}

pub(crate) fn runtime_type_import_for_surface(surface: &str) -> Option<RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS.iter().copied().find(|type_import| {
        let declared = runtime_type_surface_name(*type_import);
        declared == surface
            || (declared.starts_with("Js.")
                && (declared.replace('.', "_") == surface
                    || declared.replace('.', "::") == surface))
    })
}

pub(crate) fn runtime_type_imports() -> impl Iterator<Item = RuntimeTypeImport> {
    RUNTIME_TYPE_IMPORTS.iter().copied()
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

        let effect = runtime_type_import("std/prelude::Effect").unwrap();
        assert_eq!(effect.runtime_feature, "effect.core.type");
        assert_eq!(effect.module, "@seseragi/runtime/effect");
        assert_eq!(effect.export_name, "Effect");

        let content = runtime_type_import("std/web/dom::DomContent").unwrap();
        assert_eq!(content.runtime_feature, "web.dom.content-type");
        assert_eq!(content.module, "@seseragi/runtime/dom");
        assert_eq!(content.export_name, "DomContent");

        let binding = runtime_type_import("std/web/dom::DomBinding").unwrap();
        assert_eq!(binding.runtime_feature, "web.dom.binding-type");
        assert_eq!(binding.module, "@seseragi/runtime/dom");
        assert_eq!(binding.export_name, "DomBinding");

        let nullable = runtime_type_import_for_surface("Js.Nullable").unwrap();
        assert_eq!(nullable.canonical, "std/prelude::Js.Nullable");
        assert_eq!(nullable.export_name, "JsNullable");
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

        let debug = RuntimeTypeImport {
            canonical: "std/prelude::Debug",
            runtime_feature: "core.debug.dictionary",
            module: "@seseragi/runtime/show",
            export_name: "Debug",
        };
        assert_eq!(runtime_type_import("std/prelude::Debug"), Some(debug));
        assert_eq!(
            runtime_type_import_for_feature("core.debug.dictionary"),
            Some(debug)
        );
    }

    #[test]
    fn resolves_numeric_error_and_rounding_types() {
        for (canonical, feature, module, export_name) in [
            (
                "std/number::RoundingMode",
                "core.number.rounding-mode",
                "@seseragi/runtime/number",
                "RoundingMode",
            ),
            (
                "std/int::IntParseError",
                "core.int.parse-error",
                "@seseragi/runtime/int",
                "IntParseError",
            ),
            (
                "std/int::IntDivisionError",
                "core.int.division-error",
                "@seseragi/runtime/int",
                "IntDivisionError",
            ),
            (
                "std/int::IntPowerError",
                "core.int.power-error",
                "@seseragi/runtime/int",
                "IntPowerError",
            ),
            (
                "std/float::FloatParseError",
                "core.float64.parse-error",
                "@seseragi/runtime/float",
                "FloatParseError",
            ),
            (
                "std/float::FloatConversionError",
                "core.float64.conversion-error",
                "@seseragi/runtime/float",
                "FloatConversionError",
            ),
        ] {
            let expected = RuntimeTypeImport {
                canonical,
                runtime_feature: feature,
                module,
                export_name,
            };
            assert_eq!(runtime_type_import(canonical), Some(expected));
            assert_eq!(runtime_type_import_for_feature(feature), Some(expected));
        }
    }
}
