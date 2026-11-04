fn native_stdin_failure(tag: &'static str, field: Option<(&str, u64)>) -> NativeExit {
    let payload = match field {
        Some((name, value)) => V::Record(Shared::new(BTreeMap::from([(
            name.into(),
            V::Int(value as i64),
        )]))),
        None => V::Unit,
    };
    NativeExit::failure(V::adt(tag, payload))
}

const NATIVE_STDIN_READ_SIZE: usize = 64 * 1024;
const NATIVE_STDIN_LINE_LIMIT: usize = 1024 * 1024;
const NATIVE_STDIN_MAX_POSITION: u64 = 9_007_199_254_740_991;

type NativeStdinRead = dyn FnMut(&mut [u8]) -> std::io::Result<Option<usize>>;
struct NativeStdin {
    read: Box<NativeStdinRead>,
    buffered: Vec<u8>,
    index: usize,
    offset: u64,
    eof: bool,
    active: bool,
}
impl Default for NativeStdin {
    fn default() -> Self {
        Self {
            read: Box::new(native_poll_stdin),
            buffered: Vec::new(),
            index: 0,
            offset: 0,
            eof: false,
            active: false,
        }
    }
}
impl NativeStdin {
    fn fill(&mut self) -> std::io::Result<Option<bool>> {
        if self.index < self.buffered.len() {
            return Ok(Some(true));
        }
        if self.eof {
            return Ok(Some(false));
        }
        self.buffered.resize(NATIVE_STDIN_READ_SIZE, 0);
        let result = (self.read)(&mut self.buffered);
        self.index = 0;
        match result {
            Ok(Some(count)) => {
                self.buffered.truncate(count);
                self.eof = count == 0;
                Ok(Some(!self.eof))
            }
            result => {
                self.buffered.clear();
                result.map(|_| None)
            }
        }
    }
    fn consume(&mut self, count: usize) -> Result<(), NativeExit> {
        if self.offset > NATIVE_STDIN_MAX_POSITION - count as u64 {
            self.offset = NATIVE_STDIN_MAX_POSITION + 1;
            self.buffered.clear();
            self.index = 0;
            return Err(native_stdin_failure(
                "std/stdin::StdinPositionOverflow",
                None,
            ));
        }
        self.offset += count as u64;
        self.index += count;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum NativeStdinOperation {
    Chunk,
    Line,
}
struct NativeStdinTask {
    cursor: Rc<RefCell<NativeStdin>>,
    operation: NativeStdinOperation,
    limit: usize,
    start: u64,
    bytes: Vec<u8>,
    too_long: bool,
}
impl NativeStdinTask {
    fn start(
        cursor: Rc<RefCell<NativeStdin>>,
        operation: NativeStdinOperation,
        argument: &V,
    ) -> Result<Self, NativeExit> {
        let owner = match operation {
            NativeStdinOperation::Chunk => "std/stdin::ReadSize",
            NativeStdinOperation::Line => "std/stdin::LineLimit",
        };
        assert!(
            argument.tagged(owner),
            "Rust ABI: Stdin configuration owner"
        );
        let V::Int(limit) = argument.payload() else {
            panic!("Rust ABI: Stdin configuration payload")
        };
        let limit = usize::try_from(limit).expect("positive Stdin limit");
        let maximum = match operation {
            NativeStdinOperation::Chunk => 1024 * 1024,
            NativeStdinOperation::Line => 64 * 1024 * 1024,
        };
        assert!(
            (1..=maximum).contains(&limit),
            "Rust ABI: Stdin configuration range"
        );
        let mut state = cursor.borrow_mut();
        if state.offset > NATIVE_STDIN_MAX_POSITION {
            return Err(native_stdin_failure(
                "std/stdin::StdinPositionOverflow",
                None,
            ));
        }
        if state.active {
            return Err(native_stdin_failure("std/stdin::ConcurrentStdinRead", None));
        }
        state.active = true;
        let start = state.offset;
        drop(state);
        Ok(Self {
            cursor,
            operation,
            limit,
            start,
            bytes: Vec::new(),
            too_long: false,
        })
    }
    fn poll(&mut self) -> Option<NativeOutcome> {
        let mut state = self.cursor.borrow_mut();
        // Yield between bounded reads, including while draining an overlong line.
        for _ in 0..16 {
            match state.fill() {
                Ok(None) => return None,
                Err(_) => {
                    return Some(Err(native_stdin_failure(
                        "std/stdin::StdinReadFailure",
                        None,
                    )))
                }
                Ok(Some(false)) => {
                    return Some(if self.bytes.is_empty() && !self.too_long {
                        Ok(stream_done())
                    } else {
                        self.line_result(false)
                    });
                }
                Ok(Some(true)) => {}
            }
            let available = &state.buffered[state.index..];
            if matches!(self.operation, NativeStdinOperation::Chunk) {
                let count = self.limit.min(available.len());
                let bytes = available[..count].to_vec();
                return Some(
                    state
                        .consume(count)
                        .map(|_| stream_item(V::Bytes(Shared::new(bytes)))),
                );
            }
            let newline = available.iter().position(|byte| *byte == b'\n');
            let count = newline.map_or(available.len(), |index| index + 1);
            let segment = &available[..newline.unwrap_or(count)];
            if !self.too_long {
                if self.bytes.len() + segment.len() <= self.limit + 1 {
                    self.bytes.extend_from_slice(segment);
                } else {
                    self.too_long = true;
                    self.bytes.clear();
                }
            }
            if let Err(exit) = state.consume(count) {
                return Some(Err(exit));
            }
            if newline.is_some() {
                return Some(self.line_result(true));
            }
        }
        None
    }
    fn line_result(&self, terminated: bool) -> NativeOutcome {
        let mut bytes = self.bytes.as_slice();
        if terminated && bytes.last() == Some(&b'\r') {
            bytes = &bytes[..bytes.len() - 1];
        }
        if self.too_long || bytes.len() > self.limit {
            return Err(native_stdin_failure(
                "std/stdin::StdinLineTooLong",
                Some(("limitBytes", self.limit as u64)),
            ));
        }
        let text = std::str::from_utf8(bytes).map_err(|error| {
            native_stdin_failure(
                "std/stdin::InvalidStdinUtf8",
                Some(("offset", self.start + error.valid_up_to() as u64)),
            )
        })?;
        Ok(stream_item(V::text(
            text.strip_prefix('\u{feff}').unwrap_or(text),
        )))
    }
}
impl Drop for NativeStdinTask {
    fn drop(&mut self) {
        self.cursor.borrow_mut().active = false;
    }
}

pub fn native_stdin_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/stdin::readSize" | "std/stdin::lineLimit", [V::Int(value)]) => {
            let value = checked_int_value(*value);
            let size = name.ends_with("readSize");
            let error = if value <= 0 {
                Some(if size {
                    "std/stdin::NonPositiveReadSize"
                } else {
                    "std/stdin::NonPositiveLineLimit"
                })
            } else if value > if size { 1024 * 1024 } else { 64 * 1024 * 1024 } {
                Some(if size {
                    "std/stdin::ReadSizeTooLarge"
                } else {
                    "std/stdin::LineLimitTooLarge"
                })
            } else {
                None
            };
            match error {
                Some(error) => native_left(V::adt(error, V::Int(value))),
                None => native_right(V::adt(
                    if size {
                        "std/stdin::ReadSize"
                    } else {
                        "std/stdin::LineLimit"
                    },
                    V::Int(value),
                )),
            }
        }
        ("std/stdin::defaultReadSize", [V::Unit]) => {
            V::adt("std/stdin::ReadSize", V::Int(NATIVE_STDIN_READ_SIZE as i64))
        }
        ("std/stdin::defaultLineLimit", [V::Unit]) => V::adt(
            "std/stdin::LineLimit",
            V::Int(NATIVE_STDIN_LINE_LIMIT as i64),
        ),
        ("std/stdin::readChunk", [_]) => {
            V::effect(Instruction::Stdin(NativeStdinOperation::Chunk), arguments)
        }
        ("std/stdin::readLineWith", [_]) => {
            V::effect(Instruction::Stdin(NativeStdinOperation::Line), arguments)
        }
        ("std/prelude::readLine" | "stdin.readLine", []) | ("std/stdin::readLine", [V::Unit]) => {
            V::effect(
                Instruction::Stdin(NativeStdinOperation::Line),
                vec![V::adt(
                    "std/stdin::LineLimit",
                    V::Int(NATIVE_STDIN_LINE_LIMIT as i64),
                )],
            )
        }
        ("std/stdin::lines", [limit]) => V::fun(vec![limit.clone()], |captures, _| {
            stream_cursor(
                V::fun(captures.to_vec(), |captures, _| {
                    native_stdin_operation("std/stdin::readLineWith", captures.to_vec())
                }),
                V::fun(vec![], |_, _| stream_pure(V::Unit)),
            )
        }),
        _ => panic!("Rust ABI: unsupported Stdin operation or arity"),
    }
}

#[cfg(unix)]
fn native_poll_stdin(bytes: &mut [u8]) -> std::io::Result<Option<usize>> {
    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }
    #[cfg(any(target_os = "linux", target_os = "illumos", target_os = "solaris"))]
    type PollCount = usize;
    #[cfg(not(any(target_os = "linux", target_os = "illumos", target_os = "solaris")))]
    type PollCount = u32;
    unsafe extern "C" {
        fn poll(fds: *mut PollFd, count: PollCount, timeout: i32) -> i32;
        fn read(fd: i32, bytes: *mut u8, count: usize) -> isize;
    }
    let mut descriptor = PollFd {
        fd: 0,
        events: 1,
        revents: 0,
    };
    let ready = unsafe { poll(&mut descriptor, 1, 0) };
    if ready < 0 {
        let error = std::io::Error::last_os_error();
        return if error.kind() == std::io::ErrorKind::Interrupted {
            Ok(None)
        } else {
            Err(error)
        };
    }
    if ready == 0 {
        return Ok(None);
    }
    let count = unsafe { read(0, bytes.as_mut_ptr(), bytes.len()) };
    if count < 0 {
        let error = std::io::Error::last_os_error();
        if matches!(
            error.kind(),
            std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
        ) {
            Ok(None)
        } else {
            Err(error)
        }
    } else {
        Ok(Some(count as usize))
    }
}

#[cfg(windows)]
fn native_poll_stdin(bytes: &mut [u8]) -> std::io::Result<Option<usize>> {
    use std::ffi::c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeyEvent {
        down: i32,
        repeat: u16,
        virtual_key: u16,
        scan: u16,
        character: u16,
        modifiers: u32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    union InputEvent {
        key: KeyEvent,
        padding: [u32; 4],
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct InputRecord {
        kind: u16,
        event: InputEvent,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(kind: u32) -> *mut c_void;
        fn GetFileType(handle: *mut c_void) -> u32;
        fn PeekNamedPipe(
            handle: *mut c_void,
            buffer: *mut c_void,
            size: u32,
            read: *mut u32,
            available: *mut u32,
            remaining: *mut u32,
        ) -> i32;
        fn GetConsoleMode(handle: *mut c_void, mode: *mut u32) -> i32;
        fn GetNumberOfConsoleInputEvents(handle: *mut c_void, count: *mut u32) -> i32;
        fn PeekConsoleInputW(
            handle: *mut c_void,
            records: *mut InputRecord,
            size: u32,
            count: *mut u32,
        ) -> i32;
        fn ReadFile(
            handle: *mut c_void,
            buffer: *mut u8,
            size: u32,
            read: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;
    }
    let handle = unsafe { GetStdHandle(-10i32 as u32) };
    let mut limit = bytes.len();
    if unsafe { GetFileType(handle) } == 3 {
        let mut available = 0;
        if unsafe {
            PeekNamedPipe(
                handle,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        } == 0
        {
            let error = std::io::Error::last_os_error();
            return if error.raw_os_error() == Some(109) {
                Ok(Some(0))
            } else {
                Err(error)
            };
        }
        if available == 0 {
            return Ok(None);
        }
        limit = limit.min(available as usize);
    } else {
        let mut mode = 0;
        if unsafe { GetConsoleMode(handle, &mut mode) } != 0 {
            let mut count = 0;
            if unsafe { GetNumberOfConsoleInputEvents(handle, &mut count) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            if count == 0 {
                return Ok(None);
            }
            let mut records = vec![
                InputRecord {
                    kind: 0,
                    event: InputEvent { padding: [0; 4] }
                };
                count as usize
            ];
            if unsafe { PeekConsoleInputW(handle, records.as_mut_ptr(), count, &mut count) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            let ready = records[..count as usize].iter().any(|record| {
                if record.kind != 1 {
                    return false;
                }
                let key = unsafe { record.event.key };
                key.down != 0 && (mode & 2 == 0 || matches!(key.character, 0x0a | 0x0d))
            });
            if !ready {
                return Ok(None);
            }
            // A console key alone does not complete a line-mode read. Read
            // only after its terminator, using Rust's UTF-16 conversion.
            return std::io::Read::read(&mut std::io::stdin(), bytes).map(Some);
        }
    }
    let mut count = 0;
    if unsafe {
        ReadFile(
            handle,
            bytes.as_mut_ptr(),
            limit as u32,
            &mut count,
            std::ptr::null_mut(),
        )
    } == 0
    {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(109) {
            Ok(Some(0))
        } else {
            Err(error)
        }
    } else {
        Ok(Some(count as usize))
    }
}

#[cfg(not(any(unix, windows)))]
fn native_poll_stdin(_bytes: &mut [u8]) -> std::io::Result<Option<usize>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "native stdin is unavailable",
    ))
}

#[cfg(test)]
mod stdin_tests {
    use super::*;

    fn input(data: impl AsRef<[u8]>, size: usize) -> Rc<RefCell<NativeStdin>> {
        let data = data.as_ref().to_vec();
        let mut position = 0;
        Rc::new(RefCell::new(NativeStdin {
            read: Box::new(move |bytes| {
                let count = size.min(bytes.len()).min(data.len() - position);
                bytes[..count].copy_from_slice(&data[position..position + count]);
                position += count;
                Ok(Some(count))
            }),
            ..NativeStdin::default()
        }))
    }
    fn task(cursor: &Rc<RefCell<NativeStdin>>, chunk: bool, limit: usize) -> NativeStdinTask {
        NativeStdinTask::start(
            cursor.clone(),
            if chunk {
                NativeStdinOperation::Chunk
            } else {
                NativeStdinOperation::Line
            },
            &V::adt(
                if chunk {
                    "std/stdin::ReadSize"
                } else {
                    "std/stdin::LineLimit"
                },
                V::Int(limit as i64),
            ),
        )
        .unwrap_or_else(|_| panic!("read lease failed"))
    }
    fn read(cursor: &Rc<RefCell<NativeStdin>>, chunk: bool, limit: usize) -> NativeOutcome {
        let mut task = task(cursor, chunk, limit);
        for _ in 0..4096 {
            if let Some(value) = task.poll() {
                return value;
            }
        }
        panic!("read did not finish")
    }
    fn line(cursor: &Rc<RefCell<NativeStdin>>) -> V {
        read(cursor, false, NATIVE_STDIN_LINE_LIMIT).unwrap_or_else(|_| panic!("readLine failed"))
    }
    fn error(result: NativeOutcome, tag: &str, field: Option<(&str, u64)>) {
        let Err(NativeExit::Failure(error)) = result else {
            panic!("expected {tag}")
        };
        assert!(error.tagged(tag));
        if let Some((name, value)) = field {
            assert_eq!(error.payload().field(name).render(), value.to_string());
        }
    }

    #[test]
    fn configuration_keeps_exact_defaults_and_validated_opaque_byte_counts() {
        for (operation, owner, value) in [
            ("std/stdin::defaultReadSize", "std/stdin::ReadSize", 65536),
            (
                "std/stdin::defaultLineLimit",
                "std/stdin::LineLimit",
                1048576,
            ),
        ] {
            let result = native_stdin_operation(operation, vec![V::Unit]);
            assert!(result.tagged(owner));
            assert_eq!(result.payload().render(), value.to_string());
        }
        for (operation, owner, maximum) in [
            ("std/stdin::readSize", "std/stdin::ReadSize", 1048576),
            ("std/stdin::lineLimit", "std/stdin::LineLimit", 67108864),
        ] {
            for value in [1, maximum] {
                let result = native_stdin_operation(operation, vec![V::Int(value)]);
                assert!(result.tagged("std/prelude::Right"));
                assert!(result.payload().tagged(owner));
                assert_eq!(result.payload().payload().render(), value.to_string());
            }
        }
    }

    #[test]
    fn read_line_preserves_empty_lines_crlf_final_unterminated_line_and_repeated_eof() {
        for size in [1, 2, 3, NATIVE_STDIN_READ_SIZE] {
            let data = "雪🙂\r\n\nlast\r".as_bytes();
            let input = input(data, size);
            for expected in [Some("雪🙂"), Some(""), Some("last\r"), None, None] {
                let result = line(&input);
                if let Some(expected) = expected {
                    assert!(result.tagged("std/prelude::Just"));
                    assert_eq!(native_text(&result.payload()), expected);
                } else {
                    assert!(result.tagged("std/prelude::Nothing"));
                }
            }
            assert_eq!(input.borrow().offset, data.len() as u64);
        }
        assert!(line(&input([], 1)).tagged("std/prelude::Nothing"));
    }

    #[test]
    fn read_line_limits_memory_drains_failed_line_and_reports_absolute_utf8_offset() {
        let mut data = vec![b'x'; 1024 * 1024 + 2];
        data.extend_from_slice(b"\nvalid\n\xff\nlast");
        let cursor = input(data, NATIVE_STDIN_READ_SIZE);
        error(
            read(&cursor, false, NATIVE_STDIN_LINE_LIMIT),
            "std/stdin::StdinLineTooLong",
            Some(("limitBytes", 1048576)),
        );
        assert_eq!(native_text(&line(&cursor).payload()), "valid");
        let start = cursor.borrow().offset;
        error(
            read(&cursor, false, NATIVE_STDIN_LINE_LIMIT),
            "std/stdin::InvalidStdinUtf8",
            Some(("offset", start)),
        );
        assert_eq!(native_text(&line(&cursor).payload()), "last");
        for suffix in [b"\n".as_slice(), b"\r\n".as_slice(), b"".as_slice()] {
            let mut bytes = vec![b'x'; 1024 * 1024];
            bytes.extend_from_slice(suffix);
            assert_eq!(
                native_text(&line(&input(bytes, NATIVE_STDIN_READ_SIZE)).payload()).len(),
                1024 * 1024
            );
        }
    }

    #[test]
    fn chunks_share_line_cursor_and_keep_all_bytes_without_decoding() {
        let cursor = input(b"\xff\0\r\nsnow\nlast", 3);
        let first = read(&cursor, true, 2).ok().unwrap();
        assert_eq!(native_bytes(&first.payload()), &[255, 0]);
        assert_eq!(native_text(&line(&cursor).payload()), "");
        assert_eq!(native_text(&line(&cursor).payload()), "snow");
        let tail = read(&cursor, true, 2).ok().unwrap();
        assert_eq!(native_bytes(&tail.payload()), b"la");
        assert_eq!(native_text(&line(&cursor).payload()), "st");
        for chunk in [false, true, true, false] {
            assert!(read(&cursor, chunk, 3)
                .ok()
                .unwrap()
                .tagged("std/prelude::Nothing"));
        }
    }

    #[test]
    fn line_limit_counts_bytes_before_bom_removal_and_only_strips_cr_before_lf() {
        for size in [1, 2, 64] {
            for (bytes, limit, expected) in [
                ("雪\r\n", 3, Some("雪")),
                ("雪\n", 2, None),
                ("x\r", 1, None),
                ("x\r\n", 1, Some("x")),
                ("\u{feff}x\n", 4, Some("x")),
                ("\u{feff}x\n", 3, None),
                ("\r\r\n", 1, Some("\r")),
                ("\n", 1, Some("")),
            ] {
                let result = read(&input(bytes, size), false, limit);
                if let Some(expected) = expected {
                    assert_eq!(native_text(&result.ok().unwrap().payload()), expected);
                } else {
                    error(
                        result,
                        "std/stdin::StdinLineTooLong",
                        Some(("limitBytes", limit as u64)),
                    );
                }
            }
        }
    }

    #[test]
    fn invalid_utf8_offsets_and_error_precedence_match_byte_cursor() {
        for invalid in [
            vec![0x80],
            vec![0xc0, 0xaf],
            vec![0xc2],
            vec![0xc3, 0x28],
            vec![0xe0, 0x80, 0x80],
            vec![0xed, 0xa0, 0x80],
            vec![0xe2, 0x82],
            vec![0xf0, 0x80, 0x80, 0x80],
            vec![0xf4, 0x90, 0x80, 0x80],
            vec![0xf5, 0x80, 0x80, 0x80],
        ] {
            let data = [b"skip\nA".as_slice(), &invalid, b"\nok\n"].concat();
            let cursor = input(data, 1);
            assert_eq!(native_text(&line(&cursor).payload()), "skip");
            error(
                read(&cursor, false, 64),
                "std/stdin::InvalidStdinUtf8",
                Some(("offset", 6)),
            );
            assert_eq!(native_text(&line(&cursor).payload()), "ok");
        }
        error(
            read(&input(b"\xffxx\n", 1), false, 1),
            "std/stdin::StdinLineTooLong",
            Some(("limitBytes", 1)),
        );
    }

    #[test]
    fn overflow_is_terminal_and_precedes_an_active_read() {
        let cursor = input(b"abc\n", 1);
        cursor.borrow_mut().offset = NATIVE_STDIN_MAX_POSITION - 1;
        assert_eq!(
            native_bytes(&read(&cursor, true, 1).ok().unwrap().payload()),
            b"a"
        );
        error(
            read(&cursor, false, 10),
            "std/stdin::StdinPositionOverflow",
            None,
        );
        for chunk in [true, false] {
            let result = NativeStdinTask::start(
                cursor.clone(),
                if chunk {
                    NativeStdinOperation::Chunk
                } else {
                    NativeStdinOperation::Line
                },
                &V::adt(
                    if chunk {
                        "std/stdin::ReadSize"
                    } else {
                        "std/stdin::LineLimit"
                    },
                    V::Int(1),
                ),
            )
            .map(|_| V::Unit);
            error(result, "std/stdin::StdinPositionOverflow", None);
        }
        cursor.borrow_mut().active = true;
        error(
            NativeStdinTask::start(
                cursor.clone(),
                NativeStdinOperation::Line,
                &V::adt("std/stdin::LineLimit", V::Int(1)),
            )
            .map(|_| V::Unit),
            "std/stdin::StdinPositionOverflow",
            None,
        );
    }

    #[test]
    fn pending_read_has_one_lease_and_drop_releases_it_without_consuming_input() {
        let cursor = input(b"kept\n", 64);
        let first = task(&cursor, false, 64);
        error(
            NativeStdinTask::start(
                cursor.clone(),
                NativeStdinOperation::Chunk,
                &V::adt("std/stdin::ReadSize", V::Int(1)),
            )
            .map(|_| V::Unit),
            "std/stdin::ConcurrentStdinRead",
            None,
        );
        drop(first);
        assert_eq!(native_text(&line(&cursor).payload()), "kept");
        cursor.borrow_mut().read = Box::new(|_| Err(std::io::Error::other("read failed")));
        cursor.borrow_mut().eof = false;
        for chunk in [true, false] {
            error(read(&cursor, chunk, 3), "std/stdin::StdinReadFailure", None);
            assert!(!cursor.borrow().active);
        }
    }
}
