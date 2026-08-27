#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeDisplayDictionary {
    pub(crate) semantic_identity: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

macro_rules! display_dictionary {
    ($identity:literal, $feature:literal, $local:literal, $export:literal) => {
        RuntimeDisplayDictionary {
            semantic_identity: $identity,
            runtime_feature: $feature,
            local_name: $local,
            module: "@seseragi/runtime/show",
            export_name: $export,
            source_map_name: $export,
        }
    };
}

const RUNTIME_DISPLAY_DICTIONARIES: &[RuntimeDisplayDictionary] = &[
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Int>",
        runtime_feature: "core.int.show",
        local_name: "_ssrg_show_intShow",
        module: "@seseragi/runtime/show",
        export_name: "intShow",
        source_map_name: "intShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Int>",
        runtime_feature: "core.int.debug",
        local_name: "_ssrg_debug_intDebug",
        module: "@seseragi/runtime/show",
        export_name: "intDebug",
        source_map_name: "intDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Float>",
        runtime_feature: "core.float64.show",
        local_name: "_ssrg_show_floatShow",
        module: "@seseragi/runtime/show",
        export_name: "floatShow",
        source_map_name: "floatShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Float>",
        runtime_feature: "core.float64.debug",
        local_name: "_ssrg_debug_floatDebug",
        module: "@seseragi/runtime/show",
        export_name: "floatDebug",
        source_map_name: "floatDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Never>",
        runtime_feature: "core.never.show",
        local_name: "_ssrg_show_neverShow",
        module: "@seseragi/runtime/show",
        export_name: "neverShow",
        source_map_name: "neverShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Never>",
        runtime_feature: "core.never.debug",
        local_name: "_ssrg_debug_neverDebug",
        module: "@seseragi/runtime/show",
        export_name: "neverDebug",
        source_map_name: "neverDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::String>",
        runtime_feature: "core.string.show",
        local_name: "_ssrg_show_stringShow",
        module: "@seseragi/runtime/show",
        export_name: "stringShow",
        source_map_name: "stringShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::ConsoleError>",
        runtime_feature: "effect.console.error.show",
        local_name: "_ssrg_show_consoleErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "consoleErrorShow",
        source_map_name: "consoleErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::ConsoleError>",
        runtime_feature: "effect.console.error.debug",
        local_name: "_ssrg_debug_consoleErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "consoleErrorDebug",
        source_map_name: "consoleErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::StdinError>",
        runtime_feature: "effect.stdin.error.show",
        local_name: "_ssrg_show_stdinErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "stdinErrorShow",
        source_map_name: "stdinErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::StdinError>",
        runtime_feature: "effect.stdin.error.debug",
        local_name: "_ssrg_debug_stdinErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "stdinErrorDebug",
        source_map_name: "stdinErrorDebug",
    },
    display_dictionary!(
        "Show<std/stdin::StdinConfigError>",
        "stdin.config-error.show",
        "_ssrg_show_stdinConfigErrorShow",
        "stdinConfigErrorShow"
    ),
    display_dictionary!(
        "Debug<std/stdin::StdinConfigError>",
        "stdin.config-error.debug",
        "_ssrg_debug_stdinConfigErrorDebug",
        "stdinConfigErrorDebug"
    ),
    display_dictionary!(
        "Show<std/log::LogError>",
        "logger.error.show",
        "_ssrg_show_logErrorShow",
        "logErrorShow"
    ),
    display_dictionary!(
        "Debug<std/log::LogError>",
        "logger.error.debug",
        "_ssrg_debug_logErrorDebug",
        "logErrorDebug"
    ),
    display_dictionary!(
        "Show<std/process::ProcessSignal>",
        "process.signal.show",
        "_ssrg_show_processSignalShow",
        "processSignalShow"
    ),
    display_dictionary!(
        "Debug<std/process::ProcessSignal>",
        "process.signal.debug",
        "_ssrg_debug_processSignalDebug",
        "processSignalDebug"
    ),
    display_dictionary!(
        "Show<std/process::ProcessError>",
        "process.error.show",
        "_ssrg_show_processErrorShow",
        "processErrorShow"
    ),
    display_dictionary!(
        "Debug<std/process::ProcessError>",
        "process.error.debug",
        "_ssrg_debug_processErrorDebug",
        "processErrorDebug"
    ),
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/web/navigation::UrlBuildError>",
        runtime_feature: "web.navigation.url-error.show",
        local_name: "_ssrg_show_urlBuildErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "urlBuildErrorShow",
        source_map_name: "urlBuildErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/web/navigation::UrlBuildError>",
        runtime_feature: "web.navigation.url-error.debug",
        local_name: "_ssrg_debug_urlBuildErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "urlBuildErrorDebug",
        source_map_name: "urlBuildErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/web/navigation::NavigationError>",
        runtime_feature: "web.navigation.error.show",
        local_name: "_ssrg_show_navigationErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "navigationErrorShow",
        source_map_name: "navigationErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/web/navigation::NavigationError>",
        runtime_feature: "web.navigation.error.debug",
        local_name: "_ssrg_debug_navigationErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "navigationErrorDebug",
        source_map_name: "navigationErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/web/storage::StorageArea>",
        runtime_feature: "web.storage.area.show",
        local_name: "_ssrg_show_storageAreaShow",
        module: "@seseragi/runtime/show",
        export_name: "storageAreaShow",
        source_map_name: "storageAreaShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/web/storage::StorageArea>",
        runtime_feature: "web.storage.area.debug",
        local_name: "_ssrg_debug_storageAreaDebug",
        module: "@seseragi/runtime/show",
        export_name: "storageAreaDebug",
        source_map_name: "storageAreaDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/web/storage::StorageError>",
        runtime_feature: "web.storage.error.show",
        local_name: "_ssrg_show_storageErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "storageErrorShow",
        source_map_name: "storageErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/web/storage::StorageError>",
        runtime_feature: "web.storage.error.debug",
        local_name: "_ssrg_debug_storageErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "storageErrorDebug",
        source_map_name: "storageErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/web/dom::DomError>",
        runtime_feature: "web.dom.error.show",
        local_name: "_ssrg_show_domErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "domErrorShow",
        source_map_name: "domErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/web/dom::DomError>",
        runtime_feature: "web.dom.error.debug",
        local_name: "_ssrg_debug_domErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "domErrorDebug",
        source_map_name: "domErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/web/dom::DomRuntimeError::Show",
        runtime_feature: "web.dom.runtime-error.show",
        local_name: "_ssrg_show_domRuntimeErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "domRuntimeErrorShow",
        source_map_name: "domRuntimeErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/web/dom::DomRuntimeError::Debug",
        runtime_feature: "web.dom.runtime-error.debug",
        local_name: "_ssrg_debug_domRuntimeErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "domRuntimeErrorDebug",
        source_map_name: "domRuntimeErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/web/html::HtmlBuildError>",
        runtime_feature: "web.html.build-error.show",
        local_name: "_ssrg_show_htmlBuildErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "htmlBuildErrorShow",
        source_map_name: "htmlBuildErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/web/html::HtmlBuildError>",
        runtime_feature: "web.html.build-error.debug",
        local_name: "_ssrg_debug_htmlBuildErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "htmlBuildErrorDebug",
        source_map_name: "htmlBuildErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/bytes::ByteError>",
        runtime_feature: "core.bytes.byte-error.show",
        local_name: "_ssrg_show_byteErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "byteErrorShow",
        source_map_name: "byteErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/bytes::ByteError>",
        runtime_feature: "core.bytes.byte-error.debug",
        local_name: "_ssrg_debug_byteErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "byteErrorDebug",
        source_map_name: "byteErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/bytes::BytesSliceError>",
        runtime_feature: "core.bytes.slice-error.show",
        local_name: "_ssrg_show_bytesSliceErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "bytesSliceErrorShow",
        source_map_name: "bytesSliceErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/bytes::BytesSliceError>",
        runtime_feature: "core.bytes.slice-error.debug",
        local_name: "_ssrg_debug_bytesSliceErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "bytesSliceErrorDebug",
        source_map_name: "bytesSliceErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/text::Utf8DecodeError>",
        runtime_feature: "core.text.utf8-error.show",
        local_name: "_ssrg_show_utf8DecodeErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "utf8DecodeErrorShow",
        source_map_name: "utf8DecodeErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/text::Utf8DecodeError>",
        runtime_feature: "core.text.utf8-error.debug",
        local_name: "_ssrg_debug_utf8DecodeErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "utf8DecodeErrorDebug",
        source_map_name: "utf8DecodeErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/effect::ScheduleError>",
        runtime_feature: "effect.schedule.error.show",
        local_name: "_ssrg_show_scheduleErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "scheduleErrorShow",
        source_map_name: "scheduleErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/effect::ScheduleError>",
        runtime_feature: "effect.schedule.error.debug",
        local_name: "_ssrg_debug_scheduleErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "scheduleErrorDebug",
        source_map_name: "scheduleErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/effect::ParallelismError>",
        runtime_feature: "effect.parallelism.error.show",
        local_name: "_ssrg_show_parallelismErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "parallelismErrorShow",
        source_map_name: "parallelismErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/effect::ParallelismError>",
        runtime_feature: "effect.parallelism.error.debug",
        local_name: "_ssrg_debug_parallelismErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "parallelismErrorDebug",
        source_map_name: "parallelismErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/stream::BufferCapacityError>",
        runtime_feature: "stream.buffer.capacity-error.show",
        local_name: "_ssrg_show_bufferCapacityErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "bufferCapacityErrorShow",
        source_map_name: "bufferCapacityErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/stream::BufferCapacityError>",
        runtime_feature: "stream.buffer.capacity-error.debug",
        local_name: "_ssrg_debug_bufferCapacityErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "bufferCapacityErrorDebug",
        source_map_name: "bufferCapacityErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/queue::QueueCreateError>",
        runtime_feature: "effect.queue.create-error.show",
        local_name: "_ssrg_show_queueCreateErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "queueCreateErrorShow",
        source_map_name: "queueCreateErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/queue::QueueCreateError>",
        runtime_feature: "effect.queue.create-error.debug",
        local_name: "_ssrg_debug_queueCreateErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "queueCreateErrorDebug",
        source_map_name: "queueCreateErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/queue::QueueClosed>",
        runtime_feature: "effect.queue.closed.show",
        local_name: "_ssrg_show_queueClosedShow",
        module: "@seseragi/runtime/show",
        export_name: "queueClosedShow",
        source_map_name: "queueClosedShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/queue::QueueClosed>",
        runtime_feature: "effect.queue.closed.debug",
        local_name: "_ssrg_debug_queueClosedDebug",
        module: "@seseragi/runtime/show",
        export_name: "queueClosedDebug",
        source_map_name: "queueClosedDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/semaphore::SemaphoreCreateError>",
        runtime_feature: "effect.semaphore.create-error.show",
        local_name: "_ssrg_show_semaphoreCreateErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "semaphoreCreateErrorShow",
        source_map_name: "semaphoreCreateErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/semaphore::SemaphoreCreateError>",
        runtime_feature: "effect.semaphore.create-error.debug",
        local_name: "_ssrg_debug_semaphoreCreateErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "semaphoreCreateErrorDebug",
        source_map_name: "semaphoreCreateErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/time::DurationError>",
        runtime_feature: "time.duration.error.show",
        local_name: "_ssrg_show_durationErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "durationErrorShow",
        source_map_name: "durationErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/time::DurationError>",
        runtime_feature: "time.duration.error.debug",
        local_name: "_ssrg_debug_durationErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "durationErrorDebug",
        source_map_name: "durationErrorDebug",
    },
    display_dictionary!(
        "Show<std/path::PathError>",
        "path.error.show",
        "_ssrg_show_pathErrorShow",
        "pathErrorShow"
    ),
    display_dictionary!(
        "Debug<std/path::PathError>",
        "path.error.debug",
        "_ssrg_debug_pathErrorDebug",
        "pathErrorDebug"
    ),
    display_dictionary!(
        "Show<std/fs::FileType>",
        "filesystem.file-type.show",
        "_ssrg_show_fileTypeShow",
        "fileTypeShow"
    ),
    display_dictionary!(
        "Debug<std/fs::FileType>",
        "filesystem.file-type.debug",
        "_ssrg_debug_fileTypeDebug",
        "fileTypeDebug"
    ),
    display_dictionary!(
        "Show<std/fs::FileSystemOperation>",
        "filesystem.operation.show",
        "_ssrg_show_fileSystemOperationShow",
        "fileSystemOperationShow"
    ),
    display_dictionary!(
        "Debug<std/fs::FileSystemOperation>",
        "filesystem.operation.debug",
        "_ssrg_debug_fileSystemOperationDebug",
        "fileSystemOperationDebug"
    ),
    display_dictionary!(
        "Show<std/fs::FileSystemErrorKind>",
        "filesystem.error-kind.show",
        "_ssrg_show_fileSystemErrorKindShow",
        "fileSystemErrorKindShow"
    ),
    display_dictionary!(
        "Debug<std/fs::FileSystemErrorKind>",
        "filesystem.error-kind.debug",
        "_ssrg_debug_fileSystemErrorKindDebug",
        "fileSystemErrorKindDebug"
    ),
    display_dictionary!(
        "Show<std/fs::FileSystemError>",
        "filesystem.error.show",
        "_ssrg_show_fileSystemErrorShow",
        "fileSystemErrorShow"
    ),
    display_dictionary!(
        "Debug<std/fs::FileSystemError>",
        "filesystem.error.debug",
        "_ssrg_debug_fileSystemErrorDebug",
        "fileSystemErrorDebug"
    ),
    display_dictionary!(
        "Show<std/fs::FileMetadata>",
        "filesystem.metadata.show",
        "_ssrg_show_fileMetadataShow",
        "fileMetadataShow"
    ),
    display_dictionary!(
        "Debug<std/fs::FileMetadata>",
        "filesystem.metadata.debug",
        "_ssrg_debug_fileMetadataDebug",
        "fileMetadataDebug"
    ),
    display_dictionary!(
        "Show<std/fs::DirectoryEntry>",
        "filesystem.directory-entry.show",
        "_ssrg_show_directoryEntryShow",
        "directoryEntryShow"
    ),
    display_dictionary!(
        "Debug<std/fs::DirectoryEntry>",
        "filesystem.directory-entry.debug",
        "_ssrg_debug_directoryEntryDebug",
        "directoryEntryDebug"
    ),
    display_dictionary!(
        "Show<std/fs::WriteMode>",
        "filesystem.write-mode.show",
        "_ssrg_show_writeModeShow",
        "writeModeShow"
    ),
    display_dictionary!(
        "Debug<std/fs::WriteMode>",
        "filesystem.write-mode.debug",
        "_ssrg_debug_writeModeDebug",
        "writeModeDebug"
    ),
    display_dictionary!(
        "Show<std/fs::FileTextError>",
        "filesystem.text-error.show",
        "_ssrg_show_fileTextErrorShow",
        "fileTextErrorShow"
    ),
    display_dictionary!(
        "Debug<std/fs::FileTextError>",
        "filesystem.text-error.debug",
        "_ssrg_debug_fileTextErrorDebug",
        "fileTextErrorDebug"
    ),
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/http::HttpBuildError>",
        runtime_feature: "http-client.build-error.show",
        local_name: "_ssrg_show_httpBuildErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "httpBuildErrorShow",
        source_map_name: "httpBuildErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/http::HttpBuildError>",
        runtime_feature: "http-client.build-error.debug",
        local_name: "_ssrg_debug_httpBuildErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "httpBuildErrorDebug",
        source_map_name: "httpBuildErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/http::HttpError>",
        runtime_feature: "http-client.error.show",
        local_name: "_ssrg_show_httpErrorShow",
        module: "@seseragi/runtime/show",
        export_name: "httpErrorShow",
        source_map_name: "httpErrorShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/http::HttpError>",
        runtime_feature: "http-client.error.debug",
        local_name: "_ssrg_debug_httpErrorDebug",
        module: "@seseragi/runtime/show",
        export_name: "httpErrorDebug",
        source_map_name: "httpErrorDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "@seseragi/internal::boundedShow",
        runtime_feature: "core.show.bounded",
        local_name: "_ssrg_show_boundedShow",
        module: "@seseragi/runtime/show",
        export_name: "boundedShow",
        source_map_name: "boundedShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "@seseragi/internal::boundedDebug",
        runtime_feature: "core.debug.bounded",
        local_name: "_ssrg_debug_boundedDebug",
        module: "@seseragi/runtime/show",
        export_name: "boundedDebug",
        source_map_name: "boundedDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Bool>",
        runtime_feature: "core.bool.show",
        local_name: "_ssrg_show_boolShow",
        module: "@seseragi/runtime/show",
        export_name: "boolShow",
        source_map_name: "boolShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Unit>",
        runtime_feature: "core.unit.show",
        local_name: "_ssrg_show_unitShow",
        module: "@seseragi/runtime/show",
        export_name: "unitShow",
        source_map_name: "unitShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Show<std/prelude::Char>",
        runtime_feature: "core.char.show",
        local_name: "_ssrg_show_charShow",
        module: "@seseragi/runtime/show",
        export_name: "charShow",
        source_map_name: "charShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::String>",
        runtime_feature: "core.string.debug",
        local_name: "_ssrg_debug_stringDebug",
        module: "@seseragi/runtime/show",
        export_name: "stringDebug",
        source_map_name: "stringDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Bool>",
        runtime_feature: "core.bool.debug",
        local_name: "_ssrg_debug_boolDebug",
        module: "@seseragi/runtime/show",
        export_name: "boolDebug",
        source_map_name: "boolDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Unit>",
        runtime_feature: "core.unit.debug",
        local_name: "_ssrg_debug_unitDebug",
        module: "@seseragi/runtime/show",
        export_name: "unitDebug",
        source_map_name: "unitDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "Debug<std/prelude::Char>",
        runtime_feature: "core.char.debug",
        local_name: "_ssrg_debug_charDebug",
        module: "@seseragi/runtime/show",
        export_name: "charDebug",
        source_map_name: "charDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/array::Show",
        runtime_feature: "core.array.show",
        local_name: "_ssrg_show_arrayShow",
        module: "@seseragi/runtime/show",
        export_name: "arrayShow",
        source_map_name: "arrayShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/array::Debug",
        runtime_feature: "core.array.debug",
        local_name: "_ssrg_debug_arrayDebug",
        module: "@seseragi/runtime/show",
        export_name: "arrayDebug",
        source_map_name: "arrayDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/list::Show",
        runtime_feature: "core.list.show",
        local_name: "_ssrg_show_listShow",
        module: "@seseragi/runtime/show",
        export_name: "listShow",
        source_map_name: "listShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/list::Debug",
        runtime_feature: "core.list.debug",
        local_name: "_ssrg_debug_listDebug",
        module: "@seseragi/runtime/show",
        export_name: "listDebug",
        source_map_name: "listDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/maybe::Show",
        runtime_feature: "core.maybe.show",
        local_name: "_ssrg_show_maybeShow",
        module: "@seseragi/runtime/show",
        export_name: "maybeShow",
        source_map_name: "maybeShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/maybe::Debug",
        runtime_feature: "core.maybe.debug",
        local_name: "_ssrg_debug_maybeDebug",
        module: "@seseragi/runtime/show",
        export_name: "maybeDebug",
        source_map_name: "maybeDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/either::Show",
        runtime_feature: "core.either.show",
        local_name: "_ssrg_show_eitherShow",
        module: "@seseragi/runtime/show",
        export_name: "eitherShow",
        source_map_name: "eitherShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/either::Debug",
        runtime_feature: "core.either.debug",
        local_name: "_ssrg_debug_eitherDebug",
        module: "@seseragi/runtime/show",
        export_name: "eitherDebug",
        source_map_name: "eitherDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/range::Show",
        runtime_feature: "core.range.show",
        local_name: "_ssrg_show_rangeShow",
        module: "@seseragi/runtime/show",
        export_name: "rangeShow",
        source_map_name: "rangeShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/range::Debug",
        runtime_feature: "core.range.debug",
        local_name: "_ssrg_debug_rangeDebug",
        module: "@seseragi/runtime/show",
        export_name: "rangeDebug",
        source_map_name: "rangeDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/tuple::Show",
        runtime_feature: "core.tuple.show",
        local_name: "_ssrg_show_tupleShow",
        module: "@seseragi/runtime/show",
        export_name: "tupleShow",
        source_map_name: "tupleShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/tuple::Debug",
        runtime_feature: "core.tuple.debug",
        local_name: "_ssrg_debug_tupleDebug",
        module: "@seseragi/runtime/show",
        export_name: "tupleDebug",
        source_map_name: "tupleDebug",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/record::Show",
        runtime_feature: "core.record.show",
        local_name: "_ssrg_show_recordShow",
        module: "@seseragi/runtime/show",
        export_name: "recordShow",
        source_map_name: "recordShow",
    },
    RuntimeDisplayDictionary {
        semantic_identity: "std/record::Debug",
        runtime_feature: "core.record.debug",
        local_name: "_ssrg_debug_recordDebug",
        module: "@seseragi/runtime/show",
        export_name: "recordDebug",
        source_map_name: "recordDebug",
    },
];

pub(crate) fn runtime_display_dictionary_for_feature(
    feature: &str,
) -> Option<RuntimeDisplayDictionary> {
    RUNTIME_DISPLAY_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.runtime_feature == feature)
}

pub(crate) fn runtime_display_dictionary_for_identity(
    identity: &str,
) -> Option<RuntimeDisplayDictionary> {
    RUNTIME_DISPLAY_DICTIONARIES
        .iter()
        .copied()
        .find(|dictionary| dictionary.semantic_identity == identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_complete_standard_display_dictionary_family() {
        for (identity, feature, local_name, export_name) in [
            (
                "Show<std/prelude::Int>",
                "core.int.show",
                "_ssrg_show_intShow",
                "intShow",
            ),
            (
                "Show<std/prelude::String>",
                "core.string.show",
                "_ssrg_show_stringShow",
                "stringShow",
            ),
            (
                "Debug<std/prelude::Int>",
                "core.int.debug",
                "_ssrg_debug_intDebug",
                "intDebug",
            ),
            (
                "Show<std/prelude::Float>",
                "core.float64.show",
                "_ssrg_show_floatShow",
                "floatShow",
            ),
            (
                "Debug<std/prelude::Float>",
                "core.float64.debug",
                "_ssrg_debug_floatDebug",
                "floatDebug",
            ),
            (
                "Show<std/prelude::Never>",
                "core.never.show",
                "_ssrg_show_neverShow",
                "neverShow",
            ),
            (
                "Debug<std/prelude::Never>",
                "core.never.debug",
                "_ssrg_debug_neverDebug",
                "neverDebug",
            ),
            (
                "Show<std/prelude::ConsoleError>",
                "effect.console.error.show",
                "_ssrg_show_consoleErrorShow",
                "consoleErrorShow",
            ),
            (
                "Debug<std/prelude::ConsoleError>",
                "effect.console.error.debug",
                "_ssrg_debug_consoleErrorDebug",
                "consoleErrorDebug",
            ),
            (
                "Show<std/prelude::StdinError>",
                "effect.stdin.error.show",
                "_ssrg_show_stdinErrorShow",
                "stdinErrorShow",
            ),
            (
                "Debug<std/prelude::StdinError>",
                "effect.stdin.error.debug",
                "_ssrg_debug_stdinErrorDebug",
                "stdinErrorDebug",
            ),
            (
                "Show<std/web/dom::DomError>",
                "web.dom.error.show",
                "_ssrg_show_domErrorShow",
                "domErrorShow",
            ),
            (
                "Debug<std/web/dom::DomError>",
                "web.dom.error.debug",
                "_ssrg_debug_domErrorDebug",
                "domErrorDebug",
            ),
            (
                "std/web/dom::DomRuntimeError::Show",
                "web.dom.runtime-error.show",
                "_ssrg_show_domRuntimeErrorShow",
                "domRuntimeErrorShow",
            ),
            (
                "std/web/dom::DomRuntimeError::Debug",
                "web.dom.runtime-error.debug",
                "_ssrg_debug_domRuntimeErrorDebug",
                "domRuntimeErrorDebug",
            ),
            (
                "Show<std/web/html::HtmlBuildError>",
                "web.html.build-error.show",
                "_ssrg_show_htmlBuildErrorShow",
                "htmlBuildErrorShow",
            ),
            (
                "Debug<std/web/html::HtmlBuildError>",
                "web.html.build-error.debug",
                "_ssrg_debug_htmlBuildErrorDebug",
                "htmlBuildErrorDebug",
            ),
            (
                "@seseragi/internal::boundedShow",
                "core.show.bounded",
                "_ssrg_show_boundedShow",
                "boundedShow",
            ),
            (
                "@seseragi/internal::boundedDebug",
                "core.debug.bounded",
                "_ssrg_debug_boundedDebug",
                "boundedDebug",
            ),
            (
                "Show<std/prelude::Bool>",
                "core.bool.show",
                "_ssrg_show_boolShow",
                "boolShow",
            ),
            (
                "Show<std/prelude::Unit>",
                "core.unit.show",
                "_ssrg_show_unitShow",
                "unitShow",
            ),
            (
                "Show<std/prelude::Char>",
                "core.char.show",
                "_ssrg_show_charShow",
                "charShow",
            ),
            (
                "Debug<std/prelude::String>",
                "core.string.debug",
                "_ssrg_debug_stringDebug",
                "stringDebug",
            ),
            (
                "Debug<std/prelude::Bool>",
                "core.bool.debug",
                "_ssrg_debug_boolDebug",
                "boolDebug",
            ),
            (
                "Debug<std/prelude::Unit>",
                "core.unit.debug",
                "_ssrg_debug_unitDebug",
                "unitDebug",
            ),
            (
                "Debug<std/prelude::Char>",
                "core.char.debug",
                "_ssrg_debug_charDebug",
                "charDebug",
            ),
            (
                "std/array::Show",
                "core.array.show",
                "_ssrg_show_arrayShow",
                "arrayShow",
            ),
            (
                "std/array::Debug",
                "core.array.debug",
                "_ssrg_debug_arrayDebug",
                "arrayDebug",
            ),
            (
                "std/list::Show",
                "core.list.show",
                "_ssrg_show_listShow",
                "listShow",
            ),
            (
                "std/list::Debug",
                "core.list.debug",
                "_ssrg_debug_listDebug",
                "listDebug",
            ),
            (
                "std/maybe::Show",
                "core.maybe.show",
                "_ssrg_show_maybeShow",
                "maybeShow",
            ),
            (
                "std/maybe::Debug",
                "core.maybe.debug",
                "_ssrg_debug_maybeDebug",
                "maybeDebug",
            ),
            (
                "std/either::Show",
                "core.either.show",
                "_ssrg_show_eitherShow",
                "eitherShow",
            ),
            (
                "std/either::Debug",
                "core.either.debug",
                "_ssrg_debug_eitherDebug",
                "eitherDebug",
            ),
            (
                "std/range::Show",
                "core.range.show",
                "_ssrg_show_rangeShow",
                "rangeShow",
            ),
            (
                "std/range::Debug",
                "core.range.debug",
                "_ssrg_debug_rangeDebug",
                "rangeDebug",
            ),
            (
                "std/tuple::Show",
                "core.tuple.show",
                "_ssrg_show_tupleShow",
                "tupleShow",
            ),
            (
                "std/tuple::Debug",
                "core.tuple.debug",
                "_ssrg_debug_tupleDebug",
                "tupleDebug",
            ),
            (
                "std/record::Show",
                "core.record.show",
                "_ssrg_show_recordShow",
                "recordShow",
            ),
            (
                "std/record::Debug",
                "core.record.debug",
                "_ssrg_debug_recordDebug",
                "recordDebug",
            ),
        ] {
            let dictionary = runtime_display_dictionary_for_feature(feature).unwrap();
            assert_eq!(dictionary.local_name, local_name);
            assert_eq!(dictionary.module, "@seseragi/runtime/show");
            assert_eq!(dictionary.export_name, export_name);
            assert_eq!(dictionary.source_map_name, export_name);
            assert_eq!(
                runtime_display_dictionary_for_identity(identity),
                Some(dictionary)
            );
        }
    }

    #[test]
    fn rejects_unknown_display_dictionary_features() {
        assert!(runtime_display_dictionary_for_feature("core.decimal.show").is_none());
        assert!(runtime_display_dictionary_for_identity("Show<fixture/local::Detail>").is_none());
    }
}
