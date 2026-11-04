#[cfg(all(test, unix))]
mod child_process_host_tests {
    use super::*;

    fn command(script: &str) -> V {
        let base = native_child_process_operation(
            "std/child-process::command",
            vec![V::adt(
                "std/child-process::ExecutablePath",
                native_path("/bin/sh"),
            )],
        )
        .payload();
        let command = native_child_process_operation(
            "std/child-process::addArguments",
            vec![
                V::Array(Shared::new(vec![V::text("-c"), V::text(script)])),
                base,
            ],
        )
        .payload();
        native_child_with(&command, "grace", V::Int(20))
    }
    fn empty() -> V {
        V::Bytes(Shared::new(vec![]))
    }
    fn poll(task: &mut NativeChildTask) -> NativeOutcome {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if let Some(result) = task.poll() {
                return result;
            }
            assert!(std::time::Instant::now() < deadline, "child task deadline");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[test]
    fn captured_limits_are_independent_and_inclusive() {
        let mut context = NativeContext::default();
        let result = context
            .root(native_child_run(
                command("printf 1234; printf 5678 >&2; exit 7"),
                empty(),
                V::Int(4),
            ))
            .unwrap_or_else(|_| panic!("capture failed"));
        assert_eq!(native_bytes(&result.field("stdout")), b"1234");
        assert_eq!(native_bytes(&result.field("stderr")), b"5678");
        assert_eq!(native_child_int(&result.field("status").payload()), 7);
        for (script, channel) in [
            ("printf 12345", "std/child-process::ChildStdout"),
            ("printf 12345 >&2", "std/child-process::ChildStderr"),
        ] {
            let Err(NativeExit::Failure(error)) =
                context.root(native_child_run(command(script), empty(), V::Int(4)))
            else {
                panic!("expected output limit failure")
            };
            assert!(error
                .value
                .tagged("std/child-process::ChildOutputLimitExceeded"));
            assert!(error.value.payload().field("channel").tagged(channel));
            assert_eq!(
                native_child_int(&error.value.payload().field("limitBytes")),
                4
            );
        }
    }
    #[test]
    fn cancellation_waits_for_grace_kills_reaps_and_joins_the_writer() {
        let resource = NativeChildResource::start(
            &command("trap '' TERM; printf ready; while :; do :; done"),
            false,
        )
        .unwrap_or_else(|_| panic!("spawn failed"));
        let child = V::ChildProcess(Shared::new(resource));
        let mut task = NativeChildTask::start(
            NativeChildOperation::Captured,
            vec![child.clone(), empty(), V::Int(100)],
        )
        .unwrap_or_else(|_| panic!("task failed"));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while task.output[0] != b"ready" {
            assert!(task.poll().is_none());
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        task.cancelled = true;
        let start = std::time::Instant::now();
        assert!(matches!(poll(&mut task), Err(NativeExit::Cancelled)));
        assert!(start.elapsed() >= std::time::Duration::from_millis(20));
        let state = child.child_process().0.borrow();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(state.status.unwrap().signal(), Some(9));
        assert!(state.writer.is_none());
        assert!(state.stdout.is_none() && state.stderr.is_none());
    }
    #[test]
    fn empty_writes_after_close_fail_and_repeated_close_succeeds() {
        let resource = NativeChildResource::start(&command("cat >/dev/null"), false)
            .unwrap_or_else(|_| panic!("spawn failed"));
        let child = V::ChildProcess(Shared::new(resource));
        for _ in 0..2 {
            let mut task = NativeChildTask::start(
                NativeChildOperation::Input,
                vec![
                    child.clone(),
                    V::adt("std/child-process::CloseChildStdin", V::Unit),
                ],
            )
            .unwrap_or_else(|_| panic!("task failed"));
            assert!(poll(&mut task).is_ok());
        }
        let mut task = NativeChildTask::start(
            NativeChildOperation::Input,
            vec![
                child.clone(),
                V::adt("std/child-process::WriteChildStdin", empty()),
            ],
        )
        .unwrap_or_else(|_| panic!("task failed"));
        let Err(NativeExit::Failure(error)) = poll(&mut task) else {
            panic!("expected input after close")
        };
        assert!(error
            .value
            .tagged("std/child-process::ChildInputAfterClose"));
        assert!(child.child_process().0.borrow().writer.is_none());
    }
}
