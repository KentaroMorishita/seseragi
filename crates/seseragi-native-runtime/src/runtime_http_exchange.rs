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
    // The body failed first, or its Fiber ended with a Defect; the next pull
    // reports it.
    body_failed: bool,
    // The task of the body Fiber while it runs, and whether the scheduler
    // has cancelled it because the exchange stopped the body.
    body_task: Option<usize>,
    body_cancelled: bool,
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
            // The runtime is woken at once, since the next event may wait
            // for room in the channel while the scheduler must still cancel
            // the body Fiber.
            let stop_body = || {
                worker_stopped.store(true, std::sync::atomic::Ordering::Release);
                runtime.unpark();
            };
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
                body_task: None,
                body_cancelled: false,
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
        stream.abort();
        true
    })
}

impl NativeHttpStream {
    // Stops the exchange for a failure of its body.
    fn abort(&mut self) {
        self.body_stopped
            .store(true, std::sync::atomic::Ordering::Release);
        self.body_failed = true;
        self.parts = None;
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(connection) = self.connection.lock().unwrap().as_ref() {
            let _ = connection.shutdown(std::net::Shutdown::Both);
        }
    }
}

// Lets the scheduler follow the body Fiber of an exchange. A body Fiber that
// already ended with a Defect stops the exchange at once.
fn native_http_stream_attach_body(id: i64, fiber: &V) {
    let fiber = fiber.fiber();
    let defect = match fiber.outcome.borrow().as_ref() {
        None => false,
        Some(outcome) => matches!(outcome, Err(NativeExit::Defect(_))),
    };
    let running = fiber.outcome.borrow().is_none();
    native_http_with_stream(id, |stream| {
        let Some(stream) = stream else { return };
        if defect {
            stream.abort();
        } else if running {
            stream.body_task = Some(fiber.task);
        }
    });
}

// The body Fibers to cancel because their exchange stopped the body: at the
// final response, at its end or at an HTTP failure. TS cancels the body then
// too, even while the consumer has not taken the response yet.
fn native_http_body_tasks_to_stop() -> Vec<usize> {
    NATIVE_HTTP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        streams
            .1
            .values_mut()
            .filter_map(|stream| {
                let task = stream.body_task?;
                let stopped = stream
                    .body_stopped
                    .load(std::sync::atomic::Ordering::Acquire);
                if !stopped || stream.body_failed || stream.body_cancelled {
                    return None;
                }
                stream.body_cancelled = true;
                Some(task)
            })
            .collect()
    })
}

// A body Fiber ended. A Defect, even one raised while the body was being
// stopped, stops the exchange and wakes its pull, as TS reports a Defect
// from cancelling the body. So does a cancellation the exchange did not ask
// for.
fn native_http_body_task_ended(task: usize, outcome: &NativeOutcome) {
    NATIVE_HTTP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if streams.1.is_empty() {
            return;
        }
        if let Some(stream) = streams
            .1
            .values_mut()
            .find(|stream| stream.body_task == Some(task))
        {
            stream.body_task = None;
            // A body Fiber cancelled by anything but its exchange, such as a
            // join of an interrupted Fiber, ends the body early too.
            let cancelled = matches!(outcome, Err(NativeExit::Cancelled)) && !stream.body_cancelled;
            if matches!(outcome, Err(NativeExit::Defect(_))) || cancelled {
                stream.abort();
            }
        }
    });
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
// `Left` with the body's failure or `Right` with a length mismatch. The pull
// sets this marker once it has reported a Defect of the body Fiber.
const NATIVE_HTTP_BODY_DEFECT: &str = "\0http.body.defect";

fn native_http_fail(error: V) -> V {
    V::leaf(vec![error], |captures, _| {
        Err(NativeExit::failure(captures[0].clone()))
    })
}

// Records a failure of the body unless the exchange already stopped it.
fn native_http_body_failure(id: &V, outcome: &V, failure: V) {
    if native_http_stream_fail_body(native_http_int(id)) {
        outcome.write_reference(failure);
    }
}

// Captures: the exchange handle, the body cursor and the outcome.
fn native_http_body_loop(c: &[V]) -> V {
    V::bind(
        V::effect(Instruction::Attempt, vec![stream_pull(&c[1])]),
        V::fun(c.to_vec(), |c, pulled| {
            if pulled.tagged("std/prelude::Left") {
                native_http_body_failure(&c[0], &c[2], native_left(pulled.payload()));
                return stream_pure(V::Unit);
            }
            let next = pulled.payload();
            if next.tagged("std/prelude::Nothing") {
                if let Some(mismatch) = native_http_stream_mismatch(native_http_int(&c[0])) {
                    native_http_body_failure(&c[0], &c[2], native_right(mismatch));
                    return stream_pure(V::Unit);
                }
                return V::bind(
                    V::effect(Instruction::HttpWrite, vec![c[0].clone(), V::Unit]),
                    V::fun(vec![], |_, _| stream_pure(V::Unit)),
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
                        stream_pure(V::Unit)
                    }
                }),
            )
        }),
    )
}

// The body Fiber. Its pulls, the close of the body cursor and the resources
// the body acquired all end inside its own scope. A failure while closing is
// a failure of the body. The scheduler watches how the Fiber ends, so a
// Defect anywhere, even after the body has ended or while the exchange stops
// it, stops the exchange and is reported by the next pull.
fn native_http_body_fiber(id: V, cursor: V, outcome: V) -> V {
    let body = V::effect(
        Instruction::Scoped,
        vec![V::effect(
            Instruction::Bracket,
            vec![
                stream_pure(V::Unit),
                V::fun(vec![id.clone(), cursor.clone(), outcome.clone()], |c, _| {
                    native_http_body_loop(c)
                }),
                V::fun(vec![cursor], |c, _| stream_close(&c[0])),
            ],
        )],
    );
    V::bind(
        V::effect(Instruction::Attempt, vec![body]),
        V::fun(vec![id, outcome], |c, result| {
            if result.tagged("std/prelude::Left") {
                native_http_body_failure(&c[0], &c[1], native_left(result.payload()));
            }
            stream_pure(V::Unit)
        }),
    )
}

// Captures: the exchange handle, the body outcome and the body Fiber.
fn native_http_exchange_pull(c: &[V]) -> V {
    V::bind(
        V::effect(Instruction::HttpNext, vec![c[0].clone()]),
        V::fun(c.to_vec(), |c, next| {
            if matches!(next, V::Unit) {
                let failure = c[1].read_reference();
                if !matches!(failure, V::Unit) {
                    return native_http_fail(failure);
                }
                // The body Fiber ended with a Defect or was cancelled.
                // Interrupting it reports a Defect, and closing the exchange
                // does not again; a cancelled body is a body failure.
                c[1].write_reference(V::adt(NATIVE_HTTP_BODY_DEFECT, V::Unit));
                return V::bind(
                    native_operation("std/effect::interrupt", vec![c[2].clone()]),
                    V::fun(vec![], |_, _| {
                        native_http_fail(native_right(V::adt(
                            "std/http::HttpRequestBodyFailure",
                            V::text("request body was cancelled"),
                        )))
                    }),
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
                            native_http_stream_attach_body(native_http_int(&c[0]), &fiber);
                            let captures = vec![c[0].clone(), c[1].clone(), fiber];
                            let cursor = stream_cursor(
                                V::fun(captures.clone(), |c, _| native_http_exchange_pull(c)),
                                V::fun(captures, |c, _| native_http_exchange_close(c)),
                            );
                            // The scope also closes the exchange, since a
                            // terminal operation does not close a cursor
                            // whose pull failed.
                            V::effect(
                                Instruction::AcquireRelease,
                                vec![
                                    stream_pure(cursor),
                                    V::fun(vec![], |_, cursor| stream_close(&cursor)),
                                ],
                            )
                        }),
                    )
                },
            ),
        )
    })
}

#[cfg(test)]
mod http_exchange_tests {
    use super::*;

    // A server that reads every request and never answers.
    fn silent() -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                std::thread::spawn(move || {
                    let mut rest = [0u8; 4096];
                    while matches!(std::io::Read::read(&mut stream, &mut rest), Ok(read) if read > 0)
                    {
                    }
                });
            }
        });
        base
    }

    fn post(url: &str, headers: &[(&str, &str)]) -> V {
        let url = native_http_operation("std/http::parseUrl", vec![V::text(url)]);
        let mut request = native_http_operation(
            "std/http::request",
            vec![
                native_http_operation("std/http::post", vec![]),
                url.payload(),
            ],
        );
        for (name, value) in headers {
            request = native_http_operation(
                "std/http::withRequestHeader",
                vec![V::text(name), V::text(value), request],
            )
            .payload();
        }
        request
    }

    fn bytes(text: &str) -> V {
        V::Bytes(Shared::new(text.as_bytes().to_vec()))
    }

    // Runs an exchange to its end and returns its failure.
    fn failure(body: V, request: V) -> V {
        let events = native_http_operation("std/http::exchange", vec![body, request]);
        let result = NativeContext::default()
            .root(V::effect(
                Instruction::Attempt,
                vec![native_stream_operation(
                    "std/stream::runCollect",
                    vec![events],
                )],
            ))
            .unwrap_or_else(|_| panic!("the exchange ended without a result"));
        assert!(result.tagged("std/prelude::Left"), "{}", result.render());
        result.payload()
    }

    // Runs a test on its own thread and fails if it does not end in time.
    fn bounded(test: impl FnOnce() + Send + 'static) {
        let (done, finished) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            test();
            let _ = done.send(());
        });
        if finished
            .recv_timeout(std::time::Duration::from_secs(20))
            .is_err()
        {
            if worker.is_finished() {
                std::panic::resume_unwind(worker.join().unwrap_err());
            }
            panic!("the exchange did not end");
        }
    }

    fn open_exchanges() -> usize {
        NATIVE_HTTP_STREAMS.with(|streams| streams.borrow().1.len())
    }

    #[test]
    fn failed_exchanges_release_their_connections() {
        let base = silent();
        let ab = native_stream_operation(
            "std/stream::fromArray",
            vec![V::Array(Shared::new(vec![bytes("ab")]))],
        );
        // The body fails after its first chunk; the server is still waiting.
        let failing = native_stream_operation(
            "std/stream::concat",
            vec![
                native_stream_operation(
                    "std/stream::fromEffect",
                    vec![native_operation("effect.fail", vec![V::text("boom")])],
                ),
                ab.clone(),
            ],
        );
        let error = failure(
            native_http_operation("std/http::streamBody", vec![failing]),
            post(&format!("{base}/"), &[]),
        );
        assert_eq!(error.render(), native_left(V::text("boom")).render());
        assert_eq!(open_exchanges(), 0);

        // The body ends shorter than it declares.
        let error = failure(
            native_http_operation("std/http::streamBody", vec![ab]),
            post(&format!("{base}/"), &[("content-length", "5")]),
        );
        assert!(error.tagged("std/prelude::Right"), "{}", error.render());
        assert!(
            error
                .payload()
                .tagged("std/http::HttpRequestLengthMismatch"),
            "{}",
            error.render()
        );
        assert_eq!(open_exchanges(), 0);

        // The connection is refused.
        let error = failure(
            native_http_operation("std/http::emptyBody", vec![V::Unit]),
            post("http://127.0.0.1:9/", &[]),
        );
        assert!(
            error.payload().tagged("std/http::HttpConnectionFailure"),
            "{}",
            error.render()
        );
        assert_eq!(open_exchanges(), 0);
    }

    #[test]
    fn a_defect_releasing_the_body_ends_the_exchange() {
        let base = silent();
        bounded(move || {
            // The body's only chunk holds a resource whose release dies once
            // the body has ended; the server never answers.
            let chunk = V::effect(
                Instruction::AcquireRelease,
                vec![
                    stream_pure(bytes("ab")),
                    V::fun(vec![], |_, _| native_effect_die(V::text("release failed"))),
                ],
            );
            let body = native_http_operation(
                "std/http::streamBody",
                vec![native_stream_operation(
                    "std/stream::fromEffect",
                    vec![chunk],
                )],
            );
            let events = native_http_operation(
                "std/http::exchange",
                vec![body, post(&format!("{base}/"), &[])],
            );
            let result = NativeContext::default().root(native_stream_operation(
                "std/stream::runCollect",
                vec![events],
            ));
            match result {
                Err(NativeExit::Defect(errors)) => assert!(
                    errors.iter().any(|error| error.value == "release failed"),
                    "{:?}",
                    errors.iter().map(|error| &error.value).collect::<Vec<_>>()
                ),
                Err(NativeExit::Failure(error)) => panic!("failure {}", error.value.render()),
                Err(NativeExit::Cancelled) => panic!("cancelled"),
                Ok(value) => panic!("success {}", value.render()),
            }
            assert_eq!(open_exchanges(), 0);
        });
    }

    // A server that sends `response` as soon as it has read the request
    // head, without reading the body, and keeps the connection open.
    fn answering(response: &'static [u8]) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                std::thread::spawn(move || {
                    use std::io::{Read, Write};
                    let mut head = Vec::new();
                    let mut byte = [0u8; 1];
                    while !head.ends_with(b"\r\n\r\n") {
                        if !matches!(stream.read(&mut byte), Ok(1)) {
                            return;
                        }
                        head.push(byte[0]);
                    }
                    let _ = stream.write_all(response);
                    let mut rest = [0u8; 4096];
                    while matches!(stream.read(&mut rest), Ok(read) if read > 0) {}
                });
            }
        });
        base
    }

    #[test]
    fn a_final_response_stops_the_body_while_the_consumer_waits() {
        let base = answering(
            b"HTTP/1.1 103 Early Hints\r\n\r\nHTTP/1.1 103 Early Hints\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok",
        );
        bounded(move || {
            // The body holds a resource and never produces a chunk. The
            // consumer, still handling the first 103, waits until the body
            // has released that resource, so only stopping the body at the
            // final response, before the consumer takes it, lets both go on.
            // The second 103 can fill the event channel while the worker
            // reads the final response.
            let plan = V::bind(
                native_operation("std/deferred::make", vec![V::Unit]),
                V::fun(vec![V::text(&base)], |c, released| {
                    let hold = V::bind(
                        V::effect(
                            Instruction::AcquireRelease,
                            vec![
                                stream_pure(V::Unit),
                                V::fun(vec![released.clone()], |c, _| {
                                    native_operation(
                                        "std/deferred::succeed",
                                        vec![V::Unit, c[0].clone()],
                                    )
                                }),
                            ],
                        ),
                        V::fun(vec![], |_, _| {
                            V::bind(
                                native_operation("std/deferred::make", vec![V::Unit]),
                                V::fun(vec![], |_, never| {
                                    native_operation("std/deferred::await", vec![never])
                                }),
                            )
                        }),
                    );
                    let body = native_http_operation(
                        "std/http::streamBody",
                        vec![native_stream_operation(
                            "std/stream::fromEffect",
                            vec![hold],
                        )],
                    );
                    let events = native_http_operation(
                        "std/http::exchange",
                        vec![body, post(&format!("{}/", native_text(&c[0])), &[])],
                    );
                    let handle = V::fun(vec![released], |c, event| {
                        if event.tagged("std/http::InformationalResponse") {
                            native_operation("std/deferred::await", vec![c[0].clone()])
                        } else {
                            stream_pure(V::Unit)
                        }
                    });
                    native_stream_operation("std/stream::runForEach", vec![handle, events])
                }),
            );
            let result = NativeContext::default().root(plan);
            assert!(result.is_ok(), "the exchange did not succeed");
            assert_eq!(open_exchanges(), 0);
        });
    }

    #[test]
    fn a_defect_stopping_the_body_ends_an_unfinished_response() {
        // The response body runs until the connection closes, which it
        // never does.
        let base = answering(b"HTTP/1.1 200 OK\r\n\r\npartial");
        bounded(move || {
            let hold = V::bind(
                V::effect(
                    Instruction::AcquireRelease,
                    vec![
                        stream_pure(V::Unit),
                        V::fun(vec![], |_, _| native_effect_die(V::text("release failed"))),
                    ],
                ),
                V::fun(vec![], |_, _| {
                    V::bind(
                        native_operation("std/deferred::make", vec![V::Unit]),
                        V::fun(vec![], |_, never| {
                            native_operation("std/deferred::await", vec![never])
                        }),
                    )
                }),
            );
            let body = native_http_operation(
                "std/http::streamBody",
                vec![native_stream_operation(
                    "std/stream::fromEffect",
                    vec![hold],
                )],
            );
            let events = native_http_operation(
                "std/http::exchange",
                vec![body, post(&format!("{base}/"), &[])],
            );
            let result = NativeContext::default().root(native_stream_operation(
                "std/stream::runCollect",
                vec![events],
            ));
            assert!(
                matches!(&result, Err(NativeExit::Defect(errors)) if errors.iter().any(|error| error.value == "release failed")),
                "the exchange did not report the Defect"
            );
            assert_eq!(open_exchanges(), 0);
        });
    }

    #[test]
    fn a_body_cancelled_from_inside_fails_the_exchange() {
        let base = silent();
        bounded(move || {
            // The body joins a Fiber that has been interrupted.
            let joined = V::bind(
                native_operation(
                    "std/effect::fork",
                    vec![V::bind(
                        native_operation("std/deferred::make", vec![V::Unit]),
                        V::fun(vec![], |_, never| {
                            native_operation("std/deferred::await", vec![never])
                        }),
                    )],
                ),
                V::fun(vec![], |_, worker| {
                    V::bind(
                        native_operation("std/effect::interrupt", vec![worker.clone()]),
                        V::fun(vec![worker], |c, _| {
                            native_operation("std/effect::join", vec![c[0].clone()])
                        }),
                    )
                }),
            );
            let error = failure(
                native_http_operation(
                    "std/http::streamBody",
                    vec![native_stream_operation(
                        "std/stream::fromEffect",
                        vec![joined],
                    )],
                ),
                post(&format!("{base}/"), &[]),
            );
            assert!(
                error.tagged("std/prelude::Right")
                    && error.payload().tagged("std/http::HttpRequestBodyFailure"),
                "{}",
                error.render()
            );
            assert_eq!(open_exchanges(), 0);
        });
    }
}
