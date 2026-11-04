fn native_child_record(fields: Vec<(&str, V)>) -> V {
    V::Record(Shared::new(
        fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
    ))
}
fn native_child_config(tag: &'static str, value: V) -> V {
    native_left(V::adt(tag, value))
}
fn native_child_command(value: &V) -> BTreeMap<String, V> {
    assert!(
        value.tagged("std/child-process::Command"),
        "Rust ABI: Command owner"
    );
    value.payload().record_fields().clone()
}
fn native_child_with(value: &V, name: &str, field: V) -> V {
    let mut fields = native_child_command(value);
    fields.insert(name.into(), field);
    V::adt("std/child-process::Command", V::Record(Shared::new(fields)))
}
fn native_child_environment(name: &V, value: V, command: &V) -> V {
    let fields = native_child_command(command);
    let mut environment = fields["environment"].record_fields().clone();
    environment.insert(native_text(name).into(), value);
    native_right(native_child_with(
        command,
        "environment",
        V::Record(Shared::new(environment)),
    ))
}
pub fn native_child_process_operation(name: &str, args: Vec<V>) -> V {
    match (name, args.as_slice()) {
        ("std/child-process::command", [executable]) => {
            if executable.tagged("std/child-process::SearchPath") {
                let value = executable.payload();
                let text = native_text(&value);
                if text.is_empty() {
                    return native_child_config("std/child-process::EmptyExecutableName", V::Unit);
                }
                if text.contains(['/', '\\']) {
                    return native_child_config(
                        "std/child-process::ExecutableNameContainsSeparator",
                        value,
                    );
                }
            } else {
                assert!(
                    executable.tagged("std/child-process::ExecutablePath"),
                    "Rust ABI: Executable owner"
                );
            }
            native_right(V::adt(
                "std/child-process::Command",
                native_child_record(vec![
                    ("executable", executable.clone()),
                    ("arguments", V::Array(Shared::new(vec![]))),
                    ("directory", V::Unit),
                    ("clear", V::Bool(false)),
                    ("environment", V::Record(Shared::new(BTreeMap::new()))),
                    ("grace", V::Int(5000)),
                    ("buffer", V::Int(16)),
                ]),
            ))
        }
        (
            "std/child-process::addArgument" | "std/child-process::addArguments",
            [values, command],
        ) => {
            let mut arguments =
                native_child_array(&native_child_command(command)["arguments"]).to_vec();
            let values = if name.ends_with("::addArgument") {
                vec![values.clone()]
            } else {
                native_child_array(values).to_vec()
            };
            for (index, value) in values.iter().enumerate() {
                let text = native_text(value);
                if let Some(offset) = text.find('\0') {
                    return native_child_config(
                        "std/child-process::ArgumentContainsNul",
                        native_child_record(vec![
                            ("index", V::Int((arguments.len() + index) as i64)),
                            (
                                "offset",
                                V::Int(text[..offset].encode_utf16().count() as i64),
                            ),
                        ]),
                    );
                }
            }
            arguments.extend(values);
            native_right(native_child_with(
                command,
                "arguments",
                V::Array(Shared::new(arguments)),
            ))
        }
        ("std/child-process::inDirectory", [path, command]) => {
            native_child_with(command, "directory", path.clone())
        }
        ("std/child-process::setEnvironment", [name, value, command]) => {
            if native_text(name).contains('\0') {
                return native_child_config(
                    "std/child-process::EnvironmentNameContainsNul",
                    name.clone(),
                );
            }
            if native_text(value).contains('\0') {
                return native_child_config(
                    "std/child-process::EnvironmentValueContainsNul",
                    value.clone(),
                );
            }
            native_child_environment(name, value.clone(), command)
        }
        ("std/child-process::unsetEnvironment", [name, command]) => {
            if native_text(name).contains('\0') {
                return native_child_config(
                    "std/child-process::EnvironmentNameContainsNul",
                    name.clone(),
                );
            }
            native_child_environment(name, V::Unit, command)
        }
        ("std/child-process::clearEnvironment", [command]) => {
            native_child_with(command, "clear", V::Bool(true))
        }
        ("std/child-process::terminationGrace", [duration, command]) => native_child_with(
            command,
            "grace",
            V::Int(
                (native_duration_nanoseconds(duration).expect("canonical Duration") / 1_000_000)
                    as i64,
            ),
        ),
        ("std/child-process::outputBuffer", [capacity, command]) => {
            assert!(
                capacity.tagged("std/stream::BufferCapacity"),
                "Rust ABI: BufferCapacity owner"
            );
            native_child_with(command, "buffer", capacity.payload())
        }
        ("std/child-process::captureLimit", [V::Int(limit)]) => {
            if *limit <= 0 || *limit > 9_007_199_254_740_991 {
                native_child_config("std/child-process::InvalidCaptureLimit", V::Int(*limit))
            } else {
                native_right(V::adt("std/child-process::CaptureLimit", V::Int(*limit)))
            }
        }
        ("std/child-process::defaultCaptureLimit", [V::Unit]) => {
            V::adt("std/child-process::CaptureLimit", V::Int(8 * 1024 * 1024))
        }
        ("std/child-process::runCaptured", [limit, input, command]) => {
            assert!(
                limit.tagged("std/child-process::CaptureLimit"),
                "Rust ABI: CaptureLimit owner"
            );
            native_child_run(command.clone(), input.clone(), limit.payload())
        }
        ("std/child-process::runInherited", [command]) => {
            native_child_run(command.clone(), V::Unit, V::Unit)
        }
        ("std/child-process::runStreaming", [input, command]) => {
            native_child_stream(input.clone(), command.clone())
        }
        _ => panic!("Rust ABI: unadmitted child process operation"),
    }
}

#[derive(Clone, Copy)]
enum NativeChildOperation {
    Captured,
    Inherited,
    Event,
    Input,
    Close,
}
fn native_child_action(operation: NativeChildOperation, captures: Vec<V>) -> V {
    V::effect(Instruction::ChildProcess(operation), captures)
}
fn native_child_acquire(command: V, inherited: bool) -> V {
    V::effect(
        Instruction::AcquireRelease,
        vec![
            V::leaf(vec![command, V::Bool(inherited)], |c, _| {
                NativeChildResource::start(&c[0], c[1].boolean())
                    .map(|state| V::ChildProcess(Shared::new(state)))
            }),
            V::fun(vec![], |_, child| {
                native_child_action(NativeChildOperation::Close, vec![child])
            }),
        ],
    )
}
fn native_child_run(command: V, input: V, limit: V) -> V {
    let inherited = matches!(input, V::Unit);
    V::effect(
        Instruction::Scoped,
        vec![V::bind(
            native_child_acquire(command, inherited),
            V::fun(vec![input, limit], |c, child| {
                native_child_action(
                    if matches!(c[0], V::Unit) {
                        NativeChildOperation::Inherited
                    } else {
                        NativeChildOperation::Captured
                    },
                    vec![child, c[0].clone(), c[1].clone()],
                )
            }),
        )],
    )
}
fn native_child_map_error(plan: V) -> V {
    V::effect(
        Instruction::MapError,
        vec![V::fun(vec![], |_, error| native_right(error)), plan],
    )
}
fn native_child_pump(cursor: V, child: V) -> V {
    native_stream_defer(vec![cursor, child], |c, _| {
        V::bind(
            V::effect(
                Instruction::MapError,
                vec![
                    V::fun(vec![], |_, error| native_left(error)),
                    stream_pull(&c[0]),
                ],
            ),
            V::fun(c.to_vec(), |c, item| {
                let done = item.tagged("std/prelude::Nothing");
                let event = if done {
                    V::adt("std/child-process::CloseChildStdin", V::Unit)
                } else {
                    item.payload()
                };
                V::bind(
                    native_child_map_error(native_child_action(
                        NativeChildOperation::Input,
                        vec![c[1].clone(), event],
                    )),
                    V::fun(
                        c.to_vec().into_iter().chain([V::Bool(done)]).collect(),
                        |c, _| {
                            if c[2].boolean() {
                                stream_close(&c[0])
                            } else {
                                native_child_pump(c[0].clone(), c[1].clone())
                            }
                        },
                    ),
                )
            }),
        )
    })
}
fn native_child_stream_close(input: V, child: V, fiber: V) -> V {
    // Stop the child before interrupting a blocked stdin write.
    V::bind(
        native_child_map_error(native_child_action(
            NativeChildOperation::Close,
            vec![child],
        )),
        V::fun(vec![input, fiber], |c, _| {
            V::bind(
                V::effect(
                    Instruction::Concurrent(ConcurrentOperation::Interrupt),
                    vec![c[1].clone()],
                ),
                V::fun(vec![c[0].clone()], |c, _| stream_close(&c[0])),
            )
        }),
    )
}
fn native_child_stream_cursor(input: V, child: V, error: V, fiber: V) -> V {
    let close = V::fun(vec![input, child.clone(), fiber.clone()], |c, _| {
        native_child_stream_close(c[0].clone(), c[1].clone(), c[2].clone())
    });
    let pull = V::fun(vec![child, error, fiber, close.clone()], |c, _| {
        V::bind(
            native_child_action(NativeChildOperation::Event, c[..3].to_vec()),
            V::fun(vec![c[3].clone()], |c, event| {
                if event.tagged("std/prelude::Nothing")
                    || event.payload().tagged("std/child-process::ChildExitedWith")
                {
                    V::bind(
                        c[0].apply(V::Unit),
                        V::fun(vec![event], |c, _| stream_pure(c[0].clone())),
                    )
                } else {
                    stream_pure(event)
                }
            }),
        )
    });
    stream_cursor(pull, close)
}
fn native_child_stream_open(input: V, command: V) -> V {
    V::bind(
        native_child_map_error(native_child_acquire(command, false)),
        V::fun(vec![input], |c, child| {
            let error = stream_cell(V::Unit);
            let pump = V::effect(
                Instruction::Recover,
                vec![
                    V::fun(vec![error.clone()], |c, error| {
                        c[0].write_reference(error);
                        stream_pure(V::Unit)
                    }),
                    native_child_pump(c[0].clone(), child.clone()),
                ],
            );
            V::bind(
                V::effect(
                    Instruction::Concurrent(ConcurrentOperation::Fork),
                    vec![pump],
                ),
                V::fun(vec![c[0].clone(), child, error], |c, fiber| {
                    stream_pure(native_child_stream_cursor(
                        c[0].clone(),
                        c[1].clone(),
                        c[2].clone(),
                        fiber,
                    ))
                }),
            )
        }),
    )
}
fn native_child_stream(input: V, command: V) -> V {
    V::fun(vec![input, command], |c, _| {
        let opened = V::effect(
            Instruction::MapError,
            vec![
                V::fun(vec![], |_, error| native_left(error)),
                stream_open(&c[0]),
            ],
        );
        V::bind(
            V::effect(
                Instruction::AcquireRelease,
                vec![opened, V::fun(vec![], |_, cursor| stream_close(&cursor))],
            ),
            V::fun(vec![c[1].clone()], |c, input| {
                native_child_stream_open(input, c[0].clone())
            }),
        )
    })
}

#[cfg(test)]
mod child_process_config_tests {
    use super::*;

    #[test]
    fn argument_offsets_use_utf16_and_failed_updates_preserve_the_command() {
        let command = native_child_process_operation(
            "std/child-process::command",
            vec![V::adt("std/child-process::SearchPath", V::text("tool"))],
        )
        .payload();
        let error = native_child_process_operation(
            "std/child-process::addArguments",
            vec![
                V::Array(Shared::new(vec![V::text("first"), V::text("雪🙂\0")])),
                command.clone(),
            ],
        )
        .payload();
        assert!(error.tagged("std/child-process::ArgumentContainsNul"));
        assert_eq!(native_child_int(&error.payload().field("index")), 1);
        assert_eq!(native_child_int(&error.payload().field("offset")), 3);
        assert!(native_child_array(&native_child_command(&command)["arguments"]).is_empty());
        let edited = native_child_process_operation(
            "std/child-process::setEnvironment",
            vec![V::text("name"), V::text("value"), command.clone()],
        )
        .payload();
        assert!(native_child_command(&command)["environment"]
            .record_fields()
            .is_empty());
        assert_eq!(
            native_text(&native_child_command(&edited)["environment"].field("name")),
            "value"
        );
    }
}
