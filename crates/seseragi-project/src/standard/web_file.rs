use super::*;

pub(super) fn web_file_interface() -> ModuleInterface {
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
