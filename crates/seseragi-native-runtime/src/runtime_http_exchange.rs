// `std/http.exchange` on the native target: a Stream of the events of one
// HTTP/1.1 exchange. Opening the Stream reads the request body Stream to its
// end, then a worker sends the request and passes each response event over a
// channel. Each pull waits for the next event; closing the Stream closes the
// connection. Failures are `Left` for the body and `Right` for HTTP, as on TS.

struct NativeHttpStream {
    events: std::sync::mpsc::Receiver<Result<NativeHttpEvent, NativeHttpReply>>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    connection: std::sync::Arc<std::sync::Mutex<Option<std::net::TcpStream>>>,
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

// Starts the exchange and returns its handle.
fn native_http_stream_open(request: &V, body: Vec<u8>) -> Result<i64, NativeExit> {
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
    native_http_declared_length(&headers, body.len()).map_err(native_http_right)?;
    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let connection = std::sync::Arc::new(std::sync::Mutex::new(None));
    let (send, events) = std::sync::mpsc::channel();
    let runtime = std::thread::current();
    let (worker_cancelled, worker_connection) = (cancelled.clone(), connection.clone());
    std::thread::Builder::new()
        .name("seseragi-http-exchange".into())
        .spawn(move || {
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
                    publish(Err(reply));
                    return;
                }
            };
            let mut stop_writer = || {
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
            let _ = reader.get_mut().shutdown(std::net::Shutdown::Both);
            if worker_cancelled.load(std::sync::atomic::Ordering::Acquire) {
                return;
            }
            match result {
                Ok(()) => drop(send),
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
            },
        );
        id
    }))
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

// The next event of an exchange, or None while none has arrived.
fn native_http_stream_next(id: i64) -> Option<NativeOutcome> {
    NATIVE_HTTP_STREAMS.with(|streams| {
        let streams = streams.borrow();
        let Some(stream) = streams.1.get(&id) else {
            return Some(Ok(stream_done()));
        };
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

fn native_http_exchange_stream(body: V, request: V) -> V {
    V::fun(vec![body, request], |captures, _| {
        let source = native_http_inner(&captures[0], "std/http::Body").field("stream");
        let collected = native_operation(
            "std/effect::mapError",
            vec![
                V::fun(vec![], |_, error| native_left(error)),
                native_stream_operation("std/stream::runCollect", vec![source]),
            ],
        );
        V::bind(
            collected,
            V::fun(vec![captures[1].clone()], |captures, chunks| {
                V::leaf(vec![captures[0].clone(), chunks], |captures, _| {
                    let V::Array(chunks) = &captures[1] else {
                        panic!("Rust ABI: expected collected body chunks")
                    };
                    let body: Vec<u8> = chunks
                        .get()
                        .iter()
                        .flat_map(|chunk| native_bytes(chunk).iter().copied())
                        .collect();
                    let id = native_http_stream_open(&captures[0], body)?;
                    Ok(stream_cursor(
                        V::fun(vec![V::Int(id)], |captures, _| {
                            V::effect(Instruction::HttpNext, vec![captures[0].clone()])
                        }),
                        V::fun(vec![V::Int(id)], |captures, _| {
                            V::leaf(captures.to_vec(), |captures, _| {
                                native_http_stream_close(native_http_int(&captures[0]));
                                Ok(V::Unit)
                            })
                        }),
                    ))
                })
            }),
        )
    })
}
