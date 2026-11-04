use seseragi_native_runtime::NativeJsonCodecFrame;

fn nested_frames(count: usize) {
    let _frame = NativeJsonCodecFrame::enter();
    if count > 1 {
        nested_frames(count - 1);
    }
}

#[test]
fn generated_json_codecs_can_enter_frames_and_restore_the_host_limit_after_unwinding() {
    for _ in 0..2 {
        nested_frames(128);
        let failure = std::panic::catch_unwind(|| nested_frames(129)).unwrap_err();
        assert_eq!(
            failure.downcast_ref::<&str>(),
            Some(&"JSON codec nesting exceeds host limits")
        );
    }
}
