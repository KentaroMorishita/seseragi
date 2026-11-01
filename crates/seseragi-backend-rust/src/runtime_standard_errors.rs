fn native_error_tag(value: &V) -> &'static str {
    let V::Adt(value) = value else {
        panic!("Rust ABI: standard error variant")
    };
    value.get().0.rsplit("::").next().unwrap()
}

fn native_error_record(tag: &str, value: &V, fields: &[(&str, bool)]) -> String {
    let fields = fields
        .iter()
        .map(|(name, quoted)| {
            let field = value.field(name);
            let text = if *quoted {
                native_json_string(native_text(&field))
            } else {
                field.render()
            };
            format!("{name}: {text}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{tag} {{ {fields} }}")
}

fn native_error_scalar(value: V, debug: bool) -> String {
    match value {
        V::Text(_) if debug => native_primitive_display_dictionary("Debug<std/prelude::String>")
            .field("debug")
            .apply(value)
            .render(),
        V::Char(_) if debug => native_display(|| native_char_debug(value)).render(),
        _ => native_display(|| V::text(&value.render())).render(),
    }
}

fn native_standard_error_text(name: &str, value: &V, debug: bool, child: Option<&V>) -> String {
    if matches!(name, "std/prelude::ConsoleError" | "std/log::LogError") {
        let name = if name == "std/log::LogError" {
            "LogError"
        } else {
            "ConsoleError"
        };
        return if debug {
            format!("{name} {{ message: \"<redacted>\" }}")
        } else {
            format!("{name}: {}", native_text(&value.field("message")))
        };
    }
    if name == "std/prelude::Js.Error" {
        return format!(
            "Js.Error({}): {}",
            value.field("phase").render(),
            value.field("message").render()
        );
    }
    if matches!(
        name,
        "std/path::PathError"
            | "std/fs::FileSystemError"
            | "std/fs::FileSystemErrorKind"
            | "std/fs::FileTextError"
            | "std/text::Utf8DecodeError"
    ) {
        return native_filesystem_display(name, value);
    }
    if name == "std/regex::RegexCompileError" {
        return format!(
            "RegexCompileError {{ kind: {}, offset: {} }}",
            native_standard_error_text(
                "std/regex::RegexCompileErrorKind",
                &value.field("kind"),
                debug,
                None
            ),
            value.field("offset").render()
        );
    }
    let tag = native_error_tag(value);
    let payload = value.payload();
    match name {
        "std/prelude::StdinError" => match tag {
            "InvalidStdinUtf8" => native_error_record(tag, &payload, &[("offset", false)]),
            "StdinLineTooLong" => native_error_record(tag, &payload, &[("limitBytes", false)]),
            "StdinUnavailable"
            | "StdinReadFailure"
            | "ConcurrentStdinRead"
            | "StdinPositionOverflow" => tag.into(),
            _ => panic!("Rust ABI: StdinError variant"),
        },
        "std/int::IntParseError" | "std/big-int::BigIntParseError" => match tag {
            "EmptyInt" | "IntOutsideRange" | "EmptyBigInt" => tag.into(),
            "InvalidIntDigit" | "InvalidBigIntDigit" => {
                native_error_record(tag, &payload, &[("offset", false), ("radix", false)])
            }
            _ => format!("{tag} {}", native_error_scalar(payload, false)),
        },
        "std/decimal::DecimalParseError" => {
            native_error_record(tag, &payload, &[("offset", false)])
        }
        "std/big-int::BigIntDivisionError"
        | "std/big-int::BigIntConversionError"
        | "std/decimal::DecimalArithmeticError"
        | "std/decimal::DecimalConversionError"
        | "std/entropy::EntropyError"
        | "std/queue::QueueClosed" => tag.into(),
        "std/big-int::BigIntPowerError"
        | "std/decimal::DecimalContextError"
        | "std/random::RandomConfigError"
        | "std/entropy::EntropyConfigError"
        | "std/effect::ScheduleError"
        | "std/effect::ParallelismError"
        | "std/collection::SizeError"
        | "std/stream::BufferCapacityError"
        | "std/queue::QueueCreateError"
        | "std/semaphore::SemaphoreCreateError" => {
            format!("{tag} {}", native_error_scalar(payload, false))
        }
        "std/stdin::StdinConfigError" | "std/bytes::ByteError" => {
            format!("{tag} {}", payload.render())
        }
        "std/time::DurationError" => {
            if tag == "NegativeDuration" {
                format!("{tag} {}", native_error_scalar(payload, false))
            } else {
                tag.into()
            }
        }
        "std/random::RandomRangeError" => {
            if tag == "InvalidProbability" {
                format!("{tag} {}", native_error_scalar(payload, false))
            } else {
                native_error_record(
                    tag,
                    &payload,
                    &[("lower", false), ("upperExclusive", false)],
                )
            }
        }
        "std/bytes::BytesSliceError"
        | "std/text::TextSliceError"
        | "std/text/grapheme::GraphemeSliceError" => native_error_record(
            tag,
            &payload,
            &[("start", false), ("end", false), ("length", false)],
        ),
        "std/bytes/hex::HexDecodeError" | "std/bytes/base64::Base64DecodeError" => {
            if matches!(tag, "OddHexLength" | "InvalidBase64Length") {
                format!("{tag} {}", payload.render())
            } else {
                native_error_record(tag, &payload, &[("offset", false)])
            }
        }
        "std/process::ProcessError" => match tag {
            "UnsupportedProcessSignal" | "ReservedProcessSignal" => {
                format!(
                    "{tag} {}",
                    native_display(|| V::text(native_error_tag(&payload))).render()
                )
            }
            "InvalidArgumentEncoding" => format!("{tag} {}", native_error_scalar(payload, debug)),
            "InvalidEnvironmentName" | "InvalidEnvironmentEncoding" => {
                format!("{tag} {}", native_error_scalar(payload, debug))
            }
            _ => tag.into(),
        },
        "std/child-process::ChildProcessConfigError" => match tag {
            "EmptyExecutableName" => tag.into(),
            "ArgumentContainsNul" => {
                native_error_record(tag, &payload, &[("index", false), ("offset", false)])
            }
            _ => format!(
                "{tag} {}",
                if matches!(payload, V::Text(_)) {
                    native_json_string(native_text(&payload))
                } else {
                    payload.render()
                }
            ),
        },
        "std/child-process::ChildProcessError" => match tag {
            "ChildInputAfterClose" => tag.into(),
            "UnsupportedChildSignal" => format!("{tag} {}", native_error_tag(&payload)),
            "ChildInputFailed" | "ChildWaitFailed" | "ChildTerminationFailed" => {
                format!("{tag} {}", native_json_string(native_text(&payload)))
            }
            "ChildSpawnFailed" => {
                let executable = payload.field("executable");
                let executable_name = native_error_tag(&executable);
                let path = if executable_name == "SearchPath" {
                    native_text(&executable.payload()).to_owned()
                } else {
                    native_path_text(&executable.payload()).to_owned()
                };
                format!(
                    "{tag} {{ executable: {executable_name} {}, detail: {} }}",
                    native_json_string(&path),
                    native_json_string(native_text(&payload.field("detail")))
                )
            }
            "ChildOutputReadFailed" => format!(
                "{tag} {{ channel: {}, detail: {} }}",
                native_error_tag(&payload.field("channel")),
                native_json_string(native_text(&payload.field("detail")))
            ),
            "ChildOutputLimitExceeded" => format!(
                "{tag} {{ channel: {}, limitBytes: {} }}",
                native_error_tag(&payload.field("channel")),
                payload.field("limitBytes").render()
            ),
            _ => panic!("Rust ABI: ChildProcessError variant"),
        },
        "std/time::DateTimeError" => match tag {
            "InvalidDate" => native_error_record(
                tag,
                &payload,
                &[("year", false), ("month", false), ("day", false)],
            ),
            "InvalidTime" => native_error_record(
                tag,
                &payload,
                &[
                    ("hour", false),
                    ("minute", false),
                    ("second", false),
                    ("nanosecond", false),
                ],
            ),
            "InvalidUtcOffsetSeconds" => format!("{tag} {}", payload.render()),
            "InvalidDateTimeText" => native_error_record(tag, &payload, &[("offset", false)]),
            _ => panic!("Rust ABI: DateTimeError variant"),
        },
        "std/time::TimeZoneError" => {
            if tag == "TimeZoneDatabaseVersionMismatch" {
                native_error_record(tag, &payload, &[("required", true), ("actual", true)])
            } else {
                format!("{tag} {}", native_json_string(native_text(&payload)))
            }
        }
        "std/http::HttpBuildError" => match tag {
            "HttpUrlContainsUserInfo" | "HttpUrlContainsFragment" => tag.into(),
            "InvalidHttpUrl" => native_error_record(tag, &payload, &[("offset", false)]),
            "InvalidHeaderValue" => {
                native_error_record(tag, &payload, &[("name", true), ("offset", false)])
            }
            "InvalidHttpStatus" | "InvalidHttpBodyLimit" => format!("{tag} {}", payload.render()),
            _ => format!("{tag} {}", native_json_string(native_text(&payload))),
        },
        "std/http::HttpError" => match tag {
            "HttpClientUnavailable" => tag.into(),
            "HttpRequestLengthMismatch" => {
                native_error_record(tag, &payload, &[("declared", false), ("actual", false)])
            }
            "HttpResponseBodyLimitExceeded" => {
                native_error_record(tag, &payload, &[("limitBytes", false)])
            }
            _ => format!("{tag} {}", native_json_string(native_text(&payload))),
        },
        "std/web/navigation::UrlBuildError" => match tag {
            "UrlContainsUserInfo" => tag.into(),
            "InvalidUrl" | "InvalidPercentEncoding" => {
                native_error_record(tag, &payload, &[("offset", false)])
            }
            _ => format!("{tag} {}", native_json_string(native_text(&payload))),
        },
        "std/web/navigation::NavigationError" => {
            if tag == "CrossOriginNavigation" {
                native_error_record(tag, &payload, &[("expected", true), ("actual", true)])
            } else {
                format!("{tag} {}", native_json_string(native_text(&payload)))
            }
        }
        "std/web/storage::StorageError" => {
            let area = payload.field("area");
            let area = native_error_tag(&area);
            let message = native_json_string(native_text(&payload.field("message")));
            if tag == "StorageQuotaExceeded" {
                format!(
                    "{tag} {{ area: {area}, key: {}, message: {message} }}",
                    native_json_string(native_text(&payload.field("key")))
                )
            } else {
                format!("{tag} {{ area: {area}, message: {message} }}")
            }
        }
        "std/regex::RegexCompileErrorKind" => match tag {
            "UnexpectedRegexToken" | "DuplicateCaptureName" | "UnsupportedRegexFeature" => {
                format!("{tag} {}", native_error_scalar(payload, debug))
            }
            _ => tag.into(),
        },
        "std/web/html::HtmlBuildError" => format!("{tag} {}", native_error_scalar(payload, debug)),
        "std/web/dom::DomError" => match tag {
            "DomTargetAlreadyMounted" | "DomTargetRemoved" => tag.into(),
            "HydrationMismatch" => {
                let path = payload.field("path");
                let V::Array(path) = path else {
                    panic!("Rust ABI: hydration path")
                };
                format!(
                    "{tag} {{ path: [{}], expected: {}, actual: {} }}",
                    path.get()
                        .iter()
                        .map(V::render)
                        .collect::<Vec<_>>()
                        .join(", "),
                    native_error_scalar(payload.field("expected"), debug),
                    native_error_scalar(payload.field("actual"), debug)
                )
            }
            _ => format!("{tag} {}", native_error_scalar(payload, debug)),
        },
        "std/web/dom::DomRuntimeError" => {
            let text = if tag == "DomFailure" {
                native_display(|| {
                    V::text(&native_standard_error_text(
                        "std/web/dom::DomError",
                        &payload,
                        debug,
                        None,
                    ))
                })
                .render()
            } else {
                child
                    .expect("Rust ABI: dispatch failure dictionary")
                    .apply(payload)
                    .render()
            };
            format!("{tag} {text}")
        }
        _ => panic!("Rust ABI: standard error display type {name}"),
    }
}

fn native_standard_error_equal(left: &V, right: &V) -> bool {
    match (left, right) {
        (V::Unit, V::Unit) => true,
        (V::Int(a), V::Int(b)) => a == b,
        (V::Char(a), V::Char(b)) => a == b,
        (V::Text(a), V::Text(b)) => a.get() == b.get(),
        (V::Adt(a), V::Adt(b)) => {
            a.get().0 == b.get().0 && native_standard_error_equal(&a.get().1, &b.get().1)
        }
        (V::Record(_) | V::HtmlStyleRecord(_), V::Record(_) | V::HtmlStyleRecord(_)) => {
            let a = left.record_fields();
            let b = right.record_fields();
            a.len() == b.len()
                && a.iter().all(|(key, value)| {
                    b.get(key)
                        .is_some_and(|right| native_standard_error_equal(value, right))
                })
        }
        _ => false,
    }
}

fn native_standard_error_dictionary(
    name: &'static str,
    method: &'static str,
    children: Vec<V>,
) -> V {
    if method == "eq" {
        return V::dict(vec![(
            "eq",
            V::fun(vec![], |_, left| {
                V::fun(vec![left], |c, right| {
                    V::Bool(native_standard_error_equal(&c[0], &right))
                })
            }),
        )]);
    }
    let mut captures = vec![V::text(name), V::Bool(method == "debug")];
    captures.extend(children.into_iter().map(|child| child.field(method)));
    V::dict(vec![(
        method,
        V::fun(captures, |c, value| {
            native_display(|| {
                V::text(&native_standard_error_text(
                    native_text(&c[0]),
                    &value,
                    c[1].boolean(),
                    c.get(2),
                ))
            })
        }),
    )])
}
