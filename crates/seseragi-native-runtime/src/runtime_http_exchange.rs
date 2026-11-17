// `std/http.exchange` on the native target: a Stream of the events of one
// HTTP/1.1 exchange. Opening the Stream opens the request body Stream, starts
// a worker that sends the request and reads the response, and forks a body
// Fiber. The body Fiber pulls the body one chunk at a time and hands each
// chunk to the worker's writer, so the body is sent while the response is
// read. Each pull of the exchange waits for the next response event; closing
// the Stream closes the connection and stops the body Fiber. Failures are
// `Left` for the body and `Right` for HTTP, as on TS.

struct NativeHttpStream {
    events: std::sync::mpsc::Receiver<Result<NativeHttpEvent, NativeHttpReply>>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: std::sync::Arc<std::sync::Mutex<Option<std::net::TcpStream>>>,
    // Set once the request body must stop: by the worker at the final
    // response head, at a failure or at the end of the response, and by the
    // body Fiber when the body fails. Whichever sets it first decides the
    // failure of the exchange, as TS returns the failure observed first.
    body_stopped: std::sync::Arc<std::sync::atomic::AtomicBool>,
    // The body failed first; the next pull reports it.
    body_failed: bool,
    parts: Option<std::sync::mpsc::SyncSender<NativeHttpPart>>,
    declared: Option<i64>,
    sent: i64,
}

impl Drop for NativeHttpStream {
    fn drop(&mut self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(stream) = self.connection.lock().unwrap().as_ref() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

thread_local! {
    static NATIVE_HTTP_STREAMS: RefCell<(i64, BTreeMap<i64, NativeHttpStream>)> =
        RefCell::new((0, BTreeMap::new()));
}

fn native_http_with_stream<T>(
    id: i64,
    action: impl FnOnce(Option<&mut NativeHttpStream>) -> T,
) -> T {
    NATIVE_HTTP_STREAMS.with(|streams| action(streams.borrow_mut().1.get_mut(&id)))
}

fn native_http_right(exit: NativeExit) -> NativeExit {
    match exit {
        NativeExit::Failure(error) => NativeExit::failure(native_right(error.value)),
        other => other,
    }
}

fn native_http_reply_exit(reply: NativeHttpReply) -> NativeExit {
    match reply {
        NativeHttpReply::Failure(tag, message) => {
            native_http_right(native_http_failure(tag, &message))
        }
        NativeHttpReply::Cancelled => NativeExit::Cancelled,
        _ => NativeExit::defects(vec!["unexpected HTTP exchange reply".into()]),
    }
}

// Starts the exchange and returns its handle. `length` is the length the
// Body knows in advance, or -1.
fn native_http_stream_open(request: &V, length: i64) -> Result<i64, NativeExit> {
    let request = native_http_inner(request, "std/http::Request");
    let method = native_text(native_http_inner(
        &request.field("method"),
        "std/http::Method",
    ))
    .to_owned();
    let url = native_text(native_http_inner(
        &request.field("url"),
        "std/http::HttpUrl",
    ))
    .to_owned();
    let headers = native_http_headers(&request.field("headers"));
    let declared = native_http_declared(&headers).map_err(native_http_right)?;
    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let body_stopped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let connection = std::sync::Arc::new(std::sync::Mutex::new(None));
    // One event at most waits in the channel: the worker reads the next one
    // only after the consumer has taken it, so an unread response stays in
    // the connection rather than in memory. Body chunks are handed over the
    // same way, one at a time.
    let (send, events) = std::sync::mpsc::sync_channel(1);
    let (parts, received) = std::sync::mpsc::sync_channel(1);
    let runtime = std::thread::current();
    let body = NativeHttpBody::Parts {
        parts: received,
        length: (length >= 0).then_some(length as u64),
        runtime: runtime.clone(),
    };
    let (worker_cancelled, worker_connection, worker_stopped) =
        (cancelled.clone(), connection.clone(), body_stopped.clone());
    std::thread::Builder::new()
        .name("seseragi-http-exchange".into())
        .spawn(move || {
            let stop_body = || worker_stopped.store(true, std::sync::atomic::Ordering::Release);
            let publish = |item| {
                let sent = send.send(item).is_ok();
                runtime.unpark();
                sent
            };
            let (mut reader, written) = match native_http_open(
                &method,
                &url,
                &headers,
                body,
                &worker_cancelled,
                &worker_connection,
            ) {
                Ok(opened) => opened,
                Err(reply) => {
                    stop_body();
                    publish(Err(reply));
                    return;
                }
            };
            // A final response stops a body still being sent, as the TS
            // HTTP/1 writer does; the response is still read to its end.
            let mut stop_writer = || {
                stop_body();
                if !written.load(std::sync::atomic::Ordering::Acquire) {
                    let _ = native_http_stop_writing(&worker_connection);
                }
            };
            let result = native_http_read_events(
                &mut reader,
                method == "HEAD",
                &mut stop_writer,
                &mut |event| {
                    if publish(Ok(event)) {
                        Ok(())
                    } else {
                        native_http_protocol("exchange closed")
                    }
                },
            );
            stop_body();
            let _ = reader.get_mut().shutdown(std::net::Shutdown::Both);
            if worker_cancelled.load(std::sync::atomic::Ordering::Acquire) {
                // A closed exchange or a failed body: the end of the events
                // wakes a waiting pull, which reports the body failure.
                drop(send);
                runtime.unpark();
                return;
            }
            match result {
                Ok(()) => {
                    // The end of the events is a disconnection; wake the
                    // runtime so a waiting pull observes it.
                    drop(send);
                    runtime.unpark();
                }
                Err(NativeHttpRead::Io(error)) => {
                    publish(Err(NativeHttpReply::Failure(
                        "std/http::HttpConnectionFailure",
                        error.to_string(),
                    )));
                }
                Err(NativeHttpRead::Protocol(message)) => {
                    publish(Err(NativeHttpReply::Failure(
                        "std/http::HttpProtocolFailure",
                        message,
                    )));
                }
                Err(NativeHttpRead::Limit) => {}
            }
        })
        .map_err(|error| NativeExit::defects(vec![format!("cannot start HTTP worker: {error}")]))?;
    Ok(NATIVE_HTTP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        streams.0 += 1;
        let id = streams.0;
        streams.1.insert(
            id,
            NativeHttpStream {
                events,
                cancelled,
                connection,
                body_stopped,
                body_failed: false,
                parts: Some(parts),
                declared,
                sent: 0,
            },
        );
        id
    }))
}

// Stops the exchange because its body failed. Returns false when the
// exchange had already stopped the body, so another failure came first.
fn native_http_stream_fail_body(id: i64) -> bool {
    native_http_with_stream(id, |stream| {
        let Some(stream) = stream else {
            return false;
        };
        if stream
            .body_stopped
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            return false;
        }
        stream.body_failed = true;
        stream.parts = None;
        stream
            .cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(connection) = stream.connection.lock().unwrap().as_ref() {
            let _ = connection.shutdown(std::net::Shutdown::Both);
        }
        true
    })
}

// The length mismatch at the end of a body, if its declared length differs.
fn native_http_stream_mismatch(id: i64) -> Option<V> {
    native_http_with_stream(id, |stream| {
        let stream = stream?;
        let declared = stream.declared?;
        (declared != stream.sent).then(|| native_http_length_mismatch(declared, stream.sent))
    })
}

// A request body chunk, or the end of the body, waiting to be handed to the
// writer of an exchange. The wait ends with true once the writer has the
// part, or with false once the exchange no longer takes the body.
struct NativeHttpWrite {
    id: i64,
    part: Option<NativeHttpPart>,
}

impl NativeHttpWrite {
    fn new(captures: &[V]) -> Self {
        Self {
            id: native_http_int(&captures[0]),
            part: Some(match &captures[1] {
                V::Unit => NativeHttpPart::End,
                bytes => NativeHttpPart::Chunk(native_bytes(bytes).to_vec()),
            }),
        }
    }

    fn poll(&mut self) -> Option<NativeOutcome> {
        native_http_with_stream(self.id, |stream| {
            let Some(stream) = stream else {
                return Some(Ok(V::Bool(false)));
            };
            if stream
                .body_stopped
                .load(std::sync::atomic::Ordering::Acquire)
            {
                stream.parts = None;
            }
            let Some(parts) = &stream.parts else {
                return Some(Ok(V::Bool(false)));
            };
            let part = self.part.take().expect("pending HTTP body part");
            let (length, end) = match &part {
                NativeHttpPart::Chunk(bytes) => (bytes.len() as i64, false),
                NativeHttpPart::End => (0, true),
            };
            match parts.try_send(part) {
                Ok(()) => {
                    stream.sent += length;
                    if end {
                        stream.parts = None;
                    }
                    Some(Ok(V::Bool(true)))
                }
                Err(std::sync::mpsc::TrySendError::Full(part)) => {
                    self.part = Some(part);
                    None
                }
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    stream.parts = None;
                    Some(Ok(V::Bool(false)))
                }
            }
        })
    }
}

fn native_http_event_value(event: NativeHttpEvent) -> V {
    match event {
        NativeHttpEvent::Head {
            informational,
            minor,
            status,
            headers,
        } => V::adt(
            if informational {
                "std/http::InformationalResponse"
            } else {
                "std/http::ResponseStarted"
            },
            native_http_record(vec![
                (
                    "version",
                    V::adt(
                        if minor == 0 {
                            "std/http::Http1_0"
                        } else {
                            "std/http::Http1_1"
                        },
                        V::Unit,
                    ),
                ),
                (
                    "status",
                    native_http_value("std/http::Status", V::Int(status as i64)),
                ),
                ("headers", native_http_headers_value(headers)),
            ]),
        ),
        NativeHttpEvent::Chunk(bytes) => {
            V::adt("std/http::ResponseBodyChunk", V::Bytes(Shared::new(bytes)))
        }
        NativeHttpEvent::Trailers(headers) => V::adt(
            "std/http::ResponseTrailers",
            native_http_headers_value(headers),
        ),
    }
}

// The next event of an exchange, or None while none has arrived. Unit means
// the body failed first; the pull then reports the body's failure.
fn native_http_stream_next(id: i64) -> Option<NativeOutcome> {
    native_http_with_stream(id, |stream| {
        let Some(stream) = stream else {
            return Some(Ok(stream_done()));
        };
        if stream.body_failed {
            return Some(Ok(V::Unit));
        }
        if stream
            .body_stopped
            .load(std::sync::atomic::Ordering::Acquire)
        {
            // A writer waiting for the next chunk ends.
            stream.parts = None;
        }
        match stream.events.try_recv() {
            Ok(Ok(event)) => Some(Ok(stream_item(native_http_event_value(event)))),
            Ok(Err(reply)) => Some(Err(native_http_reply_exit(reply))),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Ok(stream_done())),
        }
    })
}

fn native_http_stream_close(id: i64) {
    let stream = NATIVE_HTTP_STREAMS.with(|streams| streams.borrow_mut().1.remove(&id));
    drop(stream);
}

fn native_http_body(stream: V, length: i64) -> V {
    native_http_value(
        "std/http::Body",
        native_http_record(vec![("stream", stream), ("length", V::Int(length))]),
    )
}

// What the body Fiber leaves for the exchange: Unit while nothing failed,
// `Left` with the body's failure, `Right` with a length mismatch, or this
// marker when the body Fiber ended with a Defect.
const NATIVE_HTTP_BODY_DEFECT: &str = "\0http.body.defect";

fn native_http_fail(error: V) -> V {
    V::leaf(vec![error], |captures, _| {
        Err(NativeExit::failure(captures[0].clone()))
    })
}

// Records a failure of the body unless the exchange already stopped it.
fn native_http_body_failure(id: &V, outcome: &V, settled: &V, failure: V) {
    settled.write_reference(V::Bool(true));
    if native_http_stream_fail_body(native_http_int(id)) {
        outcome.write_reference(failure);
    }
}

// Captures: the exchange handle, the body cursor, the outcome and whether
// the body ended in a way the exchange has observed.
fn native_http_body_loop(c: &[V]) -> V {
    V::bind(
        V::effect(Instruction::Attempt, vec![stream_pull(&c[1])]),
        V::fun(c.to_vec(), |c, pulled| {
            if pulled.tagged("std/prelude::Left") {
                native_http_body_failure(&c[0], &c[2], &c[3], native_left(pulled.payload()));
                return stream_pure(V::Unit);
            }
            let next = pulled.payload();
            if next.tagged("std/prelude::Nothing") {
                if let Some(mismatch) = native_http_stream_mismatch(native_http_int(&c[0])) {
                    native_http_body_failure(&c[0], &c[2], &c[3], native_right(mismatch));
                    return stream_pure(V::Unit);
                }
                return V::bind(
                    V::effect(Instruction::HttpWrite, vec![c[0].clone(), V::Unit]),
                    V::fun(vec![c[3].clone()], |c, _| {
                        c[0].write_reference(V::Bool(true));
                        stream_pure(V::Unit)
                    }),
                );
            }
            let bytes = next.payload();
            if native_bytes(&bytes).is_empty() {
                return native_http_body_loop(c);
            }
            V::bind(
                V::effect(Instruction::HttpWrite, vec![c[0].clone(), bytes]),
                V::fun(c.to_vec(), |c, taken| {
                    if taken.boolean() {
                        native_http_body_loop(c)
                    } else {
                        // The exchange stopped the body.
                        c[3].write_reference(V::Bool(true));
                        stream_pure(V::Unit)
                    }
                }),
            )
        }),
    )
}

// The body Fiber. It always closes the body cursor. A Defect stops the
// exchange like a failure, and the exchange reports the Defect.
fn native_http_body_fiber(id: V, cursor: V, outcome: V) -> V {
    let settled = stream_cell(V::Bool(false));
    V::effect(
        Instruction::Bracket,
        vec![
            stream_pure(V::Unit),
            V::fun(
                vec![id.clone(), cursor.clone(), outcome.clone(), settled.clone()],
                |c, _| native_http_body_loop(c),
            ),
            // The exchange is stopped before the cursor closes, so a Defect
            // while closing cannot leave it waiting for the body.
            V::fun(vec![id, cursor, outcome, settled], |c, _| {
                V::bind(
                    V::leaf(c.to_vec(), |c, _| {
                        if !c[3].read_reference().boolean() {
                            native_http_body_failure(
                                &c[0],
                                &c[2],
                                &c[3],
                                V::adt(NATIVE_HTTP_BODY_DEFECT, V::Unit),
                            );
                        }
                        Ok(V::Unit)
                    }),
                    V::fun(vec![c[1].clone()], |c, _| stream_close(&c[0])),
                )
            }),
        ],
    )
}

// Stops the body Fiber without waiting for it, as TS cancels the body when
// the final response arrives.
fn native_http_stop_body(fiber: &V) -> V {
    if fiber.fiber().outcome.borrow().is_some() {
        return stream_pure(V::Unit);
    }
    native_operation(
        "std/effect::fork",
        vec![native_operation(
            "std/effect::interrupt",
            vec![fiber.clone()],
        )],
    )
}

// Captures: the exchange handle, the body outcome and the body Fiber.
fn native_http_exchange_pull(c: &[V]) -> V {
    V::bind(
        V::effect(Instruction::HttpNext, vec![c[0].clone()]),
        V::fun(c.to_vec(), |c, next| {
            if matches!(next, V::Unit) {
                let failure = c[1].read_reference();
                if !failure.tagged(NATIVE_HTTP_BODY_DEFECT) {
                    return native_http_fail(failure);
                }
                // Interrupting the finished body Fiber reports its Defect.
                return V::bind(
                    native_operation("std/effect::interrupt", vec![c[2].clone()]),
                    V::fun(vec![], |_, _| {
                        native_http_fail(native_right(V::adt(
                            "std/http::HttpRequestBodyFailure",
                            V::text("request body stopped"),
                        )))
                    }),
                );
            }
            if next.tagged("std/prelude::Just")
                && next.payload().tagged("std/http::ResponseStarted")
            {
                return V::bind(
                    native_http_stop_body(&c[2]),
                    V::fun(vec![next], |c, _| stream_pure(c[0].clone())),
                );
            }
            stream_pure(next)
        }),
    )
}

// Captures: the exchange handle, the body outcome and the body Fiber.
fn native_http_exchange_close(c: &[V]) -> V {
    V::bind(
        V::leaf(vec![c[0].clone()], |c, _| {
            native_http_stream_close(native_http_int(&c[0]));
            Ok(V::Unit)
        }),
        V::fun(c.to_vec(), |c, _| {
            // A Defect the exchange already reported is not reported again.
            if c[1].read_reference().tagged(NATIVE_HTTP_BODY_DEFECT) {
                return stream_pure(V::Unit);
            }
            V::bind(
                native_operation("std/effect::interrupt", vec![c[2].clone()]),
                V::fun(vec![], |_, _| stream_pure(V::Unit)),
            )
        }),
    )
}

fn native_http_exchange_stream(body: V, request: V) -> V {
    V::fun(vec![body, request], |captures, _| {
        let body = native_http_inner(&captures[0], "std/http::Body");
        V::bind(
            V::effect(
                Instruction::Attempt,
                vec![stream_open(&body.field("stream"))],
            ),
            V::fun(
                vec![captures[1].clone(), body.field("length")],
                |captures, opened| {
                    if opened.tagged("std/prelude::Left") {
                        return native_http_fail(native_left(opened.payload()));
                    }
                    let cursor = opened.payload();
                    let id = match native_http_stream_open(
                        &captures[0],
                        native_http_int(&captures[1]),
                    ) {
                        Ok(id) => V::Int(id),
                        Err(exit) => {
                            let failed = match exit {
                                NativeExit::Failure(error) => native_http_fail(error.value),
                                NativeExit::Defect(errors) => native_effect_die(V::text(
                                    &errors
                                        .iter()
                                        .map(|error| error.value.as_str())
                                        .collect::<Vec<_>>()
                                        .join("; "),
                                )),
                                NativeExit::Cancelled => {
                                    V::leaf(vec![], |_, _| Err(NativeExit::Cancelled))
                                }
                            };
                            return V::bind(
                                stream_close(&cursor),
                                V::fun(vec![failed], |c, _| c[0].clone()),
                            );
                        }
                    };
                    let outcome = stream_cell(V::Unit);
                    V::bind(
                        native_operation(
                            "std/effect::fork",
                            vec![native_http_body_fiber(id.clone(), cursor, outcome.clone())],
                        ),
                        V::fun(vec![id, outcome], |c, fiber| {
                            let captures = vec![c[0].clone(), c[1].clone(), fiber];
                            stream_pure(stream_cursor(
                                V::fun(captures.clone(), |c, _| native_http_exchange_pull(c)),
                                V::fun(captures, |c, _| native_http_exchange_close(c)),
                            ))
                        }),
                    )
                },
            ),
        )
    })
}
