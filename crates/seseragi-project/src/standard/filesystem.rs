use super::*;

pub(super) fn filesystem_interface() -> ModuleInterface {
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
